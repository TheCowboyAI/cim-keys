//! NATS User Aggregate State Machine
//!
//! This module defines the lifecycle state machine for NATS users.
//! Users transition through 5 states from creation to deletion.
//!
//! State Transitions:
//! - Created → Active (permissions set)
//! - Active → Suspended (administrative action)
//! - Suspended → Reactivated (permissions restored)
//! - Reactivated → Active (back to normal operation)
//! - Active/Suspended/Reactivated → Deleted (terminal)
//!
//! Invariants:
//! - Can't publish/subscribe unless Active or Reactivated
//! - Deleted is terminal state
//! - Must have permissions to be Active
//! - Must belong to an account

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state machine for NATS users
///
/// Enforces NATS user lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NatsUserState {
    /// User created but permissions not yet set
    Created {
        created_by: Uuid, // Person ID
        account_id: Uuid,
        person_id: Uuid, // Person this NATS user represents
    },

    /// User is active with permissions
    Active {
        permissions: NatsUserPermissions,
        activated_at: DateTime<Utc>,
        last_connection: Option<DateTime<Utc>>,
    },

    /// User temporarily suspended
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid, // Person ID
    },

    /// User reactivated after suspension
    Reactivated {
        permissions: NatsUserPermissions,
        reactivated_at: DateTime<Utc>,
        reactivated_by: Uuid, // Person ID
    },

    /// User permanently deleted (TERMINAL STATE)
    Deleted {
        deleted_at: DateTime<Utc>,
        deleted_by: Uuid, // Person ID
        reason: String,
    },
}

impl NatsUserState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the user active?
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            NatsUserState::Active { .. } | NatsUserState::Reactivated { .. }
        )
    }

    /// Can the user connect to NATS?
    pub fn can_connect(&self) -> bool {
        matches!(
            self,
            NatsUserState::Active { .. } | NatsUserState::Reactivated { .. }
        )
    }

    /// Can the user publish/subscribe?
    pub fn can_pubsub(&self) -> bool {
        matches!(
            self,
            NatsUserState::Active { .. } | NatsUserState::Reactivated { .. }
        )
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, NatsUserState::Deleted { .. })
    }

    /// Is the user suspended?
    pub fn is_suspended(&self) -> bool {
        matches!(self, NatsUserState::Suspended { .. })
    }

    /// Has the user been deleted?
    pub fn is_deleted(&self) -> bool {
        matches!(self, NatsUserState::Deleted { .. })
    }

    /// Can the user be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &NatsUserState) -> bool {
        match (self, target) {
            // Created → Active
            (NatsUserState::Created { .. }, NatsUserState::Active { .. }) => true,

            // Active → Suspended
            (NatsUserState::Active { .. }, NatsUserState::Suspended { .. }) => true,

            // Suspended → Reactivated
            (NatsUserState::Suspended { .. }, NatsUserState::Reactivated { .. }) => true,

            // Reactivated → Active
            (NatsUserState::Reactivated { .. }, NatsUserState::Active { .. }) => true,

            // Active/Suspended/Reactivated → Deleted
            (NatsUserState::Active { .. }, NatsUserState::Deleted { .. }) => true,
            (NatsUserState::Suspended { .. }, NatsUserState::Deleted { .. }) => true,
            (NatsUserState::Reactivated { .. }, NatsUserState::Deleted { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Activate user (set permissions)
    pub fn activate(
        &self,
        permissions: NatsUserPermissions,
        activated_at: DateTime<Utc>,
    ) -> Result<NatsUserState, StateError> {
        match self {
            NatsUserState::Created { .. } => Ok(NatsUserState::Active {
                permissions,
                activated_at,
                last_connection: None,
            }),
            NatsUserState::Reactivated { .. } => {
                // Transition from Reactivated back to Active
                Ok(NatsUserState::Active {
                    permissions,
                    activated_at,
                    last_connection: None,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "activate".to_string(),
                reason: "Can only activate from Created or Reactivated state".to_string(),
            }),
        }
    }

    /// Suspend user (temporary revocation)
    pub fn suspend(
        &self,
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid,
    ) -> Result<NatsUserState, StateError> {
        match self {
            NatsUserState::Active { .. } => Ok(NatsUserState::Suspended {
                reason,
                suspended_at,
                suspended_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "suspend".to_string(),
                reason: "Can only suspend Active users".to_string(),
            }),
        }
    }

    /// Reactivate suspended user
    pub fn reactivate(
        &self,
        permissions: NatsUserPermissions,
        reactivated_at: DateTime<Utc>,
        reactivated_by: Uuid,
    ) -> Result<NatsUserState, StateError> {
        match self {
            NatsUserState::Suspended { .. } => Ok(NatsUserState::Reactivated {
                permissions,
                reactivated_at,
                reactivated_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "reactivate".to_string(),
                reason: "Can only reactivate Suspended users".to_string(),
            }),
        }
    }

    /// Delete user (terminal state)
    pub fn delete(
        &self,
        reason: String,
        deleted_at: DateTime<Utc>,
        deleted_by: Uuid,
    ) -> Result<NatsUserState, StateError> {
        match self {
            NatsUserState::Active { .. }
            | NatsUserState::Suspended { .. }
            | NatsUserState::Reactivated { .. } => Ok(NatsUserState::Deleted {
                deleted_at,
                deleted_by,
                reason,
            }),
            NatsUserState::Deleted { .. } => Err(StateError::TerminalState(
                "User already deleted".to_string(),
            )),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "delete".to_string(),
                reason: "Can only delete Active, Suspended, or Reactivated users".to_string(),
            }),
        }
    }

    /// Record connection for active user
    pub fn record_connection(
        &self,
        connection_time: DateTime<Utc>,
    ) -> Result<NatsUserState, StateError> {
        match self {
            NatsUserState::Active {
                permissions,
                activated_at,
                ..
            } => Ok(NatsUserState::Active {
                permissions: permissions.clone(),
                activated_at: *activated_at,
                last_connection: Some(connection_time),
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "record_connection".to_string(),
                reason: "Can only record connections for Active users".to_string(),
            }),
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            NatsUserState::Created { .. } => "Created (awaiting permissions)",
            NatsUserState::Active { .. } => "Active (can connect and pub/sub)",
            NatsUserState::Suspended { .. } => "Suspended (temporarily disabled)",
            NatsUserState::Reactivated { .. } => "Reactivated (permissions restored)",
            NatsUserState::Deleted { .. } => "Deleted (TERMINAL - permanently removed)",
        }
    }

    /// Get permissions (if active or reactivated)
    pub fn permissions(&self) -> Option<&NatsUserPermissions> {
        match self {
            NatsUserState::Active { permissions, .. } => Some(permissions),
            NatsUserState::Reactivated { permissions, .. } => Some(permissions),
            _ => None,
        }
    }

    /// Get last connection time (if active and has connected)
    pub fn last_connection(&self) -> Option<&DateTime<Utc>> {
        match self {
            NatsUserState::Active {
                last_connection, ..
            } => last_connection.as_ref(),
            _ => None,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// NATS permissions for users (subset of account permissions)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsUserPermissions {
    pub publish: Vec<String>,    // Subjects allowed to publish to
    pub subscribe: Vec<String>,   // Subjects allowed to subscribe to
    pub allow_responses: bool,    // Can send response messages
    pub max_payload: Option<u64>, // Maximum message payload size in bytes
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
