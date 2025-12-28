//! Comprehensive Key State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/key.rs

use chrono::Utc;
use cim_keys::types::KeyAlgorithm;
use cim_keys::state_machines::key::{ArchivedFromState, ExpiryReason, ImportSource, KeyState, RevocationReason};
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid { Uuid::now_v7() }

fn generated_state() -> KeyState {
    KeyState::Generated {
        algorithm: KeyAlgorithm::Rsa { bits: 2048 },
        generated_at: Utc::now(),
        generated_by: test_person_id(),
    }
}

fn imported_state() -> KeyState {
    KeyState::Imported {
        source: ImportSource::File { path: "/tmp/key.pem".to_string() },
        imported_at: Utc::now(),
        imported_by: test_person_id(),
    }
}

fn active_state() -> KeyState {
    KeyState::Active {
        activated_at: Utc::now(),
        usage_count: 0,
        last_used: None,
    }
}

fn rotation_pending_state() -> KeyState {
    KeyState::RotationPending {
        new_key_id: Uuid::now_v7(),
        initiated_at: Utc::now(),
        initiated_by: test_person_id(),
    }
}

fn rotated_state() -> KeyState {
    KeyState::Rotated {
        new_key_id: Uuid::now_v7(),
        rotated_at: Utc::now(),
        rotated_by: test_person_id(),
    }
}

fn revoked_state() -> KeyState {
    KeyState::Revoked {
        reason: RevocationReason::Compromised,
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
    }
}

fn expired_state() -> KeyState {
    KeyState::Expired {
        expired_at: Utc::now(),
        expiry_reason: ExpiryReason::TimeBasedExpiry,
    }
}

fn archived_state(from: ArchivedFromState) -> KeyState {
    KeyState::Archived {
        archived_at: Utc::now(),
        archived_by: test_person_id(),
        previous_state: from,
    }
}

// State Query Tests
#[test]
fn test_is_active_for_all_states() {
    assert!(!generated_state().is_active());
    assert!(!imported_state().is_active());
    assert!(active_state().is_active());
    assert!(!rotation_pending_state().is_active());
    assert!(!rotated_state().is_active());
    assert!(!revoked_state().is_active());
    assert!(!expired_state().is_active());
    assert!(!archived_state(ArchivedFromState::Revoked).is_active());
}

#[test]
fn test_can_use_for_crypto_for_all_states() {
    assert!(!generated_state().can_use_for_crypto());
    assert!(!imported_state().can_use_for_crypto());
    assert!(active_state().can_use_for_crypto());
    assert!(!rotation_pending_state().can_use_for_crypto());
    assert!(!rotated_state().can_use_for_crypto());
    assert!(!revoked_state().can_use_for_crypto());
    assert!(!expired_state().can_use_for_crypto());
    assert!(!archived_state(ArchivedFromState::Revoked).can_use_for_crypto());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!generated_state().is_terminal());
    assert!(!imported_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!rotation_pending_state().is_terminal());
    assert!(!rotated_state().is_terminal());
    assert!(revoked_state().is_terminal());
    assert!(!expired_state().is_terminal());
    assert!(archived_state(ArchivedFromState::Revoked).is_terminal());
}

// Transition Validation Tests
#[test]
fn test_valid_transitions() {
    assert!(generated_state().can_transition_to(&active_state()));
    assert!(imported_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&rotation_pending_state()));
    assert!(rotation_pending_state().can_transition_to(&rotated_state()));
    assert!(rotated_state().can_transition_to(&archived_state(ArchivedFromState::Rotated)));
    assert!(revoked_state().can_transition_to(&archived_state(ArchivedFromState::Revoked)));
    assert!(expired_state().can_transition_to(&archived_state(ArchivedFromState::Expired)));
}

#[test]
fn test_can_transition_any_non_terminal_to_revoked() {
    let revoked = revoked_state();
    assert!(generated_state().can_transition_to(&revoked));
    assert!(imported_state().can_transition_to(&revoked));
    assert!(active_state().can_transition_to(&revoked));
    assert!(rotation_pending_state().can_transition_to(&revoked));
    assert!(rotated_state().can_transition_to(&revoked));
    assert!(expired_state().can_transition_to(&revoked));
}

#[test]
fn test_cannot_transition_terminal_to_revoked() {
    let revoked1 = revoked_state();
    let revoked2 = revoked_state();
    let archived = archived_state(ArchivedFromState::Revoked);
    assert!(!revoked1.can_transition_to(&revoked2));
    assert!(!archived.can_transition_to(&revoked2));
}

#[test]
fn test_cannot_transition_from_archived() {
    let archived = archived_state(ArchivedFromState::Revoked);
    assert!(!archived.can_transition_to(&generated_state()));
    assert!(!archived.can_transition_to(&active_state()));
}

// Metadata Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(generated_state().description(), "Generated (awaiting activation)");
    assert_eq!(imported_state().description(), "Imported (awaiting activation)");
    assert_eq!(active_state().description(), "Active (usable for cryptographic operations)");
    assert_eq!(rotation_pending_state().description(), "Rotation Pending (new key being generated)");
    assert_eq!(rotated_state().description(), "Rotated (superseded by new key)");
    assert_eq!(revoked_state().description(), "Revoked (TERMINAL - cannot be reactivated)");
    assert_eq!(expired_state().description(), "Expired (time-based expiration)");
    assert_eq!(archived_state(ArchivedFromState::Revoked).description(), "Archived (TERMINAL - long-term storage)");
}

