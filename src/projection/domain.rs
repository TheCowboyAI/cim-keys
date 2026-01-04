// Copyright (c) 2025 - Cowboy AI, LLC.

//! # Domain-Specific Projections
//!
//! Composable projections for CIM domain concepts. Each projection follows the pattern:
//!
//! ```text
//! Input (prerequisites) → Projection → Output (domain entity)
//! ```
//!
//! ## Key Insight
//!
//! Everything is a projection:
//! - **Keys**: (TrustChain, Rules, Location) → Key
//! - **Certificates**: (Key, Identity, ValidityPeriod) → Certificate
//! - **Locations**: SpatialData → Location
//! - **Persons**: (Identity, Roles, Policies) → Person
//!
//! ## Composition Example
//!
//! ```text
//! let person_with_key = validate_identity
//!     .then(assign_roles)
//!     .then(apply_policies)
//!     .then(generate_key)
//!     .then(issue_certificate);
//! ```

use crate::projection::{Projection, ProjectionError, PrerequisiteProjection};
use crate::domain::pki::{CertificateInfo, CertificateType, KeyInfo};
use crate::events::{KeyAlgorithm, KeyPurpose};
use chrono::{DateTime, Utc};
use std::marker::PhantomData;
use uuid::Uuid;

// ============================================================================
// KEY GENERATION PROJECTION
// ============================================================================

/// Input requirements for key generation
#[derive(Debug, Clone)]
pub struct KeyGenerationInput {
    /// Who will own this key
    pub owner_person_id: Uuid,
    /// What algorithm to use
    pub algorithm: KeyAlgorithm,
    /// Purpose of this key
    pub purpose: KeyPurpose,
    /// Optional YubiKey storage location
    pub yubikey_serial: Option<String>,
    /// Optional PIV slot
    pub piv_slot: Option<String>,
}

/// Prerequisites that must be met for key generation
#[derive(Debug, Clone)]
pub struct KeyGenerationPrerequisites {
    /// Trust chain must be established (root CA exists)
    pub trust_chain_established: bool,
    /// Person must be active in organization
    pub person_active: bool,
    /// Key slot must be available (if YubiKey)
    pub slot_available: bool,
    /// Required policies are satisfied
    pub policies_satisfied: bool,
}

impl KeyGenerationPrerequisites {
    /// Check if all prerequisites are met
    pub fn is_satisfied(&self) -> bool {
        self.trust_chain_established
            && self.person_active
            && self.slot_available
            && self.policies_satisfied
    }

    /// Get list of missing prerequisites
    pub fn missing(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.trust_chain_established {
            missing.push("TrustChain");
        }
        if !self.person_active {
            missing.push("PersonActive");
        }
        if !self.slot_available {
            missing.push("SlotAvailable");
        }
        if !self.policies_satisfied {
            missing.push("Policies");
        }
        missing
    }
}

/// Key generation projection
///
/// Projects (KeyGenerationInput, KeyGenerationPrerequisites) → KeyInfo
pub struct KeyGenerationProjection;

impl Projection<(KeyGenerationInput, KeyGenerationPrerequisites), KeyInfo, ProjectionError>
    for KeyGenerationProjection
{
    fn project(
        &self,
        (input, prereqs): (KeyGenerationInput, KeyGenerationPrerequisites),
    ) -> Result<KeyInfo, ProjectionError> {
        // Validate prerequisites
        if !prereqs.is_satisfied() {
            let missing = prereqs.missing();
            return Err(ProjectionError::PrerequisiteNotMet {
                name: "KeyGeneration".to_string(),
                description: format!("Missing: {}", missing.join(", ")),
            });
        }

        // Create key info
        let mut key_info = KeyInfo::new(input.algorithm, input.purpose)
            .with_owner(input.owner_person_id);

        if let (Some(serial), Some(slot)) = (input.yubikey_serial, input.piv_slot) {
            key_info = key_info.with_yubikey(serial, slot);
        }

        Ok(key_info)
    }

    fn name(&self) -> &'static str {
        "KeyGeneration"
    }
}

// ============================================================================
// CERTIFICATE PROJECTION
// ============================================================================

