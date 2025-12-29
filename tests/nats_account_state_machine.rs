//! Comprehensive NATS Account State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/nats_account.rs

use chrono::Utc;
use cim_keys::state_machines::nats_account::{NatsAccountState, NatsPermissions, StateError};
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_operator_id() -> Uuid { Uuid::now_v7() }
fn test_user_ids(count: usize) -> Vec<Uuid> {
    (0..count).map(|_| Uuid::now_v7()).collect()
}

fn test_permissions() -> NatsPermissions {
    NatsPermissions {
        publish: vec!["test.>".to_string()],
        subscribe: vec!["test.>".to_string()],
        allow_responses: true,
        max_connections: Some(100),
        max_payload: Some(1024 * 1024),
    }
}

fn created_state() -> NatsAccountState {
    NatsAccountState::Created {
        created_by: test_person_id(),
        operator_id: test_operator_id(),
    }
}

fn active_state() -> NatsAccountState {
    NatsAccountState::Active {
        permissions: test_permissions(),
        activated_at: Utc::now(),
        users: test_user_ids(2),
    }
}

fn suspended_state() -> NatsAccountState {
    NatsAccountState::Suspended {
        reason: "Security review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
    }
}

fn reactivated_state() -> NatsAccountState {
    NatsAccountState::Reactivated {
        permissions: test_permissions(),
        reactivated_at: Utc::now(),
        reactivated_by: test_person_id(),
    }
}

fn deleted_state() -> NatsAccountState {
    NatsAccountState::Deleted {
        deleted_at: Utc::now(),
        deleted_by: test_person_id(),
        reason: "Account decommissioned".to_string(),
    }
}

// State Query Tests
#[test]
fn test_is_active_for_all_states() {
    assert!(!created_state().is_active());
    assert!(active_state().is_active());
    assert!(!suspended_state().is_active());
    assert!(reactivated_state().is_active()); // Reactivated counts as active!
    assert!(!deleted_state().is_active());
}

