//! NATS Account Aggregate State Machine
//!
//! This module defines the lifecycle state machine for NATS accounts.
//! Accounts transition through 5 states from creation to deletion.
//!
//! State Transitions:
//! - Created → Active (permissions set)
//! - Active → Suspended (administrative action)
//! - Suspended → Reactivated (permissions restored)
//! - Reactivated → Active (back to normal operation)
//! - Active/Suspended/Reactivated → Deleted (terminal)
//!
//! Invariants:
//! - Can't create users unless Active or Reactivated
//! - Can't publish/subscribe unless Active or Reactivated
//! - Deleted is terminal state
//! - Must have permissions to be Active

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state machine for NATS accounts
///
/// Enforces NATS account lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NatsAccountState {
    /// Account created but permissions not yet set
    Created {
        created_at: DateTime<Utc>,
        created_by: Uuid, // Person ID
        operator_id: Uuid,
    },

    /// Account is active with permissions
    Active {
        permissions: NatsPermissions,
        activated_at: DateTime<Utc>,
        users: Vec<Uuid>, // User IDs in this account
    },

    /// Account temporarily suspended
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid, // Person ID
    },

    /// Account reactivated after suspension
    Reactivated {
        permissions: NatsPermissions,
        reactivated_at: DateTime<Utc>,
        reactivated_by: Uuid, // Person ID
    },

    /// Account permanently deleted (TERMINAL STATE)
    Deleted {
        deleted_at: DateTime<Utc>,
        deleted_by: Uuid, // Person ID
        reason: String,
    },
}

impl NatsAccountState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the account active?
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            NatsAccountState::Active { .. } | NatsAccountState::Reactivated { .. }
        )
    }

    /// Can the account create users?
    pub fn can_create_users(&self) -> bool {
        matches!(
            self,
            NatsAccountState::Active { .. } | NatsAccountState::Reactivated { .. }
        )
    }

    /// Can the account publish/subscribe?
    pub fn can_pubsub(&self) -> bool {
        matches!(
            self,
            NatsAccountState::Active { .. } | NatsAccountState::Reactivated { .. }
        )
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, NatsAccountState::Deleted { .. })
    }

    /// Is the account suspended?
    pub fn is_suspended(&self) -> bool {
        matches!(self, NatsAccountState::Suspended { .. })
    }

    /// Has the account been deleted?
    pub fn is_deleted(&self) -> bool {
        matches!(self, NatsAccountState::Deleted { .. })
    }

    /// Can the account be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &NatsAccountState) -> bool {
        match (self, target) {
            // Created → Active
            (NatsAccountState::Created { .. }, NatsAccountState::Active { .. }) => true,

            // Active → Suspended
            (NatsAccountState::Active { .. }, NatsAccountState::Suspended { .. }) => true,

            // Suspended → Reactivated
            (NatsAccountState::Suspended { .. }, NatsAccountState::Reactivated { .. }) => true,

            // Reactivated → Active
            (NatsAccountState::Reactivated { .. }, NatsAccountState::Active { .. }) => true,

            // Active/Suspended/Reactivated → Deleted
            (NatsAccountState::Active { .. }, NatsAccountState::Deleted { .. }) => true,
            (NatsAccountState::Suspended { .. }, NatsAccountState::Deleted { .. }) => true,
            (NatsAccountState::Reactivated { .. }, NatsAccountState::Deleted { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Activate account (set permissions)
    pub fn activate(
        &self,
        permissions: NatsPermissions,
        activated_at: DateTime<Utc>,
    ) -> Result<NatsAccountState, StateError> {
        match self {
            NatsAccountState::Created { .. } => Ok(NatsAccountState::Active {
                permissions,
                activated_at,
                users: Vec::new(),
            }),
            NatsAccountState::Reactivated { .. } => {
                // Transition from Reactivated back to Active
                Ok(NatsAccountState::Active {
                    permissions,
                    activated_at,
                    users: Vec::new(),
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "activate".to_string(),
                reason: "Can only activate from Created or Reactivated state".to_string(),
            }),
        }
    }

    /// Suspend account (temporary revocation)
    pub fn suspend(
        &self,
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid,
    ) -> Result<NatsAccountState, StateError> {
        match self {
            NatsAccountState::Active { .. } => Ok(NatsAccountState::Suspended {
                reason,
                suspended_at,
                suspended_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "suspend".to_string(),
                reason: "Can only suspend Active accounts".to_string(),
            }),
        }
    }

    /// Reactivate suspended account
    pub fn reactivate(
        &self,
        permissions: NatsPermissions,
        reactivated_at: DateTime<Utc>,
        reactivated_by: Uuid,
    ) -> Result<NatsAccountState, StateError> {
        match self {
            NatsAccountState::Suspended { .. } => Ok(NatsAccountState::Reactivated {
                permissions,
                reactivated_at,
                reactivated_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "reactivate".to_string(),
                reason: "Can only reactivate Suspended accounts".to_string(),
            }),
        }
    }

    /// Delete account (terminal state)
    pub fn delete(
        &self,
        reason: String,
        deleted_at: DateTime<Utc>,
        deleted_by: Uuid,
    ) -> Result<NatsAccountState, StateError> {
        match self {
            NatsAccountState::Active { .. }
            | NatsAccountState::Suspended { .. }
            | NatsAccountState::Reactivated { .. } => Ok(NatsAccountState::Deleted {
                deleted_at,
                deleted_by,
                reason,
            }),
            NatsAccountState::Deleted { .. } => Err(StateError::TerminalState(
                "Account already deleted".to_string(),
            )),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "delete".to_string(),
                reason: "Can only delete Active, Suspended, or Reactivated accounts".to_string(),
            }),
        }
    }

    /// Add a user to active account
    pub fn add_user(&self, user_id: Uuid) -> Result<NatsAccountState, StateError> {
        match self {
            NatsAccountState::Active {
                permissions,
                activated_at,
                users,
            } => {
                let mut new_users = users.clone();
                if !new_users.contains(&user_id) {
                    new_users.push(user_id);
                }

                Ok(NatsAccountState::Active {
                    permissions: permissions.clone(),
                    activated_at: *activated_at,
                    users: new_users,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "add_user".to_string(),
                reason: "Can only add users to Active accounts".to_string(),
            }),
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            NatsAccountState::Created { .. } => "Created (awaiting permissions)",
            NatsAccountState::Active { .. } => "Active (can create users and pub/sub)",
            NatsAccountState::Suspended { .. } => "Suspended (temporarily disabled)",
            NatsAccountState::Reactivated { .. } => "Reactivated (permissions restored)",
            NatsAccountState::Deleted { .. } => "Deleted (TERMINAL - permanently removed)",
        }
    }

    /// Get permissions (if active or reactivated)
    pub fn permissions(&self) -> Option<&NatsPermissions> {
        match self {
            NatsAccountState::Active { permissions, .. } => Some(permissions),
            NatsAccountState::Reactivated { permissions, .. } => Some(permissions),
            _ => None,
        }
    }

    /// Get users (if active)
    pub fn users(&self) -> Option<&[Uuid]> {
        match self {
            NatsAccountState::Active { users, .. } => Some(users),
            _ => None,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// NATS permissions for accounts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsPermissions {
    pub publish: Vec<String>,    // Subjects allowed to publish to
    pub subscribe: Vec<String>,   // Subjects allowed to subscribe to
    pub allow_responses: bool,    // Can send response messages
    pub max_connections: Option<u32>,
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
