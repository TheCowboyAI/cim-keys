// Copyright (c) 2025 - Cowboy AI, LLC.

//! Anti-Corruption Layer (ACL) for Bounded Context Integration
//!
//! This module provides adapters that translate between bounded contexts
//! using the Published Language types. ACLs prevent context leakage by:
//!
//! 1. **Translating** - Converting internal types to published references
//! 2. **Validating** - Ensuring cross-context references are valid
//! 3. **Isolating** - Preventing direct dependencies between contexts
//!
//! ## Context Map
//!
//! ```text
//! Organization Context                PKI Context
//! ┌─────────────────┐                ┌─────────────────┐
//! │   Person        │───translate───▶│   PersonRef     │
//! │   Organization  │───translate───▶│   OrgRef        │
//! └─────────────────┘                └─────────────────┘
//!         │                                  ▲
//!         │                                  │
//!         └──────────[OrgContextAdapter]─────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cim_keys::shared_kernel::acl::{OrgContextAdapter, PersonContextAdapter};
//!
//! // PKI context uses adapter to get person reference
//! let person_ref = OrgContextAdapter::person_ref(&person);
//!
//! // NATS context uses adapter to get person reference
//! let person_ref = PersonContextAdapter::to_ref(&person);
//! ```

use super::published::{
    OrganizationRef, PersonRef, LocationRef, UnitRef, RoleRef,
    KeyRef, CertificateRef,
    OperatorRef, AccountRef, UserRef,
    DeviceRef, SlotRef,
};

// Import AggregateRoot trait for Location.id() method
use cim_domain::AggregateRoot;

// ============================================================================
// ORGANIZATION CONTEXT ADAPTER
// ============================================================================

/// Anti-Corruption Layer adapter for Organization Context.
///
/// Used by downstream contexts (PKI, NATS, YubiKey) to obtain
/// references to Organization entities without direct dependencies.
pub trait OrgContextAdapter {
    /// Convert to organization reference.
    fn to_org_ref(&self) -> OrganizationRef;
}

/// Extension trait for Person entities.
pub trait PersonAdapter {
    /// Convert to person reference.
    fn to_person_ref(&self) -> PersonRef;
}

/// Extension trait for Location entities.
pub trait LocationAdapter {
    /// Convert to location reference.
    fn to_location_ref(&self) -> LocationRef;
}

/// Extension trait for OrganizationUnit entities.
pub trait UnitAdapter {
    /// Convert to unit reference.
    fn to_unit_ref(&self) -> UnitRef;
}

/// Extension trait for Role entities.
pub trait RoleAdapter {
    /// Convert to role reference.
    fn to_role_ref(&self) -> RoleRef;
}

// ============================================================================
// PKI CONTEXT ADAPTER
// ============================================================================

/// Anti-Corruption Layer adapter for PKI Context.
///
/// Used by downstream contexts to obtain references to
/// cryptographic keys and certificates.
pub trait KeyAdapter {
    /// Convert to key reference.
    fn to_key_ref(&self) -> KeyRef;
}

/// Extension trait for Certificate entities.
pub trait CertificateAdapter {
    /// Convert to certificate reference.
    fn to_cert_ref(&self) -> CertificateRef;
}

// ============================================================================
// NATS CONTEXT ADAPTER
// ============================================================================

/// Anti-Corruption Layer adapter for NATS Context.
pub trait OperatorAdapter {
    /// Convert to operator reference.
    fn to_operator_ref(&self) -> OperatorRef;
}

/// Extension trait for Account entities.
pub trait AccountAdapter {
    /// Convert to account reference.
    fn to_account_ref(&self) -> AccountRef;
}

/// Extension trait for User entities.
pub trait NatsUserAdapter {
    /// Convert to user reference.
    fn to_user_ref(&self) -> UserRef;
}

// ============================================================================
// YUBIKEY CONTEXT ADAPTER
// ============================================================================

/// Anti-Corruption Layer adapter for YubiKey Context.
pub trait DeviceAdapter {
    /// Convert to device reference.
    fn to_device_ref(&self) -> DeviceRef;
}

/// Extension trait for Slot entities.
pub trait SlotAdapter {
    /// Convert to slot reference.
    fn to_slot_ref(&self) -> SlotRef;
}

// ============================================================================
// IMPLEMENTATIONS FOR DOMAIN TYPES
// ============================================================================

// These implementations allow domain types to be converted to references.
// By implementing the adapter traits on domain types, we create a clean
// translation layer between contexts.

use crate::domain::{Person, Organization, OrganizationUnit, Location, Role};

impl OrgContextAdapter for Organization {
    fn to_org_ref(&self) -> OrganizationRef {
        OrganizationRef::new(self.id.as_uuid(), &self.name)
            .with_display_name(&self.display_name)
    }
}

impl PersonAdapter for Person {
    fn to_person_ref(&self) -> PersonRef {
        PersonRef::new(self.id.as_uuid(), &self.name, &self.email)
    }
}

