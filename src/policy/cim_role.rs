// Copyright (c) 2025 - Cowboy AI, LLC.

//! CIM Role - Claims Composition Container
//!
//! Role is the first element of the categorical product `Role × Subject` that
//! forms the base capability. Roles compose CimClaims into semantic aggregates.
//!
//! # Mathematical Structure
//!
//! CimRole forms a bounded join-semilattice like ClaimSet:
//! - `(L, ∨, ⊥)` where `∨` is claim union, `⊥` is empty role
//!
//! # Relationship to Capability
//!
//! ```text
//! Capability = fold(Policies, Role × Subject)
//!
//! where:
//!   - Role is the claims composition (what permissions)
//!   - Subject is the NATS patterns (where they apply)
//!   - Policies constrain the product via list catamorphism
//! ```
//!
//! # CIM-Specific Design
//!
//! Unlike generic RBAC roles, CIM roles are specifically for:
//! - NATS infrastructure operations
//! - PKI certificate lifecycle
//! - YubiKey hardware management
//! - Trust delegation chains

use crate::policy::cim_claims::{CimClaim, ClaimSet, ClaimDomain};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// CIM ROLE - Claims Composition Container
// ============================================================================

/// CIM Role - semantic composition of CIM-specific claims
///
/// A role answers: "What CIM operations can someone with this responsibility do?"
///
/// # Invariants
/// 1. Non-empty claim set (role must grant at least one permission)
/// 2. Valid claim domain coherence
/// 3. Temporal validity (if bounded)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimRole {
    /// Unique identifier (UUID v7)
    pub id: Uuid,

    /// Human-readable name
    pub name: String,

    /// Description of what this role enables
    pub description: String,

    /// Claims granted by this role
    pub claims: ClaimSet,

    /// Primary domain this role serves
    pub primary_domain: ClaimDomain,

    /// Lifecycle status
    pub status: CimRoleStatus,

    /// When role was created
    pub created_at: DateTime<Utc>,

    /// Who created this role
    pub created_by: Uuid,

    /// Optional temporal bounds
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,

    /// Version for optimistic concurrency
    pub version: u64,
}

/// Lifecycle status for CIM roles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CimRoleStatus {
    /// Role is active and usable
    Active,
    /// Role is deprecated, no new assignments
    Deprecated { reason: String, deprecated_at: DateTime<Utc> },
    /// Role is retired
    Retired { retired_at: DateTime<Utc> },
}

impl CimRole {
    /// Create a new active CIM role
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        claims: ClaimSet,
        primary_domain: ClaimDomain,
        created_by: Uuid,
    ) -> Result<Self, CimRoleError> {
        if claims.is_empty() {
            return Err(CimRoleError::EmptyClaims);
        }

        Ok(Self {
            id: Uuid::now_v7(),
            name: name.into(),
            description: description.into(),
            claims,
            primary_domain,
            status: CimRoleStatus::Active,
            created_at: Utc::now(),
            created_by,
            valid_from: None,
            valid_until: None,
            version: 1,
        })
    }

    /// Builder: add a claim
    pub fn with_claim(mut self, claim: CimClaim) -> Self {
        self.claims = self.claims.with(claim);
        self
    }

    /// Builder: set temporal bounds
    pub fn with_validity(
        mut self,
        from: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Self {
        self.valid_from = from;
        self.valid_until = until;
        self
    }

    /// Check if role is currently active (status and temporal)
    pub fn is_active(&self) -> bool {
        if !matches!(self.status, CimRoleStatus::Active) {
            return false;
        }

        let now = Utc::now();
        if let Some(from) = self.valid_from {
            if now < from {
                return false;
            }
        }
        if let Some(until) = self.valid_until {
            if now > until {
                return false;
            }
        }
        true
    }

    /// Get all claims as reference
    pub fn claims(&self) -> &ClaimSet {
        &self.claims
    }

    /// Check if role grants a specific claim (with implication)
    pub fn grants(&self, claim: &CimClaim) -> bool {
        self.claims.satisfies(claim)
    }

    /// Join with another role (union of claims)
    pub fn join(self, other: &CimRole) -> Self {
        let mut joined_claims = self.claims;
        joined_claims.join_mut(&other.claims);

        Self {
            id: self.id, // Keep original ID
            name: self.name,
            description: self.description,
            claims: joined_claims,
            primary_domain: self.primary_domain,
            status: self.status,
            created_at: self.created_at,
            created_by: self.created_by,
            valid_from: self.valid_from,
            valid_until: self.valid_until,
            version: self.version + 1,
        }
    }

    /// Deprecate the role
    pub fn deprecate(&mut self, reason: String) {
        self.status = CimRoleStatus::Deprecated {
            reason,
            deprecated_at: Utc::now(),
        };
        self.version += 1;
    }

    /// Retire the role
    pub fn retire(&mut self) {
        self.status = CimRoleStatus::Retired {
            retired_at: Utc::now(),
        };
        self.version += 1;
    }
}

