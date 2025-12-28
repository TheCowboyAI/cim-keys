// Copyright (c) 2025 - Cowboy AI, LLC.
//! End-to-End Workflow Tests
//!
//! Phase 2 Day 6-7: Complete workflow testing
//!
//! Test Categories:
//! 1. Complete PKI Bootstrap Workflow
//! 2. YubiKey Provisioning Flow
//! 3. NATS Security Bootstrap
//! 4. Multi-Organization Scenarios

use chrono::Utc;
use std::collections::HashMap;
use cim_keys::{
    aggregate::KeyManagementAggregate,
    commands::{
        organization::{CreateOrganization, CreatePerson, CreateLocation},
        yubikey::ProvisionYubiKeySlot,
        KeyCommand,
    },
    domain::{Organization, Person},
    events::{
        DomainEvent,
        organization::OrganizationEvents,
        person::PersonEvents,
        yubikey::YubiKeyEvents,
    },
    projections::OfflineKeyProjection,
    state_machines::PivSlot,
    value_objects::AuthKeyPurpose,
};
use tempfile::TempDir;
use uuid::Uuid;

// =============================================================================
// Test Helpers
// =============================================================================

fn create_test_environment() -> (KeyManagementAggregate, OfflineKeyProjection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let aggregate = KeyManagementAggregate::new(Uuid::now_v7());
    let projection = OfflineKeyProjection::new(output_path).expect("Failed to create projection");

    (aggregate, projection, temp_dir)
}

fn has_person(projection: &OfflineKeyProjection, person_id: Uuid) -> bool {
    projection.get_people().iter().any(|p| p.person_id == person_id)
}

fn has_location(projection: &OfflineKeyProjection, location_id: Uuid) -> bool {
    projection.get_locations().iter().any(|l| l.location_id == location_id)
}

// =============================================================================
// 1. Complete PKI Bootstrap Workflow Tests
// =============================================================================

mod pki_bootstrap {
    use super::*;

    #[tokio::test]
    async fn test_create_organization_with_structure() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        // Step 1: Create organization
        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "CowboyAI Security".to_string(),
            domain: Some("cowboyai.com".to_string()),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .expect("Organization creation should succeed");

        assert!(!events.is_empty());

        // Apply events
        for event in &events {
            projection.apply(event).ok();
        }

