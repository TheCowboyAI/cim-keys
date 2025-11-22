//! Relationship Aggregate State Machine
//!
//! This module defines the lifecycle state machine for relationships between entities.
//! Relationships transition through 6 states from proposal to archival.
//!
//! State Transitions:
//! - Proposed → Active (relationship accepted)
//! - Active → Modified (relationship parameters changed)
//! - Active ↔ Suspended (temporary suspension)
//! - Active/Modified/Suspended → Terminated (permanent end)
//! - Terminated → Archived (after retention period)
//!
//! Invariants:
//! - Can't use relationship for authorization unless Active
//! - Can't modify if Terminated
//! - Archived is terminal state
//! - Temporal validity always enforced

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state machine for relationships between entities
///
/// Enforces relationship lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipState {
    /// Relationship proposed but not yet accepted
    Proposed {
        proposed_at: DateTime<Utc>,
        proposed_by: Uuid, // Person who proposed
        pending_approval_from: Option<Uuid>,
    },

    /// Relationship is active and valid
    Active {
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>, // None = indefinite
        relationship_type: String,
        metadata: RelationshipMetadata,
    },

    /// Relationship has been modified
    Modified {
        modified_at: DateTime<Utc>,
        modified_by: Uuid, // Person who modified
        previous_version: Box<RelationshipState>,
        changes: Vec<RelationshipChange>,
    },

    /// Relationship temporarily suspended
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid, // Person who suspended
        previous_state: Box<RelationshipState>,
    },

    /// Relationship permanently terminated
    Terminated {
        reason: String,
        terminated_at: DateTime<Utc>,
        terminated_by: Uuid, // Person who terminated
    },

    /// Relationship archived (TERMINAL STATE)
    Archived {
        archived_at: DateTime<Utc>,
        archived_by: Uuid, // Person who archived
        retention_policy_id: Option<Uuid>,
    },
}

