// Copyright (c) 2025 - Cowboy AI, LLC.

//! CIM-Specific Claims - Atomic permission units following RFC 7519
//!
//! Claims are the atomic building blocks of CIM authorization.
//! Unlike generic RBAC, CIM claims are specifically for:
//! - NATS infrastructure (Operator/Account/User management)
//! - PKI operations (Key/Certificate lifecycle)
//! - YubiKey hardware (Device provisioning, slots, attestation)
//! - Trust delegation (Chain of custody, delegation depth)
//!
//! # Mathematical Structure
//!
//! Claims form a bounded join-semilattice (L, ∨, ⊥):
//! - L = set of all claims
//! - ∨ = join (combines claims additively)
//! - ⊥ = empty claim set (identity element)
//!
//! Properties:
//! - Associativity: (a ∨ b) ∨ c = a ∨ (b ∨ c)
//! - Commutativity: a ∨ b = b ∨ a
//! - Idempotence: a ∨ a = a
//! - Identity: a ∨ ⊥ = a

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;

// ============================================================================
// CLAIM DOMAIN - The four CIM-specific claim categories
// ============================================================================

/// CIM Claim - Atomic permission unit
///
/// Each variant represents a distinct domain of CIM operations.
/// Claims compose into Roles and are constrained by Policies.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CimClaim {
    /// NATS infrastructure claims
    Nats(NatsClaim),
    /// PKI operation claims
    Pki(PkiClaim),
    /// YubiKey hardware claims
    YubiKey(YubiKeyClaim),
    /// Trust delegation claims
    Trust(TrustClaim),
}

// ============================================================================
// NATS CLAIMS - Infrastructure authorization
// ============================================================================

/// NATS Infrastructure Claims
///
/// Maps to NATS JWT hierarchy: Operator → Account → User
/// Follows nsc tool permission model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NatsClaim {
    // --- Operator-level claims (system administration) ---
    /// Create/modify operators (highest privilege)
    OperatorAdmin,
    /// View operator configuration
    OperatorRead,
    /// Sign operator JWTs
    OperatorSign,

    // --- Account-level claims (tenant management) ---
    /// Create accounts under an operator
    AccountCreate,
    /// Modify account limits and permissions
    AccountAdmin,
    /// View account configuration
    AccountRead,
    /// Sign account JWTs
    AccountSign,

    // --- User-level claims (identity management) ---
    /// Create users within an account
    UserCreate,
    /// Modify user permissions
    UserAdmin,
    /// View user configuration
    UserRead,
    /// Sign user JWTs
    UserSign,

    // --- JetStream claims (persistence) ---
    /// Create/manage streams
    StreamAdmin,
    /// Create/manage consumers
    ConsumerAdmin,
    /// Read from streams
    StreamRead,
    /// Write to streams
    StreamWrite,

    // --- KV/Object Store claims ---
    /// Manage KV buckets
    KvAdmin,
    /// Read from KV
    KvRead,
    /// Write to KV
    KvWrite,
    /// Manage object stores
    ObjectStoreAdmin,
}

// ============================================================================
// PKI CLAIMS - Certificate lifecycle authorization
// ============================================================================

/// PKI Operation Claims
///
/// Authorization for certificate and key lifecycle operations.
/// Aligns with X.509 PKI hierarchy and PKCS#11 operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PkiClaim {
    // --- Certificate Authority claims ---
    /// Operate as root CA (extremely restricted)
    RootCaOperate,
    /// Operate intermediate CA
    IntermediateCaOperate,
    /// View CA configuration
    CaRead,

    // --- Certificate lifecycle claims ---
    /// Request certificate issuance
    CertificateRequest,
    /// Issue certificates (requires CA claim)
    CertificateIssue,
    /// Renew certificates
    CertificateRenew,
    /// Revoke certificates
    CertificateRevoke,
    /// View certificate details
    CertificateRead,

    // --- Key management claims ---
    /// Generate new key pairs
    KeyGenerate,
    /// Export public keys
    KeyExportPublic,
    /// Import keys (requires attestation)
    KeyImport,
    /// Destroy keys
    KeyDestroy,
    /// View key metadata
    KeyRead,

    // --- Chain verification claims ---
    /// Verify certificate chains
    ChainVerify,
    /// Validate certificate policies
    PolicyValidate,
    /// Check revocation status (CRL/OCSP)
    RevocationCheck,

    // --- CRL/OCSP claims ---
    /// Publish Certificate Revocation Lists
    CrlPublish,
    /// Respond to OCSP requests
    OcspRespond,
}

