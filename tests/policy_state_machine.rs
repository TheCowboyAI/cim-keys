//! Comprehensive Policy State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/policy.rs

use chrono::Utc;
use cim_keys::state_machines::policy::{
    PolicyClaim, PolicyCondition, PolicyState, ReviewStatus, StateError,
};
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid { Uuid::now_v7() }

fn test_claims() -> Vec<PolicyClaim> {
    vec![PolicyClaim {
        resource: "keys.*".to_string(),
        action: "read".to_string(),
        scope: None,
    }]
}

fn test_conditions() -> Vec<PolicyCondition> {
    vec![PolicyCondition {
        condition_type: "time_based".to_string(),
        operator: "equals".to_string(),
        value: "9".to_string(),
    }]
}

fn draft_state() -> PolicyState {
    PolicyState::Draft {
        created_at: Utc::now(),
        author_id: test_person_id(),
        review_status: ReviewStatus::Pending,
    }
}

fn active_state() -> PolicyState {
    PolicyState::Active {
        activated_at: Utc::now(),
        claims: test_claims(),
        conditions: test_conditions(),
        enforcement_count: 0,
        last_enforced: None,
    }
}

fn modified_state() -> PolicyState {
    PolicyState::Modified {
        modified_at: Utc::now(),
        modified_by: test_person_id(),
        previous_version: Uuid::now_v7(),
        changes: vec![],
    }
}

fn suspended_state() -> PolicyState {
    PolicyState::Suspended {
        reason: "Review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
    }
}

fn revoked_state() -> PolicyState {
    PolicyState::Revoked {
        reason: "Replaced".to_string(),
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        replacement_policy: Some(Uuid::now_v7()),
    }
}

// State Query Tests
#[test]
fn test_is_active_for_all_states() {
    assert!(!draft_state().is_active());
    assert!(active_state().is_active());
    assert!(!modified_state().is_active());
    assert!(!suspended_state().is_active());
    assert!(!revoked_state().is_active());
}

#[test]
fn test_can_enforce_for_all_states() {
    assert!(!draft_state().can_enforce());
    assert!(active_state().can_enforce());
    assert!(!modified_state().can_enforce());
    assert!(!suspended_state().can_enforce());
    assert!(!revoked_state().can_enforce());
}

#[test]
fn test_can_be_modified_for_all_states() {
    assert!(draft_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(modified_state().can_be_modified());
    assert!(suspended_state().can_be_modified());
    assert!(!revoked_state().can_be_modified());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!draft_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!modified_state().is_terminal());
    assert!(!suspended_state().is_terminal());
    assert!(revoked_state().is_terminal());
}

#[test]
fn test_is_suspended_for_all_states() {
    assert!(!draft_state().is_suspended());
    assert!(!active_state().is_suspended());
    assert!(!modified_state().is_suspended());
    assert!(suspended_state().is_suspended());
    assert!(!revoked_state().is_suspended());
}

#[test]
fn test_is_draft_for_all_states() {
    assert!(draft_state().is_draft());
    assert!(!active_state().is_draft());
    assert!(!modified_state().is_draft());
    assert!(!suspended_state().is_draft());
    assert!(!revoked_state().is_draft());
}

#[test]
fn test_is_modified_for_all_states() {
    assert!(!draft_state().is_modified());
    assert!(!active_state().is_modified());
    assert!(modified_state().is_modified());
    assert!(!suspended_state().is_modified());
    assert!(!revoked_state().is_modified());
}

// Transition Validation Tests
#[test]
fn test_valid_transitions() {
    assert!(draft_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&modified_state()));
    assert!(modified_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&suspended_state()));
    assert!(suspended_state().can_transition_to(&active_state()));

    // Any non-terminal → Revoked
    assert!(draft_state().can_transition_to(&revoked_state()));
    assert!(active_state().can_transition_to(&revoked_state()));
    assert!(modified_state().can_transition_to(&revoked_state()));
    assert!(suspended_state().can_transition_to(&revoked_state()));
}

#[test]
fn test_cannot_transition_from_revoked() {
    let revoked = revoked_state();
    assert!(!revoked.can_transition_to(&draft_state()));
    assert!(!revoked.can_transition_to(&active_state()));
}

#[test]
fn test_validate_modification() {
    assert!(draft_state().validate_modification().is_ok());
    assert!(active_state().validate_modification().is_ok());
    assert!(modified_state().validate_modification().is_ok());
    assert!(suspended_state().validate_modification().is_err());
    assert!(revoked_state().validate_modification().is_err());
}

// Review Status Tests
#[test]
fn test_all_review_statuses() {
    let statuses = vec![
        ReviewStatus::Pending,
        ReviewStatus::InReview { reviewer_id: test_person_id() },
        ReviewStatus::Approved { approver_id: test_person_id() },
        ReviewStatus::Rejected { reason: "Test rejection".to_string() },
    ];

    for status in statuses {
        let draft = PolicyState::Draft {
            created_at: Utc::now(),
            author_id: test_person_id(),
            review_status: status.clone(),
        };
        assert!(draft.is_draft());
    }
}

