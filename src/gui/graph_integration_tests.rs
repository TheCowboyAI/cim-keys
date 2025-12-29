//! Integration tests for graph-first infrastructure generation
//!
//! These tests verify the complete user stories for generating infrastructure
//! from organizational graphs.

#[cfg(test)]
mod graph_first_integration_tests {
    use crate::domain::{Organization, OrganizationUnit, Person, KeyOwnerRole, OrganizationUnitType};
    use crate::gui::graph::{OrganizationConcept, NodeType};
    use crate::gui::{graph_pki, graph_nats, graph_yubikey};
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// Helper to create a test organization
    fn create_test_org(name: &str) -> Organization {
        Organization {
            id: Uuid::now_v7(),
            name: name.to_string(),
            display_name: format!("{} Corp", name),
            description: Some(format!("Test organization {}", name)),
            parent_id: None,
            units: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Helper to create a test organizational unit
    fn create_test_unit(name: &str, unit_type: OrganizationUnitType) -> OrganizationUnit {
        OrganizationUnit {
            id: Uuid::now_v7(),
            name: name.to_string(),
            unit_type,
            parent_unit_id: None,
            responsible_person_id: None,
            nats_account_name: None,
        }
    }

    /// Helper to create a test person
    fn create_test_person(name: &str, email: &str, org_id: Uuid) -> Person {
        Person {
            id: Uuid::now_v7(),
            name: name.to_string(),
            email: email.to_string(),
            roles: vec![],
            organization_id: org_id,
            unit_ids: vec![],
            active: true,
            nats_permissions: None,
            owner_id: None,
        }
    }

    #[test]
    fn test_user_story_generate_pki_from_simple_org() {
        // User Story: As a user, I want to generate PKI from my organizational graph
        // so that certificates follow my org structure
        //
        // Given: An organization with one unit and one person
        let mut graph = OrganizationConcept::new();

        let org = create_test_org("TestCorp");
        let org_id = org.id;
        graph.add_organization_node(org);

        let unit = create_test_unit("Engineering", OrganizationUnitType::Department);
        graph.add_org_unit_node(unit);

        let person = create_test_person("Alice Engineer", "alice@test.com", org_id);
        graph.add_node(person, KeyOwnerRole::Developer);

        // When: I generate PKI from the graph
        let result = graph_pki::generate_pki_from_graph(&graph, "test-passphrase");

        // Then: PKI certificates should be generated successfully
        assert!(result.is_ok(), "PKI generation should succeed");
        let cert_nodes = result.unwrap();

        // And: We should have certificates for org (root), unit (intermediate), and person (leaf)
        assert!(cert_nodes.len() >= 3, "Should generate at least 3 certificates");

        // And: Root CA should have no parent
        let root_certs = cert_nodes.iter()
            .filter(|(_, parent)| parent.is_none())
            .count();
        assert_eq!(root_certs, 1, "Should have exactly one root CA");
    }

    #[test]
    fn test_user_story_generate_nats_from_simple_org() {
        // User Story: As a user, I want to generate NATS infrastructure from my organizational graph
        // so that authentication follows my org structure
        //
        // Given: An organization with one unit and one person
        let mut graph = OrganizationConcept::new();

        let org = create_test_org("TestCorp");
        let org_id = org.id;
        graph.add_organization_node(org);

        let unit = create_test_unit("Engineering", OrganizationUnitType::Department);
        graph.add_org_unit_node(unit);

        let person = create_test_person("Bob Developer", "bob@test.com", org_id);
        graph.add_node(person, KeyOwnerRole::Developer);

        // When: I generate NATS infrastructure from the graph
        let result = graph_nats::generate_nats_from_graph(&graph);

        // Then: NATS entities should be generated successfully
        assert!(result.is_ok(), "NATS generation should succeed");
        let nats_nodes = result.unwrap();

        // And: We should have NATS entities for org (operator), unit (account), and person (user)
        assert!(nats_nodes.len() >= 3, "Should generate at least 3 NATS entities");

        // And: Operator should have no parent (it's the root)
        let operators = nats_nodes.iter()
            .filter(|(_, parent)| parent.is_none())
            .count();
        assert_eq!(operators, 1, "Should have exactly one NATS Operator");
    }

    #[test]
    fn test_user_story_analyze_yubikey_requirements() {
        // User Story: As a user, I want to analyze YubiKey requirements from my organizational graph
        // so I know which hardware tokens to provision
        //
        // Given: An organization with people in different roles
        let mut graph = OrganizationConcept::new();

        let org = create_test_org("TestCorp");
        let org_id = org.id;
        graph.add_organization_node(org);

        // Root authority needs signature slot
        let root_person = create_test_person("Root Admin", "root@test.com", org_id);
        graph.add_node(root_person, KeyOwnerRole::RootAuthority);

        // Security admin needs all slots
        let security_person = create_test_person("Security Admin", "security@test.com", org_id);
        graph.add_node(security_person, KeyOwnerRole::SecurityAdmin);

        // Developer needs authentication slot
        let dev_person = create_test_person("Developer", "dev@test.com", org_id);
        graph.add_node(dev_person, KeyOwnerRole::Developer);

        // When: I analyze YubiKey requirements from the graph
        let result = graph_yubikey::generate_yubikey_provision_from_graph(&graph);

        // Then: YubiKey provision plans should be generated successfully
        assert!(result.is_ok(), "YubiKey analysis should succeed");
        let yubikey_nodes = result.unwrap();

        // And: We should have one provision plan per person
        assert_eq!(yubikey_nodes.len(), 3, "Should have 3 YubiKey provision plans");

        // And: Each person should have a YubiKey requirement
        for (node, _) in &yubikey_nodes {
            if let NodeType::YubiKeyStatus { slots_needed, .. } = &node.node_type {
                assert!(!slots_needed.is_empty(), "Each person should need at least one PIV slot");
            } else {
                panic!("Expected YubiKeyStatus node");
            }
        }
    }

    #[test]
    fn test_user_story_complete_infrastructure_generation() {
        // User Story: As a user, I want to generate complete infrastructure from one organizational graph
        // so I can bootstrap my entire security infrastructure at once
        //
        // Given: A complex organization with multiple units and people
        let mut graph = OrganizationConcept::new();

        let org = create_test_org("GlobalCorp");
        let org_id = org.id;
        graph.add_organization_node(org);

        // Add two organizational units
        let eng_unit = create_test_unit("Engineering", OrganizationUnitType::Department);
        graph.add_org_unit_node(eng_unit);

        let ops_unit = create_test_unit("Operations", OrganizationUnitType::Department);
        graph.add_org_unit_node(ops_unit);

        // Add people to different units with different roles
        let eng_lead = create_test_person("Engineering Lead", "eng-lead@global.com", org_id);
        graph.add_node(eng_lead, KeyOwnerRole::SecurityAdmin);

        let ops_lead = create_test_person("Operations Lead", "ops-lead@global.com", org_id);
        graph.add_node(ops_lead, KeyOwnerRole::SecurityAdmin);

        let developer = create_test_person("Developer", "dev@global.com", org_id);
        graph.add_node(developer, KeyOwnerRole::Developer);

        // When: I generate all infrastructure types
        let pki_result = graph_pki::generate_pki_from_graph(&graph, "test-pass");
        let nats_result = graph_nats::generate_nats_from_graph(&graph);
        let yubikey_result = graph_yubikey::generate_yubikey_provision_from_graph(&graph);

        // Then: All infrastructure should be generated successfully
        assert!(pki_result.is_ok(), "PKI generation should succeed");
        assert!(nats_result.is_ok(), "NATS generation should succeed");
        assert!(yubikey_result.is_ok(), "YubiKey analysis should succeed");

        // And: PKI should have certificates for all entities
        let pki_certs = pki_result.unwrap();
        assert!(pki_certs.len() >= 6, "Should have certs for org + 2 units + 3 people");

        // And: NATS should have entities for all entities
        let nats_entities = nats_result.unwrap();
        assert!(nats_entities.len() >= 6, "Should have NATS entities for org + 2 units + 3 people");

        // And: YubiKeys should be planned for all people
        let yubikey_plans = yubikey_result.unwrap();
        assert_eq!(yubikey_plans.len(), 3, "Should have YubiKey plans for 3 people");
    }

    #[test]
    fn test_user_story_role_based_slot_allocation() {
        // User Story: As a security admin, I want different roles to get appropriate PIV slots
        // so that access controls match responsibilities
        //
        // Given: People with different roles
        let root_slots = graph_yubikey::slots_for_role(&KeyOwnerRole::RootAuthority);
        let security_slots = graph_yubikey::slots_for_role(&KeyOwnerRole::SecurityAdmin);
        let developer_slots = graph_yubikey::slots_for_role(&KeyOwnerRole::Developer);
        let service_slots = graph_yubikey::slots_for_role(&KeyOwnerRole::ServiceAccount);

        // Then: Root authority should get signature slot (for CA operations)
        assert_eq!(root_slots.len(), 1);
        assert!(root_slots.contains(&graph_yubikey::PIVSlot::DigitalSignature));

        // And: Security admin should get all administrative slots
        assert!(security_slots.len() >= 3);
        assert!(security_slots.contains(&graph_yubikey::PIVSlot::Authentication));
        assert!(security_slots.contains(&graph_yubikey::PIVSlot::DigitalSignature));
        assert!(security_slots.contains(&graph_yubikey::PIVSlot::KeyManagement));

        // And: Developer should get authentication slot (for daily work)
        assert_eq!(developer_slots.len(), 1);
        assert!(developer_slots.contains(&graph_yubikey::PIVSlot::Authentication));

        // And: Service account should get card authentication slot
        assert_eq!(service_slots.len(), 1);
        assert!(service_slots.contains(&graph_yubikey::PIVSlot::CardAuthentication));
    }

    #[test]
    fn test_user_story_empty_graph_handling() {
        // User Story: As a user, I should get clear errors when trying to generate infrastructure
        // from an incomplete organizational graph
        //
        // Given: An empty graph
        let graph = OrganizationConcept::new();

        // When: I try to generate PKI
        let pki_result = graph_pki::generate_pki_from_graph(&graph, "test-pass");

        // Then: I should get a clear error message
        assert!(pki_result.is_err());
        let error = pki_result.unwrap_err();
        assert!(error.contains("No organization"), "Error should mention missing organization");

        // When: I try to generate NATS
        let nats_result = graph_nats::generate_nats_from_graph(&graph);

        // Then: I should get a clear error message
        assert!(nats_result.is_err());
        let error = nats_result.unwrap_err();
        assert!(error.contains("No organization"), "Error should mention missing organization");

        // When: I try to analyze YubiKey requirements
        let yubikey_result = graph_yubikey::generate_yubikey_provision_from_graph(&graph);

        // Then: I should get a clear error message
        assert!(yubikey_result.is_err());
        let error = yubikey_result.unwrap_err();
        assert!(error.contains("No people"), "Error should mention missing people");
    }
}