// ============================================================================
// YUBIKEY CLAIMS - Hardware security authorization
// ============================================================================

/// YubiKey Hardware Claims
///
/// Authorization for hardware security module operations.
/// Follows PIV (FIPS 201) slot model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum YubiKeyClaim {
    // --- Device lifecycle claims ---
    /// Provision new YubiKeys
    DeviceProvision,
    /// Reset YubiKey to factory state
    DeviceReset,
    /// View device inventory
    DeviceRead,
    /// Assign device to person
    DeviceAssign,
    /// Unassign device from person
    DeviceUnassign,

    // --- PIV slot claims (FIPS 201) ---
    /// Manage authentication slot (9A)
    SlotAuthentication,
    /// Manage digital signature slot (9C)
    SlotDigitalSignature,
    /// Manage key management slot (9D)
    SlotKeyManagement,
    /// Manage card authentication slot (9E)
    SlotCardAuth,
    /// Manage retired key slots (82-95)
    SlotRetired,

    // --- Attestation claims ---
    /// Generate attestation certificates
    AttestationGenerate,
    /// Verify attestation certificates
    AttestationVerify,

    // --- PIN management claims ---
    /// Set/change PIN
    PinManage,
    /// Set/change PUK
    PukManage,
    /// Set/change management key
    ManagementKeyManage,
}

// ============================================================================
// TRUST CLAIMS - Delegation and chain of custody
// ============================================================================

/// Trust Delegation Claims
///
/// Authorization for trust chain and delegation operations.
/// Supports cryptographic delegation with depth constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustClaim {
    // --- Delegation claims ---
    /// Delegate own claims to others
    DelegationGrant,
    /// Receive delegated claims
    DelegationReceive,
    /// Revoke delegations granted
    DelegationRevoke,
    /// View delegation chains
    DelegationRead,

    // --- Trust anchor claims ---
    /// Establish trust anchors
    TrustAnchorCreate,
    /// Modify trust anchors
    TrustAnchorModify,
    /// Remove trust anchors
    TrustAnchorRemove,
    /// View trust anchors
    TrustAnchorRead,

    // --- Scope claims (constrain where claims apply) ---
    /// Operate within organizational scope
    ScopeOrganization { org_id: Uuid },
    /// Operate within unit scope
    ScopeUnit { unit_id: Uuid },
    /// Operate within location scope
    ScopeLocation { location_id: Uuid },
}

// ============================================================================
// CLAIM SET - Semilattice operations
// ============================================================================

/// A set of claims forming a join-semilattice
///
/// ClaimSet is the free semilattice over CimClaim.
/// Operations: join (∨), is_empty (⊥ check), satisfies (≤ check)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimSet {
    claims: HashSet<CimClaim>,
}

impl ClaimSet {
    /// Create empty claim set (⊥ in semilattice)
    pub fn new() -> Self {
        Self {
            claims: HashSet::new(),
        }
    }

    /// Create from a single claim
    pub fn singleton(claim: CimClaim) -> Self {
        let mut claims = HashSet::new();
        claims.insert(claim);
        Self { claims }
    }

    /// Add a claim to the set (builder pattern)
    pub fn with(mut self, claim: CimClaim) -> Self {
        self.claims.insert(claim);
        self
    }

    /// Join two claim sets (∨ operation in semilattice)
    pub fn join(mut self, other: Self) -> Self {
        self.claims.extend(other.claims);
        self
    }

    /// Join with another set (mutating version)
    pub fn join_mut(&mut self, other: &Self) {
        self.claims.extend(other.claims.iter().cloned());
    }

