// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Node Coproduct - Categorical type for graph nodes
//!
//! This module implements a proper coproduct of domain types following
//! Applied Category Theory principles. The design uses:
//!
//! 1. **Injection functions**: `inject_person()`, `inject_organization()`, etc.
//! 2. **Universal property**: `FoldDomainNode` trait with `fold()` method
//! 3. **Type-safe projections**: Each injection preserves identity
//!
//! ## Related Modules
//!
//! For new code, consider using the per-context coproducts in `crate::domains::`:
//! - `organization` - Organization, Person, Location, Role, Policy entities
//! - `pki` - Certificates and Keys
//! - `nats` - Operators, Accounts, Users
//! - `yubikey` - YubiKey devices and slots
//! - `typography` - Verified themes and labelled elements (solves tofu problem)
//!
//! The `LiftableDomain` trait in `crate::lifting` provides domain → graph
//! transformations that work with the per-context entities.
//!
//! ## Categorical Foundation
//!
//! A coproduct A + B + C + ... has:
//! - Injection morphisms: ι_A: A → Sum, ι_B: B → Sum, ...
//! - Universal property: For any X with morphisms f_A: A → X, f_B: B → X, ...
//!   there exists a unique [f_A, f_B, ...]: Sum → X
//!   such that [f_A, f_B, ...] ∘ ι_A = f_A (and similarly for B, C, ...)
//!
//! The `FoldDomainNode` trait captures this universal property.

use std::fmt;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Core domain types from bootstrap
use crate::domain::{
    Person, Organization, OrganizationUnit, Location, Policy, Role, KeyOwnerRole, PolicyClaim,
};

// Bounded context types with phantom-typed IDs
use crate::domain::pki::{Certificate, CryptographicKey, CertificateId, KeyId, KeyAlgorithm, KeyPurpose};
use crate::domain::yubikey::{YubiKeyDevice, PivSlotView, YubiKeyStatus, YubiKeyDeviceId, SlotId, PIVSlot};
use crate::domain::visualization::{
    Manifest, PolicyRole, PolicyClaimView, PolicyCategory, PolicyGroup,
    ManifestId, PolicyRoleId, ClaimId, PolicyCategoryId, PolicyGroupId,
};

use crate::domain_projections::NatsIdentityProjection;
use crate::policy::SeparationClass;

/// Injection tag - identifies which domain type was injected into the coproduct.
///
/// This is crucial for the categorical structure: injections are morphisms from
/// individual domain categories into the coproduct category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Injection {
    // Core Domain Entities
    Person,
    Organization,
    OrganizationUnit,
    Location,
    Role,
    Policy,

    // NATS Infrastructure (with projections)
    NatsOperator,
    NatsAccount,
    NatsUser,
    NatsServiceAccount,

    // NATS Infrastructure (simple visualization)
    NatsOperatorSimple,
    NatsAccountSimple,
    NatsUserSimple,

    // PKI Certificates
    RootCertificate,
    IntermediateCertificate,
    LeafCertificate,

    // Cryptographic Keys
    Key,

    // YubiKey Hardware
    YubiKey,
    PivSlot,
    YubiKeyStatus,

    // Export Manifest
    Manifest,

    // Policy Roles and Claims
    PolicyRole,
    PolicyClaim,
    PolicyCategory,
    PolicyGroup,

    // Aggregate Roots (DDD bounded contexts)
    AggregateOrganization,
    AggregatePkiChain,
    AggregateNatsSecurity,
    AggregateYubiKeyProvisioning,
}

impl Injection {
    /// Get the display name for this injection type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Person => "Person",
            Self::Organization => "Organization",
            Self::OrganizationUnit => "Organizational Unit",
            Self::Location => "Location",
            Self::Role => "Role",
            Self::Policy => "Policy",
            Self::NatsOperator => "NATS Operator",
            Self::NatsAccount => "NATS Account",
            Self::NatsUser => "NATS User",
            Self::NatsServiceAccount => "NATS Service Account",
            Self::NatsOperatorSimple => "NATS Operator",
            Self::NatsAccountSimple => "NATS Account",
            Self::NatsUserSimple => "NATS User",
            Self::RootCertificate => "Root Certificate",
            Self::IntermediateCertificate => "Intermediate Certificate",
            Self::LeafCertificate => "Leaf Certificate",
            Self::Key => "Key",
            Self::YubiKey => "YubiKey",
            Self::PivSlot => "PIV Slot",
            Self::YubiKeyStatus => "YubiKey Status",
            Self::Manifest => "Manifest",
            Self::PolicyRole => "Policy Role",
            Self::PolicyClaim => "Policy Claim",
            Self::PolicyCategory => "Policy Category",
            Self::PolicyGroup => "Separation Class",
            Self::AggregateOrganization => "Organization Aggregate",
            Self::AggregatePkiChain => "PKI Chain Aggregate",
            Self::AggregateNatsSecurity => "NATS Security Aggregate",
            Self::AggregateYubiKeyProvisioning => "YubiKey Provisioning Aggregate",
        }
    }

    /// Get the layout tier for hierarchical positioning
    ///
    /// Tiers are used for visual hierarchy in graph layouts:
    /// - Tier 0: Root entities (Organization, NATS Operator, Root CA, YubiKey, PolicyGroup)
    /// - Tier 1: Intermediate entities (OrgUnit, NATS Account, Intermediate CA, Role, Policy, PivSlot, PolicyRole, PolicyCategory)
    /// - Tier 2: Leaf entities (Person, Location, NATS User, Leaf Cert, Key, Manifest, PolicyClaim)
    pub fn layout_tier(&self) -> u8 {
        match self {
            // Tier 0: Root entities and Aggregates
            Self::Organization => 0,
            Self::NatsOperator | Self::NatsOperatorSimple => 0,
            Self::RootCertificate => 0,
            Self::YubiKey => 0,
            Self::YubiKeyStatus => 0,
            Self::PolicyGroup => 0,
            Self::AggregateOrganization |
            Self::AggregatePkiChain |
            Self::AggregateNatsSecurity |
            Self::AggregateYubiKeyProvisioning => 0,

            // Tier 1: Intermediate entities
            Self::OrganizationUnit => 1,
            Self::Role => 1,
            Self::Policy => 1,
            Self::NatsAccount | Self::NatsAccountSimple => 1,
            Self::IntermediateCertificate => 1,
            Self::PivSlot => 1,
            Self::PolicyRole => 1,
            Self::PolicyCategory => 1,

            // Tier 2: Leaf entities
            Self::Person => 2,
            Self::Location => 2,
            Self::NatsUser | Self::NatsUserSimple | Self::NatsServiceAccount => 2,
            Self::LeafCertificate => 2,
            Self::Key => 2,
            Self::Manifest => 2,
            Self::PolicyClaim => 2,
        }
    }

    /// Check if this injection type is a NATS infrastructure node
    pub fn is_nats(&self) -> bool {
        matches!(
            self,
            Self::NatsOperator | Self::NatsOperatorSimple |
            Self::NatsAccount | Self::NatsAccountSimple |
            Self::NatsUser | Self::NatsUserSimple |
            Self::NatsServiceAccount
        )
    }

    /// Check if this injection type is a PKI certificate node
    pub fn is_certificate(&self) -> bool {
        matches!(
            self,
            Self::RootCertificate | Self::IntermediateCertificate | Self::LeafCertificate
        )
    }

    /// Check if this injection type is a YubiKey-related node
    pub fn is_yubikey(&self) -> bool {
        matches!(self, Self::YubiKey | Self::PivSlot | Self::YubiKeyStatus)
    }

    /// Check if this injection type is a policy-related node
    pub fn is_policy(&self) -> bool {
        matches!(
            self,
            Self::Policy | Self::PolicyRole | Self::PolicyClaim |
            Self::PolicyCategory | Self::PolicyGroup
        )
    }

    /// Get the list of injection types that can be manually created from the UI.
    ///
    /// This replaces the legacy OrganizationalNodeType::all() method.
    /// Returns only the core organizational entities that users typically create.
    pub fn creatable() -> Vec<Self> {
        vec![
            Self::Organization,
            Self::OrganizationUnit,
            Self::Person,
            Self::Location,
            Self::Role,
            Self::Policy,
        ]
    }

    /// Check if this injection type can be manually created from the UI
    pub fn is_creatable(&self) -> bool {
        matches!(
            self,
            Self::Organization |
            Self::OrganizationUnit |
            Self::Person |
            Self::Location |
            Self::Role |
            Self::Policy
        )
    }
}

impl fmt::Display for Injection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// Domain Node Inner Data
// ============================================================================

/// Inner data for each domain node type
///
/// This enum uses types from bounded context modules for DDD compliance:
/// - Organization context: Person, Organization, OrganizationUnit, Location, Role, Policy
/// - PKI context: Certificate, CryptographicKey (with phantom-typed IDs)
/// - YubiKey context: YubiKeyDevice, PivSlotView, YubiKeyStatus
/// - Visualization context: Manifest, PolicyRole, PolicyClaim, etc.
#[derive(Debug, Clone)]
pub enum DomainNodeData {
    // =========================================================================
    // ORGANIZATION BOUNDED CONTEXT
    // =========================================================================
    Person { person: Person, role: KeyOwnerRole },
    Organization(Organization),
    OrganizationUnit(OrganizationUnit),
    Location(Location),
    Role(Role),
    Policy(Policy),

    // =========================================================================
    // NATS BOUNDED CONTEXT
    // =========================================================================
    // With full projections
    NatsOperator(NatsIdentityProjection),
    NatsAccount(NatsIdentityProjection),
    NatsUser(NatsIdentityProjection),
    NatsServiceAccount(NatsIdentityProjection),

    // Simple visualization (without full projections)
    NatsOperatorSimple {
        name: String,
        organization_id: Option<Uuid>,
    },
    NatsAccountSimple {
        name: String,
        unit_id: Option<Uuid>,
        is_system: bool,
    },
    NatsUserSimple {
        name: String,
        person_id: Option<Uuid>,
        account_name: String,
    },

    // =========================================================================
    // PKI BOUNDED CONTEXT (with phantom-typed IDs)
    // =========================================================================
    /// Root CA certificate (uses CertificateId)
    RootCertificate(Certificate),
    /// Intermediate CA certificate (uses CertificateId)
    IntermediateCertificate(Certificate),
    /// Leaf/end-entity certificate (uses CertificateId)
    LeafCertificate(Certificate),
    /// Cryptographic key (uses KeyId)
    Key(CryptographicKey),

    // =========================================================================
    // YUBIKEY BOUNDED CONTEXT (with phantom-typed IDs)
    // =========================================================================
    /// YubiKey hardware device (uses YubiKeyDeviceId)
    YubiKey(YubiKeyDevice),
    /// PIV slot on a YubiKey (uses SlotId)
    PivSlot(PivSlotView),
    /// YubiKey provisioning status for a person
    YubiKeyStatus(YubiKeyStatus),