/// Input requirements for certificate issuance
#[derive(Debug, Clone)]
pub struct CertificateInput {
    /// The key to bind to this certificate
    pub key_id: crate::domain::ids::KeyId,
    /// Certificate subject (e.g., "CN=Alice, O=CowboyAI")
    pub subject: String,
    /// Type of certificate to issue
    pub cert_type: CertificateType,
    /// Issuer certificate (None for root CA)
    pub issuer_id: Option<crate::domain::ids::CertificateId>,
    /// Owner person (for leaf certs)
    pub owner_person_id: Option<Uuid>,
    /// Subject Alternative Names
    pub san: Vec<String>,
    /// Validity in days
    pub validity_days: u32,
}

/// Prerequisites for certificate issuance
#[derive(Debug, Clone)]
pub struct CertificatePrerequisites {
    /// Key must exist and be active
    pub key_exists: bool,
    /// Issuer certificate valid (or self-signed for root)
    pub issuer_valid: bool,
    /// Subject identity verified
    pub identity_verified: bool,
    /// Policy constraints satisfied
    pub policy_satisfied: bool,
}

impl CertificatePrerequisites {
    pub fn is_satisfied(&self) -> bool {
        self.key_exists && self.issuer_valid && self.identity_verified && self.policy_satisfied
    }

    pub fn missing(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.key_exists {
            missing.push("KeyExists");
        }
        if !self.issuer_valid {
            missing.push("IssuerValid");
        }
        if !self.identity_verified {
            missing.push("IdentityVerified");
        }
        if !self.policy_satisfied {
            missing.push("PolicySatisfied");
        }
        missing
    }
}

/// Certificate issuance projection
///
/// Projects (CertificateInput, CertificatePrerequisites) → CertificateInfo
pub struct CertificateProjection;

impl Projection<(CertificateInput, CertificatePrerequisites), CertificateInfo, ProjectionError>
    for CertificateProjection
{
    fn project(
        &self,
        (input, prereqs): (CertificateInput, CertificatePrerequisites),
    ) -> Result<CertificateInfo, ProjectionError> {
        if !prereqs.is_satisfied() {
            let missing = prereqs.missing();
            return Err(ProjectionError::PrerequisiteNotMet {
                name: "Certificate".to_string(),
                description: format!("Missing: {}", missing.join(", ")),
            });
        }

        let cert_info = match input.cert_type {
            CertificateType::Root => CertificateInfo::new_root(input.subject, input.validity_days),
            CertificateType::Intermediate => {
                let issuer_id = input.issuer_id.ok_or_else(|| {
                    ProjectionError::ValidationFailed {
                        field: "issuer_id".to_string(),
                        reason: "Intermediate CA requires issuer".to_string(),
                    }
                })?;
                CertificateInfo::new_intermediate(input.subject, issuer_id, input.validity_days)
            }
            CertificateType::Leaf | CertificateType::Policy => {
                let issuer_id = input.issuer_id.ok_or_else(|| {
                    ProjectionError::ValidationFailed {
                        field: "issuer_id".to_string(),
                        reason: "Leaf certificate requires issuer".to_string(),
                    }
                })?;
                let owner_id = input.owner_person_id.ok_or_else(|| {
                    ProjectionError::ValidationFailed {
                        field: "owner_person_id".to_string(),
                        reason: "Leaf certificate requires owner".to_string(),
                    }
                })?;
                CertificateInfo::new_leaf(
                    input.subject,
                    issuer_id,
                    owner_id,
                    input.san,
                    input.validity_days,
                )
            }
        };

        Ok(cert_info)
    }

    fn name(&self) -> &'static str {
        "Certificate"
    }
}

// ============================================================================
// LOCATION PROJECTION
// ============================================================================

/// Input for location projection
#[derive(Debug, Clone)]
pub struct LocationInput {
    /// Location name
    pub name: String,
    /// Location type
    pub location_type: LocationType,
    /// Physical address (optional)
    pub address: Option<AddressInput>,
    /// Geo coordinates (optional)
    pub coordinates: Option<(f64, f64)>,
    /// Virtual location URL (optional)
    pub virtual_url: Option<String>,
}

/// Location type for projection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    Physical,
    Virtual,
    Logical,
    Hybrid,
}