    /// Check if empty (⊥)
    pub fn is_empty(&self) -> bool {
        self.claims.is_empty()
    }

    /// Number of claims
    pub fn len(&self) -> usize {
        self.claims.len()
    }

    /// Check if this set contains a specific claim
    pub fn contains(&self, claim: &CimClaim) -> bool {
        self.claims.contains(claim)
    }

    /// Check if this set satisfies a required claim (with implication)
    ///
    /// A claim set satisfies a requirement if it contains the claim
    /// or contains a claim that implies it (e.g., Admin implies Read).
    pub fn satisfies(&self, required: &CimClaim) -> bool {
        self.contains(required) || self.claims.iter().any(|c| c.implies(required))
    }

    /// Check if this set satisfies all required claims
    pub fn satisfies_all(&self, required: &ClaimSet) -> bool {
        required.claims.iter().all(|r| self.satisfies(r))
    }

    /// Get all claims as iterator
    pub fn iter(&self) -> impl Iterator<Item = &CimClaim> {
        self.claims.iter()
    }

    /// Get claims by domain
    pub fn nats_claims(&self) -> impl Iterator<Item = &NatsClaim> {
        self.claims.iter().filter_map(|c| match c {
            CimClaim::Nats(n) => Some(n),
            _ => None,
        })
    }

    pub fn pki_claims(&self) -> impl Iterator<Item = &PkiClaim> {
        self.claims.iter().filter_map(|c| match c {
            CimClaim::Pki(p) => Some(p),
            _ => None,
        })
    }

    pub fn yubikey_claims(&self) -> impl Iterator<Item = &YubiKeyClaim> {
        self.claims.iter().filter_map(|c| match c {
            CimClaim::YubiKey(y) => Some(y),
            _ => None,
        })
    }

    pub fn trust_claims(&self) -> impl Iterator<Item = &TrustClaim> {
        self.claims.iter().filter_map(|c| match c {
            CimClaim::Trust(t) => Some(t),
            _ => None,
        })
    }
}

// ============================================================================
// CLAIM IMPLICATION - Partial order on claims
// ============================================================================

impl CimClaim {
    /// Check if this claim implies another (partial order ≤)
    ///
    /// A claim `a` implies claim `b` if having `a` grants all permissions of `b`.
    /// Example: OperatorAdmin implies OperatorRead
    pub fn implies(&self, other: &Self) -> bool {
        use CimClaim::*;
        use NatsClaim::*;
        use PkiClaim::*;
        use YubiKeyClaim::*;

        match (self, other) {
            // Same claim implies itself
            (a, b) if a == b => true,

            // NATS implications
            (Nats(OperatorAdmin), Nats(OperatorRead | OperatorSign)) => true,
            (Nats(AccountAdmin), Nats(AccountRead | AccountSign)) => true,
            (Nats(UserAdmin), Nats(UserRead | UserSign)) => true,
            (Nats(StreamAdmin), Nats(StreamRead | StreamWrite)) => true,
            (Nats(KvAdmin), Nats(KvRead | KvWrite)) => true,

            // PKI implications
            (Pki(RootCaOperate), Pki(IntermediateCaOperate | CaRead)) => true,
            (Pki(IntermediateCaOperate), Pki(CaRead)) => true,
            (Pki(CertificateIssue), Pki(CertificateRead)) => true,

            // YubiKey implications
            (YubiKey(DeviceProvision), YubiKey(DeviceRead | DeviceAssign)) => true,

            _ => false,
        }
    }

    /// Get the domain category of this claim
    pub fn domain(&self) -> ClaimDomain {
        match self {
            CimClaim::Nats(_) => ClaimDomain::Nats,
            CimClaim::Pki(_) => ClaimDomain::Pki,
            CimClaim::YubiKey(_) => ClaimDomain::YubiKey,
            CimClaim::Trust(_) => ClaimDomain::Trust,
        }
    }

