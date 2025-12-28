//! Comprehensive Location State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/location.rs

use chrono::Utc;
use cim_keys::state_machines::location::{
    AccessGrant, AccessLevel, LocationState, LocationType, StateError,
};
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid {
    Uuid::now_v7()
}
fn test_admin_id() -> Uuid {
    Uuid::now_v7()
}

fn planned_state() -> LocationState {
    LocationState::Planned {
        planned_at: Utc::now(),
        planned_by: test_admin_id(),
        location_type: LocationType::Physical,
    }
}

fn active_state() -> LocationState {
    LocationState::Active {
        activated_at: Utc::now(),
        access_grants: vec![AccessGrant {
            person_id: test_person_id(),
            granted_at: Utc::now(),
            granted_by: test_admin_id(),
            access_level: AccessLevel::Admin,
        }],
        assets_stored: 0,
        last_accessed: None,
    }
}

fn active_with_assets(count: u64) -> LocationState {
    LocationState::Active {
        activated_at: Utc::now(),
        access_grants: vec![AccessGrant {
            person_id: test_person_id(),
            granted_at: Utc::now(),
            granted_by: test_admin_id(),
            access_level: AccessLevel::Admin,
        }],
        assets_stored: count,
        last_accessed: None,
    }
}

fn decommissioned_state(remaining: u64) -> LocationState {
    LocationState::Decommissioned {
        reason: "Migration to new facility".to_string(),
        decommissioned_at: Utc::now(),
        decommissioned_by: test_admin_id(),
        remaining_assets: remaining,
    }
}

fn archived_state() -> LocationState {
    LocationState::Archived {
        archived_at: Utc::now(),
        archived_by: test_admin_id(),
        final_audit_id: Some(Uuid::now_v7()),
    }
}

// State Query Tests
#[test]
fn test_is_active_for_all_states() {
    assert!(!planned_state().is_active());
    assert!(active_state().is_active());
    assert!(!decommissioned_state(0).is_active());
    assert!(!archived_state().is_active());
}

#[test]
fn test_can_store_assets_for_all_states() {
    assert!(!planned_state().can_store_assets());
    assert!(active_state().can_store_assets());
    assert!(!decommissioned_state(0).can_store_assets());
    assert!(!archived_state().can_store_assets());
}

#[test]
fn test_can_grant_access_for_all_states() {
    assert!(!planned_state().can_grant_access());
    assert!(active_state().can_grant_access());
    assert!(!decommissioned_state(0).can_grant_access());
    assert!(!archived_state().can_grant_access());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!planned_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!decommissioned_state(0).is_terminal());
    assert!(archived_state().is_terminal());
}

#[test]
fn test_is_decommissioned_for_all_states() {
    assert!(!planned_state().is_decommissioned());
    assert!(!active_state().is_decommissioned());
    assert!(decommissioned_state(0).is_decommissioned());
    assert!(!archived_state().is_decommissioned());
}

