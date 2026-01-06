// Copyright (c) 2025 - Cowboy AI, LLC.
//! Comprehensive Projection Tests
//!
//! Target: 90%+ coverage of src/projections.rs
//!
//! Test Categories:
//! 1. Projection Application Correctness
//! 2. Projection Idempotency
//! 3. File System Integrity
//! 4. Projection State Query Tests

use chrono::Utc;
use cim_keys::projections::{
    OfflineKeyProjection, KeyManifest, KeyEntry, CertificateEntry,
    PersonEntry, LocationEntry, YubiKeyEntry, PkiHierarchyEntry,
    NatsOperatorEntry, NatsAccountEntry, NatsUserEntry, OrganizationInfo,
    ProjectionError,
};
use cim_keys::types::{KeyAlgorithm, KeyPurpose, KeyMetadata};
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

// =============================================================================
// Test Helpers
// =============================================================================

fn create_temp_projection() -> (TempDir, OfflineKeyProjection) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let projection = OfflineKeyProjection::new(temp_dir.path())
        .expect("Failed to create projection");
    (temp_dir, projection)
}

// =============================================================================
// 1. Projection Application Correctness Tests
// =============================================================================

mod application_correctness {
    use super::*;

    #[test]
    fn test_new_projection_creates_directory_structure() {
        let (temp_dir, _projection) = create_temp_projection();

        // Verify directory structure
        assert!(temp_dir.path().join("events").exists());
        assert!(temp_dir.path().join("keys").exists());
        assert!(temp_dir.path().join("certificates").exists());
        assert!(temp_dir.path().join("yubikeys").exists());
        assert!(temp_dir.path().join("pki").exists());
        assert!(temp_dir.path().join("nats").exists());
    }

    #[test]
    fn test_new_projection_has_empty_collections() {
        let (_temp_dir, projection) = create_temp_projection();

        assert!(projection.get_people().is_empty());
        assert!(projection.get_keys().is_empty());
        assert!(projection.get_certificates().is_empty());
        assert!(projection.get_locations().is_empty());
        assert!(projection.get_yubikeys().is_empty());
    }

    #[test]
    fn test_add_person_updates_projection() {
        let (_temp_dir, mut projection) = create_temp_projection();
        let org_id = Uuid::now_v7();

        let result = projection.add_person(
            Uuid::now_v7(),
            "Alice Smith".to_string(),
            "alice@example.com".to_string(),
            "Engineer".to_string(),
            org_id,
        );

        assert!(result.is_ok());
        assert_eq!(projection.get_people().len(), 1);
    }

    #[test]
    fn test_add_location_updates_projection() {
        let (_temp_dir, mut projection) = create_temp_projection();
        let org_id = Uuid::now_v7();

        let result = projection.add_location(
            Uuid::now_v7(),
            "HQ".to_string(),
            "Physical".to_string(),
            org_id,
            Some("123 Main St".to_string()),
            Some("Austin".to_string()),
            Some("TX".to_string()),
            Some("USA".to_string()),
            Some("78701".to_string()),
            None, // virtual_url
        );

        assert!(result.is_ok());
        assert_eq!(projection.get_locations().len(), 1);
    }

    #[test]
    fn test_set_organization_updates_projection() {
        let (_temp_dir, mut projection) = create_temp_projection();

        let result = projection.set_organization(
            "CowboyAI".to_string(),
            "cowboyai.com".to_string(),
            "USA".to_string(),
            "admin@cowboyai.com".to_string(),
        );

        assert!(result.is_ok());
        assert_eq!(projection.get_organization().name, "CowboyAI");
    }

    #[test]
    fn test_key_exists_false_for_unknown() {
        let (_temp_dir, projection) = create_temp_projection();
        let unknown_id = Uuid::now_v7();

        assert!(!projection.key_exists(&unknown_id));
    }
}

// =============================================================================
// 2. Projection Idempotency Tests
// =============================================================================

mod idempotency {
    use super::*;

