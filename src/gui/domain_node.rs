// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Node Coproduct - Categorical replacement for NodeType enum
//!
//! This module implements a proper coproduct of domain types following
//! Applied Category Theory principles. Instead of a flat enum (NodeType),
//! we use:
//!
//! 1. **Injection functions**: `inject_person()`, `inject_organization()`, etc.
//! 2. **Universal property**: `FoldDomainNode` trait with `fold()` method
//! 3. **Type-safe projections**: Each injection preserves identity
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

use crate::domain::{
    Person, Organization, OrganizationUnit, Location, Policy, Role, KeyOwnerRole,
};
use crate::domain_projections::NatsIdentityProjection;
use crate::events::{KeyAlgorithm, KeyPurpose};
use crate::policy::SeparationClass;
use super::graph_yubikey::PIVSlot;

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
        }
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
#[derive(Debug, Clone)]
pub enum DomainNodeData {
    // Core Domain Entities
    Person { person: Person, role: KeyOwnerRole },
    Organization(Organization),
    OrganizationUnit(OrganizationUnit),
    Location(Location),
    Role(Role),
    Policy(Policy),

    // NATS Infrastructure (with projections)
    NatsOperator(NatsIdentityProjection),
    NatsAccount(NatsIdentityProjection),
    NatsUser(NatsIdentityProjection),
    NatsServiceAccount(NatsIdentityProjection),

    // NATS Infrastructure (simple visualization)
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

    // PKI Certificates
    RootCertificate {
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    },
    IntermediateCertificate {
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    },
    LeafCertificate {
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
        san: Vec<String>,
    },

    // Cryptographic Keys
    Key {
        key_id: Uuid,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    },

    // YubiKey Hardware
    YubiKey {
        device_id: Uuid,
        serial: String,
        version: String,
        provisioned_at: Option<DateTime<Utc>>,
        slots_used: Vec<String>,
    },
    PivSlot {
        slot_id: Uuid,
        slot_name: String,
        yubikey_serial: String,
        has_key: bool,
        certificate_subject: Option<String>,
    },
    YubiKeyStatus {
        person_id: Uuid,
        yubikey_serial: Option<String>,
        slots_provisioned: Vec<PIVSlot>,
        slots_needed: Vec<PIVSlot>,
    },

    // Export Manifest
    Manifest {
        manifest_id: Uuid,
        name: String,
        destination: Option<std::path::PathBuf>,
        checksum: Option<String>,
    },

    // Policy Roles and Claims
    PolicyRole {
        role_id: Uuid,
        name: String,
        purpose: String,
        level: u8,
        separation_class: SeparationClass,
        claim_count: usize,
    },
    PolicyClaim {
        claim_id: Uuid,
        name: String,
        category: String,
    },
    PolicyCategory {
        category_id: Uuid,
        name: String,
        claim_count: usize,
        expanded: bool,
    },
    PolicyGroup {
        class_id: Uuid,
        name: String,
        separation_class: SeparationClass,
        role_count: usize,
        expanded: bool,
    },
}

// ============================================================================
// Domain Node Coproduct
// ============================================================================

