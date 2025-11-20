// Certificate Projections
//
// Projects domain compositions (Organization × Person × Location × KeyPurpose)
// into certificate signing requests and X.509 parameters.
//
// Functor chain:
//   DomainContext --[project]--> CSR --[to_x509]--> X509Params --[library]--> Certificate

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{KeyContext, Organization, Person, OrganizationUnit};
use crate::events::KeyPurpose;
use crate::value_objects::{CertificateSubject, PublicKey, SignatureAlgorithm};

// ============================================================================
// Certificate Signing Request (Intermediate Projection)
// ============================================================================

/// Certificate Signing Request
///
/// This is a projection of domain context into a CSR format.
/// It's an intermediate representation before library-specific params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSigningRequest {
    pub subject: CertificateSubject,
    pub public_key: PublicKey,
    pub signature_algorithm: SignatureAlgorithm,
    pub extensions: X509Extensions,
    pub validity: CertificateValidity,
}

/// Certificate validity specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidity {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
}

impl CertificateValidity {
    /// Create validity starting now for the given number of years
    pub fn from_years(years: u32) -> Self {
        let not_before = Utc::now();
        let not_after = not_before + Duration::days((years * 365) as i64);
        Self {
            not_before,
            not_after,
        }
    }

    /// Create validity for a CA certificate (10 years default)
    pub fn for_ca() -> Self {
        Self::from_years(10)
    }

    /// Create validity for an intermediate CA (5 years default)
    pub fn for_intermediate_ca() -> Self {
        Self::from_years(5)
    }

    /// Create validity for a leaf certificate (1 year default)
    pub fn for_leaf() -> Self {
        Self::from_years(1)
    }
}

/// X.509 Extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X509Extensions {
    pub key_usage: Vec<KeyUsage>,
    pub extended_key_usage: Vec<ExtendedKeyUsage>,
    pub basic_constraints: Option<BasicConstraints>,
    pub subject_alternative_names: Vec<SubjectAlternativeName>,
}

impl Default for X509Extensions {
    fn default() -> Self {
        Self {
            key_usage: Vec::new(),
            extended_key_usage: Vec::new(),
            basic_constraints: None,
            subject_alternative_names: Vec::new(),
        }
    }
}

/// Key usage extension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyUsage {
    DigitalSignature,
    NonRepudiation,
    KeyEncipherment,
    DataEncipherment,
    KeyAgreement,
    KeyCertSign,
    CRLSign,
    EncipherOnly,
    DecipherOnly,
}

/// Extended key usage extension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtendedKeyUsage {
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OCSPSigning,
}

/// Basic constraints extension
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BasicConstraints {
    pub is_ca: bool,
    pub path_len_constraint: Option<u8>,
}

/// Subject Alternative Name
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectAlternativeName {
    DnsName(String),
    Email(String),
    IpAddress(String),
    Uri(String),
}

// ============================================================================
// Projection: Domain Context → CSR
// ============================================================================

/// Projection functor: Domain → CSR
pub struct CertificateRequestProjection;

impl CertificateRequestProjection {
    /// Project domain context to Certificate Signing Request
    ///
    /// This is a pure functor mapping domain compositions to CSR format.
    /// No side effects - just data transformation.
    pub fn project_from_context(
        context: &KeyContext,
        public_key: PublicKey,
        purpose: KeyPurpose,
        validity_years: u32,
    ) -> CertificateSigningRequest {
        // Map domain context to certificate subject
        let subject = Self::build_subject(context);

        // Determine signature algorithm based on key algorithm
        let signature_algorithm = Self::determine_signature_algorithm(&public_key);

        // Build extensions based on purpose
        let extensions = Self::build_extensions(purpose, context);

        // Calculate validity period
        let validity = match purpose {
            KeyPurpose::CertificateAuthority => CertificateValidity::for_ca(),
            _ => CertificateValidity::from_years(validity_years),
        };

        CertificateSigningRequest {
            subject,
            public_key,
            signature_algorithm,
            extensions,
            validity,
        }
    }

