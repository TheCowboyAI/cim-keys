//! Comprehensive Relationship State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/relationship.rs

use chrono::Utc;
use cim_keys::state_machines::relationship::{
    RelationshipChange, RelationshipMetadata, RelationshipState, RelationshipStrength, StateError,
};
use std::collections::HashMap;
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_retention_policy_id() -> Uuid { Uuid::now_v7() }

fn test_metadata() -> RelationshipMetadata {
    let mut props = HashMap::new();
    props.insert("key1".to_string(), "value1".to_string());

    RelationshipMetadata {
        strength: RelationshipStrength::Medium,
        bidirectional: true,
        properties: props,
    }
}

fn proposed_state() -> RelationshipState {
    RelationshipState::Proposed {
        proposed_at: Utc::now(),
        proposed_by: test_person_id(),
        pending_approval_from: Some(test_person_id()),
    }
}

fn active_state() -> RelationshipState {
    RelationshipState::Active {
        valid_from: Utc::now(),
        valid_until: None,
        relationship_type: "delegation".to_string(),
        metadata: test_metadata(),
    }
}

fn modified_state() -> RelationshipState {
    let changes = vec![RelationshipChange::StrengthChanged {
        old_strength: RelationshipStrength::Medium,
        new_strength: RelationshipStrength::Strong,
    }];

    RelationshipState::Modified {
        modified_at: Utc::now(),
        modified_by: test_person_id(),
        previous_version: Box::new(active_state()),
        changes,
    }
}

fn suspended_state() -> RelationshipState {
    RelationshipState::Suspended {
        reason: "Review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
        previous_state: Box::new(active_state()),
    }
}

fn terminated_state() -> RelationshipState {
    RelationshipState::Terminated {
        reason: "Ended".to_string(),
        terminated_at: Utc::now(),
        terminated_by: test_person_id(),
    }
}

fn archived_state() -> RelationshipState {
    RelationshipState::Archived {
        archived_at: Utc::now(),
        archived_by: test_person_id(),
        retention_policy_id: Some(test_retention_policy_id()),
    }
}

// State Query Tests
#[test]
fn test_is_active_for_all_states() {
    assert!(!proposed_state().is_active());
    assert!(active_state().is_active());
    assert!(!modified_state().is_active());
    assert!(!suspended_state().is_active());
    assert!(!terminated_state().is_active());
    assert!(!archived_state().is_active());
}

#[test]
fn test_can_use_for_authorization() {
    assert!(!proposed_state().can_use_for_authorization());
    assert!(active_state().can_use_for_authorization());
    assert!(!modified_state().can_use_for_authorization());
    assert!(!suspended_state().can_use_for_authorization());
    assert!(!terminated_state().can_use_for_authorization());
    assert!(!archived_state().can_use_for_authorization());
}

#[test]
fn test_can_be_modified_for_all_states() {
    assert!(proposed_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(modified_state().can_be_modified());
    assert!(suspended_state().can_be_modified());
    assert!(terminated_state().can_be_modified());
    assert!(!archived_state().can_be_modified());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!proposed_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!modified_state().is_terminal());
    assert!(!suspended_state().is_terminal());
    assert!(!terminated_state().is_terminal());
    assert!(archived_state().is_terminal());
}

#[test]
fn test_is_suspended_for_all_states() {
    assert!(!proposed_state().is_suspended());
    assert!(!active_state().is_suspended());
    assert!(!modified_state().is_suspended());
    assert!(suspended_state().is_suspended());
    assert!(!terminated_state().is_suspended());
    assert!(!archived_state().is_suspended());
}

#[test]
fn test_is_terminated_for_all_states() {
    assert!(!proposed_state().is_terminated());
    assert!(!active_state().is_terminated());
    assert!(!modified_state().is_terminated());
    assert!(!suspended_state().is_terminated());
    assert!(terminated_state().is_terminated());
    assert!(!archived_state().is_terminated());
}

#[test]
fn test_is_proposed_for_all_states() {
    assert!(proposed_state().is_proposed());
    assert!(!active_state().is_proposed());
    assert!(!modified_state().is_proposed());
    assert!(!suspended_state().is_proposed());
    assert!(!terminated_state().is_proposed());
    assert!(!archived_state().is_proposed());
}

// Temporal Validity Tests
#[test]
fn test_is_valid_at_for_active_relationship() {
    let now = Utc::now();
    let active = RelationshipState::Active {
        valid_from: now - chrono::Duration::hours(1),
        valid_until: Some(now + chrono::Duration::hours(1)),
        relationship_type: "test".to_string(),
        metadata: test_metadata(),
    };

    assert!(active.is_valid_at(now));
    assert!(active.is_valid_at(now - chrono::Duration::minutes(30)));
    assert!(!active.is_valid_at(now - chrono::Duration::hours(2)));
    assert!(!active.is_valid_at(now + chrono::Duration::hours(2)));
}