    #[test]
    fn test_manifest_serialization_roundtrip() {
        let manifest = KeyManifest {
            version: "1.0.0".to_string(),
            updated_at: Utc::now(),
            organization: OrganizationInfo {
                name: "Test Org".to_string(),
                domain: "test.com".to_string(),
                country: "USA".to_string(),
                admin_email: "admin@test.com".to_string(),
            },
            people: vec![],
            locations: vec![],
            keys: vec![],
            certificates: vec![],
            pki_hierarchies: vec![],
            yubikeys: vec![],
            nats_operators: vec![],
            nats_accounts: vec![],
            nats_users: vec![],
            event_count: 0,
            checksum: String::new(),
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let loaded: KeyManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.version, loaded.version);
        assert_eq!(manifest.organization.name, loaded.organization.name);
    }

    #[test]
    fn test_key_entry_serialization_roundtrip() {
        let entry = KeyEntry {
            key_id: Uuid::now_v7(),
            algorithm: KeyAlgorithm::Ed25519,
            purpose: KeyPurpose::Signing,
            label: "Test Key".to_string(),
            hardware_backed: false,
            yubikey_serial: None,
            yubikey_slot: None,
            revoked: false,
            file_path: "/keys/test.pem".to_string(),
            state: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: KeyEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.key_id, loaded.key_id);
        assert_eq!(entry.label, loaded.label);
    }

    #[test]
    fn test_certificate_entry_serialization_roundtrip() {
        let entry = CertificateEntry {
            cert_id: Uuid::now_v7(),
            key_id: Uuid::now_v7(),
            subject: "CN=test".to_string(),
            issuer: Some("CN=root".to_string()),
            serial_number: "1234".to_string(),
            not_before: Utc::now(),
            not_after: Utc::now() + chrono::Duration::days(365),
            is_ca: false,
            file_path: "/certs/test.pem".to_string(),
            state: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: CertificateEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.cert_id, loaded.cert_id);
        assert_eq!(entry.subject, loaded.subject);
    }

    #[test]
    fn test_person_entry_serialization_roundtrip() {
        let entry = PersonEntry {
            person_id: Uuid::now_v7(),
            name: "Test Person".to_string(),
            email: "test@example.com".to_string(),
            role: "Engineer".to_string(),
            organization_id: Uuid::now_v7(),
            state: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: PersonEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.person_id, loaded.person_id);
        assert_eq!(entry.name, loaded.name);
    }

    #[test]
    fn test_location_entry_serialization_roundtrip() {
        let entry = LocationEntry {
            location_id: Uuid::now_v7(),
            name: "Test Location".to_string(),
            location_type: "Physical".to_string(),
            organization_id: Uuid::now_v7(),
            street: Some("123 Main St".to_string()),
            city: Some("Austin".to_string()),
            region: Some("TX".to_string()),
            country: Some("USA".to_string()),
            postal_code: Some("78701".to_string()),
            state: None,
            virtual_url: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: LocationEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.location_id, loaded.location_id);
        assert_eq!(entry.name, loaded.name);
    }

    #[test]
    fn test_yubikey_entry_serialization_roundtrip() {
        let entry = YubiKeyEntry {
            serial: "12345678".to_string(),
            provisioned_at: Utc::now(),
            slots_used: vec!["9a".to_string(), "9c".to_string()],
            config_path: "/yubikeys/12345678/config.json".to_string(),
            state: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: YubiKeyEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.serial, loaded.serial);
        assert_eq!(entry.slots_used, loaded.slots_used);
    }

    #[test]
    fn test_nats_operator_entry_serialization_roundtrip() {
        let entry = NatsOperatorEntry {
            operator_id: Uuid::now_v7(),
            name: "test-operator".to_string(),
            public_key: "OABC123".to_string(),
            organization_id: Some(Uuid::now_v7()),
            created_by: "admin".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: NatsOperatorEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.operator_id, loaded.operator_id);
        assert_eq!(entry.name, loaded.name);
    }

