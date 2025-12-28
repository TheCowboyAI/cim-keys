//! Comprehensive Organization State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/organization.rs

use chrono::Utc;
use cim_keys::state_machines::organization::{OrganizationState, StateError};
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_unit_ids(count: usize) -> Vec<Uuid> {
    (0..count).map(|_| Uuid::now_v7()).collect()
}
fn test_member_ids(count: usize) -> Vec<Uuid> {
    (0..count).map(|_| Uuid::now_v7()).collect()
}

fn draft_state() -> OrganizationState {
    OrganizationState::Draft {
        created_at: Utc::now(),
        created_by: test_person_id(),
    }
}

fn active_state() -> OrganizationState {
    OrganizationState::Active {
        activated_at: Utc::now(),
        units: test_unit_ids(2),
        members: test_member_ids(3),
    }
}

fn suspended_state() -> OrganizationState {
    OrganizationState::Suspended {
        reason: "Policy review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
    }
}

fn dissolved_state() -> OrganizationState {
    OrganizationState::Dissolved {
        dissolved_at: Utc::now(),
        dissolved_by: test_person_id(),
        reason: "Merger".to_string(),
        successor_org_id: Some(Uuid::now_v7()),
    }
}

// State Query Tests
#[test]
fn test_is_active_for_all_states() {
    assert!(!draft_state().is_active());
    assert!(active_state().is_active());
    assert!(!suspended_state().is_active());
    assert!(!dissolved_state().is_active());
}

#[test]
fn test_can_add_units_for_all_states() {
    assert!(!draft_state().can_add_units());
    assert!(active_state().can_add_units());
    assert!(!suspended_state().can_add_units());
    assert!(!dissolved_state().can_add_units());
}

#[test]
fn test_can_add_members_for_all_states() {
    assert!(!draft_state().can_add_members());
    assert!(active_state().can_add_members());
    assert!(!suspended_state().can_add_members());
    assert!(!dissolved_state().can_add_members());
}

#[test]
fn test_can_generate_keys_for_all_states() {
    assert!(!draft_state().can_generate_keys());
    assert!(active_state().can_generate_keys());
    assert!(!suspended_state().can_generate_keys());
    assert!(!dissolved_state().can_generate_keys());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!draft_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!suspended_state().is_terminal());
    assert!(dissolved_state().is_terminal());
}

#[test]
fn test_is_suspended_for_all_states() {
    assert!(!draft_state().is_suspended());
    assert!(!active_state().is_suspended());
    assert!(suspended_state().is_suspended());
    assert!(!dissolved_state().is_suspended());
}

#[test]
fn test_is_dissolved_for_all_states() {
    assert!(!draft_state().is_dissolved());
    assert!(!active_state().is_dissolved());
    assert!(!suspended_state().is_dissolved());
    assert!(dissolved_state().is_dissolved());
}

