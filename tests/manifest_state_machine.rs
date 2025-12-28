//! Comprehensive Manifest State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/manifest.rs

use chrono::Utc;
use cim_keys::state_machines::manifest::{
    ArtifactType, FailedStage, GenerationProgress, ManifestState, StateError,
};
use std::collections::HashMap;
use uuid::Uuid;

// Test Helpers
fn test_person_id() -> Uuid {
    Uuid::now_v7()
}

fn planning_state() -> ManifestState {
    ManifestState::Planning {
        artifacts: vec![ArtifactType::RootCACertificate, ArtifactType::PublicKey],
        planned_at: Utc::now(),
        planned_by: test_person_id(),
    }
}

fn generating_state() -> ManifestState {
    let mut progress = HashMap::new();
    progress.insert(ArtifactType::RootCACertificate, GenerationProgress::Pending);
    progress.insert(ArtifactType::PublicKey, GenerationProgress::Pending);

    ManifestState::Generating {
        progress,
        started_at: Utc::now(),
    }
}

fn ready_state() -> ManifestState {
    ManifestState::Ready {
        checksum: "abc123".to_string(),
        artifact_count: 2,
        total_size_bytes: 1024,
        ready_at: Utc::now(),
    }
}

fn exported_state() -> ManifestState {
    ManifestState::Exported {
        export_path: "/mnt/encrypted/manifest.json".to_string(),
        exported_at: Utc::now(),
        exported_by: test_person_id(),
    }
}

fn verified_state() -> ManifestState {
    ManifestState::Verified {
        verification_checksum: "abc123".to_string(),
        verified_at: Utc::now(),
        verified_by: test_person_id(),
    }
}

fn failed_state(stage: FailedStage) -> ManifestState {
    ManifestState::Failed {
        error: "Test error".to_string(),
        failed_at: Utc::now(),
        failed_stage: stage,
    }
}

// State Query Tests
#[test]
fn test_is_ready_for_all_states() {
    assert!(!planning_state().is_ready());
    assert!(!generating_state().is_ready());
    assert!(ready_state().is_ready());
    assert!(!exported_state().is_ready());
    assert!(!verified_state().is_ready());
    assert!(!failed_state(FailedStage::Planning).is_ready());
}

#[test]
fn test_is_exported_for_all_states() {
    assert!(!planning_state().is_exported());
    assert!(!generating_state().is_exported());
    assert!(!ready_state().is_exported());
    assert!(exported_state().is_exported());
    assert!(!verified_state().is_exported());
    assert!(!failed_state(FailedStage::Export).is_exported());
}

#[test]
fn test_is_verified_for_all_states() {
    assert!(!planning_state().is_verified());
    assert!(!generating_state().is_verified());
    assert!(!ready_state().is_verified());
    assert!(!exported_state().is_verified());
    assert!(verified_state().is_verified());
    assert!(!failed_state(FailedStage::Verification).is_verified());
}

#[test]
fn test_has_failed_for_all_states() {
    assert!(!planning_state().has_failed());
    assert!(!generating_state().has_failed());
    assert!(!ready_state().has_failed());
    assert!(!exported_state().has_failed());
    assert!(!verified_state().has_failed());
    assert!(failed_state(FailedStage::Planning).has_failed());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!planning_state().is_terminal());
    assert!(!generating_state().is_terminal());
    assert!(!ready_state().is_terminal());
    assert!(!exported_state().is_terminal());
    assert!(verified_state().is_terminal());
    assert!(failed_state(FailedStage::Planning).is_terminal());
}

#[test]
fn test_is_generating_for_all_states() {
    assert!(!planning_state().is_generating());
    assert!(generating_state().is_generating());
    assert!(!ready_state().is_generating());
    assert!(!exported_state().is_generating());
    assert!(!verified_state().is_generating());
    assert!(!failed_state(FailedStage::Generating).is_generating());
}

#[test]
fn test_can_export_for_all_states() {
    assert!(!planning_state().can_export());
    assert!(!generating_state().can_export());
    assert!(ready_state().can_export());
    assert!(!exported_state().can_export());
    assert!(!verified_state().can_export());
    assert!(!failed_state(FailedStage::Export).can_export());
}

#[test]
fn test_can_verify_for_all_states() {
    assert!(!planning_state().can_verify());
    assert!(!generating_state().can_verify());
    assert!(!ready_state().can_verify());
    assert!(exported_state().can_verify());
    assert!(!verified_state().can_verify());
    assert!(!failed_state(FailedStage::Verification).can_verify());
}

// Transition Validation Tests
#[test]
fn test_valid_transitions() {
    assert!(planning_state().can_transition_to(&generating_state()));
    assert!(generating_state().can_transition_to(&ready_state()));
    assert!(ready_state().can_transition_to(&exported_state()));
    assert!(exported_state().can_transition_to(&verified_state()));
}

