//! Manifest Aggregate State Machine
//!
//! This module defines the lifecycle state machine for export manifests.
//! Manifests transition through 6 states from planning to verification.
//!
//! State Transitions:
//! - Planning → Generating (artifacts being created)
//! - Generating → Ready (all artifacts generated)
//! - Ready → Exported (written to target location)
//! - Exported → Verified (checksums validated)
//! - Any → Failed (error occurred)
//!
//! Invariants:
//! - Can't export unless Ready
//! - Can't verify unless Exported
//! - Failed is terminal state
//! - Verified is terminal state

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Lifecycle state machine for export manifests
///
/// Enforces manifest lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ManifestState {
    /// Manifest being planned (selecting artifacts)
    Planning {
        artifacts: Vec<ArtifactType>,
        planned_at: DateTime<Utc>,
        planned_by: Uuid, // Person ID
    },

    /// Artifacts being generated
    Generating {
        progress: HashMap<ArtifactType, GenerationProgress>,
        started_at: DateTime<Utc>,
    },

    /// All artifacts generated, ready for export
    Ready {
        checksum: String,
        artifact_count: u32,
        total_size_bytes: u64,
        ready_at: DateTime<Utc>,
    },

    /// Manifest exported to target location
    Exported {
        export_path: String,
        exported_at: DateTime<Utc>,
        exported_by: Uuid, // Person ID
    },

    /// Export verified (checksums match)
    Verified {
        verification_checksum: String,
        verified_at: DateTime<Utc>,
        verified_by: Uuid, // Person ID
    },

    /// Export failed (TERMINAL STATE)
    Failed {
        error: String,
        failed_at: DateTime<Utc>,
        failed_stage: FailedStage,
    },
}