#[test]
fn test_can_be_modified_for_all_states() {
    assert!(draft_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(suspended_state().can_be_modified());
    assert!(!dissolved_state().can_be_modified());
}

// Transition Validation Tests
#[test]
fn test_can_transition_draft_to_active() {
    assert!(draft_state().can_transition_to(&active_state()));
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
fn test_can_transition_active_to_dissolved() {
    assert!(active_state().can_transition_to(&dissolved_state()));
}

#[test]
fn test_can_transition_suspended_to_dissolved() {
    assert!(suspended_state().can_transition_to(&dissolved_state()));
}

#[test]
fn test_cannot_transition_from_dissolved() {
    let dissolved = dissolved_state();
    assert!(!dissolved.can_transition_to(&draft_state()));
    assert!(!dissolved.can_transition_to(&active_state()));
    assert!(!dissolved.can_transition_to(&suspended_state()));
}

// Activation Tests
#[test]
fn test_activate_from_draft_with_units() {
    let draft = draft_state();
    let units = test_unit_ids(2);
    let result = draft.activate(Utc::now(), units.clone(), vec![]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().units(), Some(units.as_slice()));
}

#[test]
fn test_activate_from_draft_with_members() {
    let draft = draft_state();
    let members = test_member_ids(3);
    let result = draft.activate(Utc::now(), vec![], members.clone());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().members(), Some(members.as_slice()));
}

#[test]
fn test_activate_from_draft_with_units_and_members() {
    let draft = draft_state();
    let units = test_unit_ids(2);
    let members = test_member_ids(3);
    let result = draft.activate(Utc::now(), units.clone(), members.clone());
    assert!(result.is_ok());
    let org = result.unwrap();
    assert_eq!(org.units(), Some(units.as_slice()));
    assert_eq!(org.members(), Some(members.as_slice()));
}

#[test]
fn test_activate_from_draft_without_units_or_members_fails() {
    let draft = draft_state();
    let result = draft.activate(Utc::now(), vec![], vec![]);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("without units or members"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_activate_from_suspended() {
    let suspended = suspended_state();
    let units = test_unit_ids(1);
    let members = test_member_ids(2);
    let result = suspended.activate(Utc::now(), units, members);
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_activate_from_active_fails() {
    let active = active_state();
    let result = active.activate(Utc::now(), vec![], test_member_ids(1));
    assert!(result.is_err());
}

#[test]
fn test_activate_from_dissolved_fails() {
    let dissolved = dissolved_state();
    let result = dissolved.activate(Utc::now(), test_unit_ids(1), vec![]);
    assert!(result.is_err());
}

// Suspension Tests
#[test]
fn test_suspend_from_active() {
    let active = active_state();
    let reason = "Audit required".to_string();
    let result = active.suspend(reason.clone(), Utc::now(), test_person_id());
    assert!(result.is_ok());
    assert!(result.unwrap().is_suspended());
}

#[test]
fn test_suspend_from_draft_fails() {
    let draft = draft_state();
    let result = draft.suspend("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
}

#[test]
fn test_suspend_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.suspend("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
}

#[test]
fn test_suspend_from_dissolved_fails() {
    let dissolved = dissolved_state();
    let result = dissolved.suspend("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
}

// Dissolution Tests
#[test]
fn test_dissolve_from_active() {
    let active = active_state();
    let reason = "Business closure".to_string();
    let result = active.dissolve(reason, Utc::now(), test_person_id(), None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_dissolved());
}

#[test]
fn test_dissolve_from_active_with_successor() {
    let active = active_state();
    let successor_id = Uuid::now_v7();
    let result = active.dissolve("Merger".to_string(), Utc::now(), test_person_id(), Some(successor_id));
    assert!(result.is_ok());
}

#[test]
fn test_dissolve_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.dissolve("Terminated".to_string(), Utc::now(), test_person_id(), None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_dissolved());
}

#[test]
fn test_dissolve_from_draft_fails() {
    let draft = draft_state();
    let result = draft.dissolve("Test".to_string(), Utc::now(), test_person_id(), None);
    assert!(result.is_err());
}

#[test]
fn test_dissolve_from_dissolved_fails() {
    let dissolved = dissolved_state();
    let result = dissolved.dissolve("Test".to_string(), Utc::now(), test_person_id(), None);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::TerminalState(_) => {},
        _ => panic!("Expected TerminalState error"),
    }
}

// Unit Management Tests
#[test]
fn test_add_unit_to_active_org() {
    let active = active_state();
    let new_unit = Uuid::now_v7();
    let result = active.add_unit(new_unit);
    assert!(result.is_ok());
    assert!(result.unwrap().units().unwrap().contains(&new_unit));
}

#[test]
fn test_add_duplicate_unit_is_idempotent() {
    let units = test_unit_ids(2);
    let active = OrganizationState::Active {
        activated_at: Utc::now(),
        units: units.clone(),
        members: vec![],
    };
    let duplicate_unit = units[0];
    let result = active.add_unit(duplicate_unit);
    assert!(result.is_ok());
    let org_state = result.unwrap();
    let org_units = org_state.units().unwrap();
    assert_eq!(org_units.len(), 2); // Still 2, not 3
}

#[test]
fn test_add_unit_from_draft_fails() {
    let draft = draft_state();
    let result = draft.add_unit(Uuid::now_v7());
    assert!(result.is_err());
}

#[test]
fn test_add_unit_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.add_unit(Uuid::now_v7());
    assert!(result.is_err());
}

#[test]
fn test_remove_unit_from_active_org() {
    let units = test_unit_ids(3);
    let members = test_member_ids(2);
    let active = OrganizationState::Active {
        activated_at: Utc::now(),
        units: units.clone(),
        members,
    };
    let unit_to_remove = units[1];
    let result = active.remove_unit(unit_to_remove);
    assert!(result.is_ok());
    assert!(!result.unwrap().units().unwrap().contains(&unit_to_remove));
}

#[test]
fn test_remove_last_unit_with_members_succeeds() {
    let units = test_unit_ids(1);
    let members = test_member_ids(2);
    let active = OrganizationState::Active {
        activated_at: Utc::now(),
        units: units.clone(),
        members,
    };
    let result = active.remove_unit(units[0]);
    assert!(result.is_ok());
    assert!(result.unwrap().units().unwrap().is_empty());
}

#[test]
fn test_remove_last_unit_without_members_fails() {
    let units = test_unit_ids(1);
    let active = OrganizationState::Active {
        activated_at: Utc::now(),
        units: units.clone(),
        members: vec![],
    };
    let result = active.remove_unit(units[0]);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("must have units or members"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

// Member Management Tests
#[test]
fn test_add_member_to_active_org() {
    let active = active_state();
    let new_member = Uuid::now_v7();
    let result = active.add_member(new_member);
    assert!(result.is_ok());
    assert!(result.unwrap().members().unwrap().contains(&new_member));
}

#[test]
fn test_add_duplicate_member_is_idempotent() {
    let members = test_member_ids(2);
    let active = OrganizationState::Active {
        activated_at: Utc::now(),
        units: vec![],
        members: members.clone(),
    };
    let duplicate_member = members[0];
    let result = active.add_member(duplicate_member);
    assert!(result.is_ok());
    let org_state = result.unwrap();
    let org_members = org_state.members().unwrap();
    assert_eq!(org_members.len(), 2); // Still 2, not 3
}

#[test]
fn test_add_member_from_draft_fails() {
    let draft = draft_state();
    let result = draft.add_member(Uuid::now_v7());
    assert!(result.is_err());
}

#[test]
fn test_add_member_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.add_member(Uuid::now_v7());
    assert!(result.is_err());
}

// Metadata Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(draft_state().description(), "Draft (not yet operational)");
    assert_eq!(active_state().description(), "Active (operational with units/members)");
    assert_eq!(suspended_state().description(), "Suspended (temporarily halted)");
    assert_eq!(dissolved_state().description(), "Dissolved (TERMINAL - permanently closed)");
}

#[test]
fn test_units_getter() {
    assert_eq!(draft_state().units(), None);
    assert!(active_state().units().is_some());
    assert_eq!(suspended_state().units(), None);
    assert_eq!(dissolved_state().units(), None);
}

#[test]
fn test_members_getter() {
    assert_eq!(draft_state().members(), None);
    assert!(active_state().members().is_some());
    assert_eq!(suspended_state().members(), None);
    assert_eq!(dissolved_state().members(), None);
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        draft_state(),
        active_state(),
        suspended_state(),
        dissolved_state(),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: OrganizationState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// Lifecycle Tests
#[test]
fn test_complete_lifecycle() {
    // Draft → Active
    let draft = draft_state();
    let active = draft.activate(Utc::now(), test_unit_ids(1), test_member_ids(1)).unwrap();
    assert!(active.is_active());

    // Active → Suspended
    let suspended = active.suspend("Review".to_string(), Utc::now(), test_person_id()).unwrap();
    assert!(suspended.is_suspended());

    // Suspended → Active
    let reactivated = suspended.activate(Utc::now(), test_unit_ids(1), test_member_ids(1)).unwrap();
    assert!(reactivated.is_active());

    // Active → Dissolved
    let dissolved = reactivated.dissolve("Closure".to_string(), Utc::now(), test_person_id(), None).unwrap();
    assert!(dissolved.is_dissolved());
    assert!(dissolved.is_terminal());
}
