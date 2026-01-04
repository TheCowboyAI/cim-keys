// Copyright (c) 2025 - Cowboy AI, LLC.

//! Context Boundary Tests
//!
//! These tests verify that bounded contexts communicate through
//! Published Language types and Anti-Corruption Layers, not through
//! direct imports of internal types.
//!
//! # Test Philosophy
//!
//! - Published Language types must be usable without importing internal types
//! - ACL adapters must work with mock implementations
//! - Context boundaries must be respected in API design

use uuid::Uuid;

// Import ONLY Published Language types (not internal domain types)
use cim_keys::domains::organization::published::{
    OrganizationReference, PersonReference, LocationReference, RoleReference,
    OrganizationUnitReference,
};
use cim_keys::domains::pki::published::{
    KeyReference, CertificateReference, KeyOwnershipReference, TrustChainReference,
};

// Import ACL ports and adapters
use cim_keys::domains::pki::acl::{
    OrgContextPort, MockOrgContextAdapter, KeyOwnerContext,
};
use cim_keys::domains::nats::acl::{
    PersonContextPort, PkiContextPort,
    MockPersonContextAdapter, MockPkiContextAdapter,
    NatsUserContext, NatsAccountContext,
};

// ============================================================================
// PUBLISHED LANGUAGE TESTS
// ============================================================================

mod published_language_tests {
    use super::*;

    #[test]
    fn organization_reference_is_self_contained() {
        // PersonReference can be created without importing Person
        let id = Uuid::now_v7();
        let person_ref = PersonReference::new(id, "Test User", "test@example.com", true);

        assert_eq!(person_ref.id, id);
        assert_eq!(person_ref.display_name, "Test User");
        assert_eq!(person_ref.email, "test@example.com");
        assert!(person_ref.active);
    }

    #[test]
    fn pki_reference_is_self_contained() {
        // KeyReference can be created without importing CryptographicKey
        let id = Uuid::now_v7();
        let key_ref = KeyReference::new(id, "Ed25519", "SHA256:fingerprint", "Signing");

        assert_eq!(key_ref.id, id);
        assert_eq!(key_ref.algorithm, "Ed25519");
    }

    #[test]
    fn published_types_serialize_independently() {
        // Published Language types must be serializable
        let person_ref = PersonReference::new(Uuid::now_v7(), "Alice", "alice@test.com", true);
        let json = serde_json::to_string(&person_ref).unwrap();
        let deserialized: PersonReference = serde_json::from_str(&json).unwrap();

        assert_eq!(person_ref.display_name, deserialized.display_name);
    }

    #[test]
    fn published_types_are_hashable() {
        use std::collections::HashSet;

        // Published Language types must be usable as HashMap keys
        let id = Uuid::now_v7();
        let ref1 = OrganizationReference::new(id, "Org1", "Organization One");
        let ref2 = OrganizationReference::new(id, "Org1", "Organization One");

        let mut set = HashSet::new();
        set.insert(ref1);
        assert!(set.contains(&ref2));
    }
}

// ============================================================================
// PKI ACL BOUNDARY TESTS
// ============================================================================

mod pki_acl_tests {
    use super::*;

    #[test]
    fn pki_can_access_organization_through_port() {
        // PKI context accesses Organization through OrgContextPort, not direct imports
        let person_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        let person = PersonReference::new(person_id, "Security Admin", "admin@test.com", true);
        let org = OrganizationReference::new(org_id, "TestOrg", "Test Organization");

        let adapter = MockOrgContextAdapter::new()
            .with_person(person.clone())
            .with_organization(org.clone())
            .with_person_role(person_id, "SecurityAdmin");

        // Use the port interface - no direct Organization imports needed
        let retrieved_person = adapter.get_person(person_id);
        assert!(retrieved_person.is_some());

        let retrieved_org = adapter.get_organization(org_id);
        assert!(retrieved_org.is_some());

        // Authorization check through port
        assert!(adapter.person_has_role(person_id, "SecurityAdmin"));
    }