    // =========================================================================
    // VISUALIZATION BOUNDED CONTEXT (with phantom-typed IDs)
    // =========================================================================
    /// Export manifest (uses ManifestId)
    Manifest(Manifest),
    /// Policy role for graph display (uses PolicyRoleId)
    PolicyRole(PolicyRole),
    /// Policy claim for graph display (uses ClaimId)
    PolicyClaim(PolicyClaimView),
    /// Policy category grouping (uses PolicyCategoryId)
    PolicyCategory(PolicyCategory),
    /// Policy separation class group (uses PolicyGroupId)
    PolicyGroup(PolicyGroup),

    // =========================================================================
    // AGGREGATE BOUNDED CONTEXTS (DDD State)
    // =========================================================================
    /// Organization aggregate state
    AggregateOrganization {
        name: String,
        version: u64,
        people_count: usize,
        units_count: usize,
    },
    /// PKI Certificate Chain aggregate state
    AggregatePkiChain {
        name: String,
        version: u64,
        certificates_count: usize,
        keys_count: usize,
    },
    /// NATS Security aggregate state
    AggregateNatsSecurity {
        name: String,
        version: u64,
        operators_count: usize,
        accounts_count: usize,
        users_count: usize,
    },
    /// YubiKey Provisioning aggregate state
    AggregateYubiKeyProvisioning {
        name: String,
        version: u64,
        devices_count: usize,
        slots_provisioned: usize,
    },
}

// ============================================================================
// Domain Node Coproduct
// ============================================================================

/// Domain Node - A proper coproduct of all domain entity types.
///
/// Uses a categorical structure that:
/// 1. Preserves identity of the original entity
/// 2. Provides injection functions (constructors)
/// 3. Supports the universal property via `fold()`
///
/// ## Example
///
/// ```ignore
/// // Injection: Person → DomainNode
/// let node = DomainNode::inject_person(person, KeyOwnerRole::Administrator);
///
/// // Universal property: fold with handlers
/// let color = node.fold(&ColorFolder);
/// let label = node.fold(&LabelFolder);
/// ```
#[derive(Debug, Clone)]
pub struct DomainNode {
    /// The injection used to construct this node
    injection: Injection,
    /// The inner data
    data: DomainNodeData,
}

impl DomainNode {
    // ========================================================================
    // Injection Functions (ι_A: A → DomainNode)
    // ========================================================================

    /// Injection ι_Person: (Person, KeyOwnerRole) → DomainNode
    pub fn inject_person(person: Person, role: KeyOwnerRole) -> Self {
        Self {
            injection: Injection::Person,
            data: DomainNodeData::Person { person, role },
        }
    }

    /// Injection ι_Organization: Organization → DomainNode
    pub fn inject_organization(org: Organization) -> Self {
        Self {
            injection: Injection::Organization,
            data: DomainNodeData::Organization(org),
        }
    }

    /// Injection ι_OrganizationUnit: OrganizationUnit → DomainNode
    pub fn inject_organization_unit(unit: OrganizationUnit) -> Self {
        Self {
            injection: Injection::OrganizationUnit,
            data: DomainNodeData::OrganizationUnit(unit),
        }
    }

    /// Injection ι_Location: Location → DomainNode
    pub fn inject_location(loc: Location) -> Self {
        Self {
            injection: Injection::Location,
            data: DomainNodeData::Location(loc),
        }
    }

    /// Injection ι_Role: Role → DomainNode
    pub fn inject_role(role: Role) -> Self {
        Self {
            injection: Injection::Role,
            data: DomainNodeData::Role(role),
        }
    }

    /// Injection ι_Policy: Policy → DomainNode
    pub fn inject_policy(policy: Policy) -> Self {
        Self {
            injection: Injection::Policy,
            data: DomainNodeData::Policy(policy),
        }
    }

    /// Injection ι_NatsOperator: NatsIdentityProjection → DomainNode
    pub fn inject_nats_operator(projection: NatsIdentityProjection) -> Self {
        Self {
            injection: Injection::NatsOperator,
            data: DomainNodeData::NatsOperator(projection),
        }
    }

    /// Injection ι_NatsAccount: NatsIdentityProjection → DomainNode
    pub fn inject_nats_account(projection: NatsIdentityProjection) -> Self {
        Self {
            injection: Injection::NatsAccount,
            data: DomainNodeData::NatsAccount(projection),
        }
    }

    /// Injection ι_NatsUser: NatsIdentityProjection → DomainNode
    pub fn inject_nats_user(projection: NatsIdentityProjection) -> Self {
        Self {
            injection: Injection::NatsUser,
            data: DomainNodeData::NatsUser(projection),
        }
    }

    /// Injection ι_NatsServiceAccount: NatsIdentityProjection → DomainNode
    pub fn inject_nats_service_account(projection: NatsIdentityProjection) -> Self {
        Self {
            injection: Injection::NatsServiceAccount,
            data: DomainNodeData::NatsServiceAccount(projection),
        }
    }

    /// Injection ι_NatsOperatorSimple: (String, Option<Uuid>) → DomainNode
    pub fn inject_nats_operator_simple(name: String, organization_id: Option<Uuid>) -> Self {
        Self {
            injection: Injection::NatsOperatorSimple,
            data: DomainNodeData::NatsOperatorSimple { name, organization_id },
        }
    }

    /// Injection ι_NatsAccountSimple: (String, Option<Uuid>, bool) → DomainNode
    pub fn inject_nats_account_simple(name: String, unit_id: Option<Uuid>, is_system: bool) -> Self {
        Self {
            injection: Injection::NatsAccountSimple,
            data: DomainNodeData::NatsAccountSimple { name, unit_id, is_system },
        }
    }

    /// Injection ι_NatsUserSimple: (String, Option<Uuid>, String) → DomainNode
    pub fn inject_nats_user_simple(name: String, person_id: Option<Uuid>, account_name: String) -> Self {
        Self {
            injection: Injection::NatsUserSimple,
            data: DomainNodeData::NatsUserSimple { name, person_id, account_name },
        }
    }

    // =========================================================================
    // PKI BOUNDED CONTEXT INJECTIONS
    // =========================================================================

    /// Injection ι_RootCertificate: Uses Certificate with phantom-typed CertificateId
    pub fn inject_root_certificate(
        cert_id: CertificateId,
        subject: String,
        _issuer: String,  // Ignored for root certs (self-signed)
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    ) -> Self {
        Self {
            injection: Injection::RootCertificate,
            data: DomainNodeData::RootCertificate(
                Certificate::root(cert_id, subject, not_before, not_after, key_usage)
            ),
        }
    }

    /// Injection ι_IntermediateCertificate: Uses Certificate with phantom-typed CertificateId
    pub fn inject_intermediate_certificate(
        cert_id: CertificateId,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    ) -> Self {
        Self {
            injection: Injection::IntermediateCertificate,
            data: DomainNodeData::IntermediateCertificate(
                Certificate::intermediate(cert_id, subject, issuer, not_before, not_after, key_usage)
            ),
        }
    }

    /// Injection ι_LeafCertificate: Uses Certificate with phantom-typed CertificateId
    pub fn inject_leaf_certificate(
        cert_id: CertificateId,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
        san: Vec<String>,
    ) -> Self {
        Self {
            injection: Injection::LeafCertificate,
            data: DomainNodeData::LeafCertificate(
                Certificate::leaf(cert_id, subject, issuer, not_before, not_after, key_usage, san)
            ),
        }
    }