#[test]
fn test_can_create_users_for_all_states() {
    assert!(!created_state().can_create_users());
    assert!(active_state().can_create_users());
    assert!(!suspended_state().can_create_users());
    assert!(reactivated_state().can_create_users());
    assert!(!deleted_state().can_create_users());
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
fn test_can_transition_created_to_active() {
    assert!(created_state().can_transition_to(&active_state()));
}

#[test]
fn test_can_transition_active_to_suspended() {
    assert!(active_state().can_transition_to(&suspended_state()));
}

#[test]
fn test_can_transition_suspended_to_reactivated() {
    assert!(suspended_state().can_transition_to(&reactivated_state()));
}

#[test]
fn test_can_transition_reactivated_to_active() {
    assert!(reactivated_state().can_transition_to(&active_state()));
}

#[test]
fn test_can_transition_active_to_deleted() {
    assert!(active_state().can_transition_to(&deleted_state()));
}

#[test]
fn test_can_transition_suspended_to_deleted() {
    assert!(suspended_state().can_transition_to(&deleted_state()));
}

#[test]
fn test_can_transition_reactivated_to_deleted() {
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

#[test]
fn test_cannot_skip_states() {
    // Cannot go directly from Created to Suspended
    assert!(!created_state().can_transition_to(&suspended_state()));

    // Cannot go directly from Created to Deleted
    assert!(!created_state().can_transition_to(&deleted_state()));

    // Cannot go directly from Active to Reactivated
    assert!(!active_state().can_transition_to(&reactivated_state()));
}

// Activation Tests
#[test]
fn test_activate_from_created() {
    let created = created_state();
    let permissions = test_permissions();
    let result = created.activate(permissions.clone(), Utc::now());

    assert!(result.is_ok());
    let active = result.unwrap();
    assert!(active.is_active());
    assert_eq!(active.users(), Some(&[] as &[Uuid]));
    assert_eq!(active.permissions(), Some(&permissions));
}

#[test]
fn test_activate_from_reactivated() {
    let reactivated = reactivated_state();
    let permissions = test_permissions();
    let result = reactivated.activate(permissions.clone(), Utc::now());

    assert!(result.is_ok());
    let active = result.unwrap();
    assert!(active.is_active());
}

#[test]
fn test_activate_from_active_fails() {
    let active = active_state();
    let result = active.activate(test_permissions(), Utc::now());

    assert!(result.is_err());
}

#[test]
fn test_activate_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.activate(test_permissions(), Utc::now());

    assert!(result.is_err());
}

#[test]
fn test_activate_from_deleted_fails() {
    let deleted = deleted_state();
    let result = deleted.activate(test_permissions(), Utc::now());

    assert!(result.is_err());
}

// Suspension Tests
#[test]
fn test_suspend_from_active() {
    let active = active_state();
    let reason = "Policy violation".to_string();
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
fn test_suspend_from_reactivated_fails() {
    let reactivated = reactivated_state();
    let result = reactivated.suspend("Test".to_string(), Utc::now(), test_person_id());

    assert!(result.is_err());
}

#[test]
fn test_suspend_from_deleted_fails() {
    let deleted = deleted_state();
    let result = deleted.suspend("Test".to_string(), Utc::now(), test_person_id());

    assert!(result.is_err());
}

// Reactivation Tests
#[test]
fn test_reactivate_from_suspended() {
    let suspended = suspended_state();
    let permissions = test_permissions();
    let result = suspended.reactivate(
        permissions.clone(),
        Utc::now(),
        test_person_id(),
    );

    assert!(result.is_ok());
    let reactivated = result.unwrap();
    assert!(reactivated.is_active());
    assert_eq!(reactivated.permissions(), Some(&permissions));
}

#[test]
fn test_reactivate_from_created_fails() {
    let created = created_state();
    let result = created.reactivate(
        test_permissions(),
        Utc::now(),
        test_person_id(),
    );

    assert!(result.is_err());
}

#[test]
fn test_reactivate_from_active_fails() {
    let active = active_state();
    let result = active.reactivate(
        test_permissions(),
        Utc::now(),
        test_person_id(),
    );

    assert!(result.is_err());
}

#[test]
fn test_reactivate_from_deleted_fails() {
    let deleted = deleted_state();
    let result = deleted.reactivate(
        test_permissions(),
        Utc::now(),
        test_person_id(),
    );

    assert!(result.is_err());
}

// Deletion Tests
#[test]
fn test_delete_from_active() {
    let active = active_state();
    let reason = "Account no longer needed".to_string();
    let result = active.delete(reason, Utc::now(), test_person_id());

    assert!(result.is_ok());
    let deleted = result.unwrap();
    assert!(deleted.is_deleted());
    assert!(deleted.is_terminal());
}

#[test]
fn test_delete_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.delete("Terminated".to_string(), Utc::now(), test_person_id());

    assert!(result.is_ok());
    assert!(result.unwrap().is_deleted());
}

#[test]
fn test_delete_from_reactivated() {
    let reactivated = reactivated_state();
    let result = reactivated.delete("Cleanup".to_string(), Utc::now(), test_person_id());

    assert!(result.is_ok());
    assert!(result.unwrap().is_deleted());
}

#[test]
fn test_delete_from_created_fails() {
    let created = created_state();
    let result = created.delete("Test".to_string(), Utc::now(), test_person_id());

    assert!(result.is_err());
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

// User Management Tests
#[test]
fn test_add_user_to_active_account() {
    let active = active_state();
    let new_user = Uuid::now_v7();

    let result = active.add_user(new_user);
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert!(updated.users().unwrap().contains(&new_user));
}

#[test]
fn test_add_duplicate_user_is_idempotent() {
    let users = test_user_ids(2);
    let active = NatsAccountState::Active {
        permissions: test_permissions(),
        activated_at: Utc::now(),
        users: users.clone(),
    };

    let duplicate_user = users[0];
    let result = active.add_user(duplicate_user);

    assert!(result.is_ok());
    let updated_state = result.unwrap();
    let updated_users = updated_state.users().unwrap();
    assert_eq!(updated_users.len(), 2); // Still 2, not 3
}

#[test]
fn test_add_user_from_created_fails() {
    let created = created_state();
    let result = created.add_user(Uuid::now_v7());

    assert!(result.is_err());
}

#[test]
fn test_add_user_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.add_user(Uuid::now_v7());

    assert!(result.is_err());
}

#[test]
fn test_add_user_from_reactivated_fails() {
    let reactivated = reactivated_state();
    let result = reactivated.add_user(Uuid::now_v7());

    assert!(result.is_err());
}

#[test]
fn test_add_user_from_deleted_fails() {
    let deleted = deleted_state();
    let result = deleted.add_user(Uuid::now_v7());

    assert!(result.is_err());
}

// Metadata Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(created_state().description(), "Created (awaiting permissions)");
    assert_eq!(active_state().description(), "Active (can create users and pub/sub)");
    assert_eq!(suspended_state().description(), "Suspended (temporarily disabled)");
    assert_eq!(reactivated_state().description(), "Reactivated (permissions restored)");
    assert_eq!(deleted_state().description(), "Deleted (TERMINAL - permanently removed)");
}

#[test]
fn test_permissions_getter() {
    let permissions = test_permissions();

    assert_eq!(created_state().permissions(), None);

    let active = NatsAccountState::Active {
        permissions: permissions.clone(),
        activated_at: Utc::now(),
        users: vec![],
    };
    assert_eq!(active.permissions(), Some(&permissions));

    assert_eq!(suspended_state().permissions(), None);

    let reactivated = NatsAccountState::Reactivated {
        permissions: permissions.clone(),
        reactivated_at: Utc::now(),
        reactivated_by: test_person_id(),
    };
    assert_eq!(reactivated.permissions(), Some(&permissions));

    assert_eq!(deleted_state().permissions(), None);
}