    #[test]
    fn key_owner_context_uses_published_types() {
        // KeyOwnerContext is a domain concept in PKI that uses Published Language types
        let owner = PersonReference::new(Uuid::now_v7(), "Key Owner", "owner@test.com", true);
        let org = OrganizationReference::new(Uuid::now_v7(), "Org", "Organization");
        let location = LocationReference::new(Uuid::now_v7(), "Vault", "SecureStorage");
        let role = RoleReference::new(Uuid::now_v7(), "RootAuthority", 10);

        let context = KeyOwnerContext::new(owner, org)
            .with_location(location)
            .with_role(role);

        // All fields are Published Language types, not internal types
        assert!(context.is_owner_active());
        assert!(context.storage_location.is_some());
        assert!(context.role.is_some());
    }
}

// ============================================================================
// NATS ACL BOUNDARY TESTS
// ============================================================================

mod nats_acl_tests {
    use super::*;

    #[test]
    fn nats_accesses_organization_through_person_port() {
        // NATS context accesses Organization through PersonContextPort
        let person_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        let unit_id = Uuid::now_v7();

        let person = PersonReference::new(person_id, "NATS User", "user@test.com", true);
        let org = OrganizationReference::new(org_id, "TestOrg", "Test Organization");
        let unit = OrganizationUnitReference::new(unit_id, "Engineering", "Department", org_id);
        let role = RoleReference::new(Uuid::now_v7(), "Developer", 5);

        let adapter = MockPersonContextAdapter::new()
            .with_person(person.clone())
            .with_organization(org.clone())
            .with_unit(unit.clone())
            .with_person_role(person_id, role.clone())
            .with_unit_member(unit_id, person_id)
            .with_nats_manager(person_id);

        // Use port interface
        let retrieved = adapter.get_person(person_id);
        assert!(retrieved.is_some());

        let members = adapter.get_unit_members(unit_id);
        assert_eq!(members.len(), 1);

        assert!(adapter.can_manage_nats(person_id));
    }

    #[test]
    fn nats_accesses_pki_through_pki_port() {
        // NATS context accesses PKI through PkiContextPort
        let person_id = Uuid::now_v7();
        let key_id = Uuid::now_v7();
        let server_id = Uuid::now_v7();

        let key = KeyReference::new(key_id, "Ed25519", "fingerprint", "NatsAuth");
        let cert = CertificateReference::new(
            Uuid::now_v7(),
            "CN=nats.test.com",
            "Leaf",
            chrono::Utc::now() + chrono::Duration::days(365),
            true,
        );
        let ownership = KeyOwnershipReference::new(key_id, person_id, Uuid::now_v7(), "Developer");

        let adapter = MockPkiContextAdapter::new()
            .with_person_key(person_id, key.clone())
            .with_server_cert(server_id, cert.clone())
            .with_key_ownership(key_id, ownership)
            .with_valid_nats_key(key_id);

        // Use port interface
        let retrieved_key = adapter.get_person_signing_key(person_id);
        assert!(retrieved_key.is_some());
        assert_eq!(retrieved_key.unwrap().algorithm, "Ed25519");

        let retrieved_cert = adapter.get_server_certificate(server_id);
        assert!(retrieved_cert.is_some());

        assert!(adapter.is_key_valid_for_nats(key_id));
    }

    #[test]
    fn nats_user_context_uses_published_types() {
        // NatsUserContext combines data from multiple contexts using Published Language
        let person = PersonReference::new(Uuid::now_v7(), "NATS User", "user@test.com", true);
        let org = OrganizationReference::new(Uuid::now_v7(), "Org", "Organization");
        let unit = OrganizationUnitReference::new(Uuid::now_v7(), "DevOps", "Team", org.id);
        let role = RoleReference::new(Uuid::now_v7(), "Operator", 7);
        let key = KeyReference::new(Uuid::now_v7(), "Ed25519", "fp", "NatsAuth");

        let context = NatsUserContext::new(person, org)
            .with_unit(unit)
            .with_role(role)
            .with_signing_key(key);

        // All fields are Published Language types
        assert!(context.is_active());
        assert!(context.has_signing_key());
        assert_eq!(context.suggested_account_name(), "DevOps");
    }

