//! Comprehensive Key Events Tests
//!
//! Target: 90%+ coverage of src/events/key.rs
//!
//! Tests all 10 event types for key lifecycle, rotation, and imports/exports.

use chrono::Utc;
use cim_keys::events::key::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use cim_keys::types::{
    KeyAlgorithm, KeyPurpose, KeyMetadata, ImportSource, KeyFormat,
    ExportDestination, RevocationReason,
};
use cim_keys::domain::{KeyOwnership, KeyOwnerRole};
use uuid::Uuid;
use std::collections::HashMap;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_key_id() -> Uuid { Uuid::now_v7() }
fn test_rotation_id() -> Uuid { Uuid::now_v7() }
fn test_secret_id() -> Uuid { Uuid::now_v7() }
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_partition_id() -> Uuid { Uuid::now_v7() }
fn test_org_id() -> Uuid { Uuid::now_v7() }

fn test_key_metadata() -> KeyMetadata {
    KeyMetadata {
        label: "Test Key".to_string(),
        description: Some("Generated for testing".to_string()),
        tags: vec!["test".to_string(), "rsa".to_string()],
        attributes: HashMap::new(),
        jwt_kid: Some("test-kid-123".to_string()),
        jwt_alg: Some("RS256".to_string()),
        jwt_use: None,
    }
}

fn test_key_ownership() -> KeyOwnership {
    KeyOwnership {
        person_id: test_person_id(),
        organization_id: test_org_id(),
        role: KeyOwnerRole::Developer,
        delegations: vec![],
    }
}

