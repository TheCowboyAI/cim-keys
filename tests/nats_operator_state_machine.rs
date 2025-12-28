//! Comprehensive NATS Operator State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/nats_operator.rs

use chrono::Utc;
use cim_keys::state_machines::nats_operator::{NatsOperatorState, StateError};
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_signing_key_id() -> Uuid { Uuid::now_v7() }
fn test_account_ids(count: usize) -> Vec<Uuid> {
    (0..count).map(|_| Uuid::now_v7()).collect()
}

fn created_state() -> NatsOperatorState {
    NatsOperatorState::Created {
        created_at: Utc::now(),
        created_by: test_person_id(),
        operator_name: "TestOperator".to_string(),
    }
}

fn keys_generated_state() -> NatsOperatorState {
    NatsOperatorState::KeysGenerated {
        signing_key_id: test_signing_key_id(),
        public_key: "ODABC123XYZ789".to_string(),
        generated_at: Utc::now(),
    }
}

fn active_state() -> NatsOperatorState {
    NatsOperatorState::Active {
        activated_at: Utc::now(),
        jwt_issued_at: Utc::now(),
        accounts: test_account_ids(2),
    }
}

fn suspended_state() -> NatsOperatorState {
    NatsOperatorState::Suspended {
        reason: "Security audit".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
    }
}

fn revoked_state() -> NatsOperatorState {
    NatsOperatorState::Revoked {
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        reason: "Operator compromised".to_string(),
        successor_operator_id: Some(Uuid::now_v7()),
    }
}

// State Query Tests
#[test]
fn test_is_active_for_all_states() {
    assert!(!created_state().is_active());
    assert!(!keys_generated_state().is_active());
    assert!(active_state().is_active());
    assert!(!suspended_state().is_active());
    assert!(!revoked_state().is_active());
}

#[test]
fn test_can_create_accounts_for_all_states() {
    assert!(!created_state().can_create_accounts());
    assert!(!keys_generated_state().can_create_accounts());
    assert!(active_state().can_create_accounts());
    assert!(!suspended_state().can_create_accounts());
    assert!(!revoked_state().can_create_accounts());
}

#[test]
fn test_can_sign_jwts_for_all_states() {
    assert!(!created_state().can_sign_jwts());
    assert!(!keys_generated_state().can_sign_jwts());
    assert!(active_state().can_sign_jwts());
    assert!(!suspended_state().can_sign_jwts());
    assert!(!revoked_state().can_sign_jwts());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!created_state().is_terminal());
    assert!(!keys_generated_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!suspended_state().is_terminal());
    assert!(revoked_state().is_terminal());
}

#[test]
fn test_is_suspended_for_all_states() {
    assert!(!created_state().is_suspended());
    assert!(!keys_generated_state().is_suspended());
    assert!(!active_state().is_suspended());
    assert!(suspended_state().is_suspended());
    assert!(!revoked_state().is_suspended());
}

#[test]
fn test_is_revoked_for_all_states() {
    assert!(!created_state().is_revoked());
    assert!(!keys_generated_state().is_revoked());
    assert!(!active_state().is_revoked());
    assert!(!suspended_state().is_revoked());
    assert!(revoked_state().is_revoked());
}