impl Default for CimRole {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: String::new(),
            description: String::new(),
            claims: ClaimSet::new(),
            primary_domain: ClaimDomain::Nats,
            status: CimRoleStatus::Active,
            created_at: Utc::now(),
            created_by: Uuid::nil(),
            valid_from: None,
            valid_until: None,
            version: 1,
        }
    }
}

impl fmt::Display for CimRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({} claims, {:?})",
            self.name,
            self.claims.len(),
            self.primary_domain
        )
    }
}

// ============================================================================
// STANDARD CIM ROLES
// ============================================================================

/// Standard CIM roles for common use cases
pub struct StandardCimRoles;

impl StandardCimRoles {
    /// Full NATS operator administrator
    pub fn nats_operator_admin(created_by: Uuid) -> CimRole {
        use crate::policy::cim_claims::NatsClaim;

        let claims = ClaimSet::new()
            .with(CimClaim::Nats(NatsClaim::OperatorAdmin))
            .with(CimClaim::Nats(NatsClaim::AccountCreate))
            .with(CimClaim::Nats(NatsClaim::AccountAdmin));

        CimRole::new(
            "NATS Operator Administrator",
            "Full control over NATS operators and accounts",
            claims,
            ClaimDomain::Nats,
            created_by,
        )
        .expect("Standard role should be valid")
    }

    /// NATS account administrator (tenant-level)
    pub fn nats_account_admin(created_by: Uuid) -> CimRole {
        use crate::policy::cim_claims::NatsClaim;

        let claims = ClaimSet::new()
            .with(CimClaim::Nats(NatsClaim::AccountAdmin))
            .with(CimClaim::Nats(NatsClaim::UserCreate))
            .with(CimClaim::Nats(NatsClaim::UserAdmin))
            .with(CimClaim::Nats(NatsClaim::StreamAdmin))
            .with(CimClaim::Nats(NatsClaim::KvAdmin));

        CimRole::new(
            "NATS Account Administrator",
            "Manage users and resources within an account",
            claims,
            ClaimDomain::Nats,
            created_by,
        )
        .expect("Standard role should be valid")
    }

    /// PKI certificate authority operator
    pub fn pki_ca_operator(created_by: Uuid) -> CimRole {
        use crate::policy::cim_claims::PkiClaim;

        let claims = ClaimSet::new()
            .with(CimClaim::Pki(PkiClaim::IntermediateCaOperate))
            .with(CimClaim::Pki(PkiClaim::CertificateIssue))
            .with(CimClaim::Pki(PkiClaim::CertificateRenew))
            .with(CimClaim::Pki(PkiClaim::CertificateRevoke))
            .with(CimClaim::Pki(PkiClaim::CrlPublish));

        CimRole::new(
            "PKI CA Operator",
            "Issue and manage certificates under intermediate CA",
            claims,
            ClaimDomain::Pki,
            created_by,
        )
        .expect("Standard role should be valid")
    }