    /// Injection ι_Key: Uses CryptographicKey with phantom-typed KeyId
    pub fn inject_key(
        key_id: KeyId,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            injection: Injection::Key,
            data: DomainNodeData::Key(CryptographicKey::new(key_id, algorithm, purpose, expires_at)),
        }
    }

    // =========================================================================
    // YUBIKEY BOUNDED CONTEXT INJECTIONS
    // =========================================================================

    /// Injection ι_YubiKey: Uses YubiKeyDevice with phantom-typed YubiKeyDeviceId
    pub fn inject_yubikey(
        device_id: YubiKeyDeviceId,
        serial: String,
        version: String,
        provisioned_at: Option<DateTime<Utc>>,
        slots_used: Vec<String>,
    ) -> Self {
        Self {
            injection: Injection::YubiKey,
            data: DomainNodeData::YubiKey(
                YubiKeyDevice::new(device_id, serial, version, provisioned_at, slots_used)
            ),
        }
    }

    /// Injection ι_PivSlot: Uses PivSlotView with phantom-typed SlotId
    pub fn inject_piv_slot(
        slot_id: SlotId,
        slot_name: String,
        yubikey_serial: String,
        has_key: bool,
        certificate_subject: Option<String>,
    ) -> Self {
        Self {
            injection: Injection::PivSlot,
            data: DomainNodeData::PivSlot(
                PivSlotView::new(slot_id, slot_name, yubikey_serial, has_key, certificate_subject)
            ),
        }
    }

    /// Injection ι_YubiKeyStatus: Uses YubiKeyStatus
    pub fn inject_yubikey_status(
        person_id: Uuid,
        yubikey_serial: Option<String>,
        slots_provisioned: Vec<PIVSlot>,
        slots_needed: Vec<PIVSlot>,
    ) -> Self {
        Self {
            injection: Injection::YubiKeyStatus,
            data: DomainNodeData::YubiKeyStatus(
                YubiKeyStatus::new(person_id, yubikey_serial, slots_provisioned, slots_needed)
            ),
        }
    }

    // =========================================================================
    // VISUALIZATION BOUNDED CONTEXT INJECTIONS
    // =========================================================================

    /// Injection ι_Manifest: Uses Manifest with phantom-typed ManifestId
    pub fn inject_manifest(
        manifest_id: ManifestId,
        name: String,
        destination: Option<std::path::PathBuf>,
        checksum: Option<String>,
    ) -> Self {
        Self {
            injection: Injection::Manifest,
            data: DomainNodeData::Manifest(
                Manifest::new(manifest_id, name, destination, checksum)
            ),
        }
    }

    /// Injection ι_PolicyRole: Uses PolicyRole with phantom-typed PolicyRoleId
    pub fn inject_policy_role(
        role_id: PolicyRoleId,
        name: String,
        purpose: String,
        level: u8,
        separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self {
        Self {
            injection: Injection::PolicyRole,
            data: DomainNodeData::PolicyRole(
                PolicyRole::new(role_id, name, purpose, level, separation_class, claim_count)
            ),
        }
    }

    /// Injection ι_PolicyClaim: Uses PolicyClaimView with phantom-typed ClaimId
    pub fn inject_policy_claim(
        claim_id: ClaimId,
        name: String,
        category: String,
    ) -> Self {
        Self {
            injection: Injection::PolicyClaim,
            data: DomainNodeData::PolicyClaim(PolicyClaimView::new(claim_id, name, category)),
        }
    }

    /// Injection ι_PolicyCategory: Uses PolicyCategory with phantom-typed PolicyCategoryId
    pub fn inject_policy_category(
        category_id: PolicyCategoryId,
        name: String,
        claim_count: usize,
        expanded: bool,
    ) -> Self {
        Self {
            injection: Injection::PolicyCategory,
            data: DomainNodeData::PolicyCategory(
                PolicyCategory::new(category_id, name, claim_count, expanded)
            ),
        }
    }

    /// Injection ι_PolicyGroup: Uses PolicyGroup with phantom-typed PolicyGroupId
    pub fn inject_policy_group(
        class_id: PolicyGroupId,
        name: String,
        separation_class: SeparationClass,
        role_count: usize,
        expanded: bool,
    ) -> Self {
        Self {
            injection: Injection::PolicyGroup,
            data: DomainNodeData::PolicyGroup(
                PolicyGroup::new(class_id, name, separation_class, role_count, expanded)
            ),
        }
    }

    // =========================================================================
    // AGGREGATE INJECTIONS (DDD Aggregate State)
    // =========================================================================

    /// Injection ι_AggregateOrganization: Organization aggregate state
    pub fn inject_aggregate_organization(
        name: String,
        version: u64,
        people_count: usize,
        units_count: usize,
    ) -> Self {
        Self {
            injection: Injection::AggregateOrganization,
            data: DomainNodeData::AggregateOrganization {
                name,
                version,
                people_count,
                units_count,
            },
        }
    }

    /// Injection ι_AggregatePkiChain: PKI Certificate Chain aggregate state
    pub fn inject_aggregate_pki_chain(
        name: String,
        version: u64,
        certificates_count: usize,
        keys_count: usize,
    ) -> Self {
        Self {
            injection: Injection::AggregatePkiChain,
            data: DomainNodeData::AggregatePkiChain {
                name,
                version,
                certificates_count,
                keys_count,
            },
        }
    }

    /// Injection ι_AggregateNatsSecurity: NATS Security aggregate state
    pub fn inject_aggregate_nats_security(
        name: String,
        version: u64,
        operators_count: usize,
        accounts_count: usize,
        users_count: usize,
    ) -> Self {
        Self {
            injection: Injection::AggregateNatsSecurity,
            data: DomainNodeData::AggregateNatsSecurity {
                name,
                version,
                operators_count,
                accounts_count,
                users_count,
            },
        }
    }

    /// Injection ι_AggregateYubiKeyProvisioning: YubiKey Provisioning aggregate state
    pub fn inject_aggregate_yubikey_provisioning(
        name: String,
        version: u64,
        devices_count: usize,
        slots_provisioned: usize,
    ) -> Self {
        Self {
            injection: Injection::AggregateYubiKeyProvisioning,
            data: DomainNodeData::AggregateYubiKeyProvisioning {
                name,
                version,
                devices_count,
                slots_provisioned,
            },
        }
    }

    // =========================================================================
    // BACKWARD-COMPATIBLE INJECTIONS (take raw Uuid, convert to phantom-typed)
    // =========================================================================
    // These methods maintain API compatibility with existing callers.
    // New code should prefer the strongly-typed versions above.

    /// Backward-compatible: inject_root_certificate with raw Uuid
    pub fn inject_root_certificate_uuid(
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    ) -> Self {
        Self::inject_root_certificate(
            CertificateId::from_uuid(cert_id), subject, issuer, not_before, not_after, key_usage
        )
    }

    /// Backward-compatible: inject_intermediate_certificate with raw Uuid
    pub fn inject_intermediate_certificate_uuid(
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    ) -> Self {
        Self::inject_intermediate_certificate(
            CertificateId::from_uuid(cert_id), subject, issuer, not_before, not_after, key_usage
        )
    }

    /// Backward-compatible: inject_leaf_certificate with raw Uuid
    pub fn inject_leaf_certificate_uuid(
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
        san: Vec<String>,
    ) -> Self {
        Self::inject_leaf_certificate(
            CertificateId::from_uuid(cert_id), subject, issuer, not_before, not_after, key_usage, san
        )
    }

    /// Backward-compatible: inject_key with raw Uuid
    pub fn inject_key_uuid(
        key_id: Uuid,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self::inject_key(KeyId::from_uuid(key_id), algorithm, purpose, expires_at)
    }

    /// Backward-compatible: inject_yubikey with raw Uuid
    pub fn inject_yubikey_uuid(
        device_id: Uuid,
        serial: String,
        version: String,
        provisioned_at: Option<DateTime<Utc>>,
        slots_used: Vec<String>,
    ) -> Self {
        Self::inject_yubikey(
            YubiKeyDeviceId::from_uuid(device_id), serial, version, provisioned_at, slots_used
        )
    }

    /// Backward-compatible: inject_piv_slot with raw Uuid
    pub fn inject_piv_slot_uuid(
        slot_id: Uuid,
        slot_name: String,
        yubikey_serial: String,
        has_key: bool,
        certificate_subject: Option<String>,
    ) -> Self {
        Self::inject_piv_slot(
            SlotId::from_uuid(slot_id), slot_name, yubikey_serial, has_key, certificate_subject
        )
    }

    /// Backward-compatible: inject_manifest with raw Uuid
    pub fn inject_manifest_uuid(
        manifest_id: Uuid,
        name: String,
        destination: Option<std::path::PathBuf>,
        checksum: Option<String>,
    ) -> Self {
        Self::inject_manifest(ManifestId::from_uuid(manifest_id), name, destination, checksum)
    }

    /// Backward-compatible: inject_policy_role with raw Uuid
    pub fn inject_policy_role_uuid(
        role_id: Uuid,
        name: String,
        purpose: String,
        level: u8,
        separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self {
        Self::inject_policy_role(
            PolicyRoleId::from_uuid(role_id), name, purpose, level, separation_class, claim_count
        )
    }

    /// Backward-compatible: inject_policy_claim with raw Uuid
    pub fn inject_policy_claim_uuid(
        claim_id: Uuid,
        name: String,
        category: String,
    ) -> Self {
        Self::inject_policy_claim(ClaimId::from_uuid(claim_id), name, category)
    }

    /// Backward-compatible: inject_policy_category with raw Uuid
    pub fn inject_policy_category_uuid(
        category_id: Uuid,
        name: String,
        claim_count: usize,
        expanded: bool,
    ) -> Self {
        Self::inject_policy_category(
            PolicyCategoryId::from_uuid(category_id), name, claim_count, expanded
        )
    }

    /// Backward-compatible: inject_policy_group with raw Uuid
    pub fn inject_policy_group_uuid(
        class_id: Uuid,
        name: String,
        separation_class: SeparationClass,
        role_count: usize,
        expanded: bool,
    ) -> Self {
        Self::inject_policy_group(
            PolicyGroupId::from_uuid(class_id), name, separation_class, role_count, expanded
        )
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the injection type used to construct this node
    pub fn injection(&self) -> Injection {
        self.injection
    }

    /// Get a reference to the inner data
    pub fn data(&self) -> &DomainNodeData {
        &self.data
    }

    /// Consume and return the inner data
    pub fn into_data(self) -> DomainNodeData {
        self.data
    }

    // ========================================================================
    // Layout Accessors (for graph layout functions)
    // ========================================================================

    /// Get YubiKey serial number if this is a YubiKey or PivSlot node
    pub fn yubikey_serial(&self) -> Option<&str> {
        match &self.data {
            DomainNodeData::YubiKey(node) => Some(&node.serial),
            DomainNodeData::PivSlot(node) => Some(&node.yubikey_serial),
            DomainNodeData::YubiKeyStatus(node) => node.yubikey_serial.as_deref(),
            _ => None,
        }
    }

    /// Get NATS account name if this is a NatsAccountSimple node
    /// (NatsAccount projections don't store the name directly)
    pub fn nats_account_name(&self) -> Option<&str> {
        match &self.data {
            DomainNodeData::NatsAccountSimple { name, .. } => Some(name),
            // NatsAccount projection doesn't have a direct name field
            _ => None,
        }
    }

    /// Get the parent account name if this is a NATS user node
    pub fn nats_user_account_name(&self) -> Option<&str> {
        match &self.data {
            DomainNodeData::NatsUserSimple { account_name, .. } => Some(account_name),
            // NatsUser projection doesn't store account_name directly
            _ => None,
        }
    }

    /// Get the name/identifier for NATS simple nodes (operators, accounts, users)
    /// (Full projection nodes don't expose name directly; use node.label instead)
    pub fn nats_name(&self) -> Option<&str> {
        match &self.data {
            DomainNodeData::NatsOperatorSimple { name, .. } => Some(name),
            DomainNodeData::NatsAccountSimple { name, .. } => Some(name),
            DomainNodeData::NatsUserSimple { name, .. } => Some(name),
            // Full projections don't have a direct name field - use node.label
            _ => None,
        }
    }

    /// Get organization ID if this is an Organization node
    pub fn org_id(&self) -> Option<uuid::Uuid> {
        match &self.data {
            DomainNodeData::Organization(org) => Some(org.id),
            _ => None,
        }
    }

    /// Get Person reference if this is a Person node
    pub fn person(&self) -> Option<&Person> {
        match &self.data {
            DomainNodeData::Person { person, .. } => Some(person),
            _ => None,
        }
    }

    /// Get person name if this is a Person node (convenience method)
    pub fn person_name(&self) -> Option<&str> {
        self.person().map(|p| p.name.as_str())
    }

    /// Get Organization reference if this is an Organization node
    pub fn organization(&self) -> Option<&Organization> {
        match &self.data {
            DomainNodeData::Organization(org) => Some(org),
            _ => None,
        }
    }

    /// Get OrganizationUnit reference if this is an OrganizationUnit node
    pub fn organization_unit(&self) -> Option<&OrganizationUnit> {
        match &self.data {
            DomainNodeData::OrganizationUnit(unit) => Some(unit),
            _ => None,
        }
    }

    /// Get separation class if this is a PolicyGroup node
    pub fn separation_class(&self) -> Option<&crate::policy::SeparationClass> {
        match &self.data {
            DomainNodeData::PolicyGroup(node) => Some(&node.separation_class),
            _ => None,
        }
    }

    /// Get category name if this is a PolicyCategory node
    pub fn policy_category_name(&self) -> Option<&str> {
        match &self.data {
            DomainNodeData::PolicyCategory(node) => Some(&node.name),
            _ => None,
        }
    }

    // ========================================================================
    // Universal Property: fold
    // ========================================================================

    /// Apply the universal property of the coproduct.
    ///
    /// Given a folder F with morphisms from each component type to Output,
    /// produce the unique morphism from DomainNode to Output.
    ///
    /// This satisfies: `fold(F) ∘ inject_A(a) = F.fold_A(a)`
    pub fn fold<F: FoldDomainNode>(&self, folder: &F) -> F::Output {
        match &self.data {
            DomainNodeData::Person { person, role } => folder.fold_person(person, role),
            DomainNodeData::Organization(org) => folder.fold_organization(org),
            DomainNodeData::OrganizationUnit(unit) => folder.fold_organization_unit(unit),
            DomainNodeData::Location(loc) => folder.fold_location(loc),
            DomainNodeData::Role(role) => folder.fold_role(role),
            DomainNodeData::Policy(policy) => folder.fold_policy(policy),

            DomainNodeData::NatsOperator(proj) => folder.fold_nats_operator(proj),
            DomainNodeData::NatsAccount(proj) => folder.fold_nats_account(proj),
            DomainNodeData::NatsUser(proj) => folder.fold_nats_user(proj),
            DomainNodeData::NatsServiceAccount(proj) => folder.fold_nats_service_account(proj),

            DomainNodeData::NatsOperatorSimple { name, organization_id } =>
                folder.fold_nats_operator_simple(name, *organization_id),
            DomainNodeData::NatsAccountSimple { name, unit_id, is_system } =>
                folder.fold_nats_account_simple(name, *unit_id, *is_system),
            DomainNodeData::NatsUserSimple { name, person_id, account_name } =>
                folder.fold_nats_user_simple(name, *person_id, account_name),

            DomainNodeData::RootCertificate(node) =>
                folder.fold_root_certificate(node.id.as_uuid(), &node.subject, &node.issuer, node.not_before, node.not_after, &node.key_usage),
            DomainNodeData::IntermediateCertificate(node) =>
                folder.fold_intermediate_certificate(node.id.as_uuid(), &node.subject, &node.issuer, node.not_before, node.not_after, &node.key_usage),
            DomainNodeData::LeafCertificate(node) =>
                folder.fold_leaf_certificate(node.id.as_uuid(), &node.subject, &node.issuer, node.not_before, node.not_after, &node.key_usage, &node.san),

            DomainNodeData::Key(node) =>
                folder.fold_key(node.id.as_uuid(), &node.algorithm, &node.purpose, node.expires_at),

            DomainNodeData::YubiKey(node) =>
                folder.fold_yubikey(node.id.as_uuid(), &node.serial, &node.version, node.provisioned_at, &node.slots_used),
            DomainNodeData::PivSlot(node) =>
                folder.fold_piv_slot(node.id.as_uuid(), &node.slot_name, &node.yubikey_serial, node.has_key, node.certificate_subject.as_ref()),
            DomainNodeData::YubiKeyStatus(node) =>
                folder.fold_yubikey_status(node.person_id, node.yubikey_serial.as_ref(), &node.slots_provisioned, &node.slots_needed),

            DomainNodeData::Manifest(node) =>
                folder.fold_manifest(node.id.as_uuid(), &node.name, node.destination.as_ref(), node.checksum.as_ref()),

            DomainNodeData::PolicyRole(node) =>
                folder.fold_policy_role(node.id.as_uuid(), &node.name, &node.purpose, node.level, node.separation_class, node.claim_count),
            DomainNodeData::PolicyClaim(node) =>
                folder.fold_policy_claim(node.id.as_uuid(), &node.name, &node.category),
            DomainNodeData::PolicyCategory(node) =>
                folder.fold_policy_category(node.id.as_uuid(), &node.name, node.claim_count, node.expanded),
            DomainNodeData::PolicyGroup(node) =>
                folder.fold_policy_group(node.id.as_uuid(), &node.name, node.separation_class, node.role_count, node.expanded),

            // Aggregate state nodes
            DomainNodeData::AggregateOrganization { name, version, people_count, units_count } =>
                folder.fold_aggregate_organization(name, *version, *people_count, *units_count),
            DomainNodeData::AggregatePkiChain { name, version, certificates_count, keys_count } =>
                folder.fold_aggregate_pki_chain(name, *version, *certificates_count, *keys_count),
            DomainNodeData::AggregateNatsSecurity { name, version, operators_count, accounts_count, users_count } =>
                folder.fold_aggregate_nats_security(name, *version, *operators_count, *accounts_count, *users_count),
            DomainNodeData::AggregateYubiKeyProvisioning { name, version, devices_count, slots_provisioned } =>
                folder.fold_aggregate_yubikey_provisioning(name, *version, *devices_count, *slots_provisioned),
        }
    }
}

// ============================================================================
// Domain Node Folder Trait (Universal Property)
// ============================================================================

/// Universal property of the coproduct: For any type X with morphisms from
/// each domain component to X, there exists a unique morphism from DomainNode to X.
///
/// Implementors provide the morphism from each domain type to their Output type.
/// The `fold()` method on `DomainNode` then computes the unique induced morphism.
///
/// ## Categorical Law
///
/// For any folder F and any domain entity a:
/// ```text
/// fold(F) ∘ inject_A(a) = F.fold_A(a)
/// ```
pub trait FoldDomainNode {
    /// The target type of the fold operation
    type Output;

    // Core Domain Entities
    fn fold_person(&self, person: &Person, role: &KeyOwnerRole) -> Self::Output;
    fn fold_organization(&self, org: &Organization) -> Self::Output;
    fn fold_organization_unit(&self, unit: &OrganizationUnit) -> Self::Output;
    fn fold_location(&self, loc: &Location) -> Self::Output;
    fn fold_role(&self, role: &Role) -> Self::Output;
    fn fold_policy(&self, policy: &Policy) -> Self::Output;

    // NATS Infrastructure (with projections)
    fn fold_nats_operator(&self, proj: &NatsIdentityProjection) -> Self::Output;
    fn fold_nats_account(&self, proj: &NatsIdentityProjection) -> Self::Output;
    fn fold_nats_user(&self, proj: &NatsIdentityProjection) -> Self::Output;
    fn fold_nats_service_account(&self, proj: &NatsIdentityProjection) -> Self::Output;

    // NATS Infrastructure (simple visualization)
    fn fold_nats_operator_simple(&self, name: &str, organization_id: Option<Uuid>) -> Self::Output;
    fn fold_nats_account_simple(&self, name: &str, unit_id: Option<Uuid>, is_system: bool) -> Self::Output;
    fn fold_nats_user_simple(&self, name: &str, person_id: Option<Uuid>, account_name: &str) -> Self::Output;

    // PKI Certificates
    fn fold_root_certificate(
        &self,
        cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: &[String],
    ) -> Self::Output;

    fn fold_intermediate_certificate(
        &self,
        cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: &[String],
    ) -> Self::Output;

    fn fold_leaf_certificate(
        &self,
        cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: &[String],
        san: &[String],
    ) -> Self::Output;

    // Cryptographic Keys
    fn fold_key(
        &self,
        key_id: Uuid,
        algorithm: &KeyAlgorithm,
        purpose: &KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output;

    // YubiKey Hardware
    fn fold_yubikey(
        &self,
        device_id: Uuid,
        serial: &str,
        version: &str,
        provisioned_at: Option<DateTime<Utc>>,
        slots_used: &[String],
    ) -> Self::Output;

    fn fold_piv_slot(
        &self,
        slot_id: Uuid,
        slot_name: &str,
        yubikey_serial: &str,
        has_key: bool,
        certificate_subject: Option<&String>,
    ) -> Self::Output;

    fn fold_yubikey_status(
        &self,
        person_id: Uuid,
        yubikey_serial: Option<&String>,
        slots_provisioned: &[PIVSlot],
        slots_needed: &[PIVSlot],
    ) -> Self::Output;

    // Export Manifest
    fn fold_manifest(
        &self,
        manifest_id: Uuid,
        name: &str,
        destination: Option<&std::path::PathBuf>,
        checksum: Option<&String>,
    ) -> Self::Output;

    // Policy Roles and Claims
    fn fold_policy_role(
        &self,
        role_id: Uuid,
        name: &str,
        purpose: &str,
        level: u8,
        separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self::Output;

    fn fold_policy_claim(
        &self,
        claim_id: Uuid,
        name: &str,
        category: &str,
    ) -> Self::Output;

    fn fold_policy_category(
        &self,
        category_id: Uuid,
        name: &str,
        claim_count: usize,
        expanded: bool,
    ) -> Self::Output;

    fn fold_policy_group(
        &self,
        class_id: Uuid,
        name: &str,
        separation_class: SeparationClass,
        role_count: usize,
        expanded: bool,
    ) -> Self::Output;

    // Aggregate State Machines
    fn fold_aggregate_organization(
        &self,
        name: &str,
        version: u64,
        people_count: usize,
        units_count: usize,
    ) -> Self::Output;

    fn fold_aggregate_pki_chain(
        &self,
        name: &str,
        version: u64,
        certificates_count: usize,
        keys_count: usize,
    ) -> Self::Output;

    fn fold_aggregate_nats_security(
        &self,
        name: &str,
        version: u64,
        operators_count: usize,
        accounts_count: usize,
        users_count: usize,
    ) -> Self::Output;

    fn fold_aggregate_yubikey_provisioning(
        &self,
        name: &str,
        version: u64,
        devices_count: usize,
        slots_provisioned: usize,
    ) -> Self::Output;
}

// ============================================================================
// Visualization Folder - Separates visual concerns from domain
// ============================================================================

/// Output of visualization folder - all data needed to render a node
#[derive(Debug, Clone)]
pub struct VisualizationData {
    /// Node color (fill)
    pub color: iced::Color,
    /// Primary display text (e.g., name)
    pub primary_text: String,
    /// Secondary display text (e.g., email, description)
    pub secondary_text: String,
    /// Icon character (emoji or Material icon)
    pub icon: char,
    /// Font for the icon
    pub icon_font: iced::Font,
    /// Whether this node is expandable (has +/- indicator)
    pub expandable: bool,
    /// If expandable, whether it's currently expanded
    pub expanded: bool,
}

impl Default for VisualizationData {
    fn default() -> Self {
        Self {
            color: iced::Color::from_rgb(0.5, 0.5, 0.5),
            primary_text: String::new(),
            secondary_text: String::new(),
            icon: crate::icons::ICON_HELP,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }
}

/// Folder that produces visualization data for rendering nodes.
///
/// This separates visualization concerns from domain concerns - the domain
/// node knows nothing about colors or icons, but this folder maps each
/// domain type to its visual representation.
pub struct FoldVisualization;

impl FoldDomainNode for FoldVisualization {
    type Output = VisualizationData;

    fn fold_person(&self, person: &Person, _role: &KeyOwnerRole) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.3, 0.6, 0.9), // Blue
            primary_text: person.name.clone(),
            secondary_text: person.email.clone(),
            icon: crate::icons::ICON_PERSON,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_organization(&self, org: &Organization) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.2, 0.3, 0.6), // Dark blue
            primary_text: org.name.clone(),
            secondary_text: org.display_name.clone(),
            icon: crate::icons::ICON_BUSINESS,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_organization_unit(&self, unit: &OrganizationUnit) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.4, 0.5, 0.8), // Light blue
            primary_text: unit.name.clone(),
            secondary_text: format!("{:?}", unit.unit_type),
            icon: crate::icons::ICON_GROUP,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_location(&self, loc: &Location) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.6, 0.5, 0.4), // Brown/gray
            primary_text: loc.name.clone(),
            secondary_text: format!("{:?}", loc.location_type),
            icon: crate::icons::ICON_LOCATION,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_role(&self, role: &Role) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.6, 0.3, 0.8), // Purple
            primary_text: role.name.clone(),
            secondary_text: role.description.clone(),
            icon: crate::icons::ICON_SECURITY,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_policy(&self, policy: &Policy) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.9, 0.7, 0.2), // Gold/yellow
            primary_text: policy.name.clone(),
            secondary_text: format!("{} claims", policy.claims.len()),
            icon: crate::icons::ICON_VERIFIED,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_nats_operator(&self, identity: &NatsIdentityProjection) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(1.0, 0.2, 0.0), // Bright red (root of trust)
            primary_text: "NATS Operator".to_string(),
            secondary_text: identity.nkey.public_key.public_key()[..8].to_string(),
            icon: crate::icons::ICON_CLOUD,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_nats_account(&self, identity: &NatsIdentityProjection) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(1.0, 0.5, 0.0), // Orange (intermediate trust)
            primary_text: "NATS Account".to_string(),
            secondary_text: identity.nkey.public_key.public_key()[..8].to_string(),
            icon: crate::icons::ICON_ACCOUNT_CIRCLE,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_nats_user(&self, identity: &NatsIdentityProjection) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.2, 0.8, 1.0), // Cyan (leaf user)
            primary_text: "NATS User".to_string(),
            secondary_text: identity.nkey.public_key.public_key()[..8].to_string(),
            icon: crate::icons::ICON_PERSON,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_nats_service_account(&self, identity: &NatsIdentityProjection) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.8, 0.2, 0.8), // Magenta (service account)
            primary_text: "NATS Service".to_string(),
            secondary_text: identity.nkey.public_key.public_key()[..8].to_string(),
            icon: crate::icons::ICON_SETTINGS,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_nats_operator_simple(&self, name: &str, _organization_id: Option<Uuid>) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(1.0, 0.2, 0.0), // Bright red
            primary_text: name.to_string(),
            secondary_text: "NATS Operator".to_string(),
            icon: crate::icons::ICON_CLOUD,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_nats_account_simple(&self, name: &str, _unit_id: Option<Uuid>, is_system: bool) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(1.0, 0.5, 0.0), // Orange
            primary_text: name.to_string(),
            secondary_text: if is_system { "System Account" } else { "NATS Account" }.to_string(),
            icon: crate::icons::ICON_ACCOUNT_CIRCLE,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_nats_user_simple(&self, name: &str, _person_id: Option<Uuid>, account_name: &str) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.2, 0.8, 1.0), // Cyan
            primary_text: name.to_string(),
            secondary_text: format!("Account: {}", account_name),
            icon: crate::icons::ICON_PERSON,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_root_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.0, 0.6, 0.4), // Dark teal (root trust)
            primary_text: "Root CA".to_string(),
            secondary_text: format!("{} (expires {})", subject, not_after.format("%Y-%m-%d")),
            icon: crate::icons::ICON_VERIFIED,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_intermediate_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.2, 0.8, 0.6), // Medium teal
            primary_text: "Intermediate CA".to_string(),
            secondary_text: format!("{} (expires {})", subject, not_after.format("%Y-%m-%d")),
            icon: crate::icons::ICON_VERIFIED,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_leaf_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
        san: &[String],
    ) -> Self::Output {
        let secondary = if !san.is_empty() {
            format!("SAN: {} (expires {})", san[0], not_after.format("%Y-%m-%d"))
        } else {
            format!("expires {}", not_after.format("%Y-%m-%d"))
        };
        VisualizationData {
            color: iced::Color::from_rgb(0.4, 1.0, 0.8), // Light teal
            primary_text: format!("Certificate: {}", subject),
            secondary_text: secondary,
            icon: crate::icons::ICON_LOCK,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_key(
        &self,
        _key_id: Uuid,
        algorithm: &KeyAlgorithm,
        purpose: &KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output {
        let secondary = if let Some(exp) = expires_at {
            format!("{:?} (expires {})", algorithm, exp.format("%Y-%m-%d"))
        } else {
            format!("{:?} (no expiry)", algorithm)
        };
        VisualizationData {
            color: iced::Color::from_rgb(0.7, 0.5, 0.9), // Light purple
            primary_text: format!("Key: {:?}", purpose),
            secondary_text: secondary,
            icon: crate::icons::ICON_LOCK,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_yubikey(
        &self,
        _device_id: Uuid,
        serial: &str,
        version: &str,
        _provisioned_at: Option<DateTime<Utc>>,
        slots_used: &[String],
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.8, 0.3, 0.8), // Magenta (hardware)
            primary_text: format!("YubiKey {}", serial),
            secondary_text: format!("v{} ({} slots used)", version, slots_used.len()),
            icon: crate::icons::ICON_SECURITY,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_piv_slot(
        &self,
        _slot_id: Uuid,
        slot_name: &str,
        _yubikey_serial: &str,
        has_key: bool,
        certificate_subject: Option<&String>,
    ) -> Self::Output {
        let secondary = if has_key {
            certificate_subject.cloned().unwrap_or_else(|| "Key loaded".to_string())
        } else {
            "Empty slot".to_string()
        };
        VisualizationData {
            color: iced::Color::from_rgb(0.9, 0.5, 0.9), // Light magenta (slot)
            primary_text: slot_name.to_string(),
            secondary_text: secondary,
            icon: crate::icons::ICON_LOCK,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_yubikey_status(
        &self,
        _person_id: Uuid,
        yubikey_serial: Option<&String>,
        slots_provisioned: &[PIVSlot],
        slots_needed: &[PIVSlot],
    ) -> Self::Output {
        let secondary = if let Some(serial) = yubikey_serial {
            format!("{}/{} slots ({}))", slots_provisioned.len(), slots_needed.len(), serial)
        } else {
            format!("{}/{} slots needed", slots_provisioned.len(), slots_needed.len())
        };
        VisualizationData {
            color: iced::Color::from_rgb(0.6, 0.4, 0.7), // Purple
            primary_text: "YubiKey Status".to_string(),
            secondary_text: secondary,
            icon: crate::icons::ICON_SECURITY,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_manifest(
        &self,
        _manifest_id: Uuid,
        name: &str,
        destination: Option<&std::path::PathBuf>,
        _checksum: Option<&String>,
    ) -> Self::Output {
        let secondary = if let Some(dest) = destination {
            format!("-> {}", dest.display())
        } else {
            "No destination".to_string()
        };
        VisualizationData {
            color: iced::Color::from_rgb(0.5, 0.7, 0.5), // Green
            primary_text: format!("Manifest: {}", name),
            secondary_text: secondary,
            icon: crate::icons::ICON_BUSINESS,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_policy_role(
        &self,
        _role_id: Uuid,
        name: &str,
        purpose: &str,
        level: u8,
        separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self::Output {
        // Color by separation class
        let color = match separation_class {
            SeparationClass::Operational => iced::Color::from_rgb(0.3, 0.6, 0.9),
            SeparationClass::Administrative => iced::Color::from_rgb(0.6, 0.4, 0.8),
            SeparationClass::Audit => iced::Color::from_rgb(0.2, 0.7, 0.5),
            SeparationClass::Emergency => iced::Color::from_rgb(0.9, 0.3, 0.2),
            SeparationClass::Financial => iced::Color::from_rgb(0.9, 0.7, 0.2),
            SeparationClass::Personnel => iced::Color::from_rgb(0.8, 0.4, 0.6),
        };
        VisualizationData {
            color,
            primary_text: name.to_string(),
            secondary_text: format!("L{} | {} claims | {}", level, claim_count, purpose),
            icon: crate::icons::ICON_SECURITY,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_policy_claim(
        &self,
        _claim_id: Uuid,
        name: &str,
        category: &str,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.4, 0.8, 0.6), // Teal
            primary_text: name.to_string(),
            secondary_text: category.to_string(),
            icon: crate::icons::ICON_VERIFIED,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_policy_category(
        &self,
        _category_id: Uuid,
        name: &str,
        claim_count: usize,
        expanded: bool,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.5, 0.6, 0.8), // Blue-gray
            primary_text: name.to_string(),
            secondary_text: format!("{} claims", claim_count),
            icon: ' ',  // No icon - the +/- indicator below is the main UI element
            icon_font: iced::Font::DEFAULT,
            expandable: true,
            expanded,
        }
    }

    fn fold_policy_group(
        &self,
        _class_id: Uuid,
        name: &str,
        separation_class: SeparationClass,
        role_count: usize,
        expanded: bool,
    ) -> Self::Output {
        // Color by separation class
        let color = match separation_class {
            SeparationClass::Operational => iced::Color::from_rgb(0.3, 0.6, 0.9),
            SeparationClass::Administrative => iced::Color::from_rgb(0.6, 0.4, 0.8),
            SeparationClass::Audit => iced::Color::from_rgb(0.2, 0.7, 0.5),
            SeparationClass::Emergency => iced::Color::from_rgb(0.9, 0.3, 0.2),
            SeparationClass::Financial => iced::Color::from_rgb(0.9, 0.7, 0.2),
            SeparationClass::Personnel => iced::Color::from_rgb(0.8, 0.4, 0.6),
        };
        VisualizationData {
            color,
            primary_text: name.to_string(),
            secondary_text: format!("{} roles", role_count),
            icon: ' ',  // No icon - the +/- indicator below is the main UI element
            icon_font: iced::Font::DEFAULT,
            expandable: true,
            expanded,
        }
    }

    // Aggregate state machines
    // Note: Colors are placeholder - actual colors set in populate_aggregates_graph from ColorPalette
    fn fold_aggregate_organization(
        &self,
        name: &str,
        version: u64,
        people_count: usize,
        units_count: usize,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::WHITE,  // Placeholder - overridden by ColorPalette.aggregate_organization
            primary_text: name.to_string(),
            secondary_text: format!("v{} | {} people, {} units", version, people_count, units_count),
            icon: '🏢',
            icon_font: iced::Font::DEFAULT,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_aggregate_pki_chain(
        &self,
        name: &str,
        version: u64,
        certificates_count: usize,
        keys_count: usize,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::WHITE,  // Placeholder - overridden by ColorPalette.aggregate_pki_chain
            primary_text: name.to_string(),
            secondary_text: format!("v{} | {} certs, {} keys", version, certificates_count, keys_count),
            icon: '📜',
            icon_font: iced::Font::DEFAULT,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_aggregate_nats_security(
        &self,
        name: &str,
        version: u64,
        operators_count: usize,
        accounts_count: usize,
        users_count: usize,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::WHITE,  // Placeholder - overridden by ColorPalette.aggregate_nats_security
            primary_text: name.to_string(),
            secondary_text: format!("v{} | {} ops, {} accts, {} users", version, operators_count, accounts_count, users_count),
            icon: '🔌',
            icon_font: iced::Font::DEFAULT,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_aggregate_yubikey_provisioning(
        &self,
        name: &str,
        version: u64,
        devices_count: usize,
        slots_provisioned: usize,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::WHITE,  // Placeholder - overridden by ColorPalette.aggregate_yubikey
            primary_text: name.to_string(),
            secondary_text: format!("v{} | {} devices, {} slots", version, devices_count, slots_provisioned),
            icon: '🔑',
            icon_font: iced::Font::DEFAULT,
            expandable: false,
            expanded: false,
        }
    }
}

// ============================================================================
// Detail Panel Fold
// ============================================================================

/// Data for rendering a detail panel for a selected node
#[derive(Debug, Clone)]
pub struct DetailPanelData {
    /// Title for the detail panel (e.g., "Selected Organization:")
    pub title: String,
    /// List of (label, value) pairs to display
    pub fields: Vec<(String, String)>,
}

/// Folder that produces detail panel data from a domain node
pub struct FoldDetailPanel;

impl FoldDomainNode for FoldDetailPanel {
    type Output = DetailPanelData;

    fn fold_person(&self, person: &Person, role: &KeyOwnerRole) -> Self::Output {
        DetailPanelData {
            title: "Selected Person:".to_string(),
            fields: vec![
                ("Name".to_string(), person.name.clone()),
                ("Email".to_string(), person.email.clone()),
                ("Active".to_string(), if person.active { "✓" } else { "✗" }.to_string()),
                ("Key Role".to_string(), format!("{:?}", role)),
            ],
        }
    }

    fn fold_organization(&self, org: &Organization) -> Self::Output {
        DetailPanelData {
            title: "Selected Organization:".to_string(),
            fields: vec![
                ("Name".to_string(), org.name.clone()),
                ("Display Name".to_string(), org.display_name.clone()),
                ("Units".to_string(), org.units.len().to_string()),
            ],
        }
    }

    fn fold_organization_unit(&self, unit: &OrganizationUnit) -> Self::Output {
        DetailPanelData {
            title: "Selected Unit:".to_string(),
            fields: vec![
                ("Name".to_string(), unit.name.clone()),
                ("Type".to_string(), format!("{:?}", unit.unit_type)),
            ],
        }
    }

    fn fold_location(&self, loc: &Location) -> Self::Output {
        DetailPanelData {
            title: "Selected Location:".to_string(),
            fields: vec![
                ("Name".to_string(), loc.name.clone()),
                ("Type".to_string(), format!("{:?}", loc.location_type)),
            ],
        }
    }

    fn fold_role(&self, role: &Role) -> Self::Output {
        DetailPanelData {
            title: "Selected Role:".to_string(),
            fields: vec![
                ("Name".to_string(), role.name.clone()),
                ("Description".to_string(), role.description.clone()),
                ("Required Policies".to_string(), role.required_policies.len().to_string()),
            ],
        }
    }

    fn fold_policy(&self, policy: &Policy) -> Self::Output {
        DetailPanelData {
            title: "Selected Policy:".to_string(),
            fields: vec![
                ("Name".to_string(), policy.name.clone()),
                ("Claims".to_string(), policy.claims.len().to_string()),
                ("Conditions".to_string(), policy.conditions.len().to_string()),
                ("Priority".to_string(), policy.priority.to_string()),
                ("Enabled".to_string(), policy.enabled.to_string()),
            ],
        }
    }

    fn fold_nats_operator(&self, identity: &NatsIdentityProjection) -> Self::Output {
        DetailPanelData {
            title: "Selected NATS Operator:".to_string(),
            fields: vec![
                ("Public Key".to_string(), identity.nkey.public_key.public_key().to_string()),
                ("JWT Token".to_string(), format!("{}...", &identity.jwt.token()[..20.min(identity.jwt.token().len())])),
                ("Has Credential".to_string(), identity.credential.is_some().to_string()),
            ],
        }
    }

    fn fold_nats_account(&self, identity: &NatsIdentityProjection) -> Self::Output {
        DetailPanelData {
            title: "Selected NATS Account:".to_string(),
            fields: vec![
                ("Public Key".to_string(), identity.nkey.public_key.public_key().to_string()),
                ("JWT Token".to_string(), format!("{}...", &identity.jwt.token()[..20.min(identity.jwt.token().len())])),
                ("Has Credential".to_string(), identity.credential.is_some().to_string()),
            ],
        }
    }

    fn fold_nats_user(&self, identity: &NatsIdentityProjection) -> Self::Output {
        DetailPanelData {
            title: "Selected NATS User:".to_string(),
            fields: vec![
                ("Public Key".to_string(), identity.nkey.public_key.public_key().to_string()),
                ("JWT Token".to_string(), format!("{}...", &identity.jwt.token()[..20.min(identity.jwt.token().len())])),
                ("Has Credential".to_string(), identity.credential.is_some().to_string()),
            ],
        }
    }

    fn fold_nats_service_account(&self, identity: &NatsIdentityProjection) -> Self::Output {
        DetailPanelData {
            title: "Selected Service Account:".to_string(),
            fields: vec![
                ("Public Key".to_string(), identity.nkey.public_key.public_key().to_string()),
                ("JWT Token".to_string(), format!("{}...", &identity.jwt.token()[..20.min(identity.jwt.token().len())])),
                ("Has Credential".to_string(), identity.credential.is_some().to_string()),
            ],
        }
    }

    fn fold_nats_operator_simple(&self, name: &str, organization_id: Option<Uuid>) -> Self::Output {
        DetailPanelData {
            title: "Selected NATS Operator:".to_string(),
            fields: vec![
                ("Name".to_string(), name.to_string()),
                ("Organization".to_string(), organization_id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string())),
                ("Note".to_string(), "(Visualization only - no crypto keys)".to_string()),
            ],
        }
    }

    fn fold_nats_account_simple(&self, name: &str, unit_id: Option<Uuid>, is_system: bool) -> Self::Output {
        DetailPanelData {
            title: "Selected NATS Account:".to_string(),
            fields: vec![
                ("Name".to_string(), name.to_string()),
                ("Unit".to_string(), unit_id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string())),
                ("System Account".to_string(), is_system.to_string()),
                ("Note".to_string(), "(Visualization only - no crypto keys)".to_string()),
            ],
        }
    }

    fn fold_nats_user_simple(&self, name: &str, person_id: Option<Uuid>, account_name: &str) -> Self::Output {
        DetailPanelData {
            title: "Selected NATS User:".to_string(),
            fields: vec![
                ("Name".to_string(), name.to_string()),
                ("Account".to_string(), account_name.to_string()),
                ("Person".to_string(), person_id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string())),
                ("Note".to_string(), "(Visualization only - no crypto keys)".to_string()),
            ],
        }
    }

    fn fold_root_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: &[String],
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Root CA Certificate:".to_string(),
            fields: vec![
                ("Subject".to_string(), subject.to_string()),
                ("Issuer".to_string(), issuer.to_string()),
                ("Valid From".to_string(), not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                ("Valid Until".to_string(), not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                ("Key Usage".to_string(), key_usage.join(", ")),
            ],
        }
    }

    fn fold_intermediate_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: &[String],
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Intermediate CA Certificate:".to_string(),
            fields: vec![
                ("Subject".to_string(), subject.to_string()),
                ("Issuer".to_string(), issuer.to_string()),
                ("Valid From".to_string(), not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                ("Valid Until".to_string(), not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                ("Key Usage".to_string(), key_usage.join(", ")),
            ],
        }
    }

    fn fold_leaf_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: &[String],
        san: &[String],
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Leaf Certificate:".to_string(),
            fields: vec![
                ("Subject".to_string(), subject.to_string()),
                ("Issuer".to_string(), issuer.to_string()),
                ("Valid From".to_string(), not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                ("Valid Until".to_string(), not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                ("Key Usage".to_string(), key_usage.join(", ")),
                ("Subject Alt Names".to_string(), if san.is_empty() { "none".to_string() } else { san.join(", ") }),
            ],
        }
    }

    fn fold_key(
        &self,
        key_id: Uuid,
        algorithm: &KeyAlgorithm,
        purpose: &KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Key:".to_string(),
            fields: vec![
                ("ID".to_string(), key_id.to_string()),
                ("Algorithm".to_string(), format!("{:?}", algorithm)),
                ("Purpose".to_string(), format!("{:?}", purpose)),
                ("Expires".to_string(), expires_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_else(|| "Never".to_string())),
            ],
        }
    }

    fn fold_yubikey(
        &self,
        _device_id: Uuid,
        serial: &str,
        version: &str,
        provisioned_at: Option<DateTime<Utc>>,
        slots_used: &[String],
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected YubiKey:".to_string(),
            fields: vec![
                ("Serial".to_string(), serial.to_string()),
                ("Version".to_string(), version.to_string()),
                ("Provisioned".to_string(), provisioned_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_else(|| "Not provisioned".to_string())),
                ("Slots Used".to_string(), slots_used.join(", ")),
            ],
        }
    }

    fn fold_piv_slot(
        &self,
        _slot_id: Uuid,
        slot_name: &str,
        yubikey_serial: &str,
        has_key: bool,
        certificate_subject: Option<&String>,
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected PIV Slot:".to_string(),
            fields: vec![
                ("Slot".to_string(), slot_name.to_string()),
                ("YubiKey".to_string(), yubikey_serial.to_string()),
                ("Status".to_string(), if has_key { "Key loaded" } else { "Empty" }.to_string()),
                ("Certificate".to_string(), certificate_subject.cloned().unwrap_or_else(|| "None".to_string())),
            ],
        }
    }

    fn fold_yubikey_status(
        &self,
        person_id: Uuid,
        yubikey_serial: Option<&String>,
        slots_provisioned: &[PIVSlot],
        slots_needed: &[PIVSlot],
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected YubiKey Status:".to_string(),
            fields: vec![
                ("Person ID".to_string(), person_id.to_string()),
                ("Serial".to_string(), yubikey_serial.cloned().unwrap_or_else(|| "Not detected".to_string())),
                ("Provisioned Slots".to_string(), slots_provisioned.len().to_string()),
                ("Needed Slots".to_string(), slots_needed.len().to_string()),
            ],
        }
    }

    fn fold_manifest(
        &self,
        manifest_id: Uuid,
        name: &str,
        destination: Option<&std::path::PathBuf>,
        checksum: Option<&String>,
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Manifest:".to_string(),
            fields: vec![
                ("ID".to_string(), manifest_id.to_string()),
                ("Name".to_string(), name.to_string()),
                ("Destination".to_string(), destination.map(|p| p.display().to_string()).unwrap_or_else(|| "None".to_string())),
                ("Checksum".to_string(), checksum.cloned().unwrap_or_else(|| "None".to_string())),
            ],
        }
    }

    fn fold_policy_role(
        &self,
        role_id: Uuid,
        name: &str,
        purpose: &str,
        level: u8,
        separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Policy Role:".to_string(),
            fields: vec![
                ("ID".to_string(), role_id.to_string()),
                ("Name".to_string(), name.to_string()),
                ("Purpose".to_string(), purpose.to_string()),
                ("Level".to_string(), level.to_string()),
                ("Separation Class".to_string(), format!("{:?}", separation_class)),
                ("Claims".to_string(), claim_count.to_string()),
            ],
        }
    }

    fn fold_policy_claim(
        &self,
        claim_id: Uuid,
        name: &str,
        category: &str,
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Claim:".to_string(),
            fields: vec![
                ("ID".to_string(), claim_id.to_string()),
                ("Name".to_string(), name.to_string()),
                ("Category".to_string(), category.to_string()),
            ],
        }
    }

    fn fold_policy_category(
        &self,
        category_id: Uuid,
        name: &str,
        claim_count: usize,
        expanded: bool,
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Category:".to_string(),
            fields: vec![
                ("ID".to_string(), category_id.to_string()),
                ("Name".to_string(), name.to_string()),
                ("Claims".to_string(), claim_count.to_string()),
                ("Expanded".to_string(), if expanded { "Yes" } else { "No" }.to_string()),
                ("Action".to_string(), "Click to toggle expansion".to_string()),
            ],
        }
    }

    fn fold_policy_group(
        &self,
        class_id: Uuid,
        name: &str,
        separation_class: SeparationClass,
        role_count: usize,
        expanded: bool,
    ) -> Self::Output {
        DetailPanelData {
            title: "Selected Separation Class:".to_string(),
            fields: vec![
                ("ID".to_string(), class_id.to_string()),
                ("Name".to_string(), name.to_string()),
                ("Class".to_string(), format!("{:?}", separation_class)),
                ("Roles".to_string(), role_count.to_string()),
                ("Expanded".to_string(), if expanded { "Yes" } else { "No" }.to_string()),
                ("Action".to_string(), "Click to toggle expansion".to_string()),
            ],
        }
    }

    // Aggregate state machines
    fn fold_aggregate_organization(&self, name: &str, version: u64, people_count: usize, units_count: usize) -> Self::Output {
        DetailPanelData {
            title: "Organization Aggregate:".to_string(),
            fields: vec![
                ("Name".to_string(), name.to_string()),
                ("Version".to_string(), version.to_string()),
                ("People".to_string(), people_count.to_string()),
                ("Units".to_string(), units_count.to_string()),
            ],
        }
    }

    fn fold_aggregate_pki_chain(&self, name: &str, version: u64, certificates_count: usize, keys_count: usize) -> Self::Output {
        DetailPanelData {
            title: "PKI Certificate Chain Aggregate:".to_string(),
            fields: vec![
                ("Name".to_string(), name.to_string()),
                ("Version".to_string(), version.to_string()),
                ("Certificates".to_string(), certificates_count.to_string()),
                ("Keys".to_string(), keys_count.to_string()),
            ],
        }
    }

    fn fold_aggregate_nats_security(&self, name: &str, version: u64, operators_count: usize, accounts_count: usize, users_count: usize) -> Self::Output {
        DetailPanelData {
            title: "NATS Security Aggregate:".to_string(),
            fields: vec![
                ("Name".to_string(), name.to_string()),
                ("Version".to_string(), version.to_string()),
                ("Operators".to_string(), operators_count.to_string()),
                ("Accounts".to_string(), accounts_count.to_string()),
                ("Users".to_string(), users_count.to_string()),
            ],
        }
    }

    fn fold_aggregate_yubikey_provisioning(&self, name: &str, version: u64, devices_count: usize, slots_provisioned: usize) -> Self::Output {
        DetailPanelData {
            title: "YubiKey Provisioning Aggregate:".to_string(),
            fields: vec![
                ("Name".to_string(), name.to_string()),
                ("Version".to_string(), version.to_string()),
                ("Devices".to_string(), devices_count.to_string()),
                ("Slots Provisioned".to_string(), slots_provisioned.to_string()),
            ],
        }
    }
}

/// Helper method on DomainNode for getting detail panel data
impl DomainNode {
    /// Get detail panel data using the FoldDetailPanel catamorphism
    pub fn detail_panel(&self) -> DetailPanelData {
        self.fold(&FoldDetailPanel)
    }
}

// ============================================================================
// FoldSearchableText - Extracts searchable text for node filtering/search
// ============================================================================

/// Data for search/filter matching on nodes
#[derive(Debug, Clone)]
pub struct SearchableText {
    /// Text fields from the node data (name, email, subject, etc.)
    pub fields: Vec<String>,
    /// Type-specific keywords (e.g., "nats operator", "certificate", "yubikey")
    pub keywords: Vec<String>,
}

impl SearchableText {
    /// Check if any field or keyword contains the query (case-insensitive)
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.fields.iter().any(|f| f.to_lowercase().contains(&query_lower)) ||
        self.keywords.iter().any(|k| k.contains(&query_lower))
    }
}

/// Folder that produces searchable text from a domain node
pub struct FoldSearchableText;

impl FoldDomainNode for FoldSearchableText {
    type Output = SearchableText;

    fn fold_person(&self, person: &Person, _role: &KeyOwnerRole) -> Self::Output {
        SearchableText {
            fields: vec![person.name.clone(), person.email.clone()],
            keywords: vec!["person".to_string()],
        }
    }

    fn fold_organization(&self, org: &Organization) -> Self::Output {
        let mut fields = vec![org.name.clone(), org.display_name.clone()];
        if let Some(desc) = &org.description {
            fields.push(desc.clone());
        }
        SearchableText {
            fields,
            keywords: vec!["organization".to_string(), "org".to_string()],
        }
    }

    fn fold_organization_unit(&self, unit: &OrganizationUnit) -> Self::Output {
        SearchableText {
            fields: vec![unit.name.clone()],
            keywords: vec!["unit".to_string(), "department".to_string()],
        }
    }

    fn fold_location(&self, loc: &Location) -> Self::Output {
        SearchableText {
            fields: vec![loc.name.clone()],
            keywords: vec!["location".to_string()],
        }
    }

    fn fold_role(&self, role: &Role) -> Self::Output {
        SearchableText {
            fields: vec![role.name.clone()],
            keywords: vec!["role".to_string()],
        }
    }

    fn fold_policy(&self, policy: &Policy) -> Self::Output {
        SearchableText {
            fields: vec![policy.name.clone()],
            keywords: vec!["policy".to_string()],
        }
    }

    fn fold_nats_operator(&self, _proj: &NatsIdentityProjection) -> Self::Output {
        SearchableText {
            fields: vec![],
            keywords: vec!["nats operator".to_string(), "operator".to_string()],
        }
    }

    fn fold_nats_account(&self, _proj: &NatsIdentityProjection) -> Self::Output {
        SearchableText {
            fields: vec![],
            keywords: vec!["nats account".to_string(), "account".to_string()],
        }
    }

    fn fold_nats_user(&self, _proj: &NatsIdentityProjection) -> Self::Output {
        SearchableText {
            fields: vec![],
            keywords: vec!["nats user".to_string(), "user".to_string()],
        }
    }

    fn fold_nats_service_account(&self, _proj: &NatsIdentityProjection) -> Self::Output {
        SearchableText {
            fields: vec![],
            keywords: vec!["service account".to_string(), "service".to_string()],
        }
    }

    fn fold_nats_operator_simple(&self, name: &str, _organization_id: Option<Uuid>) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["nats operator".to_string(), "operator".to_string()],
        }
    }

    fn fold_nats_account_simple(&self, name: &str, _unit_id: Option<Uuid>, _is_system: bool) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["nats account".to_string(), "account".to_string()],
        }
    }

    fn fold_nats_user_simple(&self, name: &str, _person_id: Option<Uuid>, account_name: &str) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string(), account_name.to_string()],
            keywords: vec!["nats user".to_string(), "user".to_string()],
        }
    }

    fn fold_root_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        SearchableText {
            fields: vec![subject.to_string()],
            keywords: vec!["root".to_string(), "certificate".to_string(), "ca".to_string()],
        }
    }

    fn fold_intermediate_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        SearchableText {
            fields: vec![subject.to_string()],
            keywords: vec!["intermediate".to_string(), "certificate".to_string(), "ca".to_string()],
        }
    }

    fn fold_leaf_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
        _san: &[String],
    ) -> Self::Output {
        SearchableText {
            fields: vec![subject.to_string()],
            keywords: vec!["leaf".to_string(), "certificate".to_string()],
        }
    }

    fn fold_yubikey(
        &self,
        _device_id: Uuid,
        serial: &str,
        _version: &str,
        _provisioned_at: Option<DateTime<Utc>>,
        _slots_used: &[String],
    ) -> Self::Output {
        SearchableText {
            fields: vec![serial.to_string()],
            keywords: vec!["yubikey".to_string()],
        }
    }

    fn fold_piv_slot(
        &self,
        _slot_id: Uuid,
        slot_name: &str,
        _yubikey_serial: &str,
        _has_key: bool,
        _certificate_subject: Option<&String>,
    ) -> Self::Output {
        SearchableText {
            fields: vec![slot_name.to_string()],
            keywords: vec!["piv".to_string(), "slot".to_string()],
        }
    }

    fn fold_yubikey_status(
        &self,
        _person_id: Uuid,
        yubikey_serial: Option<&String>,
        _slots_provisioned: &[PIVSlot],
        _slots_needed: &[PIVSlot],
    ) -> Self::Output {
        let mut fields = vec![];
        if let Some(serial) = yubikey_serial {
            fields.push(serial.clone());
        }
        SearchableText {
            fields,
            keywords: vec!["yubikey".to_string(), "status".to_string()],
        }
    }

    fn fold_key(
        &self,
        _key_id: Uuid,
        algorithm: &KeyAlgorithm,
        purpose: &KeyPurpose,
        _expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output {
        SearchableText {
            fields: vec![format!("{:?}", algorithm), format!("{:?}", purpose)],
            keywords: vec!["key".to_string()],
        }
    }

    fn fold_manifest(
        &self,
        _manifest_id: Uuid,
        name: &str,
        _destination: Option<&std::path::PathBuf>,
        _checksum: Option<&String>,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["manifest".to_string(), "export".to_string()],
        }
    }

    fn fold_policy_role(
        &self,
        _role_id: Uuid,
        name: &str,
        purpose: &str,
        _level: u8,
        _separation_class: SeparationClass,
        _claim_count: usize,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string(), purpose.to_string()],
            keywords: vec!["role".to_string(), "policy".to_string()],
        }
    }

    fn fold_policy_claim(
        &self,
        _claim_id: Uuid,
        name: &str,
        category: &str,
    ) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string(), category.to_string()],
            keywords: vec!["claim".to_string()],
        }
    }

    fn fold_policy_category(&self, _category_id: Uuid, name: &str, _claim_count: usize, _expanded: bool) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["category".to_string()],
        }
    }

    fn fold_policy_group(&self, _class_id: Uuid, name: &str, _separation_class: SeparationClass, _role_count: usize, _expanded: bool) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["class".to_string(), "separation".to_string()],
        }
    }

    fn fold_aggregate_organization(&self, name: &str, _version: u64, _people_count: usize, _units_count: usize) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["aggregate".to_string(), "organization".to_string()],
        }
    }

    fn fold_aggregate_pki_chain(&self, name: &str, _version: u64, _certificates_count: usize, _keys_count: usize) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["aggregate".to_string(), "pki".to_string(), "certificate".to_string()],
        }
    }

    fn fold_aggregate_nats_security(&self, name: &str, _version: u64, _operators_count: usize, _accounts_count: usize, _users_count: usize) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["aggregate".to_string(), "nats".to_string(), "security".to_string()],
        }
    }

    fn fold_aggregate_yubikey_provisioning(&self, name: &str, _version: u64, _devices_count: usize, _slots_provisioned: usize) -> Self::Output {
        SearchableText {
            fields: vec![name.to_string()],
            keywords: vec!["aggregate".to_string(), "yubikey".to_string(), "provisioning".to_string()],
        }
    }
}