#[test]
fn test_can_be_modified_for_all_states() {
    assert!(created_state().can_be_modified());
    assert!(keys_generated_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(suspended_state().can_be_modified());
    assert!(!revoked_state().can_be_modified());
}

// Transition Validation Tests
#[test]
fn test_can_transition_created_to_keys_generated() {
    assert!(created_state().can_transition_to(&keys_generated_state()));
}

#[test]
fn test_can_transition_keys_generated_to_active() {
    assert!(keys_generated_state().can_transition_to(&active_state()));
}

#[test]
fn test_can_transition_active_to_suspended() {
    assert!(active_state().can_transition_to(&suspended_state()));
}

#[test]
fn test_can_transition_suspended_to_active() {
    assert!(suspended_state().can_transition_to(&active_state()));
}

#[test]
fn test_can_transition_active_to_revoked() {
    assert!(active_state().can_transition_to(&revoked_state()));
}

#[test]
fn test_can_transition_suspended_to_revoked() {
    assert!(suspended_state().can_transition_to(&revoked_state()));
}

#[test]
fn test_cannot_transition_from_revoked() {
    let revoked = revoked_state();
    assert!(!revoked.can_transition_to(&created_state()));
    assert!(!revoked.can_transition_to(&keys_generated_state()));
    assert!(!revoked.can_transition_to(&active_state()));
    assert!(!revoked.can_transition_to(&suspended_state()));
}

#[test]
fn test_cannot_skip_states() {
    // Cannot go directly from Created to Active
    assert!(!created_state().can_transition_to(&active_state()));

    // Cannot go from Created to Suspended
    assert!(!created_state().can_transition_to(&suspended_state()));

    // Cannot go from KeysGenerated to Suspended
    assert!(!keys_generated_state().can_transition_to(&suspended_state()));
}

// Key Generation Tests
#[test]
fn test_generate_keys_from_created() {
    let created = created_state();
    let key_id = test_signing_key_id();
    let public_key = "ODABC123XYZ789".to_string();

    let result = created.generate_keys(key_id, public_key.clone(), Utc::now());
    assert!(result.is_ok());

    let keys_gen = result.unwrap();
    assert!(matches!(keys_gen, NatsOperatorState::KeysGenerated { .. }));
    assert_eq!(keys_gen.signing_key_id(), Some(&key_id));
}

#[test]
fn test_generate_keys_fails_with_empty_public_key() {
    let created = created_state();
    let result = created.generate_keys(test_signing_key_id(), "".to_string(), Utc::now());

    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("empty"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_generate_keys_from_active_fails() {
    let active = active_state();
    let result = active.generate_keys(
        test_signing_key_id(),
        "ODABC123".to_string(),
        Utc::now(),
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::InvalidTransition { .. } => {},
        _ => panic!("Expected InvalidTransition"),
    }
}

// Activation Tests
#[test]
fn test_activate_from_keys_generated() {
    let keys_gen = keys_generated_state();
    let result = keys_gen.activate(Utc::now(), Utc::now());

    assert!(result.is_ok());
    let active = result.unwrap();
    assert!(active.is_active());
    assert_eq!(active.accounts(), Some(&[] as &[Uuid]));
}

#[test]
fn test_activate_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.activate(Utc::now(), Utc::now());

    assert!(result.is_ok());
    let active = result.unwrap();
    assert!(active.is_active());
}

#[test]
fn test_activate_from_created_fails() {
    let created = created_state();
    let result = created.activate(Utc::now(), Utc::now());

    assert!(result.is_err());
}

#[test]
fn test_activate_from_revoked_fails() {
    let revoked = revoked_state();
    let result = revoked.activate(Utc::now(), Utc::now());

    assert!(result.is_err());
}

// Suspension Tests
#[test]
fn test_suspend_from_active() {
    let active = active_state();
    let reason = "Security review".to_string();
    let result = active.suspend(reason.clone(), Utc::now(), test_person_id());

    assert!(result.is_ok());
    let suspended = result.unwrap();
    assert!(suspended.is_suspended());
}

#[test]
fn test_suspend_from_created_fails() {
    let created = created_state();
    let result = created.suspend("Test".to_string(), Utc::now(), test_person_id());

    assert!(result.is_err());
}

#[test]
fn test_suspend_from_keys_generated_fails() {
    let keys_gen = keys_generated_state();
    let result = keys_gen.suspend("Test".to_string(), Utc::now(), test_person_id());

    assert!(result.is_err());
}

#[test]
fn test_suspend_from_revoked_fails() {
    let revoked = revoked_state();
    let result = revoked.suspend("Test".to_string(), Utc::now(), test_person_id());

    assert!(result.is_err());
}

// Revocation Tests
#[test]
fn test_revoke_from_active() {
    let active = active_state();
    let reason = "Operator compromised".to_string();
    let successor_id = Uuid::now_v7();

    let result = active.revoke(
        reason,
        Utc::now(),
        test_person_id(),
        Some(successor_id),
    );

    assert!(result.is_ok());
    let revoked = result.unwrap();
    assert!(revoked.is_revoked());
    assert!(revoked.is_terminal());
}

#[test]
fn test_revoke_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.revoke(
        "Terminated".to_string(),
        Utc::now(),
        test_person_id(),
        None,
    );

    assert!(result.is_ok());
    assert!(result.unwrap().is_revoked());
}

#[test]
fn test_revoke_from_created_fails() {
    let created = created_state();
    let result = created.revoke(
        "Test".to_string(),
        Utc::now(),
        test_person_id(),
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_revoke_from_keys_generated_fails() {
    let keys_gen = keys_generated_state();
    let result = keys_gen.revoke(
        "Test".to_string(),
        Utc::now(),
        test_person_id(),
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_revoke_from_revoked_fails() {
    let revoked = revoked_state();
    let result = revoked.revoke(
        "Test".to_string(),
        Utc::now(),
        test_person_id(),
        None,
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::TerminalState(_) => {},
        _ => panic!("Expected TerminalState error"),
    }
}

// Account Management Tests
#[test]
fn test_add_account_to_active_operator() {
    let active = active_state();
    let new_account = Uuid::now_v7();

    let result = active.add_account(new_account);
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert!(updated.accounts().unwrap().contains(&new_account));
}

#[test]
fn test_add_duplicate_account_is_idempotent() {
    let accounts = test_account_ids(2);
    let active = NatsOperatorState::Active {
        activated_at: Utc::now(),
        jwt_issued_at: Utc::now(),
        accounts: accounts.clone(),
    };

    let duplicate_account = accounts[0];
    let result = active.add_account(duplicate_account);

    assert!(result.is_ok());
    let updated = result.unwrap();
    let updated_accounts = updated.accounts().unwrap();
    assert_eq!(updated_accounts.len(), 2); // Still 2, not 3
}

#[test]
fn test_add_account_from_created_fails() {
    let created = created_state();
    let result = created.add_account(Uuid::now_v7());

    assert!(result.is_err());
}

#[test]
fn test_add_account_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.add_account(Uuid::now_v7());

    assert!(result.is_err());
}

#[test]
fn test_add_account_from_revoked_fails() {
    let revoked = revoked_state();
    let result = revoked.add_account(Uuid::now_v7());

    assert!(result.is_err());
}

// Metadata Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(created_state().description(), "Created (awaiting key generation)");
    assert_eq!(keys_generated_state().description(), "Keys Generated (ready for activation)");
    assert_eq!(active_state().description(), "Active (can sign account JWTs)");
    assert_eq!(suspended_state().description(), "Suspended (temporarily disabled)");
    assert_eq!(revoked_state().description(), "Revoked (TERMINAL - permanently disabled)");
}

#[test]
fn test_signing_key_id_getter() {
    let key_id = test_signing_key_id();
    let keys_gen = NatsOperatorState::KeysGenerated {
        signing_key_id: key_id,
        public_key: "ODABC123".to_string(),
        generated_at: Utc::now(),
    };

    assert_eq!(created_state().signing_key_id(), None);
    assert_eq!(keys_gen.signing_key_id(), Some(&key_id));
    assert_eq!(active_state().signing_key_id(), None);
    assert_eq!(suspended_state().signing_key_id(), None);
    assert_eq!(revoked_state().signing_key_id(), None);
}

#[test]
fn test_accounts_getter() {
    assert_eq!(created_state().accounts(), None);
    assert_eq!(keys_generated_state().accounts(), None);
    assert!(active_state().accounts().is_some());
    assert_eq!(suspended_state().accounts(), None);
    assert_eq!(revoked_state().accounts(), None);
}

// Lifecycle Workflow Tests
#[test]
fn test_complete_lifecycle_created_to_revoked() {
    // Created → KeysGenerated
    let created = created_state();
    let keys_gen = created
        .generate_keys(test_signing_key_id(), "ODABC123".to_string(), Utc::now())
        .unwrap();
    assert!(matches!(keys_gen, NatsOperatorState::KeysGenerated { .. }));

    // KeysGenerated → Active
    let active = keys_gen.activate(Utc::now(), Utc::now()).unwrap();
    assert!(active.is_active());

    // Active → Add accounts
    let with_account1 = active.add_account(Uuid::now_v7()).unwrap();
    let with_account2 = with_account1.add_account(Uuid::now_v7()).unwrap();
    assert_eq!(with_account2.accounts().unwrap().len(), 2);

    // Active → Suspended
    let suspended = with_account2
        .suspend("Security review".to_string(), Utc::now(), test_person_id())
        .unwrap();
    assert!(suspended.is_suspended());

    // Suspended → Active (reactivation)
    let reactivated = suspended.activate(Utc::now(), Utc::now()).unwrap();
    assert!(reactivated.is_active());

    // Active → Revoked
    let revoked = reactivated
        .revoke("Compromised".to_string(), Utc::now(), test_person_id(), None)
        .unwrap();
    assert!(revoked.is_revoked());
    assert!(revoked.is_terminal());
}

#[test]
fn test_lifecycle_suspended_to_revoked() {
    // Active → Suspended → Revoked (without reactivation)
    let active = active_state();
    let suspended = active
        .suspend("Review".to_string(), Utc::now(), test_person_id())
        .unwrap();
    let revoked = suspended
        .revoke("Terminated".to_string(), Utc::now(), test_person_id(), None)
        .unwrap();

    assert!(revoked.is_terminal());
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        created_state(),
        keys_generated_state(),
        active_state(),
        suspended_state(),
        revoked_state(),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: NatsOperatorState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// Edge Case Tests
#[test]
fn test_active_operator_with_many_accounts() {
    let active = active_state();
    let mut current = active;

    // Add 10 accounts
    for _ in 0..10 {
        current = current.add_account(Uuid::now_v7()).unwrap();
    }

    assert_eq!(current.accounts().unwrap().len(), 12); // 2 initial + 10 new
    assert!(current.is_active());
}

#[test]
fn test_revoked_with_successor() {
    let successor_id = Uuid::now_v7();
    let revoked = NatsOperatorState::Revoked {
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        reason: "Migration".to_string(),
        successor_operator_id: Some(successor_id),
    };

    assert!(revoked.is_revoked());
    assert!(revoked.is_terminal());
}

#[test]
fn test_revoked_without_successor() {
    let revoked = NatsOperatorState::Revoked {
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        reason: "Decommissioned".to_string(),
        successor_operator_id: None,
    };

    assert!(revoked.is_revoked());
    assert!(revoked.is_terminal());
}