    /// Project for Root CA certificate
    pub fn project_root_ca(
        organization: &Organization,
        public_key: PublicKey,
    ) -> CertificateSigningRequest {
        let subject = CertificateSubject {
            common_name: format!("{} Root CA", organization.name),
            organization: Some(organization.name.clone()),
            organizational_unit: None,
            country: Some(organization.metadata.get("country")
                .cloned()
                .unwrap_or_else(|| "US".to_string())),
            state: organization.metadata.get("state").cloned(),
            locality: organization.metadata.get("city").cloned(),
            email: organization.metadata.get("email").cloned(),
        };

        let signature_algorithm = Self::determine_signature_algorithm(&public_key);

        let extensions = X509Extensions {
            key_usage: vec![
                KeyUsage::KeyCertSign,
                KeyUsage::CRLSign,
                KeyUsage::DigitalSignature,
            ],
            extended_key_usage: Vec::new(),
            basic_constraints: Some(BasicConstraints {
                is_ca: true,
                path_len_constraint: None, // Unlimited path length for root
            }),
            subject_alternative_names: organization.metadata.get("email")
                .map(|email| vec![SubjectAlternativeName::Email(email.clone())])
                .unwrap_or_default(),
        };

        CertificateSigningRequest {
            subject,
            public_key,
            signature_algorithm,
            extensions,
            validity: CertificateValidity::for_ca(),
        }
    }

    /// Project for Intermediate CA certificate
    pub fn project_intermediate_ca(
        organization: &Organization,
        unit: &OrganizationUnit,
        public_key: PublicKey,
        path_len_constraint: Option<u8>,
    ) -> CertificateSigningRequest {
        let subject = CertificateSubject {
            common_name: format!("{} {} Intermediate CA", organization.name, unit.name),
            organization: Some(organization.name.clone()),
            organizational_unit: Some(unit.name.clone()),
            country: Some(organization.metadata.get("country")
                .cloned()
                .unwrap_or_else(|| "US".to_string())),
            state: organization.metadata.get("state").cloned(),
            locality: organization.metadata.get("city").cloned(),
            email: organization.metadata.get("email").cloned(),
        };

        let signature_algorithm = Self::determine_signature_algorithm(&public_key);

        let extensions = X509Extensions {
            key_usage: vec![
                KeyUsage::KeyCertSign,
                KeyUsage::CRLSign,
                KeyUsage::DigitalSignature,
            ],
            extended_key_usage: Vec::new(),
            basic_constraints: Some(BasicConstraints {
                is_ca: true,
                path_len_constraint,
            }),
            subject_alternative_names: organization.metadata.get("email")
                .map(|email| vec![SubjectAlternativeName::Email(email.clone())])
                .unwrap_or_default(),
        };

        CertificateSigningRequest {
            subject,
            public_key,
            signature_algorithm,
            extensions,
            validity: CertificateValidity::for_intermediate_ca(),
        }
    }

    /// Project for Person leaf certificate
    pub fn project_person_certificate(
        person: &Person,
        organization: &Organization,
        unit: Option<&OrganizationUnit>,
        public_key: PublicKey,
        purpose: KeyPurpose,
    ) -> CertificateSigningRequest {
        let subject = CertificateSubject {
            common_name: person.name.clone(),
            organization: Some(organization.name.clone()),
            organizational_unit: unit.map(|u| u.name.clone()),
            country: Some(organization.metadata.get("country")
                .cloned()
                .unwrap_or_else(|| "US".to_string())),
            state: organization.metadata.get("state").cloned(),
            locality: organization.metadata.get("city").cloned(),
            email: Some(person.email.clone()),
        };

        let signature_algorithm = Self::determine_signature_algorithm(&public_key);

        let extensions = Self::build_person_extensions(purpose, person);

        CertificateSigningRequest {
            subject,
            public_key,
            signature_algorithm,
            extensions,
            validity: CertificateValidity::for_leaf(),
        }
    }

    // ========================================================================
    // Helper Methods (Pure Functions)
    // ========================================================================

    fn build_subject(_context: &KeyContext) -> CertificateSubject {
        // KeyContext doesn't have metadata directly
        // This function should be replaced by specific projections
        // using Organization, Person, etc. directly
        CertificateSubject {
            common_name: "Unknown".to_string(),
            organization: None,
            organizational_unit: None,
            country: None,
            state: None,
            locality: None,
            email: None,
        }
    }

