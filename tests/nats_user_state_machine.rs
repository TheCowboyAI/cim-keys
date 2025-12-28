//! Comprehensive NATS User State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/nats_user.rs

use chrono::Utc;
use cim_keys::state_machines::nats_user::{NatsUserState, NatsUserPermissions, StateError};
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_account_id() -> Uuid { Uuid::now_v7() }

fn test_permissions() -> NatsUserPermissions {
    NatsUserPermissions {
        publish: vec!["user.>".to_string()],
        subscribe: vec!["user.>".to_string()],
        allow_responses: true,
        max_payload: Some(512 * 1024),
    }
}

fn created_state() -> NatsUserState {
    NatsUserState::Created {
        created_at: Utc::now(),
        created_by: test_person_id(),
        account_id: test_account_id(),
        person_id: test_person_id(),
    }
}

fn active_state() -> NatsUserState {
    NatsUserState::Active {
        permissions: test_permissions(),
        activated_at: Utc::now(),
        last_connection: None,
    }
}

fn suspended_state() -> NatsUserState {
    NatsUserState::Suspended {
        reason: "Account review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
    }
}

fn reactivated_state() -> NatsUserState {
    NatsUserState::Reactivated {
        permissions: test_permissions(),
        reactivated_at: Utc::now(),
        reactivated_by: test_person_id(),
    }
}

fn deleted_state() -> NatsUserState {
    NatsUserState::Deleted {
        deleted_at: Utc::now(),
        deleted_by: test_person_id(),
        reason: "User removed".to_string(),
    }
}

// State Query Tests
#[test]
fn test_is_active_for_all_states() {
    assert!(!created_state().is_active());
    assert!(active_state().is_active());
    assert!(!suspended_state().is_active());
    assert!(reactivated_state().is_active());
    assert!(!deleted_state().is_active());
}

#[test]
fn test_can_connect_for_all_states() {
    assert!(!created_state().can_connect());
    assert!(active_state().can_connect());
    assert!(!suspended_state().can_connect());
    assert!(reactivated_state().can_connect());
    assert!(!deleted_state().can_connect());
}

#[test]
fn test_can_pubsub_for_all_states() {
    assert!(!created_state().can_pubsub());
    assert!(active_state().can_pubsub());
    assert!(!suspended_state().can_pubsub());
    assert!(reactivated_state().can_pubsub());
    assert!(!deleted_state().can_pubsub());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!created_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!suspended_state().is_terminal());
    assert!(!reactivated_state().is_terminal());
    assert!(deleted_state().is_terminal());
}

#[test]
fn test_is_suspended_for_all_states() {
    assert!(!created_state().is_suspended());
    assert!(!active_state().is_suspended());
    assert!(suspended_state().is_suspended());
    assert!(!reactivated_state().is_suspended());
    assert!(!deleted_state().is_suspended());
}

#[test]
fn test_is_deleted_for_all_states() {
    assert!(!created_state().is_deleted());
    assert!(!active_state().is_deleted());
    assert!(!suspended_state().is_deleted());
    assert!(!reactivated_state().is_deleted());
    assert!(deleted_state().is_deleted());
}

#[test]
fn test_can_be_modified_for_all_states() {
    assert!(created_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(suspended_state().can_be_modified());
    assert!(reactivated_state().can_be_modified());
    assert!(!deleted_state().can_be_modified());
}

// Transition Validation Tests
#[test]
fn test_valid_transitions() {
    assert!(created_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&suspended_state()));
    assert!(suspended_state().can_transition_to(&reactivated_state()));
    assert!(reactivated_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&deleted_state()));
    assert!(suspended_state().can_transition_to(&deleted_state()));
    assert!(reactivated_state().can_transition_to(&deleted_state()));
}

#[test]
fn test_cannot_transition_from_deleted() {
    let deleted = deleted_state();
    assert!(!deleted.can_transition_to(&created_state()));
    assert!(!deleted.can_transition_to(&active_state()));
    assert!(!deleted.can_transition_to(&suspended_state()));
    assert!(!deleted.can_transition_to(&reactivated_state()));
}