/// Helper method on DomainNode for getting searchable text
impl DomainNode {
    /// Get searchable text using the FoldSearchableText catamorphism
    pub fn searchable_text(&self) -> SearchableText {
        self.fold(&FoldSearchableText)
    }
}

// ============================================================================
// PropertyUpdate - Mutation support for domain nodes
// ============================================================================

/// Property updates for mutable domain node types.
///
/// This structure captures all mutable properties that can be updated via
/// the PropertyCard UI. Using Option<T> allows partial updates - only
/// specified properties are changed.
///
/// ## Mutable Types and Their Properties
///
/// | Type | name | description | email | enabled | claims |
/// |------|------|-------------|-------|---------|--------|
/// | Organization | ✓ | ✓ | - | - | - |
/// | OrganizationUnit | ✓ | - | - | - | - |
/// | Person | ✓ | - | ✓ | ✓ (active) | - |
/// | Location | ✓ | - | - | - | - |
/// | Role | ✓ | ✓ | - | ✓ (active) | - |
/// | Policy | ✓ | ✓ | - | ✓ (enabled) | ✓ |
///
/// All other types (NATS, PKI, YubiKey, etc.) are read-only and ignore updates.
#[derive(Debug, Clone, Default)]
pub struct PropertyUpdate {
    /// New name for the entity
    pub name: Option<String>,
    /// New description (Organization, Role, Policy)
    pub description: Option<String>,
    /// New email (Person only)
    pub email: Option<String>,
    /// New enabled/active state (Person, Role, Policy)
    pub enabled: Option<bool>,
    /// New claims list (Policy only)
    pub claims: Option<Vec<PolicyClaim>>,
}

