//! Person Aggregate State Machine
//!
//! This module defines the lifecycle state machine for person identities.
//! Persons transition through 5 states from creation to archival.
//!
//! State Transitions:
//! - Created → Active (RoleAssigned)
//! - Active ↔ Suspended (bidirectional administrative action)
//! - Suspended → Deactivated (PersonDeactivated)
//! - Deactivated → Archived (after retention period)
//!
//! Invariants:
//! - Can't assign roles unless Active
//! - Can't generate keys unless Active
//! - Can't establish relationships if Deactivated
//! - Archived is terminal state

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state machine for person identities
///
/// Enforces person lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PersonState {
    /// Person created but not yet assigned any roles
    Created {
        created_at: DateTime<Utc>,
        created_by: Uuid, // Person or System ID
    },

    /// Person is active with assigned roles and permissions
    Active {
        roles: Vec<Uuid>,             // Role IDs assigned to this person
        activated_at: DateTime<Utc>,
        last_activity: Option<DateTime<Utc>>,
    },

    /// Person temporarily suspended (access revoked but can be restored)
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid, // Admin who suspended
        previous_roles: Vec<Uuid>, // Roles to restore on reactivation
    },

    /// Person deactivated (employment ended, access permanently revoked)
    Deactivated {
        reason: String,
        deactivated_at: DateTime<Utc>,
        deactivated_by: Uuid, // Admin who deactivated
    },

    /// Person archived for long-term retention (TERMINAL STATE)
    Archived {
        archived_at: DateTime<Utc>,
        archived_by: Uuid, // Admin who archived
        retention_policy_id: Option<Uuid>,
    },
}

impl PersonState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the person active?
    pub fn is_active(&self) -> bool {
        matches!(self, PersonState::Active { .. })
    }

    /// Can the person perform actions (sign, encrypt, access systems)?
    pub fn can_perform_actions(&self) -> bool {
        matches!(self, PersonState::Active { .. })
    }

    /// Can roles be assigned to this person?
    pub fn can_assign_roles(&self) -> bool {
        matches!(
            self,
            PersonState::Created { .. } | PersonState::Active { .. }
        )
    }

    /// Can keys be generated for this person?
    pub fn can_generate_keys(&self) -> bool {
        matches!(self, PersonState::Active { .. })
    }

    /// Can relationships be established with this person?
    pub fn can_establish_relationships(&self) -> bool {
        !matches!(
            self,
            PersonState::Deactivated { .. } | PersonState::Archived { .. }
        )
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, PersonState::Archived { .. })
    }

    /// Is the person suspended?
    pub fn is_suspended(&self) -> bool {
        matches!(self, PersonState::Suspended { .. })
    }

    /// Is the person deactivated?
    pub fn is_deactivated(&self) -> bool {
        matches!(self, PersonState::Deactivated { .. })
    }

    /// Can the person be modified (roles, permissions)?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &PersonState) -> bool {
        match (self, target) {
            // Created → Active (role assignment)
            (PersonState::Created { .. }, PersonState::Active { .. }) => true,

            // Active → Suspended
            (PersonState::Active { .. }, PersonState::Suspended { .. }) => true,

            // Suspended → Active (reactivation)
            (PersonState::Suspended { .. }, PersonState::Active { .. }) => true,

            // Suspended → Deactivated
            (PersonState::Suspended { .. }, PersonState::Deactivated { .. }) => true,

            // Active → Deactivated
            (PersonState::Active { .. }, PersonState::Deactivated { .. }) => true,

            // Deactivated → Archived
            (PersonState::Deactivated { .. }, PersonState::Archived { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Activate a person by assigning roles
    pub fn activate(
        &self,
        roles: Vec<Uuid>,
        activated_at: DateTime<Utc>,
    ) -> Result<PersonState, StateError> {
        match self {
            PersonState::Created { .. } => {
                if roles.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Cannot activate person without roles".to_string(),
                    ));
                }

                Ok(PersonState::Active {
                    roles,
                    activated_at,
                    last_activity: None,
                })
            }
            PersonState::Suspended {
                previous_roles, ..
            } => {
                // Reactivation from suspension
                let roles_to_restore = if !roles.is_empty() {
                    roles // Use new roles if provided
                } else {
                    previous_roles.clone() // Restore previous roles
                };

                Ok(PersonState::Active {
                    roles: roles_to_restore,
                    activated_at,
                    last_activity: None,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "activate".to_string(),
                reason: "Can only activate from Created or Suspended state".to_string(),
            }),
        }
    }

    /// Suspend a person (temporary access revocation)
    pub fn suspend(
        &self,
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid,
    ) -> Result<PersonState, StateError> {
        match self {
            PersonState::Active { roles, .. } => Ok(PersonState::Suspended {
                reason,
                suspended_at,
                suspended_by,
                previous_roles: roles.clone(),
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "suspend".to_string(),
                reason: "Can only suspend Active persons".to_string(),
            }),
        }
    }

    /// Deactivate a person (permanent access revocation)
    pub fn deactivate(
        &self,
        reason: String,
        deactivated_at: DateTime<Utc>,
        deactivated_by: Uuid,
    ) -> Result<PersonState, StateError> {
        match self {
            PersonState::Active { .. } | PersonState::Suspended { .. } => {
                Ok(PersonState::Deactivated {
                    reason,
                    deactivated_at,
                    deactivated_by,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "deactivate".to_string(),
                reason: "Can only deactivate Active or Suspended persons".to_string(),
            }),
        }
    }

    /// Archive a person (terminal state for long-term retention)
    pub fn archive(
        &self,
        archived_at: DateTime<Utc>,
        archived_by: Uuid,
        retention_policy_id: Option<Uuid>,
    ) -> Result<PersonState, StateError> {
        match self {
            PersonState::Deactivated { .. } => Ok(PersonState::Archived {
                archived_at,
                archived_by,
                retention_policy_id,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "archive".to_string(),
                reason: "Can only archive Deactivated persons".to_string(),
            }),
        }
    }

    /// Record activity for an active person
    pub fn record_activity(&self, activity_at: DateTime<Utc>) -> Result<PersonState, StateError> {
        match self {
            PersonState::Active {
                roles,
                activated_at,
                ..
            } => Ok(PersonState::Active {
                roles: roles.clone(),
                activated_at: *activated_at,
                last_activity: Some(activity_at),
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "record_activity".to_string(),
                reason: "Can only record activity for Active persons".to_string(),
            }),
        }
    }

    /// Update roles for an active person
    pub fn update_roles(&self, new_roles: Vec<Uuid>) -> Result<PersonState, StateError> {
        match self {
            PersonState::Active {
                activated_at,
                last_activity,
                ..
            } => {
                if new_roles.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Cannot remove all roles - use suspend or deactivate instead".to_string(),
                    ));
                }

                Ok(PersonState::Active {
                    roles: new_roles,
                    activated_at: *activated_at,
                    last_activity: *last_activity,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "update_roles".to_string(),
                reason: "Can only update roles for Active persons".to_string(),
            }),
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            PersonState::Created { .. } => "Created (awaiting role assignment)",
            PersonState::Active { .. } => "Active (has roles and permissions)",
            PersonState::Suspended { .. } => "Suspended (temporarily revoked access)",
            PersonState::Deactivated { .. } => "Deactivated (permanently revoked access)",
            PersonState::Archived { .. } => "Archived (TERMINAL - long-term retention)",
        }
    }

    /// Get assigned roles (if active)
    pub fn roles(&self) -> Option<&[Uuid]> {
        match self {
            PersonState::Active { roles, .. } => Some(roles),
            _ => None,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

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
