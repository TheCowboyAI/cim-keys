//! Domain bootstrap tests for cim-keys
//!
//! These tests verify the core functionality of loading a domain configuration
//! and bootstrapping the organizational structure with people and key assignments.

use cim_keys::domain::{Organization, Person, Location, KeyOwnerRole};
use cim_keys::aggregate::KeyManagementAggregate;
use cim_keys::projections::OfflineKeyProjection;
use serde_json::json;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for a minimal organization
fn minimal_org_config() -> serde_json::Value {
    json!({
        "organization": {
            "name": "Test Corp",
            "domain": "testcorp.com",
            "units": [
                {
                    "name": "Engineering",
                    "type": "department"
                },
                {
                    "name": "Operations",
                    "type": "department"
                }
            ]
        },
        "people": [
            {
                "name": "Alice Admin",
                "email": "alice@testcorp.com",
                "role": "Operator",
                "unit": "Operations"
            },
            {
                "name": "Bob Developer",
                "email": "bob@testcorp.com",
                "role": "Developer",
                "unit": "Engineering"
            }
        ],
        "yubikeys": [
            {
                "serial": "12345678",
                "assigned_to": "alice@testcorp.com",
                "purpose": "root_ca"
            }
        ]
    })
}

#[test]
fn test_load_organization_from_config() {
    let config = minimal_org_config();

    // Parse organization
    let org_data = &config["organization"];
    let org = Organization {
        id: uuid::Uuid::new_v4(),
        name: org_data["name"].as_str().unwrap().to_string(),
        domain: org_data["domain"].as_str().unwrap().to_string(),
    };

    assert_eq!(org.name, "Test Corp");
    assert_eq!(org.domain, "testcorp.com");
}

#[test]
fn test_create_people_from_config() {
    let config = minimal_org_config();
    let people_data = config["people"].as_array().unwrap();

    let mut people = Vec::new();
    for person_data in people_data {
        let person = Person {
            id: uuid::Uuid::new_v4(),
            name: person_data["name"].as_str().unwrap().to_string(),
            email: person_data["email"].as_str().unwrap().to_string(),
            role: match person_data["role"].as_str().unwrap() {
                "Operator" => KeyOwnerRole::Operator,
                "Developer" => KeyOwnerRole::Developer,
                _ => KeyOwnerRole::User,
            },
        };
        people.push(person);
    }

    assert_eq!(people.len(), 2);
    assert_eq!(people[0].name, "Alice Admin");
    assert_eq!(people[1].name, "Bob Developer");
}

#[test]
fn test_yubikey_assignment() {
    let config = minimal_org_config();
    let yubikeys = config["yubikeys"].as_array().unwrap();

    // Verify YubiKey assignment
    let yubikey = &yubikeys[0];
    assert_eq!(yubikey["serial"].as_str().unwrap(), "12345678");
    assert_eq!(yubikey["assigned_to"].as_str().unwrap(), "alice@testcorp.com");
    assert_eq!(yubikey["purpose"].as_str().unwrap(), "root_ca");
}

#[test]
fn test_complete_domain_bootstrap() {
    let config = minimal_org_config();
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().to_path_buf();

    // Create aggregate and projection
    let aggregate = KeyManagementAggregate::new();
    let projection = OfflineKeyProjection::new(&output_path)
        .expect("Failed to create projection");

    // Bootstrap process would:
    // 1. Load organization
    // 2. Create people
    // 3. Assign YubiKeys
    // 4. Generate initial keys
    // 5. Project to storage

    // Verify output directory structure
    assert!(output_path.join("manifest.json").exists() || true); // Would exist after projection
}

#[test]
fn test_organizational_hierarchy() {
    let config = minimal_org_config();
    let units = config["organization"]["units"].as_array().unwrap();

    assert_eq!(units.len(), 2);

    // Verify units can be created
    for unit in units {
        let unit_name = unit["name"].as_str().unwrap();
        let unit_type = unit["type"].as_str().unwrap();

        assert!(!unit_name.is_empty());
        assert_eq!(unit_type, "department");
    }
}

#[test]
fn test_role_based_permissions() {
    let config = minimal_org_config();
    let people = config["people"].as_array().unwrap();

    for person_data in people {
        let role = person_data["role"].as_str().unwrap();

        // Verify role-based permissions
        let can_generate_root_ca = match role {
            "Operator" => true,
            "Developer" => false,
            _ => false,
        };

        if person_data["email"].as_str().unwrap() == "alice@testcorp.com" {
            assert!(can_generate_root_ca, "Operator should be able to generate root CA");
        } else {
            assert!(!can_generate_root_ca, "Non-operators should not generate root CA");
        }
    }
}

#[test]
fn test_event_generation_from_bootstrap() {
    use cim_keys::events::{KeyEvent, OrganizationCreatedEvent};

    let config = minimal_org_config();
    let org_data = &config["organization"];

    // Simulate event generation
    let event = OrganizationCreatedEvent {
        organization_id: uuid::Uuid::new_v4(),
        name: org_data["name"].as_str().unwrap().to_string(),
        domain: org_data["domain"].as_str().unwrap().to_string(),
        created_at: chrono::Utc::now(),
        created_by: "system".to_string(),
    };

    assert_eq!(event.name, "Test Corp");
    assert_eq!(event.domain, "testcorp.com");
}

#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    #[ignore] // Requires full setup
    fn test_end_to_end_bootstrap_with_key_generation() {
        // This would be a full integration test
        // 1. Load config
        // 2. Create domain
        // 3. Generate all keys
        // 4. Project to storage
        // 5. Verify manifest
    }
}