/// Address input
#[derive(Debug, Clone)]
pub struct AddressInput {
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
    pub postal_code: Option<String>,
}

/// Location output from projection
#[derive(Debug, Clone)]
pub struct LocationOutput {
    pub id: Uuid,
    pub name: String,
    pub location_type: LocationType,
    pub created_at: DateTime<Utc>,
    pub address: Option<AddressInput>,
    pub coordinates: Option<(f64, f64)>,
    pub virtual_url: Option<String>,
}

/// Location projection
///
/// Projects LocationInput → LocationOutput
pub struct LocationProjection;

impl Projection<LocationInput, LocationOutput, ProjectionError> for LocationProjection {
    fn project(&self, input: LocationInput) -> Result<LocationOutput, ProjectionError> {
        // Validate based on location type
        match input.location_type {
            LocationType::Physical => {
                if input.address.is_none() && input.coordinates.is_none() {
                    return Err(ProjectionError::ValidationFailed {
                        field: "address".to_string(),
                        reason: "Physical location requires address or coordinates".to_string(),
                    });
                }
            }
            LocationType::Virtual => {
                if input.virtual_url.is_none() {
                    return Err(ProjectionError::ValidationFailed {
                        field: "virtual_url".to_string(),
                        reason: "Virtual location requires URL".to_string(),
                    });
                }
            }
            LocationType::Logical | LocationType::Hybrid => {
                // Logical/Hybrid can have any combination
            }
        }

        Ok(LocationOutput {
            id: Uuid::now_v7(),
            name: input.name,
            location_type: input.location_type,
            created_at: Utc::now(),
            address: input.address,
            coordinates: input.coordinates,
            virtual_url: input.virtual_url,
        })
    }

    fn name(&self) -> &'static str {
        "Location"
    }
}

// ============================================================================
// PERSON PROJECTION
// ============================================================================

/// Input for person projection
#[derive(Debug, Clone)]
pub struct PersonInput {
    /// Person's name
    pub name: String,
    /// Email address
    pub email: String,
    /// Assigned roles
    pub roles: Vec<String>,
    /// Organization they belong to
    pub organization_id: Uuid,
    /// Unit within organization
    pub unit_id: Option<Uuid>,
}

/// Prerequisites for person creation
#[derive(Debug, Clone)]
pub struct PersonPrerequisites {
    /// Organization exists and is active
    pub organization_exists: bool,
    /// Email is unique
    pub email_unique: bool,
    /// Roles are valid for organization
    pub roles_valid: bool,
}

impl PersonPrerequisites {
    pub fn is_satisfied(&self) -> bool {
        self.organization_exists && self.email_unique && self.roles_valid
    }

    pub fn missing(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.organization_exists {
            missing.push("OrganizationExists");
        }
        if !self.email_unique {
            missing.push("EmailUnique");
        }
        if !self.roles_valid {
            missing.push("RolesValid");
        }
        missing
    }
}

/// Person output from projection
#[derive(Debug, Clone)]
pub struct PersonOutput {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
    pub organization_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Person projection
///
/// Projects (PersonInput, PersonPrerequisites) → PersonOutput
pub struct PersonProjection;

impl Projection<(PersonInput, PersonPrerequisites), PersonOutput, ProjectionError>
    for PersonProjection
{
    fn project(
        &self,
        (input, prereqs): (PersonInput, PersonPrerequisites),
    ) -> Result<PersonOutput, ProjectionError> {
        if !prereqs.is_satisfied() {
            let missing = prereqs.missing();
            return Err(ProjectionError::PrerequisiteNotMet {
                name: "Person".to_string(),
                description: format!("Missing: {}", missing.join(", ")),
            });
        }

        Ok(PersonOutput {
            id: Uuid::now_v7(),
            name: input.name,
            email: input.email,
            roles: input.roles,
            organization_id: input.organization_id,
            unit_id: input.unit_id,
            created_at: Utc::now(),
        })
    }

    fn name(&self) -> &'static str {
        "Person"
    }
}

// ============================================================================
// COMPOSED WORKFLOW PROJECTIONS
// ============================================================================

