//! Comprehensive Person State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/person.rs
//!
//! Tests all:
//! - Valid state transitions
//! - Invalid state transition rejections
//! - State query methods
//! - State invariants
//! - Lifecycle workflows
//! - Edge cases and error conditions

use chrono::Utc;
use cim_keys::state_machines::person::{PersonState, StateError};
use uuid::Uuid;

// ============================================================================
// Test Helpers
// ============================================================================

fn test_admin_id() -> Uuid {
    Uuid::now_v7()
}

fn test_role_ids(count: usize) -> Vec<Uuid> {
    (0..count).map(|_| Uuid::now_v7()).collect()
}

fn created_state() -> PersonState {
    PersonState::Created {
        created_at: Utc::now(),
        created_by: test_admin_id(),
    }
}

fn active_state() -> PersonState {
    PersonState::Active {
        roles: test_role_ids(2),
        activated_at: Utc::now(),
        last_activity: None,
    }
}

fn suspended_state() -> PersonState {
    PersonState::Suspended {
        reason: "Policy violation".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_admin_id(),
        previous_roles: test_role_ids(2),
    }
}

fn deactivated_state() -> PersonState {
    PersonState::Deactivated {
        reason: "Employment ended".to_string(),
        deactivated_at: Utc::now(),
        deactivated_by: test_admin_id(),
    }
}

fn archived_state() -> PersonState {
    PersonState::Archived {
        archived_at: Utc::now(),
        archived_by: test_admin_id(),
        retention_policy_id: Some(Uuid::now_v7()),
    }
}

// ============================================================================
// State Query Tests
// ============================================================================

#[test]
fn test_is_active_for_all_states() {
    assert!(!created_state().is_active());
    assert!(active_state().is_active());
    assert!(!suspended_state().is_active());
    assert!(!deactivated_state().is_active());
    assert!(!archived_state().is_active());
}

#[test]
fn test_can_perform_actions_for_all_states() {
    assert!(!created_state().can_perform_actions());
    assert!(active_state().can_perform_actions());
    assert!(!suspended_state().can_perform_actions());
    assert!(!deactivated_state().can_perform_actions());
    assert!(!archived_state().can_perform_actions());
}

#[test]
fn test_can_assign_roles_for_all_states() {
    assert!(created_state().can_assign_roles());
    assert!(active_state().can_assign_roles());
    assert!(!suspended_state().can_assign_roles());
    assert!(!deactivated_state().can_assign_roles());
    assert!(!archived_state().can_assign_roles());
}

#[test]
fn test_can_generate_keys_for_all_states() {
    assert!(!created_state().can_generate_keys());
    assert!(active_state().can_generate_keys());
    assert!(!suspended_state().can_generate_keys());
    assert!(!deactivated_state().can_generate_keys());
    assert!(!archived_state().can_generate_keys());
}

#[test]
fn test_can_establish_relationships_for_all_states() {
    assert!(created_state().can_establish_relationships());
    assert!(active_state().can_establish_relationships());
    assert!(suspended_state().can_establish_relationships());
    assert!(!deactivated_state().can_establish_relationships());
    assert!(!archived_state().can_establish_relationships());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!created_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!suspended_state().is_terminal());
    assert!(!deactivated_state().is_terminal());
    assert!(archived_state().is_terminal());
}

#[test]
fn test_is_suspended_for_all_states() {
    assert!(!created_state().is_suspended());
    assert!(!active_state().is_suspended());
    assert!(suspended_state().is_suspended());
    assert!(!deactivated_state().is_suspended());
    assert!(!archived_state().is_suspended());
}

#[test]
fn test_is_deactivated_for_all_states() {
    assert!(!created_state().is_deactivated());
    assert!(!active_state().is_deactivated());
    assert!(!suspended_state().is_deactivated());
    assert!(deactivated_state().is_deactivated());
    assert!(!archived_state().is_deactivated());
}