    /// Root CA operator (highest PKI privilege)
    pub fn pki_root_ca_operator(created_by: Uuid) -> CimRole {
        use crate::policy::cim_claims::PkiClaim;

        let claims = ClaimSet::new()
            .with(CimClaim::Pki(PkiClaim::RootCaOperate))
            .with(CimClaim::Pki(PkiClaim::IntermediateCaOperate))
            .with(CimClaim::Pki(PkiClaim::CertificateIssue));

        CimRole::new(
            "PKI Root CA Operator",
            "Operate root CA and create intermediate CAs",
            claims,
            ClaimDomain::Pki,
            created_by,
        )
        .expect("Standard role should be valid")
    }

    /// Key custodian - generates and manages keys
    pub fn key_custodian(created_by: Uuid) -> CimRole {
        use crate::policy::cim_claims::PkiClaim;

        let claims = ClaimSet::new()
            .with(CimClaim::Pki(PkiClaim::KeyGenerate))
            .with(CimClaim::Pki(PkiClaim::KeyRead))
            .with(CimClaim::Pki(PkiClaim::KeyExportPublic))
            .with(CimClaim::Pki(PkiClaim::CertificateRequest));

        CimRole::new(
            "Key Custodian",
            "Generate keys and request certificates",
            claims,
            ClaimDomain::Pki,
            created_by,
        )
        .expect("Standard role should be valid")
    }

    /// YubiKey provisioner
    pub fn yubikey_provisioner(created_by: Uuid) -> CimRole {
        use crate::policy::cim_claims::YubiKeyClaim;

        let claims = ClaimSet::new()
            .with(CimClaim::YubiKey(YubiKeyClaim::DeviceProvision))
            .with(CimClaim::YubiKey(YubiKeyClaim::DeviceAssign))
            .with(CimClaim::YubiKey(YubiKeyClaim::SlotAuthentication))
            .with(CimClaim::YubiKey(YubiKeyClaim::SlotDigitalSignature))
            .with(CimClaim::YubiKey(YubiKeyClaim::SlotKeyManagement))
            .with(CimClaim::YubiKey(YubiKeyClaim::AttestationGenerate));

        CimRole::new(
            "YubiKey Provisioner",
            "Provision and configure YubiKey devices",
            claims,
            ClaimDomain::YubiKey,
            created_by,
        )
        .expect("Standard role should be valid")
    }

    /// Trust administrator - manages delegation chains
    pub fn trust_admin(created_by: Uuid) -> CimRole {
        use crate::policy::cim_claims::TrustClaim;

        let claims = ClaimSet::new()
            .with(CimClaim::Trust(TrustClaim::DelegationGrant))
            .with(CimClaim::Trust(TrustClaim::DelegationRevoke))
            .with(CimClaim::Trust(TrustClaim::DelegationRead))
            .with(CimClaim::Trust(TrustClaim::TrustAnchorCreate))
            .with(CimClaim::Trust(TrustClaim::TrustAnchorModify));

        CimRole::new(
            "Trust Administrator",
            "Manage trust anchors and delegation chains",
            claims,
            ClaimDomain::Trust,
            created_by,
        )
        .expect("Standard role should be valid")
    }

    /// Read-only auditor
    pub fn auditor(created_by: Uuid) -> CimRole {
        use crate::policy::cim_claims::{NatsClaim, PkiClaim, YubiKeyClaim, TrustClaim};

        let claims = ClaimSet::new()
            .with(CimClaim::Nats(NatsClaim::OperatorRead))
            .with(CimClaim::Nats(NatsClaim::AccountRead))
            .with(CimClaim::Nats(NatsClaim::UserRead))
            .with(CimClaim::Pki(PkiClaim::CaRead))
            .with(CimClaim::Pki(PkiClaim::CertificateRead))
            .with(CimClaim::Pki(PkiClaim::KeyRead))
            .with(CimClaim::YubiKey(YubiKeyClaim::DeviceRead))
            .with(CimClaim::Trust(TrustClaim::DelegationRead))
            .with(CimClaim::Trust(TrustClaim::TrustAnchorRead));

        CimRole::new(
            "CIM Auditor",
            "Read-only access to all CIM domains for audit purposes",
            claims,
            ClaimDomain::Trust, // Cross-domain, but primarily for trust verification
            created_by,
        )
        .expect("Standard role should be valid")
    }
}