impl LocationAdapter for Location {
    fn to_location_ref(&self) -> LocationRef {
        // Location is from cim_domain_location, uses AggregateRoot trait
        LocationRef::new(
            self.id().into(),  // EntityId -> Uuid via From impl
            &self.name,
            format!("{:?}", self.location_type),
        )
    }
}

// NOTE: UnitAdapter for OrganizationUnit is NOT implemented here because
// OrganizationUnit doesn't carry organization_id (it's embedded in Organization).
// Use the explicit factory method below when you have both unit and org context.

/// Create a UnitRef from an OrganizationUnit with explicit organization context.
///
/// OrganizationUnit in bootstrap config doesn't have organization_id because
/// units are embedded in Organization. Use this when you have both pieces.
pub fn unit_to_ref(unit: &OrganizationUnit, organization_id: uuid::Uuid) -> UnitRef {
    UnitRef::new(unit.id.as_uuid(), &unit.name, organization_id)
}

// NOTE: RoleAdapter for Role is NOT implemented here because bootstrap Role
// doesn't have a level field. Use the explicit factory method below.

/// Create a RoleRef from a bootstrap Role with explicit level.
///
/// The bootstrap Role type doesn't have a level field (that's in policy::Role).
/// Use this when you need to specify the level explicitly.
pub fn role_to_ref(role: &Role, level: u8) -> RoleRef {
    RoleRef::new(role.id.as_uuid(), &role.name, level)
}

// ============================================================================
// KEY OWNERSHIP - REFACTORED TO USE REFERENCES
// ============================================================================

/// Key ownership using Published Language references.
///
/// This replaces the old pattern of direct Uuid references with
/// type-safe references that preserve context boundaries.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyOwnership {
    /// The key being owned
    pub key: KeyRef,
    /// The person who owns the key
    pub owner: PersonRef,
    /// The organization context
    pub organization: OrganizationRef,
    /// Storage location (if any)
    pub storage_location: Option<LocationRef>,
}

impl KeyOwnership {
    /// Create new key ownership record.
    pub fn new(key: KeyRef, owner: PersonRef, organization: OrganizationRef) -> Self {
        Self {
            key,
            owner,
            organization,
            storage_location: None,
        }
    }

    /// Add storage location.
    pub fn stored_at(mut self, location: LocationRef) -> Self {
        self.storage_location = Some(location);
        self
    }
}

// ============================================================================
// NATS USER MAPPING - REFACTORED TO USE REFERENCES
// ============================================================================

/// NATS user to person mapping using Published Language references.
///
/// This replaces direct cross-context imports with clean references.
#[derive(Debug, Clone, PartialEq)]
pub struct NatsUserMapping {
    /// The NATS user
    pub user: UserRef,
    /// The associated person
    pub person: PersonRef,
    /// The NATS account
    pub account: AccountRef,
}

impl NatsUserMapping {
    /// Create new user-person mapping.
    pub fn new(user: UserRef, person: PersonRef, account: AccountRef) -> Self {
        Self { user, person, account }
    }
}

// ============================================================================
// YUBIKEY ASSIGNMENT - REFACTORED TO USE REFERENCES
// ============================================================================

/// YubiKey device assignment using Published Language references.
#[derive(Debug, Clone, PartialEq)]
pub struct YubiKeyAssignment {
    /// The YubiKey device
    pub device: DeviceRef,
    /// The assigned person
    pub owner: PersonRef,
    /// Keys stored in slots
    pub slot_keys: Vec<(SlotRef, KeyRef)>,
}

impl YubiKeyAssignment {
    /// Create new device assignment.
    pub fn new(device: DeviceRef, owner: PersonRef) -> Self {
        Self {
            device,
            owner,
            slot_keys: Vec::new(),
        }
    }

    /// Add a key in a slot.
    pub fn with_slot_key(mut self, slot: SlotRef, key: KeyRef) -> Self {
        self.slot_keys.push((slot, key));
        self
    }
}

// ============================================================================
// CERTIFICATE CHAIN - REFACTORED TO USE REFERENCES
// ============================================================================

/// Certificate chain using Published Language references.
#[derive(Debug, Clone, PartialEq)]
pub struct CertificateChain {
    /// Root CA
    pub root: CertificateRef,
    /// Intermediate CAs (ordered from root to leaf)
    pub intermediates: Vec<CertificateRef>,
    /// Leaf certificate
    pub leaf: Option<CertificateRef>,
    /// Organization that owns this chain
    pub organization: OrganizationRef,
}

impl CertificateChain {
    /// Create a new certificate chain starting with root.
    pub fn new(root: CertificateRef, organization: OrganizationRef) -> Self {
        Self {
            root,
            intermediates: Vec::new(),
            leaf: None,
            organization,
        }
    }

    /// Add an intermediate CA.
    pub fn with_intermediate(mut self, intermediate: CertificateRef) -> Self {
        self.intermediates.push(intermediate);
        self
    }

    /// Set the leaf certificate.
    pub fn with_leaf(mut self, leaf: CertificateRef) -> Self {
        self.leaf = Some(leaf);
        self
    }