#[test]
fn test_can_be_modified_for_all_states() {
    assert!(planned_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(decommissioned_state(0).can_be_modified());
    assert!(!archived_state().can_be_modified());
}

#[test]
fn test_asset_count_for_all_states() {
    assert_eq!(planned_state().asset_count(), 0);
    assert_eq!(active_state().asset_count(), 0);
    assert_eq!(active_with_assets(5).asset_count(), 5);
    assert_eq!(decommissioned_state(3).asset_count(), 3);
    assert_eq!(archived_state().asset_count(), 0);
}

// Transition Validation Tests
#[test]
fn test_valid_transitions() {
    // Planned → Active
    assert!(planned_state().can_transition_to(&active_state()));

    // Active → Decommissioned
    assert!(active_state().can_transition_to(&decommissioned_state(0)));

    // Decommissioned (0 assets) → Archived
    assert!(decommissioned_state(0).can_transition_to(&archived_state()));
}

#[test]
fn test_cannot_archive_with_remaining_assets() {
    let decomm_with_assets = decommissioned_state(5);
    let archived = archived_state();
    assert!(!decomm_with_assets.can_transition_to(&archived));
}

#[test]
fn test_cannot_transition_from_archived() {
    let archived = archived_state();
    assert!(!archived.can_transition_to(&planned_state()));
    assert!(!archived.can_transition_to(&active_state()));
}

#[test]
fn test_invalid_direct_transitions() {
    // Planned → Decommissioned (must go through Active)
    assert!(!planned_state().can_transition_to(&decommissioned_state(0)));

    // Planned → Archived
    assert!(!planned_state().can_transition_to(&archived_state()));

    // Active → Archived (must go through Decommissioned)
    assert!(!active_state().can_transition_to(&archived_state()));
}

// Activation Tests
#[test]
fn test_activate_planned_location() {
    let planned = planned_state();
    let grants = vec![AccessGrant {
        person_id: test_person_id(),
        granted_at: Utc::now(),
        granted_by: test_admin_id(),
        access_level: AccessLevel::Admin,
    }];

    let result = planned.activate(Utc::now(), grants);
    assert!(result.is_ok());
    let active = result.unwrap();
    assert!(active.is_active());
    assert_eq!(active.asset_count(), 0);
}

#[test]
fn test_cannot_activate_without_access_grants() {
    let planned = planned_state();
    let result = planned.activate(Utc::now(), vec![]);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("without access grants"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_cannot_activate_non_planned() {
    let active = active_state();
    let grants = vec![AccessGrant {
        person_id: test_person_id(),
        granted_at: Utc::now(),
        granted_by: test_admin_id(),
        access_level: AccessLevel::Admin,
    }];
    let result = active.activate(Utc::now(), grants);
    assert!(result.is_err());
}

// Decommission Tests
#[test]
fn test_decommission_active_location() {
    let active = active_with_assets(10);
    let result = active.decommission(
        "End of lease".to_string(),
        Utc::now(),
        test_admin_id(),
    );
    assert!(result.is_ok());
    let decomm = result.unwrap();
    assert!(decomm.is_decommissioned());
    assert_eq!(decomm.asset_count(), 10);
}

#[test]
fn test_cannot_decommission_non_active() {
    let planned = planned_state();
    let result = planned.decommission(
        "Test".to_string(),
        Utc::now(),
        test_admin_id(),
    );
    assert!(result.is_err());
}

// Archive Tests
#[test]
fn test_archive_empty_decommissioned_location() {
    let decomm = decommissioned_state(0);
    let result = decomm.archive(Utc::now(), test_admin_id(), Some(Uuid::now_v7()));
    assert!(result.is_ok());
    let archived = result.unwrap();
    assert!(archived.is_terminal());
}

#[test]
fn test_archive_method_fails_with_remaining_assets() {
    let decomm = decommissioned_state(5);
    let result = decomm.archive(Utc::now(), test_admin_id(), None);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("5 remaining assets"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_cannot_archive_non_decommissioned() {
    let active = active_state();
    let result = active.archive(Utc::now(), test_admin_id(), None);
    assert!(result.is_err());
}

// Access Grant Tests
#[test]
fn test_grant_access_to_active_location() {
    let active = active_state();
    let new_grant = AccessGrant {
        person_id: test_person_id(),
        granted_at: Utc::now(),
        granted_by: test_admin_id(),
        access_level: AccessLevel::ReadWrite,
    };

    let result = active.grant_access(new_grant);
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.access_grants().unwrap().len(), 2);
}

#[test]
fn test_cannot_grant_duplicate_access() {
    let person = test_person_id();
    let active = LocationState::Active {
        activated_at: Utc::now(),
        access_grants: vec![AccessGrant {
            person_id: person,
            granted_at: Utc::now(),
            granted_by: test_admin_id(),
            access_level: AccessLevel::ReadOnly,
        }],
        assets_stored: 0,
        last_accessed: None,
    };

    let duplicate_grant = AccessGrant {
        person_id: person,
        granted_at: Utc::now(),
        granted_by: test_admin_id(),
        access_level: AccessLevel::Admin,
    };

    let result = active.grant_access(duplicate_grant);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("already has access"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_cannot_grant_access_to_non_active() {
    let planned = planned_state();
    let grant = AccessGrant {
        person_id: test_person_id(),
        granted_at: Utc::now(),
        granted_by: test_admin_id(),
        access_level: AccessLevel::Admin,
    };
    let result = planned.grant_access(grant);
    assert!(result.is_err());
}

// Revoke Access Tests
#[test]
fn test_revoke_access_from_active_location() {
    let person = test_person_id();
    let active = LocationState::Active {
        activated_at: Utc::now(),
        access_grants: vec![
            AccessGrant {
                person_id: person,
                granted_at: Utc::now(),
                granted_by: test_admin_id(),
                access_level: AccessLevel::ReadOnly,
            },
            AccessGrant {
                person_id: test_person_id(),
                granted_at: Utc::now(),
                granted_by: test_admin_id(),
                access_level: AccessLevel::Admin,
            },
        ],
        assets_stored: 0,
        last_accessed: None,
    };

    let result = active.revoke_access(person);
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.access_grants().unwrap().len(), 1);
}

#[test]
fn test_cannot_revoke_nonexistent_access() {
    let active = active_state();
    let nonexistent_person = test_person_id();
    let result = active.revoke_access(nonexistent_person);
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("does not have access"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_cannot_revoke_access_from_non_active() {
    let planned = planned_state();
    let result = planned.revoke_access(test_person_id());
    assert!(result.is_err());
}

// Asset Management Tests
#[test]
fn test_add_asset_to_active_location() {
    let active = active_state();
    let result = active.add_asset();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().asset_count(), 1);
}

#[test]
fn test_cannot_add_asset_to_non_active() {
    let planned = planned_state();
    assert!(planned.add_asset().is_err());

    let decomm = decommissioned_state(0);
    assert!(decomm.add_asset().is_err());

    let archived = archived_state();
    assert!(archived.add_asset().is_err());
}

#[test]
fn test_remove_asset_from_active_location() {
    let active = active_with_assets(5);
    let result = active.remove_asset();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().asset_count(), 4);
}

#[test]
fn test_remove_asset_from_decommissioned_location() {
    let decomm = decommissioned_state(3);
    let result = decomm.remove_asset();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().asset_count(), 2);
}

#[test]
fn test_cannot_remove_asset_when_empty() {
    let active = active_state(); // 0 assets
    let result = active.remove_asset();
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("No assets to remove"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_cannot_remove_asset_from_planned_or_archived() {
    let planned = planned_state();
    assert!(planned.remove_asset().is_err());

    let archived = archived_state();
    assert!(archived.remove_asset().is_err());
}

// Record Access Tests
#[test]
fn test_record_access_to_active_location() {
    let active = active_state();
    let accessed_at = Utc::now();
    let result = active.record_access(accessed_at);
    assert!(result.is_ok());

    let updated = result.unwrap();
    match updated {
        LocationState::Active { last_accessed, .. } => {
            assert!(last_accessed.is_some());
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_cannot_record_access_to_non_active() {
    let planned = planned_state();
    assert!(planned.record_access(Utc::now()).is_err());

    let decomm = decommissioned_state(0);
    assert!(decomm.record_access(Utc::now()).is_err());

    let archived = archived_state();
    assert!(archived.record_access(Utc::now()).is_err());
}

// Description Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(planned_state().description(), "Planned (not yet operational)");
    assert_eq!(
        active_state().description(),
        "Active (operational, can store assets)"
    );
    assert_eq!(
        decommissioned_state(0).description(),
        "Decommissioned (no new assets, existing assets need migration)"
    );
    assert_eq!(
        archived_state().description(),
        "Archived (TERMINAL - all assets removed)"
    );
}

// LocationType Tests
#[test]
fn test_all_location_types() {
    let types = vec![
        LocationType::Physical,
        LocationType::Virtual,
        LocationType::Logical,
        LocationType::Hybrid,
    ];

    for loc_type in types {
        let planned = LocationState::Planned {
            planned_at: Utc::now(),
            planned_by: test_admin_id(),
            location_type: loc_type,
        };
        assert!(!planned.is_active());
    }
}

// AccessLevel Tests
#[test]
fn test_all_access_levels() {
    let levels = vec![AccessLevel::ReadOnly, AccessLevel::ReadWrite, AccessLevel::Admin];

    for level in levels {
        let grant = AccessGrant {
            person_id: test_person_id(),
            granted_at: Utc::now(),
            granted_by: test_admin_id(),
            access_level: level,
        };

        let active = LocationState::Active {
            activated_at: Utc::now(),
            access_grants: vec![grant],
            assets_stored: 0,
            last_accessed: None,
        };

        assert_eq!(active.access_grants().unwrap().len(), 1);
    }
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        planned_state(),
        active_state(),
        active_with_assets(10),
        decommissioned_state(5),
        archived_state(),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: LocationState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// Lifecycle Workflow Tests
#[test]
fn test_complete_lifecycle_planned_to_archived() {
    // Start: Planned
    let planned = planned_state();
    assert!(!planned.is_active());

    // Activate with access grants
    let grants = vec![AccessGrant {
        person_id: test_person_id(),
        granted_at: Utc::now(),
        granted_by: test_admin_id(),
        access_level: AccessLevel::Admin,
    }];
    let active = planned.activate(Utc::now(), grants).unwrap();
    assert!(active.is_active());

    // Add some assets
    let with_assets = active.add_asset().unwrap().add_asset().unwrap();
    assert_eq!(with_assets.asset_count(), 2);

    // Decommission
    let decomm = with_assets
        .decommission("Facility closing".to_string(), Utc::now(), test_admin_id())
        .unwrap();
    assert!(decomm.is_decommissioned());
    assert_eq!(decomm.asset_count(), 2);

    // Remove all assets
    let decomm1 = decomm.remove_asset().unwrap();
    let decomm_empty = decomm1.remove_asset().unwrap();
    assert_eq!(decomm_empty.asset_count(), 0);

    // Archive
    let archived = decomm_empty
        .archive(Utc::now(), test_admin_id(), Some(Uuid::now_v7()))
        .unwrap();
    assert!(archived.is_terminal());
}

// Edge Cases
#[test]
fn test_access_grants_returns_none_for_non_active() {
    assert!(planned_state().access_grants().is_none());
    assert!(decommissioned_state(0).access_grants().is_none());
    assert!(archived_state().access_grants().is_none());
}

#[test]
fn test_multiple_asset_operations() {
    let active = active_state();

    // Add multiple assets
    let loc1 = active.add_asset().unwrap();
    let loc2 = loc1.add_asset().unwrap();
    let loc3 = loc2.add_asset().unwrap();
    assert_eq!(loc3.asset_count(), 3);

    // Remove multiple assets
    let loc4 = loc3.remove_asset().unwrap();
    let loc5 = loc4.remove_asset().unwrap();
    assert_eq!(loc5.asset_count(), 1);
}