// ============================================================================
// ERROR TYPE
// ============================================================================

/// Errors that can occur in CIM role operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CimRoleError {
    /// Role has no claims
    EmptyClaims,
    /// Role is not active
    RoleNotActive,
    /// Invalid temporal bounds
    InvalidTemporalBounds,
}

impl fmt::Display for CimRoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CimRoleError::EmptyClaims => write!(f, "CIM role must have at least one claim"),
            CimRoleError::RoleNotActive => write!(f, "CIM role is not active"),
            CimRoleError::InvalidTemporalBounds => {
                write!(f, "valid_from must be before valid_until")
            }
        }
    }
}

impl std::error::Error for CimRoleError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::cim_claims::NatsClaim;

    #[test]
    fn test_cim_role_creation() {
        let claims = ClaimSet::singleton(CimClaim::Nats(NatsClaim::OperatorRead));
        let role = CimRole::new(
            "Test Role",
            "A test role",
            claims,
            ClaimDomain::Nats,
            Uuid::now_v7(),
        );

        assert!(role.is_ok());
        let role = role.unwrap();
        assert_eq!(role.name, "Test Role");
        assert!(role.is_active());
    }

    #[test]
    fn test_empty_role_fails() {
        let result = CimRole::new(
            "Empty",
            "No claims",
            ClaimSet::new(),
            ClaimDomain::Nats,
            Uuid::now_v7(),
        );

        assert!(matches!(result, Err(CimRoleError::EmptyClaims)));
    }

    #[test]
    fn test_role_grants_with_implication() {
        let claims = ClaimSet::singleton(CimClaim::Nats(NatsClaim::OperatorAdmin));
        let role = CimRole::new(
            "Admin",
            "Admin role",
            claims,
            ClaimDomain::Nats,
            Uuid::now_v7(),
        )
        .unwrap();

        // Admin implies Read
        assert!(role.grants(&CimClaim::Nats(NatsClaim::OperatorAdmin)));
        assert!(role.grants(&CimClaim::Nats(NatsClaim::OperatorRead)));
    }

    #[test]
    fn test_role_join() {
        let creator = Uuid::now_v7();
        let role_a = CimRole::new(
            "A",
            "Role A",
            ClaimSet::singleton(CimClaim::Nats(NatsClaim::OperatorRead)),
            ClaimDomain::Nats,
            creator,
        )
        .unwrap();

        let role_b = CimRole::new(
            "B",
            "Role B",
            ClaimSet::singleton(CimClaim::Nats(NatsClaim::AccountRead)),
            ClaimDomain::Nats,
            creator,
        )
        .unwrap();

        let joined = role_a.join(&role_b);
        assert!(joined.grants(&CimClaim::Nats(NatsClaim::OperatorRead)));
        assert!(joined.grants(&CimClaim::Nats(NatsClaim::AccountRead)));
    }

    #[test]
    fn test_standard_roles() {
        let creator = Uuid::now_v7();

        let nats_admin = StandardCimRoles::nats_operator_admin(creator);
        assert!(nats_admin.grants(&CimClaim::Nats(NatsClaim::OperatorAdmin)));

        let pki_ca = StandardCimRoles::pki_ca_operator(creator);
        assert!(pki_ca.grants(&CimClaim::Pki(crate::policy::cim_claims::PkiClaim::CertificateIssue)));

        let auditor = StandardCimRoles::auditor(creator);
        assert!(auditor.grants(&CimClaim::Nats(NatsClaim::OperatorRead)));
        assert!(!auditor.grants(&CimClaim::Nats(NatsClaim::OperatorAdmin)));
    }
}