/// Domain Node - A proper coproduct of all domain entity types.
///
/// This replaces the `NodeType` enum with a categorical structure that:
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

    /// Injection ι_RootCertificate
    pub fn inject_root_certificate(
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    ) -> Self {
        Self {
            injection: Injection::RootCertificate,
            data: DomainNodeData::RootCertificate {
                cert_id, subject, issuer, not_before, not_after, key_usage,
            },
        }
    }

    /// Injection ι_IntermediateCertificate
    pub fn inject_intermediate_certificate(
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
    ) -> Self {
        Self {
            injection: Injection::IntermediateCertificate,
            data: DomainNodeData::IntermediateCertificate {
                cert_id, subject, issuer, not_before, not_after, key_usage,
            },
        }
    }

    /// Injection ι_LeafCertificate
    pub fn inject_leaf_certificate(
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        key_usage: Vec<String>,
        san: Vec<String>,
    ) -> Self {
        Self {
            injection: Injection::LeafCertificate,
            data: DomainNodeData::LeafCertificate {
                cert_id, subject, issuer, not_before, not_after, key_usage, san,
            },
        }
    }

    /// Injection ι_Key
    pub fn inject_key(
        key_id: Uuid,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            injection: Injection::Key,
            data: DomainNodeData::Key { key_id, algorithm, purpose, expires_at },
        }
    }

    /// Injection ι_YubiKey
    pub fn inject_yubikey(
        device_id: Uuid,
        serial: String,
        version: String,
        provisioned_at: Option<DateTime<Utc>>,
        slots_used: Vec<String>,
    ) -> Self {
        Self {
            injection: Injection::YubiKey,
            data: DomainNodeData::YubiKey {
                device_id, serial, version, provisioned_at, slots_used,
            },
        }
    }

    /// Injection ι_PivSlot
    pub fn inject_piv_slot(
        slot_id: Uuid,
        slot_name: String,
        yubikey_serial: String,
        has_key: bool,
        certificate_subject: Option<String>,
    ) -> Self {
        Self {
            injection: Injection::PivSlot,
            data: DomainNodeData::PivSlot {
                slot_id, slot_name, yubikey_serial, has_key, certificate_subject,
            },
        }
    }

    /// Injection ι_YubiKeyStatus
    pub fn inject_yubikey_status(
        person_id: Uuid,
        yubikey_serial: Option<String>,
        slots_provisioned: Vec<PIVSlot>,
        slots_needed: Vec<PIVSlot>,
    ) -> Self {
        Self {
            injection: Injection::YubiKeyStatus,
            data: DomainNodeData::YubiKeyStatus {
                person_id, yubikey_serial, slots_provisioned, slots_needed,
            },
        }
    }

    /// Injection ι_Manifest
    pub fn inject_manifest(
        manifest_id: Uuid,
        name: String,
        destination: Option<std::path::PathBuf>,
        checksum: Option<String>,
    ) -> Self {
        Self {
            injection: Injection::Manifest,
            data: DomainNodeData::Manifest { manifest_id, name, destination, checksum },
        }
    }

    /// Injection ι_PolicyRole
    pub fn inject_policy_role(
        role_id: Uuid,
        name: String,
        purpose: String,
        level: u8,
        separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self {
        Self {
            injection: Injection::PolicyRole,
            data: DomainNodeData::PolicyRole {
                role_id, name, purpose, level, separation_class, claim_count,
            },
        }
    }

    /// Injection ι_PolicyClaim
    pub fn inject_policy_claim(
        claim_id: Uuid,
        name: String,
        category: String,
    ) -> Self {
        Self {
            injection: Injection::PolicyClaim,
            data: DomainNodeData::PolicyClaim { claim_id, name, category },
        }
    }

    /// Injection ι_PolicyCategory
    pub fn inject_policy_category(
        category_id: Uuid,
        name: String,
        claim_count: usize,
        expanded: bool,
    ) -> Self {
        Self {
            injection: Injection::PolicyCategory,
            data: DomainNodeData::PolicyCategory { category_id, name, claim_count, expanded },
        }
    }

    /// Injection ι_PolicyGroup
    pub fn inject_policy_group(
        class_id: Uuid,
        name: String,
        separation_class: SeparationClass,
        role_count: usize,
        expanded: bool,
    ) -> Self {
        Self {
            injection: Injection::PolicyGroup,
            data: DomainNodeData::PolicyGroup {
                class_id, name, separation_class, role_count, expanded,
            },
        }
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
    // Conversion from NodeType (for gradual migration)
    // ========================================================================

    /// Convert from the legacy NodeType enum to DomainNode.
    ///
    /// This enables gradual migration - existing code using NodeType can
    /// be converted to DomainNode incrementally.
    pub fn from_node_type(node_type: &super::graph::NodeType) -> Self {
        use super::graph::NodeType;
        match node_type {
            NodeType::Person { person, role } => {
                Self::inject_person(person.clone(), role.clone())
            }
            NodeType::Organization(org) => {
                Self::inject_organization(org.clone())
            }
            NodeType::OrganizationalUnit(unit) => {
                Self::inject_organization_unit(unit.clone())
            }
            NodeType::Location(loc) => {
                Self::inject_location(loc.clone())
            }
            NodeType::Role(role) => {
                Self::inject_role(role.clone())
            }
            NodeType::Policy(policy) => {
                Self::inject_policy(policy.clone())
            }
            NodeType::NatsOperator(proj) => {
                Self::inject_nats_operator(proj.clone())
            }
            NodeType::NatsAccount(proj) => {
                Self::inject_nats_account(proj.clone())
            }
            NodeType::NatsUser(proj) => {
                Self::inject_nats_user(proj.clone())
            }
            NodeType::NatsServiceAccount(proj) => {
                Self::inject_nats_service_account(proj.clone())
            }
            NodeType::NatsOperatorSimple { name, organization_id } => {
                Self::inject_nats_operator_simple(name.clone(), *organization_id)
            }
            NodeType::NatsAccountSimple { name, unit_id, is_system } => {
                Self::inject_nats_account_simple(name.clone(), *unit_id, *is_system)
            }
            NodeType::NatsUserSimple { name, person_id, account_name } => {
                Self::inject_nats_user_simple(name.clone(), *person_id, account_name.clone())
            }
            NodeType::RootCertificate { cert_id, subject, issuer, not_before, not_after, key_usage } => {
                Self::inject_root_certificate(
                    *cert_id, subject.clone(), issuer.clone(),
                    *not_before, *not_after, key_usage.clone(),
                )
            }
            NodeType::IntermediateCertificate { cert_id, subject, issuer, not_before, not_after, key_usage } => {
                Self::inject_intermediate_certificate(
                    *cert_id, subject.clone(), issuer.clone(),
                    *not_before, *not_after, key_usage.clone(),
                )
            }
            NodeType::LeafCertificate { cert_id, subject, issuer, not_before, not_after, key_usage, san } => {
                Self::inject_leaf_certificate(
                    *cert_id, subject.clone(), issuer.clone(),
                    *not_before, *not_after, key_usage.clone(), san.clone(),
                )
            }
            NodeType::Key { key_id, algorithm, purpose, expires_at } => {
                Self::inject_key(*key_id, algorithm.clone(), purpose.clone(), *expires_at)
            }
            NodeType::YubiKey { device_id, serial, version, provisioned_at, slots_used } => {
                Self::inject_yubikey(
                    *device_id, serial.clone(), version.clone(),
                    *provisioned_at, slots_used.clone(),
                )
            }
            NodeType::PivSlot { slot_id, slot_name, yubikey_serial, has_key, certificate_subject } => {
                Self::inject_piv_slot(
                    *slot_id, slot_name.clone(), yubikey_serial.clone(),
                    *has_key, certificate_subject.clone(),
                )
            }
            NodeType::YubiKeyStatus { person_id, yubikey_serial, slots_provisioned, slots_needed } => {
                Self::inject_yubikey_status(
                    *person_id, yubikey_serial.clone(),
                    slots_provisioned.clone(), slots_needed.clone(),
                )
            }
            NodeType::Manifest { manifest_id, name, destination, checksum } => {
                Self::inject_manifest(
                    *manifest_id, name.clone(),
                    destination.clone(), checksum.clone(),
                )
            }
            NodeType::PolicyRole { role_id, name, purpose, level, separation_class, claim_count } => {
                Self::inject_policy_role(
                    *role_id, name.clone(), purpose.clone(),
                    *level, *separation_class, *claim_count,
                )
            }
            NodeType::PolicyClaim { claim_id, name, category } => {
                Self::inject_policy_claim(*claim_id, name.clone(), category.clone())
            }
            NodeType::PolicyCategory { category_id, name, claim_count, expanded } => {
                Self::inject_policy_category(*category_id, name.clone(), *claim_count, *expanded)
            }
            NodeType::PolicyGroup { class_id, name, separation_class, role_count, expanded } => {
                Self::inject_policy_group(
                    *class_id, name.clone(), *separation_class, *role_count, *expanded,
                )
            }
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

            DomainNodeData::RootCertificate { cert_id, subject, issuer, not_before, not_after, key_usage } =>
                folder.fold_root_certificate(*cert_id, subject, issuer, *not_before, *not_after, key_usage),
            DomainNodeData::IntermediateCertificate { cert_id, subject, issuer, not_before, not_after, key_usage } =>
                folder.fold_intermediate_certificate(*cert_id, subject, issuer, *not_before, *not_after, key_usage),
            DomainNodeData::LeafCertificate { cert_id, subject, issuer, not_before, not_after, key_usage, san } =>
                folder.fold_leaf_certificate(*cert_id, subject, issuer, *not_before, *not_after, key_usage, san),

            DomainNodeData::Key { key_id, algorithm, purpose, expires_at } =>
                folder.fold_key(*key_id, algorithm, purpose, *expires_at),

            DomainNodeData::YubiKey { device_id, serial, version, provisioned_at, slots_used } =>
                folder.fold_yubikey(*device_id, serial, version, *provisioned_at, slots_used),
            DomainNodeData::PivSlot { slot_id, slot_name, yubikey_serial, has_key, certificate_subject } =>
                folder.fold_piv_slot(*slot_id, slot_name, yubikey_serial, *has_key, certificate_subject.as_ref()),
            DomainNodeData::YubiKeyStatus { person_id, yubikey_serial, slots_provisioned, slots_needed } =>
                folder.fold_yubikey_status(*person_id, yubikey_serial.as_ref(), slots_provisioned, slots_needed),

            DomainNodeData::Manifest { manifest_id, name, destination, checksum } =>
                folder.fold_manifest(*manifest_id, name, destination.as_ref(), checksum.as_ref()),

            DomainNodeData::PolicyRole { role_id, name, purpose, level, separation_class, claim_count } =>
                folder.fold_policy_role(*role_id, name, purpose, *level, *separation_class, *claim_count),
            DomainNodeData::PolicyClaim { claim_id, name, category } =>
                folder.fold_policy_claim(*claim_id, name, category),
            DomainNodeData::PolicyCategory { category_id, name, claim_count, expanded } =>
                folder.fold_policy_category(*category_id, name, *claim_count, *expanded),
            DomainNodeData::PolicyGroup { class_id, name, separation_class, role_count, expanded } =>
                folder.fold_policy_group(*class_id, name, *separation_class, *role_count, *expanded),
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
            secondary_text: format!("@{}", account_name),
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
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.0, 0.6, 0.4), // Dark teal (root trust)
            primary_text: format!("Root CA: {}", subject),
            secondary_text: "Root Certificate".to_string(),
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
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.2, 0.8, 0.6), // Medium teal
            primary_text: format!("Intermediate CA: {}", subject),
            secondary_text: "Intermediate Certificate".to_string(),
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
        _not_after: DateTime<Utc>,
        _key_usage: &[String],
        _san: &[String],
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.4, 1.0, 0.8), // Light teal
            primary_text: format!("Certificate: {}", subject),
            secondary_text: "Leaf Certificate".to_string(),
            icon: crate::icons::ICON_VERIFIED,
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
        _expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.7, 0.5, 0.9), // Light purple
            primary_text: format!("{:?}", purpose),
            secondary_text: format!("{:?}", algorithm),
            icon: crate::icons::ICON_KEY,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
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
        VisualizationData {
            color: iced::Color::from_rgb(0.8, 0.3, 0.8), // Magenta (hardware)
            primary_text: format!("YubiKey {}", serial),
            secondary_text: "Hardware Token".to_string(),
            icon: crate::icons::ICON_USB,
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
        _certificate_subject: Option<&String>,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.9, 0.5, 0.9), // Light magenta (slot)
            primary_text: slot_name.to_string(),
            secondary_text: if has_key { "Key Present" } else { "Empty" }.to_string(),
            icon: crate::icons::ICON_MEMORY,
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
        let status = if yubikey_serial.is_some() {
            format!("{}/{} slots", slots_provisioned.len(), slots_needed.len())
        } else {
            "No YubiKey".to_string()
        };
        VisualizationData {
            color: iced::Color::from_rgb(0.6, 0.4, 0.7), // Purple
            primary_text: "YubiKey Status".to_string(),
            secondary_text: status,
            icon: crate::icons::ICON_USB,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_manifest(
        &self,
        _manifest_id: Uuid,
        name: &str,
        _destination: Option<&std::path::PathBuf>,
        _checksum: Option<&String>,
    ) -> Self::Output {
        VisualizationData {
            color: iced::Color::from_rgb(0.5, 0.7, 0.5), // Green
            primary_text: name.to_string(),
            secondary_text: "Export Manifest".to_string(),
            icon: crate::icons::ICON_DOWNLOAD,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: false,
            expanded: false,
        }
    }

    fn fold_policy_role(
        &self,
        _role_id: Uuid,
        name: &str,
        _purpose: &str,
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
            secondary_text: format!("Level {} • {} claims", level, claim_count),
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
            icon: crate::icons::ICON_CHECK_CIRCLE,
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
            icon: crate::icons::ICON_FOLDER,
            icon_font: crate::icons::MATERIAL_ICONS,
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
            icon: crate::icons::ICON_FOLDER,
            icon_font: crate::icons::MATERIAL_ICONS,
            expandable: true,
            expanded,
        }
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