#[test]
fn test_users_getter() {
    assert_eq!(created_state().users(), None);
    assert!(active_state().users().is_some());
    assert_eq!(suspended_state().users(), None);
    assert_eq!(reactivated_state().users(), None);
    assert_eq!(deleted_state().users(), None);
}

// Permissions Tests
#[test]
fn test_nats_permissions_structure() {
    let perms = NatsPermissions {
        publish: vec!["app.>".to_string(), "system.heartbeat".to_string()],
        subscribe: vec!["app.>".to_string(), "system.>".to_string()],
        allow_responses: true,
        max_connections: Some(50),
        max_payload: Some(512 * 1024),
    };

    assert_eq!(perms.publish.len(), 2);
    assert_eq!(perms.subscribe.len(), 2);
    assert!(perms.allow_responses);
    assert_eq!(perms.max_connections, Some(50));
    assert_eq!(perms.max_payload, Some(512 * 1024));
}

#[test]
fn test_nats_permissions_unlimited() {
    let perms = NatsPermissions {
        publish: vec!["*.>".to_string()],
        subscribe: vec!["*.>".to_string()],
        allow_responses: true,
        max_connections: None,
        max_payload: None,
    };

    assert!(perms.max_connections.is_none());
    assert!(perms.max_payload.is_none());
}

// Lifecycle Workflow Tests
#[test]
fn test_complete_lifecycle_created_to_deleted() {
    // Created → Active
    let created = created_state();
    let active = created.activate(test_permissions(), Utc::now()).unwrap();
    assert!(active.is_active());

    // Active → Add users
    let with_user1 = active.add_user(Uuid::now_v7()).unwrap();
    let with_user2 = with_user1.add_user(Uuid::now_v7()).unwrap();
    assert_eq!(with_user2.users().unwrap().len(), 2);

    // Active → Suspended
    let suspended = with_user2
        .suspend("Review".to_string(), Utc::now(), test_person_id())
        .unwrap();
    assert!(suspended.is_suspended());

    // Suspended → Reactivated
    let reactivated = suspended
        .reactivate(test_permissions(), Utc::now(), test_person_id())
        .unwrap();
    assert!(reactivated.is_active());

    // Reactivated → Active
    let active_again = reactivated.activate(test_permissions(), Utc::now()).unwrap();
    assert!(active_again.is_active());

    // Active → Deleted
    let deleted = active_again
        .delete("Cleanup".to_string(), Utc::now(), test_person_id())
        .unwrap();
    assert!(deleted.is_deleted());
    assert!(deleted.is_terminal());
}

#[test]
fn test_lifecycle_suspended_to_deleted() {
    // Active → Suspended → Deleted (without reactivation)
    let active = active_state();
    let suspended = active
        .suspend("Review".to_string(), Utc::now(), test_person_id())
        .unwrap();
    let deleted = suspended
        .delete("Terminated".to_string(), Utc::now(), test_person_id())
        .unwrap();

    assert!(deleted.is_terminal());
}

#[test]
fn test_lifecycle_reactivated_to_deleted() {
    // Reactivated → Deleted (without going back to Active)
    let reactivated = reactivated_state();
    let deleted = reactivated
        .delete("Cleanup".to_string(), Utc::now(), test_person_id())
        .unwrap();

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
        let deserialized: NatsAccountState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

#[test]
fn test_serde_roundtrip_permissions() {
    let perms = test_permissions();
    let json = serde_json::to_string(&perms).unwrap();
    let deserialized: NatsPermissions = serde_json::from_str(&json).unwrap();
    assert_eq!(perms, deserialized);
}

// Edge Case Tests
#[test]
fn test_active_account_with_many_users() {
    let active = active_state();
    let mut current = active;

    // Add 20 users
    for _ in 0..20 {
        current = current.add_user(Uuid::now_v7()).unwrap();
    }

    assert_eq!(current.users().unwrap().len(), 22); // 2 initial + 20 new
    assert!(current.is_active());
}

#[test]
fn test_permissions_with_wildcard_subjects() {
    let perms = NatsPermissions {
        publish: vec!["app.*.events".to_string(), "system.>".to_string()],
        subscribe: vec!["app.>".to_string()],
        allow_responses: true,
        max_connections: Some(100),
        max_payload: Some(1024 * 1024),
    };

    let active = NatsAccountState::Active {
        permissions: perms.clone(),
        activated_at: Utc::now(),
        users: vec![],
    };

    assert!(active.can_pubsub());
    assert_eq!(active.permissions(), Some(&perms));
}