impl PropertyUpdate {
    /// Create a new empty property update
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name property
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description property
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the email property
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set the enabled/active property
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Set the claims property
    pub fn with_claims(mut self, claims: Vec<PolicyClaim>) -> Self {
        self.claims = Some(claims);
        self
    }

    /// Check if this update would change anything for a given injection type
    pub fn has_changes_for(&self, injection: Injection) -> bool {
        match injection {
            Injection::Organization => self.name.is_some() || self.description.is_some(),
            Injection::OrganizationUnit => self.name.is_some(),
            Injection::Person => self.name.is_some() || self.email.is_some() || self.enabled.is_some(),
            Injection::Location => self.name.is_some(),
            Injection::Role => self.name.is_some() || self.description.is_some() || self.enabled.is_some(),
            Injection::Policy => {
                self.name.is_some() || self.description.is_some() ||
                self.enabled.is_some() || self.claims.is_some()
            }
            // All other types are read-only
            _ => false,
        }
    }
}

/// Methods for updating domain node properties
impl DomainNode {
    /// Apply property updates to this domain node, returning a new updated node.
    ///
    /// This method respects the mutability of each domain type:
    /// - Mutable types (Organization, OrganizationUnit, Person, Location, Role, Policy)
    ///   apply relevant updates and return a new node
    /// - Read-only types (NATS, PKI, YubiKey, etc.) return a clone unchanged
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let update = PropertyUpdate::new()
    ///     .with_name("New Name")
    ///     .with_email("new@example.com");
    ///
    /// let updated_node = node.with_properties(&update);
    /// ```
    pub fn with_properties(&self, update: &PropertyUpdate) -> Self {
        match &self.data {
            // Mutable types - apply relevant updates
            DomainNodeData::Organization(org) => {
                let mut updated = org.clone();
                if let Some(name) = &update.name {
                    updated.name = name.clone();
                    updated.display_name = name.clone();
                }
                if let Some(description) = &update.description {
                    updated.description = Some(description.clone());
                }
                Self::inject_organization(updated)
            }

            DomainNodeData::OrganizationUnit(unit) => {
                let mut updated = unit.clone();
                if let Some(name) = &update.name {
                    updated.name = name.clone();
                }
                Self::inject_organization_unit(updated)
            }

            DomainNodeData::Person { person, role } => {
                let mut updated = person.clone();
                if let Some(name) = &update.name {
                    updated.name = name.clone();
                }
                if let Some(email) = &update.email {
                    updated.email = email.clone();
                }
                if let Some(enabled) = update.enabled {
                    updated.active = enabled;
                }
                Self::inject_person(updated, role.clone())
            }

            DomainNodeData::Location(loc) => {
                let mut updated = loc.clone();
                if let Some(name) = &update.name {
                    updated.name = name.clone();
                }
                Self::inject_location(updated)
            }

            DomainNodeData::Role(role) => {
                let mut updated = role.clone();
                if let Some(name) = &update.name {
                    updated.name = name.clone();
                }
                if let Some(description) = &update.description {
                    updated.description = description.clone();
                }
                if let Some(enabled) = update.enabled {
                    updated.active = enabled;
                }
                Self::inject_role(updated)
            }

            DomainNodeData::Policy(policy) => {
                let mut updated = policy.clone();
                if let Some(name) = &update.name {
                    updated.name = name.clone();
                }
                if let Some(description) = &update.description {
                    updated.description = description.clone();
                }
                if let Some(enabled) = update.enabled {
                    updated.enabled = enabled;
                }
                if let Some(claims) = &update.claims {
                    updated.claims = claims.clone();
                }
                Self::inject_policy(updated)
            }

            // Read-only types - return clone unchanged
            _ => self.clone(),
        }
    }