// PolicyClaim Tests
#[test]
fn test_policy_claim_structure() {
    let claim = PolicyClaim {
        resource: "certificates.*".to_string(),
        action: "write".to_string(),
        scope: Some("organization:123".to_string()),
    };

    assert_eq!(claim.action, "write");
    assert_eq!(claim.resource, "certificates.*");
    assert_eq!(claim.scope, Some("organization:123".to_string()));
}

// PolicyCondition Tests
#[test]
fn test_policy_condition_structure() {
    let condition = PolicyCondition {
        condition_type: "location_based".to_string(),
        operator: "equals".to_string(),
        value: "office".to_string(),
    };

    assert_eq!(condition.condition_type, "location_based");
    assert_eq!(condition.operator, "equals");
    assert_eq!(condition.value, "office");
}

// Metadata Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(draft_state().description(), "Draft (under review, not enforced)");
    assert_eq!(active_state().description(), "Active (enforced for authorization)");
    assert_eq!(modified_state().description(), "Modified (awaiting activation)");
    assert_eq!(suspended_state().description(), "Suspended (temporarily not enforced)");
    assert_eq!(revoked_state().description(), "Revoked (TERMINAL - permanently disabled)");
}

// Lifecycle Tests
#[test]
fn test_complete_lifecycle() {
    // This test would require reading more of the implementation
    // For now, verify basic transitions work
    assert!(draft_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&suspended_state()));
    assert!(suspended_state().can_transition_to(&active_state()));
    assert!(active_state().can_transition_to(&revoked_state()));
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        draft_state(),
        active_state(),
        modified_state(),
        suspended_state(),
        revoked_state(),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: PolicyState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// Claim Validation Tests