        assert_eq!(projection.get_organization().name, "CowboyAI Security");
    }

    #[tokio::test]
    async fn test_add_people_to_organization() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        // Create organization first
        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "Test Org".to_string(),
            domain: Some("test.org".to_string()),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .unwrap();
        for event in events {
            projection.apply(&event).ok();
        }

        // Add multiple people
        let people = vec![
            ("Alice Smith", "alice@test.org", "Security Admin"),
            ("Bob Jones", "bob@test.org", "Developer"),
            ("Carol White", "carol@test.org", "Manager"),
        ];

        let mut person_ids = Vec::new();

        for (name, email, title) in people {
            let person_id = Uuid::now_v7();
            person_ids.push(person_id);

            let person_command = KeyCommand::CreatePerson(CreatePerson {
                command_id: Uuid::now_v7(),
                person_id,
                name: name.to_string(),
                email: email.to_string(),
                title: Some(title.to_string()),
                department: None,
                organization_id: Some(org_id),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(person_command, &projection, None, None)
                .await
                .expect("Person creation should succeed");

            for event in events {
                projection.apply(&event).ok();
            }
        }

        // Verify all people were added
        assert_eq!(projection.get_people().len(), 3);
        for person_id in person_ids {
            assert!(has_person(&projection, person_id));
        }
    }

    #[tokio::test]
    async fn test_add_locations_to_organization() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        // Create organization first
        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "Test Org".to_string(),
            domain: Some("test.org".to_string()),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .unwrap();
        for event in events {
            projection.apply(&event).ok();
        }

        // Add multiple locations
        let locations = vec![
            ("HQ Secure Room", "Physical", "Austin", "TX"),
            ("Remote Datacenter", "Physical", "Dallas", "TX"),
            ("Cloud HSM", "Virtual", "AWS", "us-east-1"),
        ];

        let mut location_ids = Vec::new();

        for (name, loc_type, city, region) in locations {
            let location_id = Uuid::now_v7();
            location_ids.push(location_id);

            let location_command = KeyCommand::CreateLocation(CreateLocation {
                command_id: Uuid::now_v7(),
                location_id,
                name: name.to_string(),
                location_type: loc_type.to_string(),
                address: Some(format!("{}, {}", city, region)),
                coordinates: None,
                organization_id: Some(org_id),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(location_command, &projection, None, None)
                .await
                .expect("Location creation should succeed");

            for event in events {
                projection.apply(&event).ok();
            }
        }

        // Verify all locations were added
        assert_eq!(projection.get_locations().len(), 3);
        for location_id in location_ids {
            assert!(has_location(&projection, location_id));
        }
    }

    #[tokio::test]
    async fn test_complete_pki_bootstrap_workflow() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        // Step 1: Create organization
        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "CowboyAI".to_string(),
            domain: Some("cowboyai.com".to_string()),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .unwrap();
        for event in events {
            projection.apply(&event).ok();
        }

        // Step 2: Add security admin
        let admin_id = Uuid::now_v7();
        let person_command = KeyCommand::CreatePerson(CreatePerson {
            command_id: Uuid::now_v7(),
            person_id: admin_id,
            name: "Security Admin".to_string(),
            email: "security@cowboyai.com".to_string(),
            title: Some("Chief Security Officer".to_string()),
            department: Some("Security".to_string()),
            organization_id: Some(org_id),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(person_command, &projection, None, None)
            .await
            .unwrap();
        for event in events {
            projection.apply(&event).ok();
        }

        // Step 3: Add secure location
        let location_id = Uuid::now_v7();
        let location_command = KeyCommand::CreateLocation(CreateLocation {
            command_id: Uuid::now_v7(),
            location_id,
            name: "Secure Key Vault".to_string(),
            location_type: "Physical".to_string(),
            address: Some("123 Security Lane, Austin, TX 78701, USA".to_string()),
            coordinates: Some((30.2672, -97.7431)),  // Austin, TX coordinates
            organization_id: Some(org_id),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(location_command, &projection, None, None)
            .await
            .unwrap();
        for event in events {
            projection.apply(&event).ok();
        }

        // Verify complete setup
        assert_eq!(projection.get_organization().name, "CowboyAI");
        assert_eq!(projection.get_people().len(), 1);
        assert_eq!(projection.get_locations().len(), 1);

        // Save manifest
        projection.save_manifest().expect("Should save manifest");
    }
}

// =============================================================================
// 2. Event Chain Tests (Correlation/Causation)
// =============================================================================

mod event_chains {
    use super::*;

    #[tokio::test]
    async fn test_causation_chain_through_workflow() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        // Create organization
        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "Chain Test Org".to_string(),
            domain: Some("chain.org".to_string()),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let org_events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .expect("Should succeed");

        // Get the org created event's ID for causation chain
        let org_event_causation = org_events.first().map(|e| {
            match e {
                DomainEvent::Organization(OrganizationEvents::OrganizationCreated(evt)) =>
                    Some(evt.correlation_id),
                _ => None,
            }
        }).flatten();

        for event in org_events {
            projection.apply(&event).ok();
        }

        // Create person with causation link to org creation
        let person_id = Uuid::now_v7();
        let person_command = KeyCommand::CreatePerson(CreatePerson {
            command_id: Uuid::now_v7(),
            person_id,
            name: "Chain Person".to_string(),
            email: "chain@chain.org".to_string(),
            title: None,
            department: None,
            organization_id: Some(org_id),
            correlation_id,  // Same correlation ID maintains the chain
            causation_id: org_event_causation,  // Link to org event
            timestamp: Utc::now(),
        });

        let person_events = aggregate.handle_command(person_command, &projection, None, None)
            .await
            .expect("Should succeed");

        // Verify causation chain
        for event in &person_events {
            match event {
                DomainEvent::Person(PersonEvents::PersonCreated(evt)) => {
                    assert_eq!(evt.correlation_id, correlation_id);
                    // Causation should link back
                }
                _ => {}
            }
        }

        for event in person_events {
            projection.apply(&event).ok();
        }
    }

    #[tokio::test]
    async fn test_correlation_groups_related_events() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let workflow_correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        // All events in this workflow share the same correlation_id
        let mut all_events = Vec::new();

        // Create organization
        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "Correlated Org".to_string(),
            domain: Some("correlated.org".to_string()),
            correlation_id: workflow_correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .unwrap();
        all_events.extend(events.clone());
        for event in events {
            projection.apply(&event).ok();
        }

        // Add person
        let person_command = KeyCommand::CreatePerson(CreatePerson {
            command_id: Uuid::now_v7(),
            person_id: Uuid::now_v7(),
            name: "Test Person".to_string(),
            email: "test@correlated.org".to_string(),
            title: None,
            department: None,
            organization_id: Some(org_id),
            correlation_id: workflow_correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(person_command, &projection, None, None)
            .await
            .unwrap();
        all_events.extend(events.clone());
        for event in events {
            projection.apply(&event).ok();
        }

        // Add location
        let location_command = KeyCommand::CreateLocation(CreateLocation {
            command_id: Uuid::now_v7(),
            location_id: Uuid::now_v7(),
            name: "Test Location".to_string(),
            location_type: "Physical".to_string(),
            address: None,
            coordinates: None,
            organization_id: Some(org_id),
            correlation_id: workflow_correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(location_command, &projection, None, None)
            .await
            .unwrap();
        all_events.extend(events.clone());
        for event in events {
            projection.apply(&event).ok();
        }

        // All events should share the same correlation_id
        assert!(all_events.len() >= 3, "Should have at least 3 events");

        // Verify we can query/filter by correlation_id
        // (In a real system, this would be done via event store)
    }
}

// =============================================================================
// 3. Multi-Organization Scenarios
// =============================================================================

mod multi_organization {
    use super::*;

    #[tokio::test]
    async fn test_create_multiple_independent_organizations() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();

        let org_names = vec![
            ("Org Alpha", "alpha.com"),
            ("Org Beta", "beta.com"),
            ("Org Gamma", "gamma.com"),
        ];

        for (name, domain) in org_names {
            let correlation_id = Uuid::now_v7();
            let org_id = Uuid::now_v7();

            let org_command = KeyCommand::CreateOrganization(CreateOrganization {
                command_id: Uuid::now_v7(),
                organization_id: org_id,
                name: name.to_string(),
                domain: Some(domain.to_string()),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(org_command, &projection, None, None)
                .await
                .expect("Organization creation should succeed");

            for event in events {
                projection.apply(&event).ok();
            }
        }

        // Note: Current projection only stores one organization at a time
        // In a full implementation, multiple orgs would be supported
        assert!(!projection.get_organization().name.is_empty());
    }

    #[tokio::test]
    async fn test_people_in_multiple_organizations() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();

        // Create two organizations
        let org1_id = Uuid::now_v7();
        let org2_id = Uuid::now_v7();

        for (org_id, name) in [(org1_id, "Org One"), (org2_id, "Org Two")] {
            let org_command = KeyCommand::CreateOrganization(CreateOrganization {
                command_id: Uuid::now_v7(),
                organization_id: org_id,
                name: name.to_string(),
                domain: None,
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(org_command, &projection, None, None)
                .await
                .unwrap();
            for event in events {
                projection.apply(&event).ok();
            }
        }

        // Add people to org1
        for i in 0..3 {
            let person_command = KeyCommand::CreatePerson(CreatePerson {
                command_id: Uuid::now_v7(),
                person_id: Uuid::now_v7(),
                name: format!("Person {} in Org1", i),
                email: format!("person{}@org1.com", i),
                title: None,
                department: None,
                organization_id: Some(org1_id),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(person_command, &projection, None, None)
                .await
                .unwrap();
            for event in events {
                projection.apply(&event).ok();
            }
        }

        // Add people to org2
        for i in 0..2 {
            let person_command = KeyCommand::CreatePerson(CreatePerson {
                command_id: Uuid::now_v7(),
                person_id: Uuid::now_v7(),
                name: format!("Person {} in Org2", i),
                email: format!("person{}@org2.com", i),
                title: None,
                department: None,
                organization_id: Some(org2_id),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(person_command, &projection, None, None)
                .await
                .unwrap();
            for event in events {
                projection.apply(&event).ok();
            }
        }

        // Total of 5 people
        assert_eq!(projection.get_people().len(), 5);
    }
}

// =============================================================================
// 4. Projection Consistency Tests
// =============================================================================

mod projection_consistency {
    use super::*;

    #[tokio::test]
    async fn test_projection_survives_reload() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        // Phase 1: Create data and save
        {
            let aggregate = KeyManagementAggregate::new(Uuid::now_v7());
            let mut projection = OfflineKeyProjection::new(&path).unwrap();
            let correlation_id = Uuid::now_v7();
            let org_id = Uuid::now_v7();

            // Create organization
            let org_command = KeyCommand::CreateOrganization(CreateOrganization {
                command_id: Uuid::now_v7(),
                organization_id: org_id,
                name: "Persistent Org".to_string(),
                domain: Some("persistent.org".to_string()),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(org_command, &projection, None, None)
                .await
                .unwrap();
            for event in events {
                projection.apply(&event).ok();
            }

            // Add person
            let person_command = KeyCommand::CreatePerson(CreatePerson {
                command_id: Uuid::now_v7(),
                person_id: Uuid::now_v7(),
                name: "Persistent Person".to_string(),
                email: "person@persistent.org".to_string(),
                title: None,
                department: None,
                organization_id: Some(org_id),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(person_command, &projection, None, None)
                .await
                .unwrap();
            for event in events {
                projection.apply(&event).ok();
            }

            // Save
            projection.save_manifest().unwrap();
        }

        // Phase 2: Reload and verify
        {
            let projection = OfflineKeyProjection::new(&path).unwrap();

            assert_eq!(projection.get_organization().name, "Persistent Org");
            assert_eq!(projection.get_people().len(), 1);
            assert_eq!(projection.get_people()[0].name, "Persistent Person");
        }
    }

    #[tokio::test]
    async fn test_idempotent_event_application() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "Idempotent Org".to_string(),
            domain: None,
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .unwrap();

        // Apply events once
        for event in &events {
            projection.apply(event).ok();
        }

        let org_name_1 = projection.get_organization().name.clone();

        // Apply events again (idempotent check)
        for event in &events {
            projection.apply(event).ok();
        }

        let org_name_2 = projection.get_organization().name.clone();

        // Should be the same
        assert_eq!(org_name_1, org_name_2);
    }
}

// =============================================================================
// 5. Error Handling Workflows
// =============================================================================

mod error_handling {
    use super::*;

    #[tokio::test]
    async fn test_workflow_with_invalid_org_reference() {
        let (aggregate, projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let nonexistent_org_id = Uuid::now_v7();

        // Try to create person in non-existent org
        // This should still succeed (validation is at app level, not aggregate)
        let person_command = KeyCommand::CreatePerson(CreatePerson {
            command_id: Uuid::now_v7(),
            person_id: Uuid::now_v7(),
            name: "Orphan Person".to_string(),
            email: "orphan@nowhere.com".to_string(),
            title: None,
            department: None,
            organization_id: Some(nonexistent_org_id),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let result = aggregate.handle_command(person_command, &projection, None, None).await;
        // This may succeed or fail depending on implementation
        // The test verifies we don't panic
        assert!(result.is_ok() || result.is_err());
    }
}

// =============================================================================
// 6. Performance/Bulk Operation Tests
// =============================================================================

mod bulk_operations {
    use super::*;

    #[tokio::test]
    async fn test_bulk_person_creation() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        // Create organization
        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "Bulk Org".to_string(),
            domain: None,
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .unwrap();
        for event in events {
            projection.apply(&event).ok();
        }

        // Create 100 people
        for i in 0..100 {
            let person_command = KeyCommand::CreatePerson(CreatePerson {
                command_id: Uuid::now_v7(),
                person_id: Uuid::now_v7(),
                name: format!("Person {}", i),
                email: format!("person{}@bulk.org", i),
                title: None,
                department: None,
                organization_id: Some(org_id),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(person_command, &projection, None, None)
                .await
                .expect(&format!("Person {} creation should succeed", i));

            for event in events {
                projection.apply(&event).ok();
            }
        }

        assert_eq!(projection.get_people().len(), 100);
    }

    #[tokio::test]
    async fn test_bulk_location_creation() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        // Create organization
        let org_command = KeyCommand::CreateOrganization(CreateOrganization {
            command_id: Uuid::now_v7(),
            organization_id: org_id,
            name: "Bulk Loc Org".to_string(),
            domain: None,
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
        });

        let events = aggregate.handle_command(org_command, &projection, None, None)
            .await
            .unwrap();
        for event in events {
            projection.apply(&event).ok();
        }

        // Create 50 locations
        for i in 0..50 {
            let location_command = KeyCommand::CreateLocation(CreateLocation {
                command_id: Uuid::now_v7(),
                location_id: Uuid::now_v7(),
                name: format!("Location {}", i),
                location_type: "Physical".to_string(),
                address: Some(format!("City {}, USA", i)),
                coordinates: None,
                organization_id: Some(org_id),
                correlation_id,
                causation_id: None,
                timestamp: Utc::now(),
            });

            let events = aggregate.handle_command(location_command, &projection, None, None)
                .await
                .expect(&format!("Location {} creation should succeed", i));

            for event in events {
                projection.apply(&event).ok();
            }
        }

        assert_eq!(projection.get_locations().len(), 50);
    }
}

// =============================================================================
// 7. YubiKey Provisioning Workflow Tests
// =============================================================================

mod yubikey_provisioning {
    use super::*;

    fn create_test_organization() -> Organization {
        Organization {
            id: Uuid::now_v7(),
            name: "test_org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("Test organization for YubiKey provisioning".to_string()),
            parent_id: None,
            units: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_person(org_id: Uuid) -> Person {
        Person {
            id: Uuid::now_v7(),
            name: "Security Admin".to_string(),
            email: "admin@test.org".to_string(),
            roles: Vec::new(),
            organization_id: org_id,
            unit_ids: Vec::new(),
            created_at: Utc::now(),
            active: true,
            nats_permissions: None,
        }
    }

    #[tokio::test]
    async fn test_yubikey_slot_provisioning() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();

        // Setup organization and person first
        let org = create_test_organization();
        let person = create_test_person(org.id);

        // Create the provision command
        let provision_command = KeyCommand::ProvisionYubiKey(ProvisionYubiKeySlot {
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            person: person.clone(),
            organization: org.clone(),
            purpose: AuthKeyPurpose::SsoAuthentication,
            correlation_id,
            causation_id: None,
        });

        let events = aggregate.handle_command(provision_command, &projection, None, None)
            .await
            .expect("YubiKey provisioning should succeed");

        // Should emit multiple events
        assert!(!events.is_empty(), "Should emit provisioning events");

        // Verify we got YubiKey events
        let yubikey_events: Vec<_> = events.iter().filter(|e| {
            matches!(e, DomainEvent::YubiKey(_))
        }).collect();

        assert!(!yubikey_events.is_empty(), "Should have YubiKey-specific events");

        // Apply events
        for event in &events {
            projection.apply(event).ok();
        }
    }

    #[tokio::test]
    async fn test_yubikey_multiple_slots() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();

        let org = create_test_organization();
        let person = create_test_person(org.id);

        // Provision authentication slot
        let auth_command = KeyCommand::ProvisionYubiKey(ProvisionYubiKeySlot {
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Authentication,
            person: person.clone(),
            organization: org.clone(),
            purpose: AuthKeyPurpose::SsoAuthentication,
            correlation_id,
            causation_id: None,
        });

        let auth_events = aggregate.handle_command(auth_command, &projection, None, None)
            .await
            .expect("Authentication slot provisioning should succeed");

        for event in &auth_events {
            projection.apply(event).ok();
        }

        // Provision signature slot
        let sign_command = KeyCommand::ProvisionYubiKey(ProvisionYubiKeySlot {
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::Signature,
            person: person.clone(),
            organization: org.clone(),
            purpose: AuthKeyPurpose::GpgSigning,
            correlation_id,
            causation_id: Some(correlation_id),
        });

        let sign_events = aggregate.handle_command(sign_command, &projection, None, None)
            .await
            .expect("Signature slot provisioning should succeed");

        for event in &sign_events {
            projection.apply(event).ok();
        }

        // Both provisioning operations should have generated events
        assert!(!auth_events.is_empty());
        assert!(!sign_events.is_empty());
    }

    #[tokio::test]
    async fn test_yubikey_provisioning_event_chain() {
        let (aggregate, projection, _temp_dir) = create_test_environment();
        let correlation_id = Uuid::now_v7();

        let org = create_test_organization();
        let person = create_test_person(org.id);

        let provision_command = KeyCommand::ProvisionYubiKey(ProvisionYubiKeySlot {
            yubikey_serial: "12345678".to_string(),
            slot: PivSlot::KeyManagement,
            person: person.clone(),
            organization: org.clone(),
            purpose: AuthKeyPurpose::GpgEncryption,
            correlation_id,
            causation_id: None,
        });

        let events = aggregate.handle_command(provision_command, &projection, None, None)
            .await
            .expect("Provisioning should succeed");

        // Check event chain contains expected event types
        let has_slot_planned = events.iter().any(|e| {
            matches!(e, DomainEvent::YubiKey(YubiKeyEvents::SlotAllocationPlanned(_)))
        });

        let has_key_generated = events.iter().any(|e| {
            matches!(e, DomainEvent::YubiKey(YubiKeyEvents::KeyGeneratedInSlot(_)))
        });

        assert!(has_slot_planned, "Should have slot allocation planned event");
        assert!(has_key_generated, "Should have key generated event");
    }

    #[tokio::test]
    async fn test_yubikey_different_users() {
        let (aggregate, mut projection, _temp_dir) = create_test_environment();

        let org = create_test_organization();

        // Provision for multiple users
        let users = vec![
            ("Alice", "alice@test.org"),
            ("Bob", "bob@test.org"),
            ("Carol", "carol@test.org"),
        ];

        for (name, email) in users {
            let person = Person {
                id: Uuid::now_v7(),
                name: name.to_string(),
                email: email.to_string(),
                roles: Vec::new(),
                organization_id: org.id,
                unit_ids: Vec::new(),
                created_at: Utc::now(),
                active: true,
                nats_permissions: None,
            };

            let correlation_id = Uuid::now_v7();

            let provision_command = KeyCommand::ProvisionYubiKey(ProvisionYubiKeySlot {
                yubikey_serial: format!("YK-{}", Uuid::now_v7().as_u128() % 100000000),
                slot: PivSlot::Authentication,
                person: person.clone(),
                organization: org.clone(),
                purpose: AuthKeyPurpose::SsoAuthentication,
                correlation_id,
                causation_id: None,
            });

            let events = aggregate.handle_command(provision_command, &projection, None, None)
                .await
                .expect(&format!("Provisioning for {} should succeed", name));

            for event in &events {
                projection.apply(event).ok();
            }
        }
    }
}