    #[test]
    fn test_nats_account_entry_serialization_roundtrip() {
        let entry = NatsAccountEntry {
            account_id: Uuid::now_v7(),
            operator_id: Uuid::now_v7(),
            name: "test-account".to_string(),
            public_key: "AABC123".to_string(),
            is_system: false,
            organization_unit_id: None,
            created_by: "admin".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: NatsAccountEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.account_id, loaded.account_id);
        assert_eq!(entry.name, loaded.name);
    }

    #[test]
    fn test_nats_user_entry_serialization_roundtrip() {
        let entry = NatsUserEntry {
            user_id: Uuid::now_v7(),
            account_id: Uuid::now_v7(),
            name: "test-user".to_string(),
            public_key: "UABC123".to_string(),
            person_id: Some(Uuid::now_v7()),
            created_by: "admin".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: NatsUserEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.user_id, loaded.user_id);
        assert_eq!(entry.name, loaded.name);
    }

    #[test]
    fn test_pki_hierarchy_entry_serialization_roundtrip() {
        let entry = PkiHierarchyEntry {
            hierarchy_name: "test-pki".to_string(),
            root_ca_id: Uuid::now_v7(),
            intermediate_ca_ids: vec![Uuid::now_v7(), Uuid::now_v7()],
            directory_path: "/pki/test-pki".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let loaded: PkiHierarchyEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.hierarchy_name, loaded.hierarchy_name);
        assert_eq!(entry.intermediate_ca_ids.len(), loaded.intermediate_ca_ids.len());
    }

    #[test]
    fn test_organization_info_serialization_roundtrip() {
        let info = OrganizationInfo {
            name: "Test Corp".to_string(),
            domain: "test.com".to_string(),
            country: "USA".to_string(),
            admin_email: "admin@test.com".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let loaded: OrganizationInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.name, loaded.name);
        assert_eq!(info.domain, loaded.domain);
    }
}

// =============================================================================
// 3. File System Integrity Tests
// =============================================================================

mod filesystem_integrity {
    use super::*;

    #[test]
    fn test_directory_structure_complete() {
        let (temp_dir, _projection) = create_temp_projection();
        let root = temp_dir.path();

        // All required directories exist
        let required_dirs = ["events", "keys", "certificates", "yubikeys", "pki", "nats"];
        for dir in &required_dirs {
            assert!(root.join(dir).is_dir(), "Missing directory: {}", dir);
        }
    }

    #[test]
    fn test_save_manifest_creates_file() {
        let (temp_dir, projection) = create_temp_projection();

        projection.save_manifest().unwrap();

        let manifest_path = temp_dir.path().join("manifest.json");
        assert!(manifest_path.exists());
    }

    #[test]
    fn test_manifest_json_valid() {
        let (temp_dir, projection) = create_temp_projection();

        projection.save_manifest().unwrap();

        let manifest_path = temp_dir.path().join("manifest.json");
        let content = fs::read_to_string(&manifest_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_object());
        assert!(parsed.get("version").is_some());
        assert!(parsed.get("keys").is_some());
        assert!(parsed.get("certificates").is_some());
    }

    #[test]
    fn test_event_log_directory_exists() {
        let (temp_dir, _projection) = create_temp_projection();

        let events_dir = temp_dir.path().join("events");
        assert!(events_dir.is_dir());
    }

    #[test]
    fn test_projection_handles_missing_manifest() {
        let temp_dir = TempDir::new().unwrap();

        let projection = OfflineKeyProjection::new(temp_dir.path());
        assert!(projection.is_ok());
    }

    #[test]
    fn test_projection_reload_preserves_organization() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();

        // Create projection and set organization
        {
            let mut projection = OfflineKeyProjection::new(&path).unwrap();
            projection.set_organization(
                "CowboyAI".to_string(),
                "cowboyai.com".to_string(),
                "USA".to_string(),
                "admin@cowboyai.com".to_string(),
            ).unwrap();
            projection.save_manifest().unwrap();
        }