/// Complete person onboarding projection
///
/// This composes multiple projections into a complete workflow:
/// PersonInput → Person → Key → Certificate
pub struct PersonOnboardingProjection<P, K, C> {
    person_proj: P,
    key_proj: K,
    cert_proj: C,
    _phantom: PhantomData<fn() -> ()>,
}

impl<P, K, C> PersonOnboardingProjection<P, K, C>
where
    P: Projection<(PersonInput, PersonPrerequisites), PersonOutput, ProjectionError>,
    K: Projection<(KeyGenerationInput, KeyGenerationPrerequisites), KeyInfo, ProjectionError>,
    C: Projection<(CertificateInput, CertificatePrerequisites), CertificateInfo, ProjectionError>,
{
    pub fn new(person_proj: P, key_proj: K, cert_proj: C) -> Self {
        Self {
            person_proj,
            key_proj,
            cert_proj,
            _phantom: PhantomData,
        }
    }
}

/// Onboarding workflow input
#[derive(Debug, Clone)]
pub struct OnboardingInput {
    pub person: PersonInput,
    pub person_prereqs: PersonPrerequisites,
    pub key_algorithm: KeyAlgorithm,
    pub key_purpose: KeyPurpose,
    pub key_prereqs: KeyGenerationPrerequisites,
    pub cert_validity_days: u32,
    pub cert_prereqs: CertificatePrerequisites,
    pub issuer_cert_id: crate::domain::ids::CertificateId,
}

/// Complete onboarding result
#[derive(Debug, Clone)]
pub struct OnboardingOutput {
    pub person: PersonOutput,
    pub key: KeyInfo,
    pub certificate: CertificateInfo,
}

impl<P, K, C> Projection<OnboardingInput, OnboardingOutput, ProjectionError>
    for PersonOnboardingProjection<P, K, C>
where
    P: Projection<(PersonInput, PersonPrerequisites), PersonOutput, ProjectionError>,
    K: Projection<(KeyGenerationInput, KeyGenerationPrerequisites), KeyInfo, ProjectionError>,
    C: Projection<(CertificateInput, CertificatePrerequisites), CertificateInfo, ProjectionError>,
{
    fn project(&self, input: OnboardingInput) -> Result<OnboardingOutput, ProjectionError> {
        // Step 1: Create person
        let person = self
            .person_proj
            .project((input.person.clone(), input.person_prereqs))
            .map_err(|e| ProjectionError::CompositionError {
                stage: "Person".to_string(),
                inner: Box::new(e),
            })?;

        // Step 2: Generate key for person
        let key_input = KeyGenerationInput {
            owner_person_id: person.id,
            algorithm: input.key_algorithm,
            purpose: input.key_purpose,
            yubikey_serial: None,
            piv_slot: None,
        };
        let key = self
            .key_proj
            .project((key_input, input.key_prereqs))
            .map_err(|e| ProjectionError::CompositionError {
                stage: "KeyGeneration".to_string(),
                inner: Box::new(e),
            })?;

        // Step 3: Issue certificate
        let cert_input = CertificateInput {
            key_id: key.id.clone(),
            subject: format!("CN={}, EMAIL={}", person.name, person.email),
            cert_type: CertificateType::Leaf,
            issuer_id: Some(input.issuer_cert_id),
            owner_person_id: Some(person.id),
            san: vec![format!("email:{}", person.email)],
            validity_days: input.cert_validity_days,
        };
        let certificate = self
            .cert_proj
            .project((cert_input, input.cert_prereqs))
            .map_err(|e| ProjectionError::CompositionError {
                stage: "Certificate".to_string(),
                inner: Box::new(e),
            })?;

        Ok(OnboardingOutput {
            person,
            key,
            certificate,
        })
    }

    fn name(&self) -> &'static str {
        "PersonOnboarding"
    }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/// Create a key generation projection
pub fn key_generation() -> KeyGenerationProjection {
    KeyGenerationProjection
}

/// Create a certificate projection
pub fn certificate() -> CertificateProjection {
    CertificateProjection
}

/// Create a location projection
pub fn location() -> LocationProjection {
    LocationProjection
}

/// Create a person projection
pub fn person() -> PersonProjection {
    PersonProjection
}