// Activation Tests
#[test]
fn test_activate_from_created() {
    let created = created_state();
    let result = created.activate(test_permissions(), Utc::now());
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_activate_from_reactivated() {
    let reactivated = reactivated_state();
    let result = reactivated.activate(test_permissions(), Utc::now());
    assert!(result.is_ok());
}

#[test]
fn test_activate_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.activate(test_permissions(), Utc::now());
    assert!(result.is_err());
}

// Suspension Tests
#[test]
fn test_suspend_from_active() {
    let active = active_state();
    let result = active.suspend("Review".to_string(), Utc::now(), test_person_id());
    assert!(result.is_ok());
    assert!(result.unwrap().is_suspended());
}

#[test]
fn test_suspend_from_created_fails() {
    let created = created_state();
    let result = created.suspend("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
}

// Reactivation Tests
#[test]
fn test_reactivate_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.reactivate(test_permissions(), Utc::now(), test_person_id());
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_reactivate_from_active_fails() {
    let active = active_state();
    let result = active.reactivate(test_permissions(), Utc::now(), test_person_id());
    assert!(result.is_err());
}

// Deletion Tests
#[test]
fn test_delete_from_active() {
    let active = active_state();
    let result = active.delete("Removed".to_string(), Utc::now(), test_person_id());
    assert!(result.is_ok());
    assert!(result.unwrap().is_deleted());
}

#[test]
fn test_delete_from_deleted_fails() {
    let deleted = deleted_state();
    let result = deleted.delete("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::TerminalState(_) => {},
        _ => panic!("Expected TerminalState error"),
    }
}

// Connection Recording Tests
#[test]
fn test_record_connection_from_active() {
    let active = active_state();
    let conn_time = Utc::now();
    let result = active.record_connection(conn_time);
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.last_connection(), Some(&conn_time));
}

#[test]
fn test_record_connection_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.record_connection(Utc::now());
    assert!(result.is_err());
}

// Metadata Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(created_state().description(), "Created (awaiting permissions)");
    assert_eq!(active_state().description(), "Active (can connect and pub/sub)");
    assert_eq!(suspended_state().description(), "Suspended (temporarily disabled)");
    assert_eq!(reactivated_state().description(), "Reactivated (permissions restored)");
    assert_eq!(deleted_state().description(), "Deleted (TERMINAL - permanently removed)");
}

#[test]
fn test_permissions_getter() {
    assert_eq!(created_state().permissions(), None);
    assert!(active_state().permissions().is_some());
    assert_eq!(suspended_state().permissions(), None);
    assert!(reactivated_state().permissions().is_some());
    assert_eq!(deleted_state().permissions(), None);
}

#[test]
fn test_last_connection_getter() {
    assert_eq!(created_state().last_connection(), None);
    assert_eq!(active_state().last_connection(), None);

    let active_with_conn = NatsUserState::Active {
        permissions: test_permissions(),
        activated_at: Utc::now(),
        last_connection: Some(Utc::now()),
    };
    assert!(active_with_conn.last_connection().is_some());
}

// Lifecycle Tests
#[test]
fn test_complete_lifecycle() {
    let created = created_state();
    let active = created.activate(test_permissions(), Utc::now()).unwrap();
    let with_conn = active.record_connection(Utc::now()).unwrap();
    let suspended = with_conn.suspend("Review".to_string(), Utc::now(), test_person_id()).unwrap();
    let reactivated = suspended.reactivate(test_permissions(), Utc::now(), test_person_id()).unwrap();
    let active_again = reactivated.activate(test_permissions(), Utc::now()).unwrap();
    let deleted = active_again.delete("Cleanup".to_string(), Utc::now(), test_person_id()).unwrap();
    assert!(deleted.is_terminal());
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        created_state(),
        active_state(),
        suspended_state(),
        reactivated_state(),
        deleted_state(),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: NatsUserState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}