    /// Check if this node type is mutable (supports property updates)
    pub fn is_mutable(&self) -> bool {
        matches!(
            self.injection,
            Injection::Organization |
            Injection::OrganizationUnit |
            Injection::Person |
            Injection::Location |
            Injection::Role |
            Injection::Policy
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test folder that extracts the injection type
    struct FoldInjection;

    impl FoldDomainNode for FoldInjection {
        type Output = Injection;

        fn fold_person(&self, _: &Person, _: &KeyOwnerRole) -> Self::Output { Injection::Person }
        fn fold_organization(&self, _: &Organization) -> Self::Output { Injection::Organization }
        fn fold_organization_unit(&self, _: &OrganizationUnit) -> Self::Output { Injection::OrganizationUnit }
        fn fold_location(&self, _: &Location) -> Self::Output { Injection::Location }
        fn fold_role(&self, _: &Role) -> Self::Output { Injection::Role }
        fn fold_policy(&self, _: &Policy) -> Self::Output { Injection::Policy }
        fn fold_nats_operator(&self, _: &NatsIdentityProjection) -> Self::Output { Injection::NatsOperator }
        fn fold_nats_account(&self, _: &NatsIdentityProjection) -> Self::Output { Injection::NatsAccount }
        fn fold_nats_user(&self, _: &NatsIdentityProjection) -> Self::Output { Injection::NatsUser }
        fn fold_nats_service_account(&self, _: &NatsIdentityProjection) -> Self::Output { Injection::NatsServiceAccount }
        fn fold_nats_operator_simple(&self, _: &str, _: Option<Uuid>) -> Self::Output { Injection::NatsOperatorSimple }
        fn fold_nats_account_simple(&self, _: &str, _: Option<Uuid>, _: bool) -> Self::Output { Injection::NatsAccountSimple }
        fn fold_nats_user_simple(&self, _: &str, _: Option<Uuid>, _: &str) -> Self::Output { Injection::NatsUserSimple }
        fn fold_root_certificate(&self, _: Uuid, _: &str, _: &str, _: DateTime<Utc>, _: DateTime<Utc>, _: &[String]) -> Self::Output { Injection::RootCertificate }
        fn fold_intermediate_certificate(&self, _: Uuid, _: &str, _: &str, _: DateTime<Utc>, _: DateTime<Utc>, _: &[String]) -> Self::Output { Injection::IntermediateCertificate }
        fn fold_leaf_certificate(&self, _: Uuid, _: &str, _: &str, _: DateTime<Utc>, _: DateTime<Utc>, _: &[String], _: &[String]) -> Self::Output { Injection::LeafCertificate }
        fn fold_key(&self, _: Uuid, _: &KeyAlgorithm, _: &KeyPurpose, _: Option<DateTime<Utc>>) -> Self::Output { Injection::Key }
        fn fold_yubikey(&self, _: Uuid, _: &str, _: &str, _: Option<DateTime<Utc>>, _: &[String]) -> Self::Output { Injection::YubiKey }
        fn fold_piv_slot(&self, _: Uuid, _: &str, _: &str, _: bool, _: Option<&String>) -> Self::Output { Injection::PivSlot }
        fn fold_yubikey_status(&self, _: Uuid, _: Option<&String>, _: &[PIVSlot], _: &[PIVSlot]) -> Self::Output { Injection::YubiKeyStatus }
        fn fold_manifest(&self, _: Uuid, _: &str, _: Option<&std::path::PathBuf>, _: Option<&String>) -> Self::Output { Injection::Manifest }
        fn fold_policy_role(&self, _: Uuid, _: &str, _: &str, _: u8, _: SeparationClass, _: usize) -> Self::Output { Injection::PolicyRole }
        fn fold_policy_claim(&self, _: Uuid, _: &str, _: &str) -> Self::Output { Injection::PolicyClaim }
        fn fold_policy_category(&self, _: Uuid, _: &str, _: usize, _: bool) -> Self::Output { Injection::PolicyCategory }
        fn fold_policy_group(&self, _: Uuid, _: &str, _: SeparationClass, _: usize, _: bool) -> Self::Output { Injection::PolicyGroup }
        fn fold_aggregate_organization(&self, _: &str, _: u64, _: usize, _: usize) -> Self::Output { Injection::AggregateOrganization }
        fn fold_aggregate_pki_chain(&self, _: &str, _: u64, _: usize, _: usize) -> Self::Output { Injection::AggregatePkiChain }
        fn fold_aggregate_nats_security(&self, _: &str, _: u64, _: usize, _: usize, _: usize) -> Self::Output { Injection::AggregateNatsSecurity }
        fn fold_aggregate_yubikey_provisioning(&self, _: &str, _: u64, _: usize, _: usize) -> Self::Output { Injection::AggregateYubiKeyProvisioning }
    }

    #[test]
    fn test_injection_preserved() {
        // Test that fold ∘ inject_A = fold_A (categorical law)
        let node = DomainNode::inject_nats_operator_simple("test".to_string(), None);
        let result = node.fold(&FoldInjection);
        assert_eq!(result, Injection::NatsOperatorSimple);
        assert_eq!(node.injection(), Injection::NatsOperatorSimple);
    }

    #[test]
    fn test_injection_display() {
        assert_eq!(Injection::Person.to_string(), "Person");
        assert_eq!(Injection::NatsOperator.to_string(), "NATS Operator");
        assert_eq!(Injection::RootCertificate.to_string(), "Root Certificate");
    }
}