    /// Get URI representation (RFC 7519 style)
    pub fn uri(&self) -> String {
        match self {
            CimClaim::Nats(n) => format!("urn:cim:nats:{}", n.name()),
            CimClaim::Pki(p) => format!("urn:cim:pki:{}", p.name()),
            CimClaim::YubiKey(y) => format!("urn:cim:yubikey:{}", y.name()),
            CimClaim::Trust(t) => format!("urn:cim:trust:{}", t.name()),
        }
    }
}

/// Claim domain categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimDomain {
    Nats,
    Pki,
    YubiKey,
    Trust,
}

impl fmt::Display for ClaimDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaimDomain::Nats => write!(f, "NATS Infrastructure"),
            ClaimDomain::Pki => write!(f, "PKI Operations"),
            ClaimDomain::YubiKey => write!(f, "YubiKey Hardware"),
            ClaimDomain::Trust => write!(f, "Trust Delegation"),
        }
    }
}

// ============================================================================
// NAME HELPERS
// ============================================================================

impl NatsClaim {
    pub fn name(&self) -> &'static str {
        match self {
            NatsClaim::OperatorAdmin => "operator:admin",
            NatsClaim::OperatorRead => "operator:read",
            NatsClaim::OperatorSign => "operator:sign",
            NatsClaim::AccountCreate => "account:create",
            NatsClaim::AccountAdmin => "account:admin",
            NatsClaim::AccountRead => "account:read",
            NatsClaim::AccountSign => "account:sign",
            NatsClaim::UserCreate => "user:create",
            NatsClaim::UserAdmin => "user:admin",
            NatsClaim::UserRead => "user:read",
            NatsClaim::UserSign => "user:sign",
            NatsClaim::StreamAdmin => "stream:admin",
            NatsClaim::ConsumerAdmin => "consumer:admin",
            NatsClaim::StreamRead => "stream:read",
            NatsClaim::StreamWrite => "stream:write",
            NatsClaim::KvAdmin => "kv:admin",
            NatsClaim::KvRead => "kv:read",
            NatsClaim::KvWrite => "kv:write",
            NatsClaim::ObjectStoreAdmin => "object:admin",
        }
    }
}

impl PkiClaim {
    pub fn name(&self) -> &'static str {
        match self {
            PkiClaim::RootCaOperate => "ca:root:operate",
            PkiClaim::IntermediateCaOperate => "ca:intermediate:operate",
            PkiClaim::CaRead => "ca:read",
            PkiClaim::CertificateRequest => "certificate:request",
            PkiClaim::CertificateIssue => "certificate:issue",
            PkiClaim::CertificateRenew => "certificate:renew",
            PkiClaim::CertificateRevoke => "certificate:revoke",
            PkiClaim::CertificateRead => "certificate:read",
            PkiClaim::KeyGenerate => "key:generate",
            PkiClaim::KeyExportPublic => "key:export:public",
            PkiClaim::KeyImport => "key:import",
            PkiClaim::KeyDestroy => "key:destroy",
            PkiClaim::KeyRead => "key:read",
            PkiClaim::ChainVerify => "chain:verify",
            PkiClaim::PolicyValidate => "policy:validate",
            PkiClaim::RevocationCheck => "revocation:check",
            PkiClaim::CrlPublish => "crl:publish",
            PkiClaim::OcspRespond => "ocsp:respond",
        }
    }
}

impl YubiKeyClaim {
    pub fn name(&self) -> &'static str {
        match self {
            YubiKeyClaim::DeviceProvision => "device:provision",
            YubiKeyClaim::DeviceReset => "device:reset",
            YubiKeyClaim::DeviceRead => "device:read",
            YubiKeyClaim::DeviceAssign => "device:assign",
            YubiKeyClaim::DeviceUnassign => "device:unassign",
            YubiKeyClaim::SlotAuthentication => "slot:9a",
            YubiKeyClaim::SlotDigitalSignature => "slot:9c",
            YubiKeyClaim::SlotKeyManagement => "slot:9d",
            YubiKeyClaim::SlotCardAuth => "slot:9e",
            YubiKeyClaim::SlotRetired => "slot:retired",
            YubiKeyClaim::AttestationGenerate => "attestation:generate",
            YubiKeyClaim::AttestationVerify => "attestation:verify",
            YubiKeyClaim::PinManage => "pin:manage",
            YubiKeyClaim::PukManage => "puk:manage",
            YubiKeyClaim::ManagementKeyManage => "mgmt-key:manage",
        }
    }
}