    fn determine_signature_algorithm(public_key: &PublicKey) -> SignatureAlgorithm {
        use crate::events::KeyAlgorithm;

        match &public_key.algorithm {
            KeyAlgorithm::Ed25519 => SignatureAlgorithm::Ed25519,
            KeyAlgorithm::Ecdsa { curve } if curve == "P-256" => SignatureAlgorithm::EcdsaSha256,
            KeyAlgorithm::Rsa { bits } if *bits >= 2048 => SignatureAlgorithm::RsaSha256,
            KeyAlgorithm::Rsa { .. } => SignatureAlgorithm::RsaSha256, // Fallback
            _ => SignatureAlgorithm::RsaSha256, // Default fallback
        }
    }

    fn build_extensions(purpose: KeyPurpose, _context: &KeyContext) -> X509Extensions {
        let mut extensions = X509Extensions::default();

        match purpose {
            KeyPurpose::CertificateAuthority => {
                extensions.key_usage = vec![
                    KeyUsage::KeyCertSign,
                    KeyUsage::CRLSign,
                    KeyUsage::DigitalSignature,
                ];
                extensions.basic_constraints = Some(BasicConstraints {
                    is_ca: true,
                    path_len_constraint: Some(0),
                });
            }
            KeyPurpose::Signing => {
                extensions.key_usage = vec![
                    KeyUsage::DigitalSignature,
                    KeyUsage::NonRepudiation,
                ];
                extensions.extended_key_usage = vec![
                    ExtendedKeyUsage::CodeSigning,
                    ExtendedKeyUsage::EmailProtection,
                ];
            }
            KeyPurpose::Encryption => {
                extensions.key_usage = vec![
                    KeyUsage::KeyEncipherment,
                    KeyUsage::DataEncipherment,
                ];
            }
            KeyPurpose::Authentication => {
                extensions.key_usage = vec![
                    KeyUsage::DigitalSignature,
                    KeyUsage::KeyAgreement,
                ];
                extensions.extended_key_usage = vec![
                    ExtendedKeyUsage::ClientAuth,
                    ExtendedKeyUsage::ServerAuth,
                ];
            }
            KeyPurpose::KeyAgreement => {
                extensions.key_usage = vec![
                    KeyUsage::KeyAgreement,
                ];
            }
            _ => {
                // Default to signing
                extensions.key_usage = vec![
                    KeyUsage::DigitalSignature,
                ];
            }
        }

        extensions
    }

    fn build_person_extensions(purpose: KeyPurpose, person: &Person) -> X509Extensions {
        let mut extensions = X509Extensions::default();

        match purpose {
            KeyPurpose::Signing => {
                extensions.key_usage = vec![
                    KeyUsage::DigitalSignature,
                    KeyUsage::NonRepudiation,
                ];
                extensions.extended_key_usage = vec![
                    ExtendedKeyUsage::CodeSigning,
                    ExtendedKeyUsage::EmailProtection,
                ];
                extensions.subject_alternative_names = vec![
                    SubjectAlternativeName::Email(person.email.clone()),
                ];
            }
            KeyPurpose::Authentication => {
                extensions.key_usage = vec![
                    KeyUsage::DigitalSignature,
                ];
                extensions.extended_key_usage = vec![
                    ExtendedKeyUsage::ClientAuth,
                ];
                extensions.subject_alternative_names = vec![
                    SubjectAlternativeName::Email(person.email.clone()),
                ];
            }
            KeyPurpose::Encryption => {
                extensions.key_usage = vec![
                    KeyUsage::KeyEncipherment,
                    KeyUsage::DataEncipherment,
                ];
                extensions.extended_key_usage = vec![
                    ExtendedKeyUsage::EmailProtection,
                ];
                extensions.subject_alternative_names = vec![
                    SubjectAlternativeName::Email(person.email.clone()),
                ];
            }
            _ => {
                // Default
                extensions.key_usage = vec![
                    KeyUsage::DigitalSignature,
                ];
                extensions.subject_alternative_names = vec![
                    SubjectAlternativeName::Email(person.email.clone()),
                ];
            }
        }

        extensions
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validity_from_years() {
        let validity = CertificateValidity::from_years(1);
        let duration = validity.not_after - validity.not_before;
        assert_eq!(duration.num_days(), 365);
    }

    #[test]
    fn test_ca_validity() {
        let validity = CertificateValidity::for_ca();
        let duration = validity.not_after - validity.not_before;
        assert_eq!(duration.num_days(), 3650); // 10 years
    }
}
