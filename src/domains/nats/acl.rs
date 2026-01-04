// Copyright (c) 2025 - Cowboy AI, LLC.

//! Anti-Corruption Layer for NATS Bounded Context
//!
//! This module provides the Anti-Corruption Layer (ACL) that translates
//! between the NATS bounded context and other contexts (Organization, PKI).
//!
//! # DDD Pattern: Anti-Corruption Layer
//!
//! An ACL prevents domain model pollution by translating between
//! bounded contexts. The NATS context uses its own internal types
//! and accesses other contexts through well-defined ports.
//!
//! # Architecture
//!
//! ```text
//! Organization Context           PKI Context              NATS Context
//! ┌──────────────────┐          ┌──────────────┐         ┌──────────────┐
//! │  Person          │          │  Key         │         │  NatsUser    │
//! │  OrgUnit         │ ─ACL─►   │  Certificate │ ─ACL─►  │  uses        │
//! │  (internal)      │          │  (internal)  │         │  References  │
//! └──────────────────┘          └──────────────┘         └──────────────┘
//!                PersonContextAdapter     PkiContextAdapter
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use cim_keys::domains::nats::acl::{PersonContextPort, PkiContextPort};
//!
//! // Configure adapters at application layer
//! let nats_service = NatsUserService::new(person_adapter, pki_adapter);
//!
//! // NATS context never imports Person or Key directly
//! let user = nats_service.create_user_for_person(person_id)?;
//! ```

use uuid::Uuid;
use crate::domains::organization::published::{
    PersonReference, OrganizationReference, OrganizationUnitReference, RoleReference,
};
use crate::domains::pki::published::{
    KeyReference, CertificateReference, KeyOwnershipReference,
};

// ============================================================================
// PERSON CONTEXT PORT (Interface for Organization context)
// ============================================================================

/// Port for accessing Person/Organization context from NATS context.
///
/// NATS users map to Organization persons, and NATS accounts map to
/// organizational units. This port provides that translation.
pub trait PersonContextPort: Send + Sync {
    /// Get person reference by ID.
    fn get_person(&self, person_id: Uuid) -> Option<PersonReference>;

    /// Get organization reference by ID.
    fn get_organization(&self, org_id: Uuid) -> Option<OrganizationReference>;

    /// Get organization unit reference by ID.
    ///
    /// NATS accounts map to organizational units.
    fn get_unit(&self, unit_id: Uuid) -> Option<OrganizationUnitReference>;

    /// Get person's primary role.
    ///
    /// Used to determine NATS user permissions.
    fn get_person_role(&self, person_id: Uuid) -> Option<RoleReference>;

    /// Get all persons in an organizational unit.
    ///
    /// Used to list potential NATS users for an account.
    fn get_unit_members(&self, unit_id: Uuid) -> Vec<PersonReference>;

    /// Check if person is authorized to manage NATS.
    fn can_manage_nats(&self, person_id: Uuid) -> bool;
}

// ============================================================================
// PKI CONTEXT PORT (Interface for PKI context)
// ============================================================================

/// Port for accessing PKI context from NATS context.
///
/// NATS users need signing keys for authentication. This port provides
/// access to PKI context without direct imports.
pub trait PkiContextPort: Send + Sync {
    /// Get the signing key for a person.
    ///
    /// NATS users authenticate with NKey signatures.
    fn get_person_signing_key(&self, person_id: Uuid) -> Option<KeyReference>;

    /// Get the TLS certificate for NATS server.
    fn get_server_certificate(&self, server_id: Uuid) -> Option<CertificateReference>;

    /// Get key ownership for a specific key.
    fn get_key_ownership(&self, key_id: Uuid) -> Option<KeyOwnershipReference>;

    /// Check if a key is valid for NATS authentication.
    fn is_key_valid_for_nats(&self, key_id: Uuid) -> bool;
}

// ============================================================================
// NATS USER CONTEXT (Domain concept using Published Language)
// ============================================================================

/// NATS user context using Published Language types.
///
/// This provides all the context needed to create or manage a NATS user
/// without importing from Organization or PKI contexts directly.
#[derive(Debug, Clone)]
pub struct NatsUserContext {
    /// Person this NATS user represents
    pub person: PersonReference,
    /// Organization the person belongs to
    pub organization: OrganizationReference,
    /// Organizational unit (maps to NATS account)
    pub unit: Option<OrganizationUnitReference>,
    /// Person's role (determines permissions)
    pub role: Option<RoleReference>,
    /// Signing key for authentication
    pub signing_key: Option<KeyReference>,
}

impl NatsUserContext {
    /// Create a new NATS user context.
    pub fn new(person: PersonReference, organization: OrganizationReference) -> Self {
        Self {
            person,
            organization,
            unit: None,
            role: None,
            signing_key: None,
        }
    }