impl TrustClaim {
    pub fn name(&self) -> String {
        match self {
            TrustClaim::DelegationGrant => "delegation:grant".to_string(),
            TrustClaim::DelegationReceive => "delegation:receive".to_string(),
            TrustClaim::DelegationRevoke => "delegation:revoke".to_string(),
            TrustClaim::DelegationRead => "delegation:read".to_string(),
            TrustClaim::TrustAnchorCreate => "anchor:create".to_string(),
            TrustClaim::TrustAnchorModify => "anchor:modify".to_string(),
            TrustClaim::TrustAnchorRemove => "anchor:remove".to_string(),
            TrustClaim::TrustAnchorRead => "anchor:read".to_string(),
            TrustClaim::ScopeOrganization { org_id } => format!("scope:org:{}", org_id),
            TrustClaim::ScopeUnit { unit_id } => format!("scope:unit:{}", unit_id),
            TrustClaim::ScopeLocation { location_id } => format!("scope:location:{}", location_id),
        }
    }
}

// ============================================================================
// DISPLAY IMPLEMENTATIONS
// ============================================================================

impl fmt::Display for CimClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CimClaim::Nats(n) => write!(f, "NATS: {}", n),
            CimClaim::Pki(p) => write!(f, "PKI: {}", p),
            CimClaim::YubiKey(y) => write!(f, "YubiKey: {}", y),
            CimClaim::Trust(t) => write!(f, "Trust: {}", t),
        }
    }
}

impl fmt::Display for NatsClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for PkiClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for YubiKeyClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for TrustClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_set_semilattice_identity() {
        let set = ClaimSet::new();
        let claim = CimClaim::Nats(NatsClaim::OperatorRead);

        // a ∨ ⊥ = a
        let joined = ClaimSet::singleton(claim.clone()).join(ClaimSet::new());
        assert!(joined.contains(&claim));
        assert_eq!(joined.len(), 1);
    }

    #[test]
    fn test_claim_set_semilattice_idempotence() {
        let claim = CimClaim::Nats(NatsClaim::OperatorRead);

        // a ∨ a = a
        let set = ClaimSet::singleton(claim.clone()).with(claim.clone());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_claim_implication() {
        let admin = CimClaim::Nats(NatsClaim::OperatorAdmin);
        let read = CimClaim::Nats(NatsClaim::OperatorRead);

        assert!(admin.implies(&read));
        assert!(!read.implies(&admin));
    }

    #[test]
    fn test_claim_set_satisfies_with_implication() {
        let set = ClaimSet::singleton(CimClaim::Nats(NatsClaim::OperatorAdmin));

        // Admin implies Read, so set should satisfy Read
        assert!(set.satisfies(&CimClaim::Nats(NatsClaim::OperatorRead)));
        assert!(set.satisfies(&CimClaim::Nats(NatsClaim::OperatorSign)));
    }

    #[test]
    fn test_claim_uri() {
        let claim = CimClaim::Nats(NatsClaim::OperatorAdmin);
        assert_eq!(claim.uri(), "urn:cim:nats:operator:admin");

        let pki_claim = CimClaim::Pki(PkiClaim::CertificateIssue);
        assert_eq!(pki_claim.uri(), "urn:cim:pki:certificate:issue");
    }

    #[test]
    fn test_claim_domains() {
        assert_eq!(
            CimClaim::Nats(NatsClaim::OperatorAdmin).domain(),
            ClaimDomain::Nats
        );
        assert_eq!(
            CimClaim::Pki(PkiClaim::KeyGenerate).domain(),
            ClaimDomain::Pki
        );
        assert_eq!(
            CimClaim::YubiKey(YubiKeyClaim::DeviceProvision).domain(),
            ClaimDomain::YubiKey
        );
    }
}