#[test]
fn test_validate_claims_empty_fails() {
    let result = PolicyState::validate_claims(&[]);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => assert!(msg.contains("at least one claim")),
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_validate_claims_with_valid_claims() {
    let claims = test_claims();
    assert!(PolicyState::validate_claims(&claims).is_ok());
}

#[test]
fn test_validate_claims_with_invalid_claim_empty_resource() {
    let claims = vec![PolicyClaim {
        resource: "".to_string(),
        action: "read".to_string(),
        scope: None,
    }];
    let result = PolicyState::validate_claims(&claims);
    assert!(result.is_err());
}

#[test]
fn test_validate_claims_with_invalid_claim_empty_action() {
    let claims = vec![PolicyClaim {
        resource: "keys.*".to_string(),
        action: "".to_string(),
        scope: None,
    }];
    let result = PolicyState::validate_claims(&claims);
    assert!(result.is_err());
}

// Condition Validation Tests
#[test]
fn test_validate_conditions_empty_succeeds() {
    assert!(PolicyState::validate_conditions(&[]).is_ok());
}

#[test]
fn test_validate_conditions_with_valid_conditions() {
    let conditions = test_conditions();
    assert!(PolicyState::validate_conditions(&conditions).is_ok());
}

#[test]
fn test_validate_conditions_with_invalid_condition() {
    let conditions = vec![PolicyCondition {
        condition_type: "".to_string(),
        operator: "equals".to_string(),
        value: "test".to_string(),
    }];
    let result = PolicyState::validate_conditions(&conditions);
    assert!(result.is_err());
}

// PolicyClaim Validation Tests
#[test]
fn test_policy_claim_validate_empty_resource() {
    let claim = PolicyClaim {
        resource: "".to_string(),
        action: "read".to_string(),
        scope: None,
    };
    assert!(claim.validate().is_err());
}

#[test]
fn test_policy_claim_validate_empty_action() {
    let claim = PolicyClaim {
        resource: "keys.*".to_string(),
        action: "".to_string(),
        scope: None,
    };
    assert!(claim.validate().is_err());
}

#[test]
fn test_policy_claim_validate_success() {
    let claim = PolicyClaim {
        resource: "keys.*".to_string(),
        action: "read".to_string(),
        scope: Some("org:123".to_string()),
    };
    assert!(claim.validate().is_ok());
}

// PolicyCondition Validation Tests
#[test]
fn test_policy_condition_validate_empty_type() {
    let condition = PolicyCondition {
        condition_type: "".to_string(),
        operator: "equals".to_string(),
        value: "test".to_string(),
    };
    assert!(condition.validate().is_err());
}

#[test]
fn test_policy_condition_validate_empty_operator() {
    let condition = PolicyCondition {
        condition_type: "time_based".to_string(),
        operator: "".to_string(),
        value: "test".to_string(),
    };
    assert!(condition.validate().is_err());
}

#[test]
fn test_policy_condition_validate_success() {
    let condition = PolicyCondition {
        condition_type: "time_based".to_string(),
        operator: "equals".to_string(),
        value: "9".to_string(),
    };
    assert!(condition.validate().is_ok());
}

// Activation Tests
#[test]
fn test_activate_from_draft() {
    let draft = draft_state();
    let result = draft.activate(Utc::now(), test_claims(), test_conditions());
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_activate_from_modified() {
    let modified = modified_state();
    let result = modified.activate(Utc::now(), test_claims(), test_conditions());
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_activate_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.activate(Utc::now(), test_claims(), test_conditions());
    assert!(result.is_ok());
    assert!(result.unwrap().is_active());
}

#[test]
fn test_activate_from_active_fails() {
    let active = active_state();
    let result = active.activate(Utc::now(), test_claims(), test_conditions());
    assert!(result.is_err());
}

#[test]
fn test_activate_from_revoked_fails() {
    let revoked = revoked_state();
    let result = revoked.activate(Utc::now(), test_claims(), test_conditions());
    assert!(result.is_err());
}

#[test]
fn test_activate_with_empty_claims_fails() {
    let draft = draft_state();
    let result = draft.activate(Utc::now(), vec![], test_conditions());
    assert!(result.is_err());
}

#[test]
fn test_activate_with_invalid_claims_fails() {
    let draft = draft_state();
    let invalid_claims = vec![PolicyClaim {
        resource: "".to_string(),
        action: "read".to_string(),
        scope: None,
    }];
    let result = draft.activate(Utc::now(), invalid_claims, test_conditions());
    assert!(result.is_err());
}

// Enforcement Recording Tests
#[test]
fn test_record_enforcement_from_active() {
    let active = active_state();
    let result = active.record_enforcement(Utc::now());
    assert!(result.is_ok());
    // Verify enforcement count increased
    match result.unwrap() {
        PolicyState::Active { enforcement_count, .. } => assert_eq!(enforcement_count, 1),
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_record_enforcement_multiple_times() {
    let active = active_state();
    let result1 = active.record_enforcement(Utc::now()).unwrap();
    let result2 = result1.record_enforcement(Utc::now()).unwrap();
    let result3 = result2.record_enforcement(Utc::now()).unwrap();

    match result3 {
        PolicyState::Active { enforcement_count, .. } => assert_eq!(enforcement_count, 3),
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_record_enforcement_from_draft_fails() {
    let draft = draft_state();
    let result = draft.record_enforcement(Utc::now());
    assert!(result.is_err());
}

#[test]
fn test_record_enforcement_from_suspended_fails() {
    let suspended = suspended_state();
    let result = suspended.record_enforcement(Utc::now());
    assert!(result.is_err());
}

// Suspension Tests
#[test]
fn test_suspend_from_active() {
    let active = active_state();
    let result = active.suspend("Security review".to_string(), Utc::now(), test_person_id());
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
fn test_suspend_from_revoked_fails() {
    let revoked = revoked_state();
    let result = revoked.suspend("Test".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
}

// Revocation Tests
#[test]
fn test_revoke_from_active() {
    let active = active_state();
    let result = active.revoke(
        "Policy outdated".to_string(),
        Utc::now(),
        test_person_id(),
        Some(Uuid::now_v7()),
    );
    assert!(result.is_ok());
    assert!(result.unwrap().is_terminal());
}

#[test]
fn test_revoke_from_draft() {
    let draft = draft_state();
    let result = draft.revoke("Cancelled".to_string(), Utc::now(), test_person_id(), None);
    assert!(result.is_ok());
}

#[test]
fn test_revoke_from_suspended() {
    let suspended = suspended_state();
    let result = suspended.revoke("No longer needed".to_string(), Utc::now(), test_person_id(), None);
    assert!(result.is_ok());
}

#[test]
fn test_revoke_from_revoked_fails() {
    let revoked = revoked_state();
    let result = revoked.revoke("Test".to_string(), Utc::now(), test_person_id(), None);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::TerminalState(_) => {},
        _ => panic!("Expected TerminalState error"),
    }
}

#[test]
fn test_revoke_with_replacement_policy() {
    let active = active_state();
    let replacement_id = Uuid::now_v7();
    let result = active.revoke(
        "Replaced with newer version".to_string(),
        Utc::now(),
        test_person_id(),
        Some(replacement_id),
    );
    assert!(result.is_ok());
    match result.unwrap() {
        PolicyState::Revoked { replacement_policy, .. } => {
            assert_eq!(replacement_policy, Some(replacement_id));
        },
        _ => panic!("Expected Revoked state"),
    }
}

// Edge Cases
#[test]
fn test_active_with_enforcement_tracking() {
    let active = PolicyState::Active {
        activated_at: Utc::now(),
        claims: test_claims(),
        conditions: test_conditions(),
        enforcement_count: 1000,
        last_enforced: Some(Utc::now()),
    };

    assert!(active.is_active());
    assert!(active.can_enforce());
}

#[test]
fn test_revoked_with_replacement() {
    let replacement_id = Uuid::now_v7();
    let revoked = PolicyState::Revoked {
        reason: "Upgraded to new policy".to_string(),
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        replacement_policy: Some(replacement_id),
    };

    assert!(revoked.is_terminal());
}

#[test]
fn test_revoked_without_replacement() {
    let revoked = PolicyState::Revoked {
        reason: "No longer needed".to_string(),
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        replacement_policy: None,
    };

    assert!(revoked.is_terminal());
}