impl RelationshipState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the relationship active?
    pub fn is_active(&self) -> bool {
        matches!(self, RelationshipState::Active { .. })
    }

    /// Can the relationship be used for authorization decisions?
    pub fn can_use_for_authorization(&self) -> bool {
        match self {
            RelationshipState::Active {
                valid_until, ..
            } => {
                // Check temporal validity
                if let Some(until) = valid_until {
                    Utc::now() <= *until
                } else {
                    true // Indefinite validity
                }
            }
            _ => false,
        }
    }

    /// Can the relationship be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, RelationshipState::Archived { .. })
    }

    /// Is the relationship suspended?
    pub fn is_suspended(&self) -> bool {
        matches!(self, RelationshipState::Suspended { .. })
    }

    /// Is the relationship terminated?
    pub fn is_terminated(&self) -> bool {
        matches!(self, RelationshipState::Terminated { .. })
    }

    /// Is the relationship proposed (pending acceptance)?
    pub fn is_proposed(&self) -> bool {
        matches!(self, RelationshipState::Proposed { .. })
    }

    /// Check if relationship is temporally valid at a given time
    pub fn is_valid_at(&self, check_time: DateTime<Utc>) -> bool {
        match self {
            RelationshipState::Active {
                valid_from,
                valid_until,
                ..
            } => {
                let after_start = check_time >= *valid_from;
                let before_end = valid_until
                    .map(|until| check_time <= until)
                    .unwrap_or(true);

                after_start && before_end
            }
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &RelationshipState) -> bool {
        match (self, target) {
            // Proposed → Active (accepted)
            (RelationshipState::Proposed { .. }, RelationshipState::Active { .. }) => true,

            // Active → Modified
            (RelationshipState::Active { .. }, RelationshipState::Modified { .. }) => true,

            // Modified → Active (changes applied)
            (RelationshipState::Modified { .. }, RelationshipState::Active { .. }) => true,

            // Active → Suspended
            (RelationshipState::Active { .. }, RelationshipState::Suspended { .. }) => true,

            // Suspended → Active (reactivation)
            (RelationshipState::Suspended { .. }, RelationshipState::Active { .. }) => true,

            // Active/Modified/Suspended → Terminated
            (RelationshipState::Active { .. }, RelationshipState::Terminated { .. }) => true,
            (RelationshipState::Modified { .. }, RelationshipState::Terminated { .. }) => true,
            (RelationshipState::Suspended { .. }, RelationshipState::Terminated { .. }) => true,

            // Terminated → Archived
            (RelationshipState::Terminated { .. }, RelationshipState::Archived { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Accept a proposed relationship (activate it)
    pub fn accept(
        &self,
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
        relationship_type: String,
        metadata: RelationshipMetadata,
    ) -> Result<RelationshipState, StateError> {
        match self {
            RelationshipState::Proposed { .. } => {
                // Validate temporal bounds
                if let Some(until) = valid_until {
                    if until <= valid_from {
                        return Err(StateError::ValidationFailed(
                            "valid_until must be after valid_from".to_string(),
                        ));
                    }
                }

                Ok(RelationshipState::Active {
                    valid_from,
                    valid_until,
                    relationship_type,
                    metadata,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "accept".to_string(),
                reason: "Can only accept Proposed relationships".to_string(),
            }),
        }
    }

    /// Modify an active relationship
    pub fn modify(
        &self,
        modified_at: DateTime<Utc>,
        modified_by: Uuid,
        changes: Vec<RelationshipChange>,
    ) -> Result<RelationshipState, StateError> {
        match self {
            RelationshipState::Active { .. } => {
                if changes.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Cannot modify relationship without changes".to_string(),
                    ));
                }

                Ok(RelationshipState::Modified {
                    modified_at,
                    modified_by,
                    previous_version: Box::new(self.clone()),
                    changes,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "modify".to_string(),
                reason: "Can only modify Active relationships".to_string(),
            }),
        }
    }

    /// Apply modifications and return to Active state
    pub fn apply_modifications(
        &self,
        new_state_params: (DateTime<Utc>, Option<DateTime<Utc>>, String, RelationshipMetadata),
    ) -> Result<RelationshipState, StateError> {
        match self {
            RelationshipState::Modified { .. } => {
                let (valid_from, valid_until, relationship_type, metadata) = new_state_params;

                // Validate temporal bounds
                if let Some(until) = valid_until {
                    if until <= valid_from {
                        return Err(StateError::ValidationFailed(
                            "valid_until must be after valid_from".to_string(),
                        ));
                    }
                }

                Ok(RelationshipState::Active {
                    valid_from,
                    valid_until,
                    relationship_type,
                    metadata,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "apply_modifications".to_string(),
                reason: "Can only apply modifications to Modified relationships".to_string(),
            }),
        }
    }

    /// Suspend an active relationship
    pub fn suspend(
        &self,
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid,
    ) -> Result<RelationshipState, StateError> {
        match self {
            RelationshipState::Active { .. } => Ok(RelationshipState::Suspended {
                reason,
                suspended_at,
                suspended_by,
                previous_state: Box::new(self.clone()),
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "suspend".to_string(),
                reason: "Can only suspend Active relationships".to_string(),
            }),
        }
    }

    /// Reactivate a suspended relationship
    pub fn reactivate(&self) -> Result<RelationshipState, StateError> {
        match self {
            RelationshipState::Suspended {
                previous_state, ..
            } => Ok(*previous_state.clone()),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "reactivate".to_string(),
                reason: "Can only reactivate Suspended relationships".to_string(),
            }),
        }
    }

    /// Terminate a relationship (permanent end)
    pub fn terminate(
        &self,
        reason: String,
        terminated_at: DateTime<Utc>,
        terminated_by: Uuid,
    ) -> Result<RelationshipState, StateError> {
        match self {
            RelationshipState::Active { .. }
            | RelationshipState::Modified { .. }
            | RelationshipState::Suspended { .. } => Ok(RelationshipState::Terminated {
                reason,
                terminated_at,
                terminated_by,
            }),
            RelationshipState::Terminated { .. } => Err(StateError::TerminalState(
                "Relationship already terminated".to_string(),
            )),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "terminate".to_string(),
                reason: "Can only terminate Active, Modified, or Suspended relationships"
                    .to_string(),
            }),
        }
    }

    /// Archive a terminated relationship (terminal state)
    pub fn archive(
        &self,
        archived_at: DateTime<Utc>,
        archived_by: Uuid,
        retention_policy_id: Option<Uuid>,
    ) -> Result<RelationshipState, StateError> {
        match self {
            RelationshipState::Terminated { .. } => Ok(RelationshipState::Archived {
                archived_at,
                archived_by,
                retention_policy_id,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "archive".to_string(),
                reason: "Can only archive Terminated relationships".to_string(),
            }),
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            RelationshipState::Proposed { .. } => "Proposed (awaiting acceptance)",
            RelationshipState::Active { .. } => "Active (valid and usable)",
            RelationshipState::Modified { .. } => "Modified (changes pending application)",
            RelationshipState::Suspended { .. } => "Suspended (temporarily inactive)",
            RelationshipState::Terminated { .. } => "Terminated (permanently ended)",
            RelationshipState::Archived { .. } => "Archived (TERMINAL - long-term retention)",
        }
    }

    /// Get relationship metadata (if active)
    pub fn metadata(&self) -> Option<&RelationshipMetadata> {
        match self {
            RelationshipState::Active { metadata, .. } => Some(metadata),
            _ => None,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Metadata about a relationship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipMetadata {
    pub strength: RelationshipStrength,
    pub bidirectional: bool,
    pub properties: std::collections::HashMap<String, String>,
}

/// Strength of a relationship
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipStrength {
    Weak,
    Medium,
    Strong,
}

/// Description of a relationship change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipChange {
    /// Validity period changed
    ValidityChanged {
        old_from: DateTime<Utc>,
        old_until: Option<DateTime<Utc>>,
        new_from: DateTime<Utc>,
        new_until: Option<DateTime<Utc>>,
    },
    /// Relationship type changed
    TypeChanged {
        old_type: String,
        new_type: String,
    },
    /// Metadata property added
    PropertyAdded { key: String, value: String },
    /// Metadata property removed
    PropertyRemoved { key: String },
    /// Metadata property modified
    PropertyModified {
        key: String,
        old_value: String,
        new_value: String,
    },
    /// Strength changed
    StrengthChanged {
        old_strength: RelationshipStrength,
        new_strength: RelationshipStrength,
    },
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