#[test]
fn test_any_non_terminal_can_fail() {
    let failed = failed_state(FailedStage::Generating);
    assert!(planning_state().can_transition_to(&failed));
    assert!(generating_state().can_transition_to(&failed));
    assert!(ready_state().can_transition_to(&failed));
    assert!(exported_state().can_transition_to(&failed));
}

#[test]
fn test_terminal_states_cannot_transition() {
    let verified = verified_state();
    assert!(!verified.can_transition_to(&planning_state()));
    assert!(!verified.can_transition_to(&failed_state(FailedStage::Verification)));

    let failed = failed_state(FailedStage::Export);
    assert!(!failed.can_transition_to(&ready_state()));
    assert!(!failed.can_transition_to(&verified_state()));
}

// Start Generation Tests
#[test]
fn test_start_generating_from_planning() {
    let planning = planning_state();
    let result = planning.start_generating(Utc::now());
    assert!(result.is_ok());
    let generating = result.unwrap();
    assert!(generating.is_generating());
}

#[test]
fn test_cannot_start_generating_without_artifacts() {
    let planning = ManifestState::Planning {
        artifacts: vec![],
        planned_at: Utc::now(),
        planned_by: test_person_id(),
    };
    let result = planning.start_generating(Utc::now());
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("without artifacts"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_cannot_start_generating_from_non_planning() {
    assert!(ready_state().start_generating(Utc::now()).is_err());
}

// Complete Artifact Tests
#[test]
fn test_complete_artifact_during_generation() {
    let generating = generating_state();
    let result = generating.complete_artifact(ArtifactType::RootCACertificate, Uuid::now_v7());
    assert!(result.is_ok());
}

#[test]
fn test_cannot_complete_unknown_artifact() {
    let generating = generating_state();
    let result = generating.complete_artifact(ArtifactType::DidDocument, Uuid::now_v7());
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("not in manifest"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_cannot_complete_artifact_outside_generating() {
    let planning = planning_state();
    let result = planning.complete_artifact(ArtifactType::PublicKey, Uuid::now_v7());
    assert!(result.is_err());
}

// Mark Ready Tests
#[test]
fn test_mark_ready_when_all_completed() {
    let mut progress = HashMap::new();
    progress.insert(
        ArtifactType::RootCACertificate,
        GenerationProgress::Completed {
            artifact_id: Uuid::now_v7(),
        },
    );
    progress.insert(
        ArtifactType::PublicKey,
        GenerationProgress::Completed {
            artifact_id: Uuid::now_v7(),
        },
    );

    let generating = ManifestState::Generating {
        progress,
        started_at: Utc::now(),
    };

    let result = generating.mark_ready("checksum".to_string(), 2, 2048, Utc::now());
    assert!(result.is_ok());
    let ready = result.unwrap();
    assert!(ready.is_ready());
}

#[test]
fn test_cannot_mark_ready_with_incomplete_artifacts() {
    let generating = generating_state(); // Has Pending artifacts
    let result = generating.mark_ready("checksum".to_string(), 2, 2048, Utc::now());
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::ValidationFailed(msg) => {
            assert!(msg.contains("not all artifacts completed"));
        }
        _ => panic!("Expected ValidationFailed"),
    }
}

#[test]
fn test_cannot_mark_ready_from_non_generating() {
    let planning = planning_state();
    let result = planning.mark_ready("checksum".to_string(), 2, 2048, Utc::now());
    assert!(result.is_err());
}

// Export Tests
#[test]
fn test_export_from_ready() {
    let ready = ready_state();
    let result = ready.export(
        "/export/path".to_string(),
        Utc::now(),
        test_person_id(),
    );
    assert!(result.is_ok());
    let exported = result.unwrap();
    assert!(exported.is_exported());
}

#[test]
fn test_cannot_export_from_non_ready() {
    let planning = planning_state();
    let result = planning.export(
        "/export/path".to_string(),
        Utc::now(),
        test_person_id(),
    );
    assert!(result.is_err());
}

// Verify Tests
#[test]
fn test_verify_from_exported() {
    let exported = exported_state();
    let result = exported.verify("checksum".to_string(), Utc::now(), test_person_id());
    assert!(result.is_ok());
    let verified = result.unwrap();
    assert!(verified.is_verified());
    assert!(verified.is_terminal());
}

#[test]
fn test_cannot_verify_from_non_exported() {
    let ready = ready_state();
    let result = ready.verify("checksum".to_string(), Utc::now(), test_person_id());
    assert!(result.is_err());
}

// Fail Tests
#[test]
fn test_fail_from_any_non_terminal() {
    let states = vec![
        planning_state(),
        generating_state(),
        ready_state(),
        exported_state(),
    ];

    for state in states {
        let result = state.fail(
            "Error occurred".to_string(),
            Utc::now(),
            FailedStage::Generating,
        );
        assert!(result.is_ok());
        let failed = result.unwrap();
        assert!(failed.has_failed());
        assert!(failed.is_terminal());
    }
}

#[test]
fn test_cannot_fail_from_terminal() {
    let verified = verified_state();
    let result = verified.fail(
        "Error".to_string(),
        Utc::now(),
        FailedStage::Verification,
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        StateError::TerminalState(_) => {}
        _ => panic!("Expected TerminalState"),
    }

    let failed = failed_state(FailedStage::Export);
    let result = failed.fail("Error".to_string(), Utc::now(), FailedStage::Export);
    assert!(result.is_err());
}

// Description Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(
        planning_state().description(),
        "Planning (selecting artifacts)"
    );
    assert_eq!(
        generating_state().description(),
        "Generating (creating artifacts)"
    );
    assert_eq!(ready_state().description(), "Ready (all artifacts generated)");
    assert_eq!(
        exported_state().description(),
        "Exported (written to target location)"
    );
    assert_eq!(
        verified_state().description(),
        "Verified (TERMINAL - checksums validated)"
    );
    assert_eq!(
        failed_state(FailedStage::Export).description(),
        "Failed (TERMINAL - error occurred)"
    );
}

// ArtifactType Tests
#[test]
fn test_all_artifact_types() {
    let types = vec![
        ArtifactType::RootCACertificate,
        ArtifactType::IntermediateCACertificate,
        ArtifactType::LeafCertificate,
        ArtifactType::PublicKey,
        ArtifactType::EncryptedPrivateKey,
        ArtifactType::NatsOperatorJWT,
        ArtifactType::NatsAccountJWT,
        ArtifactType::NatsUserCreds,
        ArtifactType::DidDocument,
        ArtifactType::VerifiableCredential,
        ArtifactType::ManifestFile,
    ];

    for artifact_type in types {
        let planning = ManifestState::Planning {
            artifacts: vec![artifact_type],
            planned_at: Utc::now(),
            planned_by: test_person_id(),
        };
        assert!(!planning.is_ready());
    }
}

// GenerationProgress Tests
#[test]
fn test_all_generation_progress_states() {
    let progress_states = vec![
        GenerationProgress::Pending,
        GenerationProgress::InProgress { percent: 50 },
        GenerationProgress::Completed {
            artifact_id: Uuid::now_v7(),
        },
        GenerationProgress::Failed {
            error: "Test error".to_string(),
        },
    ];

    for progress in progress_states {
        let mut map = HashMap::new();
        map.insert(ArtifactType::PublicKey, progress);

        let generating = ManifestState::Generating {
            progress: map,
            started_at: Utc::now(),
        };
        assert!(generating.is_generating());
    }
}

// FailedStage Tests
#[test]
fn test_all_failed_stages() {
    let stages = vec![
        FailedStage::Planning,
        FailedStage::Generating,
        FailedStage::Export,
        FailedStage::Verification,
    ];

    for stage in stages {
        let failed = failed_state(stage);
        assert!(failed.has_failed());
        assert!(failed.is_terminal());
    }
}

// Helper Methods Tests
#[test]
fn test_generation_progress_method() {
    let generating = generating_state();
    assert!(generating.generation_progress().is_some());

    assert!(planning_state().generation_progress().is_none());
    assert!(ready_state().generation_progress().is_none());
}

#[test]
fn test_checksum_method() {
    let ready = ready_state();
    assert_eq!(ready.checksum(), Some("abc123"));

    let verified = verified_state();
    assert_eq!(verified.checksum(), Some("abc123"));

    assert!(planning_state().checksum().is_none());
    assert!(generating_state().checksum().is_none());
    assert!(exported_state().checksum().is_none());
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        planning_state(),
        generating_state(),
        ready_state(),
        exported_state(),
        verified_state(),
        failed_state(FailedStage::Export),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ManifestState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// Complete Lifecycle Tests
#[test]
fn test_complete_lifecycle_planning_to_verified() {
    // Planning
    let planning = planning_state();

    // Start generating
    let generating = planning.start_generating(Utc::now()).unwrap();
    assert!(generating.is_generating());

    // Complete artifacts
    let gen1 = generating
        .complete_artifact(ArtifactType::RootCACertificate, Uuid::now_v7())
        .unwrap();
    let gen2 = gen1
        .complete_artifact(ArtifactType::PublicKey, Uuid::now_v7())
        .unwrap();

    // Mark ready
    let ready = gen2
        .mark_ready("checksum123".to_string(), 2, 4096, Utc::now())
        .unwrap();
    assert!(ready.is_ready());

    // Export
    let exported = ready
        .export(
            "/mnt/export/manifest.json".to_string(),
            Utc::now(),
            test_person_id(),
        )
        .unwrap();
    assert!(exported.is_exported());

    // Verify
    let verified = exported
        .verify("checksum123".to_string(), Utc::now(), test_person_id())
        .unwrap();
    assert!(verified.is_verified());
    assert!(verified.is_terminal());
}

#[test]
fn test_lifecycle_with_failure() {
    let planning = planning_state();
    let generating = planning.start_generating(Utc::now()).unwrap();

    // Fail during generation
    let failed = generating
        .fail("Disk full".to_string(), Utc::now(), FailedStage::Generating)
        .unwrap();

    assert!(failed.has_failed());
    assert!(failed.is_terminal());
}