    /// Set the organizational unit.
    pub fn with_unit(mut self, unit: OrganizationUnitReference) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Set the role.
    pub fn with_role(mut self, role: RoleReference) -> Self {
        self.role = Some(role);
        self
    }

    /// Set the signing key.
    pub fn with_signing_key(mut self, key: KeyReference) -> Self {
        self.signing_key = Some(key);
        self
    }

    /// Check if user has a signing key.
    pub fn has_signing_key(&self) -> bool {
        self.signing_key.is_some()
    }

    /// Check if user is active.
    pub fn is_active(&self) -> bool {
        self.person.active
    }

    /// Get suggested NATS account name from unit.
    pub fn suggested_account_name(&self) -> String {
        self.unit
            .as_ref()
            .map(|u| u.name.clone())
            .unwrap_or_else(|| self.organization.name.clone())
    }
}

// ============================================================================
// NATS ACCOUNT CONTEXT (Domain concept using Published Language)
// ============================================================================

/// NATS account context using Published Language types.
///
/// This provides all the context needed to create or manage a NATS account
/// mapped from an organizational unit.
#[derive(Debug, Clone)]
pub struct NatsAccountContext {
    /// Organizational unit this account represents
    pub unit: OrganizationUnitReference,
    /// Organization the unit belongs to
    pub organization: OrganizationReference,
    /// Members of this account (persons in the unit)
    pub members: Vec<PersonReference>,
    /// Whether this is a system account
    pub is_system: bool,
}

impl NatsAccountContext {
    /// Create a new NATS account context.
    pub fn new(unit: OrganizationUnitReference, organization: OrganizationReference) -> Self {
        Self {
            unit,
            organization,
            members: Vec::new(),
            is_system: false,
        }
    }

    /// Set members.
    pub fn with_members(mut self, members: Vec<PersonReference>) -> Self {
        self.members = members;
        self
    }

    /// Mark as system account.
    pub fn as_system(mut self) -> Self {
        self.is_system = true;
        self
    }

    /// Get suggested NATS account name.
    pub fn suggested_name(&self) -> String {
        self.unit.name.clone()
    }
}

// ============================================================================
// MOCK ADAPTERS FOR TESTING
// ============================================================================

/// Mock adapter for Person context.
#[derive(Debug, Default)]
pub struct MockPersonContextAdapter {
    persons: std::collections::HashMap<Uuid, PersonReference>,
    organizations: std::collections::HashMap<Uuid, OrganizationReference>,
    units: std::collections::HashMap<Uuid, OrganizationUnitReference>,
    person_roles: std::collections::HashMap<Uuid, RoleReference>,
    unit_members: std::collections::HashMap<Uuid, Vec<Uuid>>,
    nats_managers: std::collections::HashSet<Uuid>,
}

impl MockPersonContextAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_person(mut self, person: PersonReference) -> Self {
        self.persons.insert(person.id, person);
        self
    }

    pub fn with_organization(mut self, org: OrganizationReference) -> Self {
        self.organizations.insert(org.id, org);
        self
    }

    pub fn with_unit(mut self, unit: OrganizationUnitReference) -> Self {
        self.units.insert(unit.id, unit);
        self
    }

    pub fn with_person_role(mut self, person_id: Uuid, role: RoleReference) -> Self {
        self.person_roles.insert(person_id, role);
        self
    }

    pub fn with_unit_member(mut self, unit_id: Uuid, person_id: Uuid) -> Self {
        self.unit_members.entry(unit_id).or_default().push(person_id);
        self
    }

    pub fn with_nats_manager(mut self, person_id: Uuid) -> Self {
        self.nats_managers.insert(person_id);
        self
    }
}

impl PersonContextPort for MockPersonContextAdapter {
    fn get_person(&self, person_id: Uuid) -> Option<PersonReference> {
        self.persons.get(&person_id).cloned()
    }

    fn get_organization(&self, org_id: Uuid) -> Option<OrganizationReference> {
        self.organizations.get(&org_id).cloned()
    }

    fn get_unit(&self, unit_id: Uuid) -> Option<OrganizationUnitReference> {
        self.units.get(&unit_id).cloned()
    }

    fn get_person_role(&self, person_id: Uuid) -> Option<RoleReference> {
        self.person_roles.get(&person_id).cloned()
    }