#[test]
fn test_is_valid_at_for_indefinite_relationship() {
    let now = Utc::now();
    let active = RelationshipState::Active {
        valid_from: now - chrono::Duration::hours(1),
        valid_until: None, // Indefinite
        relationship_type: "test".to_string(),
        metadata: test_metadata(),
    };

    assert!(active.is_valid_at(now));
    assert!(active.is_valid_at(now + chrono::Duration::days(365))); // Valid far in future
}

// Transition Validation Tests
#[test]
fn test_valid_transitions() {
    assert!(proposed_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&modified_state()));
    assert!(modified_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&suspended_state()));
    assert!(suspended_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&terminated_state()));
    assert!(modified_state().can_transition_to(&terminated_state()));
    assert!(suspended_state().can_transition_to(&terminated_state()));
    assert!(terminated_state().can_transition_to(&archived_state()));
}

#[test]
fn test_cannot_transition_from_archived() {
    let archived = archived_state();
    assert!(!archived.can_transition_to(&proposed_state()));
    assert!(!archived.can_transition_to(&active_state()));
    assert!(!archived.can_transition_to(&terminated_state()));
}

// Accept Tests
#[test]
fn test_accept_from_proposed() {
    let proposed = proposed_state();
    let now = Utc::now();
    let result = proposed.accept(
        now,
        Some(now + chrono::Duration::hours(24)),
        "delegation".to_string(),
        test_metadata(),
    );
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_accept_with_invalid_temporal_bounds_fails() {
    let proposed = proposed_state();
    let now = Utc::now();
    let result = proposed.accept(
        now,
        Some(now - chrono::Duration::hours(1)), // Invalid: until < from
        "delegation".to_string(),
        test_metadata(),
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => assert!(msg.contains("after")),
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_accept_from_active_fails() {
    let active = active_state();
    let result = active.accept(
        Utc::now(),
        None,
        "test".to_string(),
        test_metadata(),
    );
    assert!(result.is_err());
}

// Modify Tests
#[test]
fn test_modify_from_active() {
    let active = active_state();
    let changes = vec![RelationshipChange::StrengthChanged {
        old_strength: RelationshipStrength::Medium,
        new_strength: RelationshipStrength::Strong,
    }];
    let result = active.modify(Utc::now(), test_person_id(), changes);
    assert!(result.is_ok());
}

#[test]
fn test_modify_with_no_changes_fails() {
    let active = active_state();
    let result = active.modify(Utc::now(), test_person_id(), vec![]);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => assert!(msg.contains("without changes")),
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_modify_from_proposed_fails() {
    let proposed = proposed_state();
    let changes = vec![RelationshipChange::PropertyAdded {
        key: "test".to_string(),
        value: "value".to_string(),
    }];
    let result = proposed.modify(Utc::now(), test_person_id(), changes);
    assert!(result.is_err());
}

// Apply Modifications Tests
#[test]
fn test_apply_modifications_from_modified() {
    let modified = modified_state();
    let now = Utc::now();
    let params = (
        now,
        Some(now + chrono::Duration::days(7)),
        "delegation".to_string(),
        test_metadata(),
    );
    let result = modified.apply_modifications(params);
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_apply_modifications_from_active_fails() {
    let active = active_state();
    let now = Utc::now();
    let params = (now, None, "test".to_string(), test_metadata());
    let result = active.apply_modifications(params);
    assert!(result.is_err());
}

// Suspend/Reactivate Tests
#[test]
fn test_suspend_from_active() {
    let active = active_state();
    let result = active.suspend("Review".to_string(), Utc::now(), test_person_id());
    assert!(result.is_ok());
    assert!(result.unwrap().is_suspended());
}

#[test]
fn test_reactivate_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.reactivate();
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_reactivate_from_active_fails() {
    let active = active_state();
    let result = active.reactivate();
    assert!(result.is_err());
}

// Terminate Tests
#[test]
fn test_terminate_from_active() {
    let active = active_state();
    let result = active.terminate("Ended".to_string(), Utc::now(), test_person_id());
    assert!(result.is_ok());
    assert!(result.unwrap().is_terminated());
}

#[test]
fn test_terminate_from_modified() {
    let modified = modified_state();
    let result = modified.terminate("Cancelled".to_string(), Utc::now(), test_person_id());
    assert!(result.is_ok());
}

#[test]
fn test_terminate_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.terminate("Ended".to_string(), Utc::now(), test_person_id());
    assert!(result.is_ok());
}

#[test]
fn test_terminate_from_terminated_fails() {
    let terminated = terminated_state();
    let result = terminated.terminate("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::TerminalState(_) => {},
        _ => panic!("Expected TerminalState error"),
    }
}

// Archive Tests
#[test]
fn test_archive_from_terminated() {
    let terminated = terminated_state();
    let result = terminated.archive(Utc::now(), test_person_id(), Some(test_retention_policy_id()));
    assert!(result.is_ok());
    assert!(result.unwrap().is_terminal());
}

#[test]
fn test_archive_from_active_fails() {
    let active = active_state();
    let result = active.archive(Utc::now(), test_person_id(), None);
    assert!(result.is_err());
}

// Metadata Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(proposed_state().description(), "Proposed (awaiting acceptance)");
    assert_eq!(active_state().description(), "Active (valid and usable)");
    assert_eq!(modified_state().description(), "Modified (changes pending application)");
    assert_eq!(suspended_state().description(), "Suspended (temporarily inactive)");
    assert_eq!(terminated_state().description(), "Terminated (permanently ended)");
    assert_eq!(archived_state().description(), "Archived (TERMINAL - long-term retention)");
}

#[test]
fn test_metadata_getter() {
    assert_eq!(proposed_state().metadata(), None);
    assert!(active_state().metadata().is_some());
    assert_eq!(modified_state().metadata(), None);
    assert_eq!(suspended_state().metadata(), None);
    assert_eq!(terminated_state().metadata(), None);
    assert_eq!(archived_state().metadata(), None);
}

// RelationshipChange Tests
#[test]
fn test_all_relationship_change_types() {
    let changes = vec![
        RelationshipChange::ValidityChanged {
            old_from: Utc::now(),
            old_until: None,
            new_from: Utc::now(),
            new_until: Some(Utc::now() + chrono::Duration::days(30)),
        },
        RelationshipChange::TypeChanged {
            old_type: "delegation".to_string(),
            new_type: "supervision".to_string(),
        },
        RelationshipChange::PropertyAdded {
            key: "prop1".to_string(),
            value: "val1".to_string(),
        },
        RelationshipChange::PropertyRemoved {
            key: "prop2".to_string(),
        },
        RelationshipChange::PropertyModified {
            key: "prop3".to_string(),
            old_value: "old".to_string(),
            new_value: "new".to_string(),
        },
        RelationshipChange::StrengthChanged {
            old_strength: RelationshipStrength::Weak,
            new_strength: RelationshipStrength::Strong,
        },
    ];

    let active = active_state();
    let result = active.modify(Utc::now(), test_person_id(), changes);
    assert!(result.is_ok());
}

// RelationshipStrength Tests
#[test]
fn test_relationship_strength_values() {
    let strengths = vec![
        RelationshipStrength::Weak,
        RelationshipStrength::Medium,
        RelationshipStrength::Strong,
    ];

    for strength in strengths {
        let metadata = RelationshipMetadata {
            strength,
            bidirectional: false,
            properties: HashMap::new(),
        };
        assert_eq!(metadata.strength, strength);
    }
}

// Lifecycle Tests
#[test]
fn test_complete_lifecycle() {
    // Proposed → Active
    let proposed = proposed_state();
    let active = proposed.accept(
        Utc::now(),
        None,
        "delegation".to_string(),
        test_metadata(),
    ).unwrap();

    // Active → Modified
    let changes = vec![RelationshipChange::StrengthChanged {
        old_strength: RelationshipStrength::Medium,
        new_strength: RelationshipStrength::Strong,
    }];
    let modified = active.modify(Utc::now(), test_person_id(), changes).unwrap();

    // Modified → Active (apply changes)
    let now = Utc::now();
    let active_again = modified.apply_modifications((
        now,
        None,
        "delegation".to_string(),
        test_metadata(),
    )).unwrap();

    // Active → Suspended
    let suspended = active_again.suspend("Review".to_string(), Utc::now(), test_person_id()).unwrap();

    // Suspended → Active (reactivate)
    let reactivated = suspended.reactivate().unwrap();

    // Active → Terminated
    let terminated = reactivated.terminate("Ended".to_string(), Utc::now(), test_person_id()).unwrap();

    // Terminated → Archived
    let archived = terminated.archive(Utc::now(), test_person_id(), None).unwrap();

    assert!(archived.is_terminal());
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        proposed_state(),
        active_state(),
        modified_state(),
        suspended_state(),
        terminated_state(),
        archived_state(),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: RelationshipState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// Additional Edge Cases for 95%+ Coverage
#[test]
fn test_can_use_for_authorization_expired_relationship() {
    let now = Utc::now();
    let active = RelationshipState::Active {
        valid_from: now - chrono::Duration::hours(2),
        valid_until: Some(now - chrono::Duration::hours(1)), // Expired!
        relationship_type: "test".to_string(),
        metadata: test_metadata(),
    };
    // Even though it's Active, it's expired so can't be used for authorization
    assert!(!active.can_use_for_authorization());
}

#[test]
fn test_apply_modifications_with_invalid_temporal_bounds() {
    let modified = modified_state();
    let now = Utc::now();
    let params = (
        now,
        Some(now - chrono::Duration::hours(1)), // until < from (invalid!)
        "delegation".to_string(),
        test_metadata(),
    );
    let result = modified.apply_modifications(params);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => assert!(msg.contains("after")),
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_suspend_from_proposed_fails() {
    let proposed = proposed_state();
    let result = proposed.suspend("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::InvalidTransition { .. } => {},
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[test]
fn test_suspend_from_modified_fails() {
    let modified = modified_state();
    let result = modified.suspend("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
}

#[test]
fn test_suspend_from_terminated_fails() {
    let terminated = terminated_state();
    let result = terminated.suspend("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
}
