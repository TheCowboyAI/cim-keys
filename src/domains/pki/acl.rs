// Copyright (c) 2025 - Cowboy AI, LLC.

//! Anti-Corruption Layer for PKI Bounded Context
//!
//! This module provides the Anti-Corruption Layer (ACL) that translates
//! between the PKI bounded context and the Organization bounded context.
//!
//! # DDD Pattern: Anti-Corruption Layer
//!
//! An ACL prevents domain model pollution by translating between
//! bounded contexts. The PKI context uses its own internal types
//! (via the Published Language) instead of depending on Organization's
//! internal types directly.
//!
//! # Architecture
//!
//! ```text
//! Organization Context                    PKI Context
//! ┌──────────────────┐                   ┌──────────────────┐
//! │  Person          │                   │  KeyOwnership    │
//! │  Organization    │  ───ACL───────►   │  uses            │
//! │  (internal)      │                   │  PersonReference │
//! └──────────────────┘                   └──────────────────┘
//!                      OrgContextAdapter
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use cim_keys::domains::pki::acl::{OrgContextAdapter, OrgContextPort};
//!
//! // Adapter implementation for Organization context
//! struct OrgAdapter { /* repository handle */ }
//!
//! impl OrgContextPort for OrgAdapter {
//!     fn get_person(&self, id: Uuid) -> Option<PersonReference> {
//!         // Look up person, translate to reference
//!     }
//! }
//!
//! // PKI context uses the port, never the Organization types directly
//! let ownership = key_service.get_ownership(key_id, &org_adapter)?;
//! ```

use uuid::Uuid;
use crate::domains::organization::published::{
    OrganizationReference, PersonReference, LocationReference, RoleReference,
};

// ============================================================================
// ORGANIZATION CONTEXT PORT (Interface)
// ============================================================================

/// Port for accessing Organization context from PKI context.
///
/// This trait defines the interface that the PKI context uses to
/// interact with Organization context. The actual implementation
/// (adapter) is provided at the application layer.
///
/// # DDD Pattern: Ports and Adapters
///
/// The port defines WHAT we need from Organization context.
/// The adapter (implementation) defines HOW we get it.
pub trait OrgContextPort: Send + Sync {
    /// Look up a person by ID and return their reference.
    ///
    /// Returns None if person doesn't exist or is not accessible.
    fn get_person(&self, person_id: Uuid) -> Option<PersonReference>;

    /// Look up an organization by ID and return their reference.
    fn get_organization(&self, org_id: Uuid) -> Option<OrganizationReference>;

    /// Look up a location by ID and return their reference.
    ///
    /// Used for determining where keys are physically stored.
    fn get_location(&self, location_id: Uuid) -> Option<LocationReference>;

    /// Look up a role by ID and return their reference.
    ///
    /// Used for determining key ownership permissions.
    fn get_role(&self, role_id: Uuid) -> Option<RoleReference>;

    /// Check if a person is authorized for a specific key role.
    ///
    /// This is a domain-level authorization check that PKI context
    /// delegates to Organization context.
    fn person_has_role(&self, person_id: Uuid, role_name: &str) -> bool;

    /// Get all persons in an organization.
    ///
    /// Used for listing potential key owners.
    fn get_organization_members(&self, org_id: Uuid) -> Vec<PersonReference>;
}

// ============================================================================
// KEY OWNER CONTEXT (Domain concept using Published Language)
// ============================================================================

/// Key owner context using Published Language types.
///
/// This replaces direct usage of `Person` and `Organization` in PKI context
/// with reference types from the Published Language.
#[derive(Debug, Clone)]
pub struct KeyOwnerContext {
    /// Person who owns the key (via Published Language)
    pub owner: PersonReference,
    /// Organization the person belongs to (via Published Language)
    pub organization: OrganizationReference,
    /// Storage location if known (via Published Language)
    pub storage_location: Option<LocationReference>,
    /// Role that grants key ownership (via Published Language)
    pub role: Option<RoleReference>,
}