fn sample_key_generated() -> KeyGeneratedEvent {
    KeyGeneratedEvent {
        key_id: test_key_id(),
        algorithm: KeyAlgorithm::Ed25519,
        purpose: KeyPurpose::Signing,
        generated_at: Utc::now(),
        generated_by: "key_admin".to_string(),
        hardware_backed: false,
        metadata: test_key_metadata(),
        ownership: Some(test_key_ownership()),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_key_imported() -> KeyImportedEvent {
    KeyImportedEvent {
        key_id: test_key_id(),
        source: ImportSource::File { path: "/tmp/key.pem".to_string() },
        format: KeyFormat::Pem,
        imported_at: Utc::now(),
        imported_by: "admin".to_string(),
        metadata: test_key_metadata(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_key_exported() -> KeyExportedEvent {
    KeyExportedEvent {
        key_id: test_key_id(),
        format: KeyFormat::Pem,
        include_private: false,
        exported_at: Utc::now(),
        exported_by: "admin".to_string(),
        destination: ExportDestination::File { path: "/mnt/encrypted/keys/key.pem".to_string() },
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_key_stored_offline() -> KeyStoredOfflineEvent {
    KeyStoredOfflineEvent {
        key_id: test_key_id(),
        partition_id: test_partition_id(),
        encrypted: true,
        stored_at: Utc::now(),
        checksum: "sha256:abcd1234...".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_key_revoked() -> KeyRevokedEvent {
    KeyRevokedEvent {
        key_id: test_key_id(),
        reason: RevocationReason::KeyCompromise,
        revoked_at: Utc::now(),
        revoked_by: "security_admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_key_rotation_initiated() -> KeyRotationInitiatedEvent {
    KeyRotationInitiatedEvent {
        rotation_id: test_rotation_id(),
        old_key_id: test_key_id(),
        new_key_id: Uuid::now_v7(),
        rotation_reason: "Scheduled rotation".to_string(),
        initiated_at: Utc::now(),
        initiated_by: "key_admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_key_rotation_completed() -> KeyRotationCompletedEvent {
    KeyRotationCompletedEvent {
        rotation_id: test_rotation_id(),
        old_key_id: test_key_id(),
        new_key_id: Uuid::now_v7(),
        completed_at: Utc::now(),
        transition_period_ends: Utc::now() + chrono::Duration::days(30),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_ssh_key_generated() -> SshKeyGeneratedEvent {
    SshKeyGeneratedEvent {
        key_id: test_key_id(),
        key_type: "ed25519".to_string(),
        comment: "user@example.com".to_string(),
        generated_at: Utc::now(),
        generated_by: "user".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_gpg_key_generated() -> GpgKeyGeneratedEvent {
    GpgKeyGeneratedEvent {
        key_id: test_key_id(),
        fingerprint: "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234".to_string(),
        user_id: "John Doe <john@example.com>".to_string(),
        key_type: "RSA".to_string(),
        generated_at: Utc::now(),
        generated_by: "john".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_totp_secret_generated() -> TotpSecretGeneratedEvent {
    TotpSecretGeneratedEvent {
        secret_id: test_secret_id(),
        person_id: test_person_id(),
        algorithm: "SHA1".to_string(),
        digits: 6,
        period: 30,
        generated_at: Utc::now(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (10 event types)
// =============================================================================

#[test]
fn test_key_generated_serialization() {
    let event = sample_key_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: KeyGeneratedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.hardware_backed, deserialized.hardware_backed);
}

#[test]
fn test_key_imported_serialization() {
    let event = sample_key_imported();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: KeyImportedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.key_id, deserialized.key_id);
}

#[test]
fn test_key_exported_serialization() {
    let event = sample_key_exported();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: KeyExportedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.include_private, deserialized.include_private);
}

#[test]
fn test_key_stored_offline_serialization() {
    let event = sample_key_stored_offline();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: KeyStoredOfflineEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.encrypted, deserialized.encrypted);
}

#[test]
fn test_key_revoked_serialization() {
    let event = sample_key_revoked();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: KeyRevokedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.key_id, deserialized.key_id);
}

#[test]
fn test_key_rotation_initiated_serialization() {
    let event = sample_key_rotation_initiated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: KeyRotationInitiatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.rotation_id, deserialized.rotation_id);
    assert_eq!(event.old_key_id, deserialized.old_key_id);
    assert_eq!(event.new_key_id, deserialized.new_key_id);
}

#[test]
fn test_key_rotation_completed_serialization() {
    let event = sample_key_rotation_completed();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: KeyRotationCompletedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.rotation_id, deserialized.rotation_id);
}

#[test]
fn test_ssh_key_generated_serialization() {
    let event = sample_ssh_key_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: SshKeyGeneratedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.key_type, deserialized.key_type);
}

#[test]
fn test_gpg_key_generated_serialization() {
    let event = sample_gpg_key_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: GpgKeyGeneratedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.key_id, deserialized.key_id);
    assert_eq!(event.fingerprint, deserialized.fingerprint);
}

#[test]
fn test_totp_secret_generated_serialization() {
    let event = sample_totp_secret_generated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: TotpSecretGeneratedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.secret_id, deserialized.secret_id);
    assert_eq!(event.person_id, deserialized.person_id);
}

// =============================================================================
// KeyEvents Enum Serialization
// =============================================================================

#[test]
fn test_key_events_enum_serialization() {
    let events = vec![
        KeyEvents::KeyGenerated(sample_key_generated()),
        KeyEvents::KeyImported(sample_key_imported()),
        KeyEvents::KeyExported(sample_key_exported()),
        KeyEvents::KeyStoredOffline(sample_key_stored_offline()),
        KeyEvents::KeyRevoked(sample_key_revoked()),
        KeyEvents::KeyRotationInitiated(sample_key_rotation_initiated()),
        KeyEvents::KeyRotationCompleted(sample_key_rotation_completed()),
        KeyEvents::SshKeyGenerated(sample_ssh_key_generated()),
        KeyEvents::GpgKeyGenerated(sample_gpg_key_generated()),
        KeyEvents::TotpSecretGenerated(sample_totp_secret_generated()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: KeyEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let generated = KeyGeneratedEvent {
        causation_id: None,
        correlation_id,
        ..sample_key_generated()
    };
    let exported = KeyExportedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_key_exported()
    };

    assert_eq!(generated.causation_id, None);
    assert_eq!(exported.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let generated = KeyGeneratedEvent { correlation_id, ..sample_key_generated() };
    let stored = KeyStoredOfflineEvent { correlation_id, ..sample_key_stored_offline() };
    let exported = KeyExportedEvent { correlation_id, ..sample_key_exported() };

    assert_eq!(generated.correlation_id, correlation_id);
    assert_eq!(stored.correlation_id, correlation_id);
    assert_eq!(exported.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let key = sample_key_generated();
    assert_ne!(key.key_id, Uuid::nil());
    assert_ne!(key.correlation_id, Uuid::nil());

    let rotation = sample_key_rotation_initiated();
    assert_ne!(rotation.rotation_id, Uuid::nil());
    assert_ne!(rotation.old_key_id, Uuid::nil());
    assert_ne!(rotation.new_key_id, Uuid::nil());
}

#[test]
fn test_key_metadata_fields() {
    let key = sample_key_generated();
    assert!(!key.metadata.label.is_empty());
    assert!(!key.metadata.tags.is_empty());
}

#[test]
fn test_key_ownership_fields() {
    let key = sample_key_generated();
    if let Some(ownership) = &key.ownership {
        assert_ne!(ownership.person_id, Uuid::nil());
        assert_ne!(ownership.organization_id, Uuid::nil());
    }
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let key_id = test_key_id();
    let rotation_id = test_rotation_id();
    let secret_id = test_secret_id();

    let events = vec![
        KeyEvents::KeyGenerated(KeyGeneratedEvent { key_id, ..sample_key_generated() }),
        KeyEvents::KeyImported(KeyImportedEvent { key_id, ..sample_key_imported() }),
        KeyEvents::KeyExported(KeyExportedEvent { key_id, ..sample_key_exported() }),
        KeyEvents::KeyStoredOffline(KeyStoredOfflineEvent { key_id, ..sample_key_stored_offline() }),
        KeyEvents::KeyRevoked(KeyRevokedEvent { key_id, ..sample_key_revoked() }),
        KeyEvents::KeyRotationInitiated(KeyRotationInitiatedEvent { rotation_id, ..sample_key_rotation_initiated() }),
        KeyEvents::KeyRotationCompleted(KeyRotationCompletedEvent { rotation_id, ..sample_key_rotation_completed() }),
        KeyEvents::SshKeyGenerated(SshKeyGeneratedEvent { key_id, ..sample_ssh_key_generated() }),
        KeyEvents::GpgKeyGenerated(GpgKeyGeneratedEvent { key_id, ..sample_gpg_key_generated() }),
        KeyEvents::TotpSecretGenerated(TotpSecretGeneratedEvent { secret_id, ..sample_totp_secret_generated() }),
    ];

    // Verify each event returns the correct aggregate ID
    assert_eq!(events[0].aggregate_id(), key_id);
    assert_eq!(events[1].aggregate_id(), key_id);
    assert_eq!(events[2].aggregate_id(), key_id);
    assert_eq!(events[3].aggregate_id(), key_id);
    assert_eq!(events[4].aggregate_id(), key_id);
    assert_eq!(events[5].aggregate_id(), rotation_id); // KeyRotationInitiated uses rotation_id
    assert_eq!(events[6].aggregate_id(), rotation_id); // KeyRotationCompleted uses rotation_id
    assert_eq!(events[7].aggregate_id(), key_id);
    assert_eq!(events[8].aggregate_id(), key_id);
    assert_eq!(events[9].aggregate_id(), secret_id); // TotpSecretGenerated uses secret_id
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(KeyEvents::KeyGenerated(sample_key_generated()).event_type(), "KeyGenerated");
    assert_eq!(KeyEvents::KeyImported(sample_key_imported()).event_type(), "KeyImported");
    assert_eq!(KeyEvents::KeyExported(sample_key_exported()).event_type(), "KeyExported");
    assert_eq!(KeyEvents::KeyStoredOffline(sample_key_stored_offline()).event_type(), "KeyStoredOffline");
    assert_eq!(KeyEvents::KeyRevoked(sample_key_revoked()).event_type(), "KeyRevoked");
    assert_eq!(KeyEvents::KeyRotationInitiated(sample_key_rotation_initiated()).event_type(), "KeyRotationInitiated");
    assert_eq!(KeyEvents::KeyRotationCompleted(sample_key_rotation_completed()).event_type(), "KeyRotationCompleted");
    assert_eq!(KeyEvents::SshKeyGenerated(sample_ssh_key_generated()).event_type(), "SshKeyGenerated");
    assert_eq!(KeyEvents::GpgKeyGenerated(sample_gpg_key_generated()).event_type(), "GpgKeyGenerated");
    assert_eq!(KeyEvents::TotpSecretGenerated(sample_totp_secret_generated()).event_type(), "TotpSecretGenerated");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_key_lifecycle() {
    let key_id = test_key_id();
    let correlation_id = Uuid::now_v7();

    let generated = KeyGeneratedEvent {
        key_id,
        correlation_id,
        ..sample_key_generated()
    };
    let stored = KeyStoredOfflineEvent {
        key_id,
        correlation_id,
        ..sample_key_stored_offline()
    };
    let exported = KeyExportedEvent {
        key_id,
        correlation_id,
        ..sample_key_exported()
    };
    let revoked = KeyRevokedEvent {
        key_id,
        correlation_id,
        ..sample_key_revoked()
    };

    assert_eq!(generated.key_id, key_id);
    assert_eq!(stored.key_id, key_id);
    assert_eq!(exported.key_id, key_id);
    assert_eq!(revoked.key_id, key_id);
}

#[test]
fn test_key_rotation_workflow() {
    let rotation_id = test_rotation_id();
    let old_key_id = test_key_id();
    let new_key_id = Uuid::now_v7();
    let correlation_id = Uuid::now_v7();

    let initiated = KeyRotationInitiatedEvent {
        rotation_id,
        old_key_id,
        new_key_id,
        correlation_id,
        ..sample_key_rotation_initiated()
    };
    let completed = KeyRotationCompletedEvent {
        rotation_id,
        old_key_id,
        new_key_id,
        correlation_id,
        ..sample_key_rotation_completed()
    };

    assert_eq!(initiated.rotation_id, rotation_id);
    assert_eq!(initiated.old_key_id, old_key_id);
    assert_eq!(initiated.new_key_id, new_key_id);
    assert_eq!(completed.rotation_id, rotation_id);
    assert_eq!(completed.old_key_id, old_key_id);
    assert_eq!(completed.new_key_id, new_key_id);
}

#[test]
fn test_key_import_export_workflow() {
    let key_id = test_key_id();
    let correlation_id = Uuid::now_v7();

    let imported = KeyImportedEvent {
        key_id,
        correlation_id,
        ..sample_key_imported()
    };
    let exported = KeyExportedEvent {
        key_id,
        correlation_id,
        include_private: false, // Export public only
        ..sample_key_exported()
    };

    assert_eq!(imported.key_id, key_id);
    assert_eq!(exported.key_id, key_id);
    assert!(!exported.include_private);
}

#[test]
fn test_ssh_gpg_key_workflow() {
    let correlation_id = Uuid::now_v7();

    let ssh_key = SshKeyGeneratedEvent {
        correlation_id,
        ..sample_ssh_key_generated()
    };
    let gpg_key = GpgKeyGeneratedEvent {
        correlation_id,
        ..sample_gpg_key_generated()
    };

    assert_eq!(ssh_key.correlation_id, correlation_id);
    assert_eq!(gpg_key.correlation_id, correlation_id);
}

#[test]
fn test_totp_workflow() {
    let person_id = test_person_id();
    let correlation_id = Uuid::now_v7();

    let totp = TotpSecretGeneratedEvent {
        person_id,
        correlation_id,
        ..sample_totp_secret_generated()
    };

    assert_eq!(totp.person_id, person_id);
    assert_eq!(totp.digits, 6);
    assert_eq!(totp.period, 30);
}

#[test]
fn test_hardware_backed_keys() {
    let yubikey_key = KeyGeneratedEvent {
        hardware_backed: true,
        ..sample_key_generated()
    };
    let software_key = KeyGeneratedEvent {
        hardware_backed: false,
        ..sample_key_generated()
    };

    assert!(yubikey_key.hardware_backed);
    assert!(!software_key.hardware_backed);
}