    /// Get the full chain length.
    pub fn chain_length(&self) -> usize {
        1 + self.intermediates.len() + if self.leaf.is_some() { 1 } else { 0 }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_person_ref() -> PersonRef {
        PersonRef::new(Uuid::now_v7(), "John Doe", "john@example.com")
    }

    fn sample_org_ref() -> OrganizationRef {
        OrganizationRef::new(Uuid::now_v7(), "CowboyAI")
    }

    fn sample_key_ref() -> KeyRef {
        KeyRef::new(Uuid::now_v7(), "Ed25519", "SHA256:abc123")
    }

    fn sample_cert_ref() -> CertificateRef {
        CertificateRef::new(
            Uuid::now_v7(),
            "CN=Root CA",
            "Root",
            Utc::now() + chrono::Duration::days(365),
            "fingerprint123",
        )
    }

    #[test]
    fn test_key_ownership() {
        let key = sample_key_ref();
        let owner = sample_person_ref();
        let org = sample_org_ref();

        let ownership = KeyOwnership::new(key.clone(), owner.clone(), org.clone());

        assert_eq!(ownership.key, key);
        assert_eq!(ownership.owner, owner);
        assert_eq!(ownership.organization, org);
        assert!(ownership.storage_location.is_none());
    }

    #[test]
    fn test_key_ownership_with_location() {
        let key = sample_key_ref();
        let owner = sample_person_ref();
        let org = sample_org_ref();
        let location = LocationRef::new(Uuid::now_v7(), "Safe Room", "Physical");

        let ownership = KeyOwnership::new(key, owner, org)
            .stored_at(location.clone());

        assert_eq!(ownership.storage_location, Some(location));
    }

    #[test]
    fn test_nats_user_mapping() {
        let account_id = Uuid::now_v7();
        let user = UserRef::new(Uuid::now_v7(), "jdoe", account_id);
        let person = sample_person_ref();
        let account = AccountRef::new(
            account_id,
            "engineering",
            Uuid::now_v7(),
            "AKEY123",
        );

        let mapping = NatsUserMapping::new(user.clone(), person.clone(), account.clone());

        assert_eq!(mapping.user, user);
        assert_eq!(mapping.person, person);
        assert_eq!(mapping.account, account);
    }

    #[test]
    fn test_certificate_chain() {
        let org = sample_org_ref();
        let root = sample_cert_ref();
        let intermediate = CertificateRef::new(
            Uuid::now_v7(),
            "CN=Intermediate CA",
            "Intermediate",
            Utc::now() + chrono::Duration::days(180),
            "inter123",
        ).with_issuer(root.id);

        let chain = CertificateChain::new(root, org)
            .with_intermediate(intermediate);

        assert_eq!(chain.chain_length(), 2);
        assert!(chain.leaf.is_none());
    }

    #[test]
    fn test_yubikey_assignment() {
        let device = DeviceRef::new(Uuid::now_v7(), "12345678");
        let owner = sample_person_ref();
        let slot = SlotRef::new(Uuid::now_v7(), "9A", device.id);
        let key = sample_key_ref();

        let assignment = YubiKeyAssignment::new(device.clone(), owner.clone())
            .with_slot_key(slot.clone(), key.clone());

        assert_eq!(assignment.device, device);
        assert_eq!(assignment.owner, owner);
        assert_eq!(assignment.slot_keys.len(), 1);
    }

    #[test]
    fn test_org_adapter() {
        // Use the new() constructor - ID auto-generated
        let org = Organization::new("TestOrg", "Test Organization");

        let org_ref = org.to_org_ref();

        assert_eq!(org_ref.name, "TestOrg");
        assert_eq!(org_ref.display_name, Some("Test Organization".to_string()));
    }

    #[test]
    fn test_person_adapter() {
        // Create org first, then person
        let org = Organization::new("TestOrg", "Test Org");
        let person = Person::new("Jane Doe", "jane@example.com", org.id);

        let person_ref = person.to_person_ref();

        assert_eq!(person_ref.display_name, "Jane Doe");
        assert_eq!(person_ref.email, "jane@example.com");
    }

    #[test]
    fn test_unit_to_ref_helper() {
        use crate::domain::bootstrap::OrganizationUnitType;

        let org = Organization::new("TestOrg", "Test Org");
        let unit = OrganizationUnit::new("Engineering", OrganizationUnitType::Department);

        let unit_ref = unit_to_ref(&unit, org.id.as_uuid());

        assert_eq!(unit_ref.name, "Engineering");
        assert_eq!(unit_ref.organization_id, org.id.as_uuid());
    }

    #[test]
    fn test_role_to_ref_helper() {
        let org = Organization::new("TestOrg", "Test Org");
        let creator = Person::new("Admin", "admin@test.com", org.id);
        let role = Role::new("Administrator", "System administrator", org.id, creator.id);

        let role_ref = role_to_ref(&role, 3);

        assert_eq!(role_ref.name, "Administrator");
        assert_eq!(role_ref.level, 3);
    }
}