    fn get_unit_members(&self, unit_id: Uuid) -> Vec<PersonReference> {
        self.unit_members
            .get(&unit_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.persons.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn can_manage_nats(&self, person_id: Uuid) -> bool {
        self.nats_managers.contains(&person_id)
    }
}

/// Mock adapter for PKI context.
#[derive(Debug, Default)]
pub struct MockPkiContextAdapter {
    person_keys: std::collections::HashMap<Uuid, KeyReference>,
    server_certs: std::collections::HashMap<Uuid, CertificateReference>,
    key_ownerships: std::collections::HashMap<Uuid, KeyOwnershipReference>,
    valid_nats_keys: std::collections::HashSet<Uuid>,
}

impl MockPkiContextAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_person_key(mut self, person_id: Uuid, key: KeyReference) -> Self {
        self.person_keys.insert(person_id, key);
        self
    }

    pub fn with_server_cert(mut self, server_id: Uuid, cert: CertificateReference) -> Self {
        self.server_certs.insert(server_id, cert);
        self
    }

    pub fn with_key_ownership(mut self, key_id: Uuid, ownership: KeyOwnershipReference) -> Self {
        self.key_ownerships.insert(key_id, ownership);
        self
    }

    pub fn with_valid_nats_key(mut self, key_id: Uuid) -> Self {
        self.valid_nats_keys.insert(key_id);
        self
    }
}

impl PkiContextPort for MockPkiContextAdapter {
    fn get_person_signing_key(&self, person_id: Uuid) -> Option<KeyReference> {
        self.person_keys.get(&person_id).cloned()
    }

    fn get_server_certificate(&self, server_id: Uuid) -> Option<CertificateReference> {
        self.server_certs.get(&server_id).cloned()
    }

    fn get_key_ownership(&self, key_id: Uuid) -> Option<KeyOwnershipReference> {
        self.key_ownerships.get(&key_id).cloned()
    }

    fn is_key_valid_for_nats(&self, key_id: Uuid) -> bool {
        self.valid_nats_keys.contains(&key_id)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_user_context_creation() {
        let person = PersonReference::new(Uuid::now_v7(), "Alice", "alice@test.com", true);
        let org = OrganizationReference::new(Uuid::now_v7(), "TestOrg", "Test Organization");

        let context = NatsUserContext::new(person.clone(), org);

        assert!(context.is_active());
        assert!(!context.has_signing_key());
    }

    #[test]
    fn test_nats_user_context_with_key() {
        let person = PersonReference::new(Uuid::now_v7(), "Alice", "alice@test.com", true);
        let org = OrganizationReference::new(Uuid::now_v7(), "TestOrg", "Test Organization");
        let key = KeyReference::new(Uuid::now_v7(), "Ed25519", "fingerprint", "Signing");

        let context = NatsUserContext::new(person, org)
            .with_signing_key(key);

        assert!(context.has_signing_key());
    }

    #[test]
    fn test_nats_account_context_creation() {
        let org_id = Uuid::now_v7();
        let unit = OrganizationUnitReference::new(
            Uuid::now_v7(),
            "Engineering",
            "Department",
            org_id,
        );
        let org = OrganizationReference::new(org_id, "TestOrg", "Test Organization");

        let context = NatsAccountContext::new(unit, org);

        assert_eq!(context.suggested_name(), "Engineering");
        assert!(!context.is_system);
    }

    #[test]
    fn test_mock_person_adapter_get_person() {
        let person_id = Uuid::now_v7();
        let person = PersonReference::new(person_id, "Bob", "bob@test.com", true);

        let adapter = MockPersonContextAdapter::new()
            .with_person(person.clone());

        let retrieved = adapter.get_person(person_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().display_name, "Bob");
    }

    #[test]
    fn test_mock_person_adapter_can_manage_nats() {
        let person_id = Uuid::now_v7();
        let person = PersonReference::new(person_id, "Admin", "admin@test.com", true);

        let adapter = MockPersonContextAdapter::new()
            .with_person(person)
            .with_nats_manager(person_id);

        assert!(adapter.can_manage_nats(person_id));
        assert!(!adapter.can_manage_nats(Uuid::now_v7())); // Other person
    }

    #[test]
    fn test_mock_pki_adapter_get_signing_key() {
        let person_id = Uuid::now_v7();
        let key_id = Uuid::now_v7();
        let key = KeyReference::new(key_id, "Ed25519", "SHA256:abc", "NatsAuth");

        let adapter = MockPkiContextAdapter::new()
            .with_person_key(person_id, key.clone())
            .with_valid_nats_key(key_id);

        let retrieved = adapter.get_person_signing_key(person_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().algorithm, "Ed25519");
        assert!(adapter.is_key_valid_for_nats(key_id));
    }

    #[test]
    fn test_nats_user_suggested_account_name() {
        let org_id = Uuid::now_v7();
        let person = PersonReference::new(Uuid::now_v7(), "Alice", "alice@test.com", true);
        let org = OrganizationReference::new(org_id, "TestOrg", "Test Organization");
        let unit = OrganizationUnitReference::new(Uuid::now_v7(), "DevOps", "Team", org_id);

        let context = NatsUserContext::new(person, org.clone())
            .with_unit(unit);

        assert_eq!(context.suggested_account_name(), "DevOps");

        // Without unit, falls back to org name
        let context_no_unit = NatsUserContext::new(
            PersonReference::new(Uuid::now_v7(), "Bob", "bob@test.com", true),
            org,
        );
        assert_eq!(context_no_unit.suggested_account_name(), "TestOrg");
    }
}