// ImportSource Tests
#[test]
fn test_all_import_sources() {
    let sources = vec![
        ImportSource::File { path: "/tmp/key.pem".to_string() },
        ImportSource::YubiKey { serial: "12345".to_string(), slot: "9a".to_string() },
        ImportSource::CIM { cim_id: Uuid::now_v7() },
        ImportSource::ExternalPKI { authority: "VeriSign".to_string() },
    ];

    for source in sources {
        let key = KeyState::Imported {
            source: source.clone(),
            imported_at: Utc::now(),
            imported_by: test_person_id(),
        };
        assert!(!key.is_active());
    }
}

// Revocation Reason Tests
#[test]
fn test_all_revocation_reasons() {
    let reasons = vec![
        RevocationReason::Compromised,
        RevocationReason::EmployeeTermination,
        RevocationReason::Superseded,
        RevocationReason::CessationOfOperation,
        RevocationReason::Administrative { reason: "Policy violation".to_string() },
    ];

    for reason in reasons {
        let key = KeyState::Revoked {
            reason: reason.clone(),
            revoked_at: Utc::now(),
            revoked_by: test_person_id(),
        };
        assert!(key.is_revoked());
        assert!(key.is_terminal());
    }
}

// Expiry Reason Tests
#[test]
fn test_all_expiry_reasons() {
    let reasons = vec![
        ExpiryReason::TimeBasedExpiry,
        ExpiryReason::UsageLimitExceeded,
        ExpiryReason::PolicyExpiration { policy_id: Uuid::now_v7() },
    ];

    for reason in reasons {
        let key = KeyState::Expired {
            expired_at: Utc::now(),
            expiry_reason: reason.clone(),
        };
        assert!(key.is_expired());
    }
}

// Archived State Tests
#[test]
fn test_archived_from_all_states() {
    let from_states = vec![
        ArchivedFromState::Rotated,
        ArchivedFromState::Revoked,
        ArchivedFromState::Expired,
    ];

    for from_state in from_states {
        let key = archived_state(from_state);
        assert!(key.is_terminal());
    }
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        generated_state(),
        imported_state(),
        active_state(),
        rotation_pending_state(),
        rotated_state(),
        revoked_state(),
        expired_state(),
        archived_state(ArchivedFromState::Revoked),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: KeyState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// Key Algorithm Tests
#[test]
fn test_all_key_algorithms() {
    let algorithms = vec![
        KeyAlgorithm::Ed25519,
        KeyAlgorithm::Rsa { bits: 2048 },
        KeyAlgorithm::Rsa { bits: 4096 },
    ];

    for algo in algorithms {
        let key = KeyState::Generated {
            algorithm: algo,
            generated_at: Utc::now(),
            generated_by: test_person_id(),
        };
        assert!(!key.is_active());
    }
}

// Edge Cases
#[test]
fn test_active_with_usage_tracking() {
    let key = KeyState::Active {
        activated_at: Utc::now(),
        usage_count: 1000,
        last_used: Some(Utc::now()),
    };
    assert!(key.is_active());
}

// Additional Coverage Tests for Missing Methods
#[test]
fn test_is_rotation_pending_for_all_states() {
    assert!(!generated_state().is_rotation_pending());
    assert!(!imported_state().is_rotation_pending());
    assert!(!active_state().is_rotation_pending());
    assert!(rotation_pending_state().is_rotation_pending());
    assert!(!rotated_state().is_rotation_pending());
    assert!(!revoked_state().is_rotation_pending());
    assert!(!expired_state().is_rotation_pending());
    assert!(!archived_state(ArchivedFromState::Revoked).is_rotation_pending());
}

#[test]
fn test_is_rotated_for_all_states() {
    assert!(!generated_state().is_rotated());
    assert!(!imported_state().is_rotated());
    assert!(!active_state().is_rotated());
    assert!(!rotation_pending_state().is_rotated());
    assert!(rotated_state().is_rotated());
    assert!(!revoked_state().is_rotated());
    assert!(!expired_state().is_rotated());
    assert!(!archived_state(ArchivedFromState::Rotated).is_rotated());
}

#[test]
fn test_is_expired_for_all_states() {
    assert!(!generated_state().is_expired());
    assert!(!imported_state().is_expired());
    assert!(!active_state().is_expired());
    assert!(!rotation_pending_state().is_expired());
    assert!(!rotated_state().is_expired());
    assert!(!revoked_state().is_expired());
    assert!(expired_state().is_expired());
    assert!(!archived_state(ArchivedFromState::Expired).is_expired());
}

#[test]
fn test_can_be_modified_for_all_states() {
    // Non-terminal states can be modified
    assert!(generated_state().can_be_modified());
    assert!(imported_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(rotation_pending_state().can_be_modified());
    assert!(rotated_state().can_be_modified());
    assert!(expired_state().can_be_modified());

    // Terminal states cannot be modified
    assert!(!revoked_state().can_be_modified());
    assert!(!archived_state(ArchivedFromState::Revoked).can_be_modified());
}