/// Create a complete person onboarding projection
pub fn person_onboarding() -> PersonOnboardingProjection<PersonProjection, KeyGenerationProjection, CertificateProjection> {
    PersonOnboardingProjection::new(
        PersonProjection,
        KeyGenerationProjection,
        CertificateProjection,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::{CertificateId, KeyId};

    #[test]
    fn test_key_generation_with_prerequisites() {
        let proj = key_generation();

        let input = KeyGenerationInput {
            owner_person_id: Uuid::now_v7(),
            algorithm: KeyAlgorithm::Ed25519,
            purpose: KeyPurpose::Authentication,
            yubikey_serial: None,
            piv_slot: None,
        };

        let prereqs = KeyGenerationPrerequisites {
            trust_chain_established: true,
            person_active: true,
            slot_available: true,
            policies_satisfied: true,
        };

        let result = proj.project((input, prereqs));
        assert!(result.is_ok());
        let key = result.unwrap();
        assert_eq!(key.algorithm, KeyAlgorithm::Ed25519);
    }

    #[test]
    fn test_key_generation_fails_without_prerequisites() {
        let proj = key_generation();

        let input = KeyGenerationInput {
            owner_person_id: Uuid::now_v7(),
            algorithm: KeyAlgorithm::Ed25519,
            purpose: KeyPurpose::Authentication,
            yubikey_serial: None,
            piv_slot: None,
        };

        let prereqs = KeyGenerationPrerequisites {
            trust_chain_established: false, // Missing!
            person_active: true,
            slot_available: true,
            policies_satisfied: true,
        };

        let result = proj.project((input, prereqs));
        assert!(result.is_err());
        match result {
            Err(ProjectionError::PrerequisiteNotMet { name, description }) => {
                assert_eq!(name, "KeyGeneration");
                assert!(description.contains("TrustChain"));
            }
            _ => panic!("Expected PrerequisiteNotMet error"),
        }
    }

    #[test]
    fn test_location_physical_requires_address() {
        let proj = location();

        let input = LocationInput {
            name: "HQ".to_string(),
            location_type: LocationType::Physical,
            address: None, // Missing!
            coordinates: None,
            virtual_url: None,
        };

        let result = proj.project(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_location_virtual_requires_url() {
        let proj = location();

        let input = LocationInput {
            name: "Cloud".to_string(),
            location_type: LocationType::Virtual,
            address: None,
            coordinates: None,
            virtual_url: Some("https://cloud.example.com".to_string()),
        };

        let result = proj.project(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_person_projection() {
        let proj = person();

        let input = PersonInput {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            roles: vec!["Developer".to_string()],
            organization_id: Uuid::now_v7(),
            unit_id: None,
        };

        let prereqs = PersonPrerequisites {
            organization_exists: true,
            email_unique: true,
            roles_valid: true,
        };

        let result = proj.project((input, prereqs));
        assert!(result.is_ok());
        let person = result.unwrap();
        assert_eq!(person.name, "Alice");
    }

    #[test]
    fn test_certificate_root() {
        let proj = certificate();

        let input = CertificateInput {
            key_id: KeyId::new(),
            subject: "CN=Root CA".to_string(),
            cert_type: CertificateType::Root,
            issuer_id: None,
            owner_person_id: None,
            san: Vec::new(),
            validity_days: 3650,
        };

        let prereqs = CertificatePrerequisites {
            key_exists: true,
            issuer_valid: true, // Self-signed is OK for root
            identity_verified: true,
            policy_satisfied: true,
        };

        let result = proj.project((input, prereqs));
        assert!(result.is_ok());
    }

    #[test]
    fn test_certificate_leaf_requires_issuer() {
        let proj = certificate();

        let input = CertificateInput {
            key_id: KeyId::new(),
            subject: "CN=Alice".to_string(),
            cert_type: CertificateType::Leaf,
            issuer_id: None, // Missing!
            owner_person_id: Some(Uuid::now_v7()),
            san: Vec::new(),
            validity_days: 365,
        };

        let prereqs = CertificatePrerequisites {
            key_exists: true,
            issuer_valid: true,
            identity_verified: true,
            policy_satisfied: true,
        };

        let result = proj.project((input, prereqs));
        assert!(result.is_err());
    }
}