    #[test]
    fn nats_account_context_maps_from_org_unit() {
        // NATS accounts map to organizational units using Published Language
        let org_id = Uuid::now_v7();
        let unit = OrganizationUnitReference::new(Uuid::now_v7(), "Engineering", "Department", org_id);
        let org = OrganizationReference::new(org_id, "Org", "Organization");

        let member1 = PersonReference::new(Uuid::now_v7(), "Alice", "alice@test.com", true);
        let member2 = PersonReference::new(Uuid::now_v7(), "Bob", "bob@test.com", true);

        let context = NatsAccountContext::new(unit, org)
            .with_members(vec![member1, member2]);

        assert_eq!(context.suggested_name(), "Engineering");
        assert_eq!(context.members.len(), 2);
        assert!(!context.is_system);
    }
}

// ============================================================================
// CROSS-CONTEXT WORKFLOW TESTS
// ============================================================================

mod cross_context_workflow_tests {
    use super::*;

    #[test]
    fn complete_nats_user_creation_workflow() {
        // This test demonstrates a complete workflow that crosses
        // Organization → PKI → NATS contexts using only Published Language

        // Setup: Organization context data (via adapters)
        let person_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        let unit_id = Uuid::now_v7();

        let person = PersonReference::new(person_id, "Alice Developer", "alice@cowboyai.com", true);
        let org = OrganizationReference::new(org_id, "CowboyAI", "Cowboy AI, LLC");
        let unit = OrganizationUnitReference::new(unit_id, "Platform", "Team", org_id);
        let role = RoleReference::new(Uuid::now_v7(), "Developer", 5);

        // Setup: PKI context data (via adapters)
        let key_id = Uuid::now_v7();
        let key = KeyReference::new(key_id, "Ed25519", "SHA256:abc123", "NatsAuth");

        // Wire up adapters (this would be done at application layer)
        let person_adapter = MockPersonContextAdapter::new()
            .with_person(person.clone())
            .with_organization(org.clone())
            .with_unit(unit.clone())
            .with_person_role(person_id, role.clone())
            .with_unit_member(unit_id, person_id);

        let pki_adapter = MockPkiContextAdapter::new()
            .with_person_key(person_id, key.clone())
            .with_valid_nats_key(key_id);

        // Workflow: Create NATS user for person
        // Step 1: Get person from Organization context
        let person_ref = person_adapter.get_person(person_id).expect("Person exists");
        let org_ref = person_adapter.get_organization(org_id).expect("Org exists");
        let unit_ref = person_adapter.get_unit(unit_id);
        let role_ref = person_adapter.get_person_role(person_id);

        // Step 2: Get signing key from PKI context
        let signing_key = pki_adapter.get_person_signing_key(person_id).expect("Key exists");
        assert!(pki_adapter.is_key_valid_for_nats(signing_key.id));

        // Step 3: Build NATS user context
        let mut nats_context = NatsUserContext::new(person_ref, org_ref);
        if let Some(unit) = unit_ref {
            nats_context = nats_context.with_unit(unit);
        }
        if let Some(role) = role_ref {
            nats_context = nats_context.with_role(role);
        }
        nats_context = nats_context.with_signing_key(signing_key);

        // Verify: All data flows through Published Language types
        assert!(nats_context.is_active());
        assert!(nats_context.has_signing_key());
        assert_eq!(nats_context.suggested_account_name(), "Platform");
        assert_eq!(nats_context.person.display_name, "Alice Developer");
    }

    #[test]
    fn trust_chain_reference_flows_between_contexts() {
        // Trust chains can be referenced across contexts
        let root_id = Uuid::now_v7();
        let intermediate_id = Uuid::now_v7();

        let chain = TrustChainReference::new(
            root_id,
            vec![intermediate_id],
            true,
        );

        // Chain reference contains only IDs - no internal Certificate imports
        assert!(!chain.root_cert_id.is_nil());
        assert_eq!(chain.intermediate_cert_ids.len(), 1);
        assert!(chain.is_valid);
    }
}