impl KeyOwnerContext {
    /// Create a new key owner context.
    pub fn new(owner: PersonReference, organization: OrganizationReference) -> Self {
        Self {
            owner,
            organization,
            storage_location: None,
            role: None,
        }
    }

    /// Set the storage location.
    pub fn with_location(mut self, location: LocationReference) -> Self {
        self.storage_location = Some(location);
        self
    }

    /// Set the role.
    pub fn with_role(mut self, role: RoleReference) -> Self {
        self.role = Some(role);
        self
    }

    /// Check if the owner is active.
    pub fn is_owner_active(&self) -> bool {
        self.owner.active
    }
}

// ============================================================================
// ORG CONTEXT ADAPTER (Default/Mock Implementation)
// ============================================================================

/// Mock adapter for testing and development.
///
/// This adapter provides a simple in-memory implementation of the
/// OrgContextPort trait for testing purposes.
#[derive(Debug, Default)]
pub struct MockOrgContextAdapter {
    persons: std::collections::HashMap<Uuid, PersonReference>,
    organizations: std::collections::HashMap<Uuid, OrganizationReference>,
    locations: std::collections::HashMap<Uuid, LocationReference>,
    roles: std::collections::HashMap<Uuid, RoleReference>,
    person_roles: std::collections::HashMap<(Uuid, String), bool>,
    org_members: std::collections::HashMap<Uuid, Vec<Uuid>>,
}

impl MockOrgContextAdapter {
    /// Create a new mock adapter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a person to the mock.
    pub fn with_person(mut self, person: PersonReference) -> Self {
        self.persons.insert(person.id, person);
        self
    }

    /// Add an organization to the mock.
    pub fn with_organization(mut self, org: OrganizationReference) -> Self {
        self.organizations.insert(org.id, org);
        self
    }

    /// Add a location to the mock.
    pub fn with_location(mut self, location: LocationReference) -> Self {
        self.locations.insert(location.id, location);
        self
    }

    /// Add a role to the mock.
    pub fn with_role(mut self, role: RoleReference) -> Self {
        self.roles.insert(role.id, role);
        self
    }

    /// Assign a role to a person.
    pub fn with_person_role(mut self, person_id: Uuid, role_name: &str) -> Self {
        self.person_roles.insert((person_id, role_name.to_string()), true);
        self
    }

    /// Add organization membership.
    pub fn with_org_member(mut self, org_id: Uuid, person_id: Uuid) -> Self {
        self.org_members.entry(org_id).or_default().push(person_id);
        self
    }
}

impl OrgContextPort for MockOrgContextAdapter {
    fn get_person(&self, person_id: Uuid) -> Option<PersonReference> {
        self.persons.get(&person_id).cloned()
    }

    fn get_organization(&self, org_id: Uuid) -> Option<OrganizationReference> {
        self.organizations.get(&org_id).cloned()
    }

    fn get_location(&self, location_id: Uuid) -> Option<LocationReference> {
        self.locations.get(&location_id).cloned()
    }

    fn get_role(&self, role_id: Uuid) -> Option<RoleReference> {
        self.roles.get(&role_id).cloned()
    }

    fn person_has_role(&self, person_id: Uuid, role_name: &str) -> bool {
        self.person_roles.get(&(person_id, role_name.to_string())).copied().unwrap_or(false)
    }