        // Reload projection
        {
            let projection = OfflineKeyProjection::new(&path).unwrap();
            assert_eq!(projection.get_organization().name, "CowboyAI");
        }
    }
}

// =============================================================================
// 4. State Query Tests
// =============================================================================

mod state_queries {
    use super::*;

    #[test]
    fn test_get_people_returns_all_added() {
        let (_temp_dir, mut projection) = create_temp_projection();
        let org_id = Uuid::now_v7();

        projection.add_person(Uuid::now_v7(), "Alice".to_string(), "alice@test.com".to_string(), "Dev".to_string(), org_id).unwrap();
        projection.add_person(Uuid::now_v7(), "Bob".to_string(), "bob@test.com".to_string(), "Dev".to_string(), org_id).unwrap();
        projection.add_person(Uuid::now_v7(), "Carol".to_string(), "carol@test.com".to_string(), "Dev".to_string(), org_id).unwrap();

        assert_eq!(projection.get_people().len(), 3);
    }

    #[test]
    fn test_get_locations_returns_all_added() {
        let (_temp_dir, mut projection) = create_temp_projection();
        let org_id = Uuid::now_v7();

        projection.add_location(Uuid::now_v7(), "HQ".to_string(), "Physical".to_string(), org_id, None, None, None, None, None, None).unwrap();
        projection.add_location(Uuid::now_v7(), "Branch".to_string(), "Physical".to_string(), org_id, None, None, None, None, None, None).unwrap();

        assert_eq!(projection.get_locations().len(), 2);
    }

    #[test]
    fn test_remove_location_reduces_count() {
        let (_temp_dir, mut projection) = create_temp_projection();
        let org_id = Uuid::now_v7();
        let location_id = Uuid::now_v7();

        projection.add_location(location_id, "HQ".to_string(), "Physical".to_string(), org_id, None, None, None, None, None, None).unwrap();
        assert_eq!(projection.get_locations().len(), 1);

        projection.remove_location(location_id).unwrap();
        assert_eq!(projection.get_locations().len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_location_returns_error() {
        let (_temp_dir, mut projection) = create_temp_projection();
        let unknown_id = Uuid::now_v7();

        let result = projection.remove_location(unknown_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_projection_has_default_organization() {
        let (_temp_dir, projection) = create_temp_projection();

        let org = projection.get_organization();
        // Default organization should have empty fields
        assert!(org.name.is_empty() || org.name == "");
    }
}

// =============================================================================
// 5. Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_projection_error_debug_format() {
        let error = ProjectionError::IoError("test error".to_string());
        let display = format!("{:?}", error);
        assert!(display.contains("IoError"));
    }

    #[test]
    fn test_projection_error_variants() {
        let io_error = ProjectionError::IoError("io test".to_string());
        let serialization_error = ProjectionError::SerializationError("serde test".to_string());

        assert!(format!("{:?}", io_error).contains("IoError"));
        assert!(format!("{:?}", serialization_error).contains("Serialization"));
    }
}

// =============================================================================
// 6. Multiple Entry Tests
// =============================================================================

mod multiple_entries {
    use super::*;

    #[test]
    fn test_add_multiple_people() {
        let (_temp_dir, mut projection) = create_temp_projection();
        let org_id = Uuid::now_v7();

        for i in 0..10 {
            projection.add_person(
                Uuid::now_v7(),
                format!("Person {}", i),
                format!("person{}@test.com", i),
                "Developer".to_string(),
                org_id,
            ).unwrap();
        }

        assert_eq!(projection.get_people().len(), 10);
    }

    #[test]
    fn test_add_multiple_locations() {
        let (_temp_dir, mut projection) = create_temp_projection();
        let org_id = Uuid::now_v7();

        for i in 0..5 {
            projection.add_location(
                Uuid::now_v7(),
                format!("Location {}", i),
                "Physical".to_string(),
                org_id,
                None, None, None, None, None, None, // virtual_url
            ).unwrap();
        }

        assert_eq!(projection.get_locations().len(), 5);
    }
}