impl ManifestState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the manifest ready for export?
    pub fn is_ready(&self) -> bool {
        matches!(self, ManifestState::Ready { .. })
    }

    /// Has the manifest been exported?
    pub fn is_exported(&self) -> bool {
        matches!(self, ManifestState::Exported { .. })
    }

    /// Has the export been verified?
    pub fn is_verified(&self) -> bool {
        matches!(self, ManifestState::Verified { .. })
    }

    /// Has the export failed?
    pub fn has_failed(&self) -> bool {
        matches!(self, ManifestState::Failed { .. })
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ManifestState::Verified { .. } | ManifestState::Failed { .. }
        )
    }

    /// Is generation in progress?
    pub fn is_generating(&self) -> bool {
        matches!(self, ManifestState::Generating { .. })
    }

    /// Can the manifest be exported?
    pub fn can_export(&self) -> bool {
        matches!(self, ManifestState::Ready { .. })
    }

    /// Can the export be verified?
    pub fn can_verify(&self) -> bool {
        matches!(self, ManifestState::Exported { .. })
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &ManifestState) -> bool {
        match (self, target) {
            // Planning → Generating
            (ManifestState::Planning { .. }, ManifestState::Generating { .. }) => true,

            // Generating → Ready
            (ManifestState::Generating { .. }, ManifestState::Ready { .. }) => true,

            // Ready → Exported
            (ManifestState::Ready { .. }, ManifestState::Exported { .. }) => true,

            // Exported → Verified
            (ManifestState::Exported { .. }, ManifestState::Verified { .. }) => true,

            // Any non-terminal → Failed
            (_, ManifestState::Failed { .. }) if !self.is_terminal() => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Start generating artifacts
    pub fn start_generating(
        &self,
        started_at: DateTime<Utc>,
    ) -> Result<ManifestState, StateError> {
        match self {
            ManifestState::Planning { artifacts, .. } => {
                if artifacts.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Cannot start generation without artifacts".to_string(),
                    ));
                }

                let progress: HashMap<ArtifactType, GenerationProgress> = artifacts
                    .iter()
                    .map(|artifact| (*artifact, GenerationProgress::Pending))
                    .collect();

                Ok(ManifestState::Generating {
                    progress,
                    started_at,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "start_generating".to_string(),
                reason: "Can only start generation from Planning state".to_string(),
            }),
        }
    }

    /// Mark artifact as completed
    pub fn complete_artifact(
        &self,
        artifact_type: ArtifactType,
        artifact_id: Uuid,
    ) -> Result<ManifestState, StateError> {
        match self {
            ManifestState::Generating {
                progress,
                started_at,
            } => {
                let mut new_progress = progress.clone();

                if !new_progress.contains_key(&artifact_type) {
                    return Err(StateError::ValidationFailed(format!(
                        "Artifact type {:?} not in manifest",
                        artifact_type
                    )));
                }

                new_progress.insert(
                    artifact_type,
                    GenerationProgress::Completed { artifact_id },
                );

                Ok(ManifestState::Generating {
                    progress: new_progress,
                    started_at: *started_at,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "complete_artifact".to_string(),
                reason: "Can only complete artifacts during Generating state".to_string(),
            }),
        }
    }

    /// Mark generation as complete and ready for export
    pub fn mark_ready(
        &self,
        checksum: String,
        artifact_count: u32,
        total_size_bytes: u64,
        ready_at: DateTime<Utc>,
    ) -> Result<ManifestState, StateError> {
        match self {
            ManifestState::Generating { progress, .. } => {
                // Verify all artifacts are completed
                let all_completed = progress.values().all(|p| {
                    matches!(p, GenerationProgress::Completed { .. })
                });

                if !all_completed {
                    return Err(StateError::ValidationFailed(
                        "Cannot mark ready - not all artifacts completed".to_string(),
                    ));
                }

                Ok(ManifestState::Ready {
                    checksum,
                    artifact_count,
                    total_size_bytes,
                    ready_at,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "mark_ready".to_string(),
                reason: "Can only mark ready from Generating state".to_string(),
            }),
        }
    }

    /// Export manifest to target location
    pub fn export(
        &self,
        export_path: String,
        exported_at: DateTime<Utc>,
        exported_by: Uuid,
    ) -> Result<ManifestState, StateError> {
        match self {
            ManifestState::Ready { .. } => Ok(ManifestState::Exported {
                export_path,
                exported_at,
                exported_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "export".to_string(),
                reason: "Can only export from Ready state".to_string(),
            }),
        }
    }

    /// Verify exported manifest (terminal state)
    pub fn verify(
        &self,
        verification_checksum: String,
        verified_at: DateTime<Utc>,
        verified_by: Uuid,
    ) -> Result<ManifestState, StateError> {
        match self {
            ManifestState::Exported { .. } => Ok(ManifestState::Verified {
                verification_checksum,
                verified_at,
                verified_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "verify".to_string(),
                reason: "Can only verify from Exported state".to_string(),
            }),
        }
    }

    /// Mark manifest as failed (terminal state)
    pub fn fail(
        &self,
        error: String,
        failed_at: DateTime<Utc>,
        failed_stage: FailedStage,
    ) -> Result<ManifestState, StateError> {
        if self.is_terminal() {
            return Err(StateError::TerminalState(
                "Cannot fail a manifest in terminal state".to_string(),
            ));
        }

        Ok(ManifestState::Failed {
            error,
            failed_at,
            failed_stage,
        })
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            ManifestState::Planning { .. } => "Planning (selecting artifacts)",
            ManifestState::Generating { .. } => "Generating (creating artifacts)",
            ManifestState::Ready { .. } => "Ready (all artifacts generated)",
            ManifestState::Exported { .. } => "Exported (written to target location)",
            ManifestState::Verified { .. } => "Verified (TERMINAL - checksums validated)",
            ManifestState::Failed { .. } => "Failed (TERMINAL - error occurred)",
        }
    }

    /// Get generation progress (if generating)
    pub fn generation_progress(&self) -> Option<&HashMap<ArtifactType, GenerationProgress>> {
        match self {
            ManifestState::Generating { progress, .. } => Some(progress),
            _ => None,
        }
    }

    /// Get checksum (if ready, exported, or verified)
    pub fn checksum(&self) -> Option<&str> {
        match self {
            ManifestState::Ready { checksum, .. } => Some(checksum),
            ManifestState::Verified {
                verification_checksum,
                ..
            } => Some(verification_checksum),
            _ => None,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Type of artifact in the manifest
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ArtifactType {
    RootCACertificate,
    IntermediateCACertificate,
    LeafCertificate,
    PublicKey,
    EncryptedPrivateKey,
    NatsOperatorJWT,
    NatsAccountJWT,
    NatsUserCreds,
    DidDocument,
    VerifiableCredential,
    ManifestFile,
}

/// Progress of artifact generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GenerationProgress {
    Pending,
    InProgress { percent: u8 },
    Completed { artifact_id: Uuid },
    Failed { error: String },
}

/// Stage at which manifest failed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FailedStage {
    Planning,
    Generating,
    Export,
    Verification,
}

/// Errors that can occur during state transitions
#[derive(Debug, Clone, thiserror::Error)]
pub enum StateError {
    #[error("Invalid state transition from {current} on event {event}: {reason}")]
    InvalidTransition {
        current: String,
        event: String,
        reason: String,
    },

    #[error("Terminal state reached: {0}")]
    TerminalState(String),

    #[error("State validation failed: {0}")]
    ValidationFailed(String),
}