    fn get_organization_members(&self, org_id: Uuid) -> Vec<PersonReference> {
        self.org_members
            .get(&org_id)
            .map(|member_ids| {
                member_ids
                    .iter()
                    .filter_map(|id| self.persons.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ============================================================================
// TRANSLATION FUNCTIONS
// ============================================================================

/// Translate a Person domain entity to PersonReference.
///
/// This function is used at the boundary between Organization and PKI contexts.
/// It extracts only the information PKI context needs.
pub fn person_to_reference(
    id: Uuid,
    name: &str,
    email: &str,
    active: bool,
) -> PersonReference {
    PersonReference::new(id, name, email, active)
}

/// Translate an Organization domain entity to OrganizationReference.
pub fn organization_to_reference(
    id: Uuid,
    name: &str,
    display_name: &str,
) -> OrganizationReference {
    OrganizationReference::new(id, name, display_name)
}

/// Translate a Location domain entity to LocationReference.
pub fn location_to_reference(
    id: Uuid,
    name: &str,
    location_type: &str,
) -> LocationReference {
    LocationReference::new(id, name, location_type)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_adapter_get_person() {
        let person_id = Uuid::now_v7();
        let person = PersonReference::new(person_id, "Alice", "alice@example.com", true);

        let adapter = MockOrgContextAdapter::new()
            .with_person(person.clone());

        let retrieved = adapter.get_person(person_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().display_name, "Alice");
    }

    #[test]
    fn test_mock_adapter_get_organization() {
        let org_id = Uuid::now_v7();
        let org = OrganizationReference::new(org_id, "CowboyAI", "Cowboy AI, LLC");

        let adapter = MockOrgContextAdapter::new()
            .with_organization(org.clone());

        let retrieved = adapter.get_organization(org_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "CowboyAI");
    }

    #[test]
    fn test_mock_adapter_person_has_role() {
        let person_id = Uuid::now_v7();
        let person = PersonReference::new(person_id, "Bob", "bob@example.com", true);

        let adapter = MockOrgContextAdapter::new()
            .with_person(person)
            .with_person_role(person_id, "SecurityAdmin");

        assert!(adapter.person_has_role(person_id, "SecurityAdmin"));
        assert!(!adapter.person_has_role(person_id, "Developer"));
    }

    #[test]
    fn test_mock_adapter_org_members() {
        let org_id = Uuid::now_v7();
        let person1_id = Uuid::now_v7();
        let person2_id = Uuid::now_v7();

        let org = OrganizationReference::new(org_id, "TestOrg", "Test Organization");
        let person1 = PersonReference::new(person1_id, "Alice", "alice@test.com", true);
        let person2 = PersonReference::new(person2_id, "Bob", "bob@test.com", true);

        let adapter = MockOrgContextAdapter::new()
            .with_organization(org)
            .with_person(person1)
            .with_person(person2)
            .with_org_member(org_id, person1_id)
            .with_org_member(org_id, person2_id);

        let members = adapter.get_organization_members(org_id);
        assert_eq!(members.len(), 2);
    }

    #[test]
    fn test_key_owner_context_creation() {
        let owner = PersonReference::new(Uuid::now_v7(), "Alice", "alice@test.com", true);
        let org = OrganizationReference::new(Uuid::now_v7(), "TestOrg", "Test Organization");

        let context = KeyOwnerContext::new(owner.clone(), org);

        assert!(context.is_owner_active());
        assert!(context.storage_location.is_none());
        assert!(context.role.is_none());
    }

    #[test]
    fn test_key_owner_context_with_location() {
        let owner = PersonReference::new(Uuid::now_v7(), "Alice", "alice@test.com", true);
        let org = OrganizationReference::new(Uuid::now_v7(), "TestOrg", "Test Organization");
        let location = LocationReference::new(Uuid::now_v7(), "Headquarters", "PhysicalBuilding");

        let context = KeyOwnerContext::new(owner, org)
            .with_location(location);

        assert!(context.storage_location.is_some());
        assert_eq!(context.storage_location.unwrap().name, "Headquarters");
    }

    #[test]
    fn test_translation_functions() {
        let id = Uuid::now_v7();

        let person_ref = person_to_reference(id, "Test User", "test@example.com", true);
        assert_eq!(person_ref.id, id);
        assert_eq!(person_ref.display_name, "Test User");

        let org_ref = organization_to_reference(id, "TestOrg", "Test Organization");
        assert_eq!(org_ref.id, id);
        assert_eq!(org_ref.name, "TestOrg");

        let loc_ref = location_to_reference(id, "HQ", "Building");
        assert_eq!(loc_ref.id, id);
        assert_eq!(loc_ref.name, "HQ");
    }
}