#[test]
fn test_can_be_modified_for_all_states() {
    assert!(created_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(suspended_state().can_be_modified());
    assert!(deactivated_state().can_be_modified());
    assert!(!archived_state().can_be_modified());
}

// ============================================================================
// Transition Validation Tests
// ============================================================================

#[test]
fn test_can_transition_created_to_active() {
    let created = created_state();
    let active = active_state();
    assert!(created.can_transition_to(&active));
}

#[test]
fn test_can_transition_active_to_suspended() {
    let active = active_state();
    let suspended = suspended_state();
    assert!(active.can_transition_to(&suspended));
}

#[test]
fn test_can_transition_suspended_to_active() {
    let suspended = suspended_state();
    let active = active_state();
    assert!(suspended.can_transition_to(&active));
}

#[test]
fn test_can_transition_suspended_to_deactivated() {
    let suspended = suspended_state();
    let deactivated = deactivated_state();
    assert!(suspended.can_transition_to(&deactivated));
}

#[test]
fn test_can_transition_active_to_deactivated() {
    let active = active_state();
    let deactivated = deactivated_state();
    assert!(active.can_transition_to(&deactivated));
}

#[test]
fn test_can_transition_deactivated_to_archived() {
    let deactivated = deactivated_state();
    let archived = archived_state();
    assert!(deactivated.can_transition_to(&archived));
}

#[test]
fn test_cannot_transition_created_to_suspended() {
    let created = created_state();
    let suspended = suspended_state();
    assert!(!created.can_transition_to(&suspended));
}

#[test]
fn test_cannot_transition_created_to_deactivated() {
    let created = created_state();
    let deactivated = deactivated_state();
    assert!(!created.can_transition_to(&deactivated));
}

#[test]
fn test_cannot_transition_created_to_archived() {
    let created = created_state();
    let archived = archived_state();
    assert!(!created.can_transition_to(&archived));
}

#[test]
fn test_cannot_transition_active_to_archived() {
    let active = active_state();
    let archived = archived_state();
    assert!(!active.can_transition_to(&archived));
}

#[test]
fn test_cannot_transition_suspended_to_archived() {
    let suspended = suspended_state();
    let archived = archived_state();
    assert!(!suspended.can_transition_to(&archived));
}

#[test]
fn test_cannot_transition_from_archived() {
    let archived = archived_state();
    assert!(!archived.can_transition_to(&created_state()));
    assert!(!archived.can_transition_to(&active_state()));
    assert!(!archived.can_transition_to(&suspended_state()));
    assert!(!archived.can_transition_to(&deactivated_state()));
    assert!(!archived.can_transition_to(&archived_state()));
}

// ============================================================================
// Activation Tests (Created → Active)
// ============================================================================

#[test]
fn test_activate_from_created_with_roles() {
    let created = created_state();
    let roles = test_role_ids(3);
    let activated_at = Utc::now();

    let result = created.activate(roles.clone(), activated_at);
    assert!(result.is_ok());

    let active = result.unwrap();
    assert!(active.is_active());
    assert_eq!(active.roles(), Some(roles.as_slice()));
}

#[test]
fn test_activate_from_created_without_roles_fails() {
    let created = created_state();
    let empty_roles = vec![];
    let activated_at = Utc::now();

    let result = created.activate(empty_roles, activated_at);
    assert!(result.is_err());

    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("without roles"));
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_activate_from_suspended_with_new_roles() {
    let suspended = suspended_state();
    let new_roles = test_role_ids(4);
    let activated_at = Utc::now();

    let result = suspended.activate(new_roles.clone(), activated_at);
    assert!(result.is_ok());

    let active = result.unwrap();
    assert!(active.is_active());
    assert_eq!(active.roles(), Some(new_roles.as_slice()));
}

#[test]
fn test_activate_from_suspended_restores_previous_roles() {
    let previous_roles = test_role_ids(2);
    let suspended = PersonState::Suspended {
        reason: "Test".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_admin_id(),
        previous_roles: previous_roles.clone(),
    };

    let empty_roles = vec![];
    let activated_at = Utc::now();

    let result = suspended.activate(empty_roles, activated_at);
    assert!(result.is_ok());

    let active = result.unwrap();
    assert!(active.is_active());
    assert_eq!(active.roles(), Some(previous_roles.as_slice()));
}

#[test]
fn test_activate_from_active_fails() {
    let active = active_state();
    let roles = test_role_ids(2);
    let activated_at = Utc::now();

    let result = active.activate(roles, activated_at);
    assert!(result.is_err());

    match result.unwrap_err() {
        StateError::InvalidTransition { current, event, .. } => {
            assert!(current.contains("Active"));
            assert_eq!(event, "activate");
        }
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[test]
fn test_activate_from_deactivated_fails() {
    let deactivated = deactivated_state();
    let roles = test_role_ids(2);
    let activated_at = Utc::now();

    let result = deactivated.activate(roles, activated_at);
    assert!(result.is_err());

    match result.unwrap_err() {
        StateError::InvalidTransition { .. } => {}
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[test]
fn test_activate_from_archived_fails() {
    let archived = archived_state();
    let roles = test_role_ids(2);
    let activated_at = Utc::now();

    let result = archived.activate(roles, activated_at);
    assert!(result.is_err());
}

// ============================================================================
// Suspension Tests (Active → Suspended)
// ============================================================================

#[test]
fn test_suspend_from_active() {
    let roles = test_role_ids(3);
    let active = PersonState::Active {
        roles: roles.clone(),
        activated_at: Utc::now(),
        last_activity: None,
    };

    let reason = "Security investigation".to_string();
    let suspended_at = Utc::now();
    let suspended_by = test_admin_id();

    let result = active.suspend(reason.clone(), suspended_at, suspended_by);
    assert!(result.is_ok());

    let suspended = result.unwrap();
    assert!(suspended.is_suspended());

    match suspended {
        PersonState::Suspended { previous_roles, reason: saved_reason, .. } => {
            assert_eq!(previous_roles, roles);
            assert_eq!(saved_reason, reason);
        }
        _ => panic!("Expected Suspended state"),
    }
}

#[test]
fn test_suspend_from_created_fails() {
    let created = created_state();
    let result = created.suspend("Test".to_string(), Utc::now(), test_admin_id());
    assert!(result.is_err());

    match result.unwrap_err() {
        StateError::InvalidTransition { .. } => {}
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[test]
fn test_suspend_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.suspend("Test".to_string(), Utc::now(), test_admin_id());
    assert!(result.is_err());
}

#[test]
fn test_suspend_from_deactivated_fails() {
    let deactivated = deactivated_state();
    let result = deactivated.suspend("Test".to_string(), Utc::now(), test_admin_id());
    assert!(result.is_err());
}

#[test]
fn test_suspend_from_archived_fails() {
    let archived = archived_state();
    let result = archived.suspend("Test".to_string(), Utc::now(), test_admin_id());
    assert!(result.is_err());
}

// ============================================================================
// Deactivation Tests (Active/Suspended → Deactivated)
// ============================================================================

#[test]
fn test_deactivate_from_active() {
    let active = active_state();
    let reason = "Employment terminated".to_string();
    let deactivated_at = Utc::now();
    let deactivated_by = test_admin_id();

    let result = active.deactivate(reason.clone(), deactivated_at, deactivated_by);
    assert!(result.is_ok());

    let deactivated = result.unwrap();
    assert!(deactivated.is_deactivated());

    match deactivated {
        PersonState::Deactivated { reason: saved_reason, .. } => {
            assert_eq!(saved_reason, reason);
        }
        _ => panic!("Expected Deactivated state"),
    }
}

#[test]
fn test_deactivate_from_suspended() {
    let suspended = suspended_state();
    let reason = "Suspension became permanent".to_string();
    let deactivated_at = Utc::now();
    let deactivated_by = test_admin_id();

    let result = suspended.deactivate(reason.clone(), deactivated_at, deactivated_by);
    assert!(result.is_ok());

    let deactivated = result.unwrap();
    assert!(deactivated.is_deactivated());
}

#[test]
fn test_deactivate_from_created_fails() {
    let created = created_state();
    let result = created.deactivate("Test".to_string(), Utc::now(), test_admin_id());
    assert!(result.is_err());

    match result.unwrap_err() {
        StateError::InvalidTransition { .. } => {}
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[test]
fn test_deactivate_from_deactivated_fails() {
    let deactivated = deactivated_state();
    let result = deactivated.deactivate("Test".to_string(), Utc::now(), test_admin_id());
    assert!(result.is_err());
}

#[test]
fn test_deactivate_from_archived_fails() {
    let archived = archived_state();
    let result = archived.deactivate("Test".to_string(), Utc::now(), test_admin_id());
    assert!(result.is_err());
}

// ============================================================================
// Archival Tests (Deactivated → Archived)
// ============================================================================

#[test]
fn test_archive_from_deactivated() {
    let deactivated = deactivated_state();
    let archived_at = Utc::now();
    let archived_by = test_admin_id();
    let retention_policy_id = Some(Uuid::now_v7());

    let result = deactivated.archive(archived_at, archived_by, retention_policy_id);
    assert!(result.is_ok());

    let archived = result.unwrap();
    assert!(archived.is_terminal());
    assert!(!archived.can_be_modified());

    match archived {
        PersonState::Archived { retention_policy_id: saved_policy, .. } => {
            assert_eq!(saved_policy, retention_policy_id);
        }
        _ => panic!("Expected Archived state"),
    }
}

#[test]
fn test_archive_from_deactivated_without_retention_policy() {
    let deactivated = deactivated_state();
    let archived_at = Utc::now();
    let archived_by = test_admin_id();

    let result = deactivated.archive(archived_at, archived_by, None);
    assert!(result.is_ok());

    let archived = result.unwrap();
    assert!(archived.is_terminal());
}

#[test]
fn test_archive_from_created_fails() {
    let created = created_state();
    let result = created.archive(Utc::now(), test_admin_id(), None);
    assert!(result.is_err());

    match result.unwrap_err() {
        StateError::InvalidTransition { .. } => {}
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[test]
fn test_archive_from_active_fails() {
    let active = active_state();
    let result = active.archive(Utc::now(), test_admin_id(), None);
    assert!(result.is_err());
}

#[test]
fn test_archive_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.archive(Utc::now(), test_admin_id(), None);
    assert!(result.is_err());
}

#[test]
fn test_archive_from_archived_fails() {
    let archived = archived_state();
    let result = archived.archive(Utc::now(), test_admin_id(), None);
    assert!(result.is_err());
}

// ============================================================================
// Activity Recording Tests
// ============================================================================

#[test]
fn test_record_activity_for_active_person() {
    let active = active_state();
    let activity_at = Utc::now();

    let result = active.record_activity(activity_at);
    assert!(result.is_ok());

    let updated = result.unwrap();
    match updated {
        PersonState::Active { last_activity, .. } => {
            assert_eq!(last_activity, Some(activity_at));
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_record_activity_updates_existing_activity() {
    let first_activity = Utc::now();
    let active = PersonState::Active {
        roles: test_role_ids(2),
        activated_at: Utc::now(),
        last_activity: Some(first_activity),
    };

    let second_activity = Utc::now();
    let result = active.record_activity(second_activity);
    assert!(result.is_ok());

    let updated = result.unwrap();
    match updated {
        PersonState::Active { last_activity, .. } => {
            assert_eq!(last_activity, Some(second_activity));
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_record_activity_from_created_fails() {
    let created = created_state();
    let result = created.record_activity(Utc::now());
    assert!(result.is_err());
}

#[test]
fn test_record_activity_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.record_activity(Utc::now());
    assert!(result.is_err());
}

#[test]
fn test_record_activity_from_deactivated_fails() {
    let deactivated = deactivated_state();
    let result = deactivated.record_activity(Utc::now());
    assert!(result.is_err());
}

#[test]
fn test_record_activity_from_archived_fails() {
    let archived = archived_state();
    let result = archived.record_activity(Utc::now());
    assert!(result.is_err());
}

// ============================================================================
// Role Update Tests
// ============================================================================

#[test]
fn test_update_roles_for_active_person() {
    let active = active_state();
    let new_roles = test_role_ids(5);

    let result = active.update_roles(new_roles.clone());
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.roles(), Some(new_roles.as_slice()));
}

#[test]
fn test_update_roles_preserves_activity() {
    let last_activity = Utc::now();
    let active = PersonState::Active {
        roles: test_role_ids(2),
        activated_at: Utc::now(),
        last_activity: Some(last_activity),
    };

    let new_roles = test_role_ids(3);
    let result = active.update_roles(new_roles);
    assert!(result.is_ok());

    let updated = result.unwrap();
    match updated {
        PersonState::Active { last_activity: saved_activity, .. } => {
            assert_eq!(saved_activity, Some(last_activity));
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_update_roles_with_empty_list_fails() {
    let active = active_state();
    let empty_roles = vec![];

    let result = active.update_roles(empty_roles);
    assert!(result.is_err());

    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("Cannot remove all roles"));
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_update_roles_from_created_fails() {
    let created = created_state();
    let result = created.update_roles(test_role_ids(2));
    assert!(result.is_err());
}

#[test]
fn test_update_roles_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.update_roles(test_role_ids(2));
    assert!(result.is_err());
}

#[test]
fn test_update_roles_from_deactivated_fails() {
    let deactivated = deactivated_state();
    let result = deactivated.update_roles(test_role_ids(2));
    assert!(result.is_err());
}

#[test]
fn test_update_roles_from_archived_fails() {
    let archived = archived_state();
    let result = archived.update_roles(test_role_ids(2));
    assert!(result.is_err());
}

// ============================================================================
// Metadata Tests
// ============================================================================

#[test]
fn test_description_for_all_states() {
    assert_eq!(
        created_state().description(),
        "Created (awaiting role assignment)"
    );
    assert_eq!(
        active_state().description(),
        "Active (has roles and permissions)"
    );
    assert_eq!(
        suspended_state().description(),
        "Suspended (temporarily revoked access)"
    );
    assert_eq!(
        deactivated_state().description(),
        "Deactivated (permanently revoked access)"
    );
    assert_eq!(
        archived_state().description(),
        "Archived (TERMINAL - long-term retention)"
    );
}

#[test]
fn test_roles_getter() {
    assert_eq!(created_state().roles(), None);
    assert!(active_state().roles().is_some());
    assert_eq!(suspended_state().roles(), None);
    assert_eq!(deactivated_state().roles(), None);
    assert_eq!(archived_state().roles(), None);
}

#[test]
fn test_roles_returns_correct_roles() {
    let roles = test_role_ids(4);
    let active = PersonState::Active {
        roles: roles.clone(),
        activated_at: Utc::now(),
        last_activity: None,
    };

    assert_eq!(active.roles(), Some(roles.as_slice()));
}

// ============================================================================
// Lifecycle Workflow Tests
// ============================================================================

#[test]
fn test_complete_lifecycle_created_to_archived() {
    // Created → Active
    let created = created_state();
    let roles = test_role_ids(3);
    let active = created.activate(roles, Utc::now()).unwrap();
    assert!(active.is_active());

    // Active → Suspended
    let suspended = active
        .suspend("Investigation".to_string(), Utc::now(), test_admin_id())
        .unwrap();
    assert!(suspended.is_suspended());

    // Suspended → Deactivated
    let deactivated = suspended
        .deactivate("Terminated".to_string(), Utc::now(), test_admin_id())
        .unwrap();
    assert!(deactivated.is_deactivated());

    // Deactivated → Archived
    let archived = deactivated
        .archive(Utc::now(), test_admin_id(), Some(Uuid::now_v7()))
        .unwrap();
    assert!(archived.is_terminal());
}

#[test]
fn test_lifecycle_created_to_active_to_deactivated_to_archived() {
    // Created → Active
    let created = created_state();
    let active = created.activate(test_role_ids(2), Utc::now()).unwrap();

    // Active → Deactivated (skip suspension)
    let deactivated = active
        .deactivate("Resigned".to_string(), Utc::now(), test_admin_id())
        .unwrap();

    // Deactivated → Archived
    let archived = deactivated
        .archive(Utc::now(), test_admin_id(), None)
        .unwrap();
    assert!(archived.is_terminal());
}

#[test]
fn test_lifecycle_suspend_and_reactivate() {
    // Created → Active
    let created = created_state();
    let roles = test_role_ids(3);
    let active = created.activate(roles.clone(), Utc::now()).unwrap();

    // Active → Suspended
    let suspended = active
        .suspend("Security review".to_string(), Utc::now(), test_admin_id())
        .unwrap();

    // Suspended → Active (reactivation)
    let reactivated = suspended.activate(vec![], Utc::now()).unwrap();
    assert!(reactivated.is_active());

    // Roles should be restored
    assert_eq!(reactivated.roles(), Some(roles.as_slice()));
}

#[test]
fn test_lifecycle_multiple_role_updates() {
    // Created → Active
    let created = created_state();
    let initial_roles = test_role_ids(2);
    let active = created.activate(initial_roles, Utc::now()).unwrap();

    // Update roles multiple times
    let roles_v2 = test_role_ids(3);
    let active_v2 = active.update_roles(roles_v2.clone()).unwrap();
    assert_eq!(active_v2.roles(), Some(roles_v2.as_slice()));

    let roles_v3 = test_role_ids(4);
    let active_v3 = active_v2.update_roles(roles_v3.clone()).unwrap();
    assert_eq!(active_v3.roles(), Some(roles_v3.as_slice()));
}

#[test]
fn test_lifecycle_activity_tracking() {
    // Created → Active
    let created = created_state();
    let active = created.activate(test_role_ids(2), Utc::now()).unwrap();

    // Record activity
    let activity1 = Utc::now();
    let active_v2 = active.record_activity(activity1).unwrap();

    // Record more activity
    let activity2 = Utc::now();
    let active_v3 = active_v2.record_activity(activity2).unwrap();

    match active_v3 {
        PersonState::Active { last_activity, .. } => {
            assert_eq!(last_activity, Some(activity2));
        }
        _ => panic!("Expected Active state"),
    }
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_serde_roundtrip_created_state() {
    let created = created_state();
    let json = serde_json::to_string(&created).unwrap();
    let deserialized: PersonState = serde_json::from_str(&json).unwrap();
    assert_eq!(created, deserialized);
}

#[test]
fn test_serde_roundtrip_active_state() {
    let active = active_state();
    let json = serde_json::to_string(&active).unwrap();
    let deserialized: PersonState = serde_json::from_str(&json).unwrap();
    assert_eq!(active, deserialized);
}

#[test]
fn test_serde_roundtrip_suspended_state() {
    let suspended = suspended_state();
    let json = serde_json::to_string(&suspended).unwrap();
    let deserialized: PersonState = serde_json::from_str(&json).unwrap();
    assert_eq!(suspended, deserialized);
}

#[test]
fn test_serde_roundtrip_deactivated_state() {
    let deactivated = deactivated_state();
    let json = serde_json::to_string(&deactivated).unwrap();
    let deserialized: PersonState = serde_json::from_str(&json).unwrap();
    assert_eq!(deactivated, deserialized);
}

#[test]
fn test_serde_roundtrip_archived_state() {
    let archived = archived_state();
    let json = serde_json::to_string(&archived).unwrap();
    let deserialized: PersonState = serde_json::from_str(&json).unwrap();
    assert_eq!(archived, deserialized);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_activate_with_single_role() {
    let created = created_state();
    let single_role = vec![Uuid::now_v7()];
    let result = created.activate(single_role.clone(), Utc::now());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().roles(), Some(single_role.as_slice()));
}

#[test]
fn test_activate_with_many_roles() {
    let created = created_state();
    let many_roles = test_role_ids(100);
    let result = created.activate(many_roles.clone(), Utc::now());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().roles(), Some(many_roles.as_slice()));
}

#[test]
fn test_suspend_with_empty_reason() {
    let active = active_state();
    let result = active.suspend("".to_string(), Utc::now(), test_admin_id());
    assert!(result.is_ok()); // Empty reason is allowed
}

#[test]
fn test_deactivate_with_long_reason() {
    let active = active_state();
    let long_reason = "x".repeat(10000);
    let result = active.deactivate(long_reason.clone(), Utc::now(), test_admin_id());
    assert!(result.is_ok());

    match result.unwrap() {
        PersonState::Deactivated { reason, .. } => {
            assert_eq!(reason, long_reason);
        }
        _ => panic!("Expected Deactivated state"),
    }
}

#[test]
fn test_archive_with_retention_policy() {
    let deactivated = deactivated_state();
    let policy_id = Uuid::now_v7();
    let result = deactivated.archive(Utc::now(), test_admin_id(), Some(policy_id));
    assert!(result.is_ok());

    match result.unwrap() {
        PersonState::Archived { retention_policy_id, .. } => {
            assert_eq!(retention_policy_id, Some(policy_id));
        }
        _ => panic!("Expected Archived state"),
    }
}
