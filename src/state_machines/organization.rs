//! Organization Aggregate State Machine
//!
//! This module defines the lifecycle state machine for organizations.
//! Organizations transition through 4 states from draft to dissolution.
//!
//! State Transitions:
//! - Draft → Active (first unit/person added)
//! - Active → Suspended (administrative action)
//! - Suspended → Active (reactivated)
//! - Active/Suspended → Dissolved (terminal)
//!
//! Invariants:
//! - Can't add units or people unless Active
//! - Can't generate organizational keys unless Active
//! - Dissolved is terminal state
//! - Must have at least one unit or person to be Active

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state machine for organizations
///
/// Enforces organizational lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrganizationState {
    /// Organization created but not yet operational
    Draft {
        created_at: DateTime<Utc>,
        created_by: Uuid, // Person ID
    },

    /// Organization is active and operational
    Active {
        activated_at: DateTime<Utc>,
        units: Vec<Uuid>,     // Organizational unit IDs
        members: Vec<Uuid>,   // Person IDs
    },

    /// Organization temporarily suspended
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid, // Admin who suspended
    },

    /// Organization permanently dissolved (TERMINAL STATE)
    Dissolved {
        dissolved_at: DateTime<Utc>,
        dissolved_by: Uuid,   // Admin who dissolved
        reason: String,
        successor_org_id: Option<Uuid>, // If org was merged/acquired
    },
}

impl OrganizationState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the organization active?
    pub fn is_active(&self) -> bool {
        matches!(self, OrganizationState::Active { .. })
    }

    /// Can organizational units be added?
    pub fn can_add_units(&self) -> bool {
        matches!(self, OrganizationState::Active { .. })
    }

    /// Can members (people) be added?
    pub fn can_add_members(&self) -> bool {
        matches!(self, OrganizationState::Active { .. })
    }

    /// Can organizational keys be generated?
    pub fn can_generate_keys(&self) -> bool {
        matches!(self, OrganizationState::Active { .. })
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, OrganizationState::Dissolved { .. })
    }

    /// Is the organization suspended?
    pub fn is_suspended(&self) -> bool {
        matches!(self, OrganizationState::Suspended { .. })
    }

    /// Is the organization dissolved?
    pub fn is_dissolved(&self) -> bool {
        matches!(self, OrganizationState::Dissolved { .. })
    }

    /// Can the organization be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &OrganizationState) -> bool {
        match (self, target) {
            // Draft → Active (first unit/member added)
            (OrganizationState::Draft { .. }, OrganizationState::Active { .. }) => true,

            // Active → Suspended
            (OrganizationState::Active { .. }, OrganizationState::Suspended { .. }) => true,

            // Suspended → Active (reactivation)
            (OrganizationState::Suspended { .. }, OrganizationState::Active { .. }) => true,

            // Active → Dissolved
            (OrganizationState::Active { .. }, OrganizationState::Dissolved { .. }) => true,

            // Suspended → Dissolved
            (OrganizationState::Suspended { .. }, OrganizationState::Dissolved { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Activate an organization (first unit or member added)
    pub fn activate(
        &self,
        activated_at: DateTime<Utc>,
        initial_units: Vec<Uuid>,
        initial_members: Vec<Uuid>,
    ) -> Result<OrganizationState, StateError> {
        match self {
            OrganizationState::Draft { .. } => {
                if initial_units.is_empty() && initial_members.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Cannot activate organization without units or members".to_string(),
                    ));
                }

                Ok(OrganizationState::Active {
                    activated_at,
                    units: initial_units,
                    members: initial_members,
                })
            }
            OrganizationState::Suspended { .. } => {
                // Reactivation from suspension
                Ok(OrganizationState::Active {
                    activated_at,
                    units: initial_units,
                    members: initial_members,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "activate".to_string(),
                reason: "Can only activate from Draft or Suspended state".to_string(),
            }),
        }
    }

    /// Suspend an organization (temporary operational halt)
    pub fn suspend(
        &self,
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid,
    ) -> Result<OrganizationState, StateError> {
        match self {
            OrganizationState::Active { .. } => Ok(OrganizationState::Suspended {
                reason,
                suspended_at,
                suspended_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "suspend".to_string(),
                reason: "Can only suspend Active organizations".to_string(),
            }),
        }
    }

    /// Dissolve an organization (terminal state)
    pub fn dissolve(
        &self,
        reason: String,
        dissolved_at: DateTime<Utc>,
        dissolved_by: Uuid,
        successor_org_id: Option<Uuid>,
    ) -> Result<OrganizationState, StateError> {
        match self {
            OrganizationState::Active { .. } | OrganizationState::Suspended { .. } => {
                Ok(OrganizationState::Dissolved {
                    dissolved_at,
                    dissolved_by,
                    reason,
                    successor_org_id,
                })
            }
            OrganizationState::Dissolved { .. } => Err(StateError::TerminalState(
                "Organization already dissolved".to_string(),
            )),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "dissolve".to_string(),
                reason: "Can only dissolve Active or Suspended organizations".to_string(),
            }),
        }
    }

    /// Add a unit to an active organization
    pub fn add_unit(&self, unit_id: Uuid) -> Result<OrganizationState, StateError> {
        match self {
            OrganizationState::Active {
                activated_at,
                units,
                members,
            } => {
                let mut new_units = units.clone();
                if !new_units.contains(&unit_id) {
                    new_units.push(unit_id);
                }

                Ok(OrganizationState::Active {
                    activated_at: *activated_at,
                    units: new_units,
                    members: members.clone(),
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "add_unit".to_string(),
                reason: "Can only add units to Active organizations".to_string(),
            }),
        }
    }

    /// Add a member to an active organization
    pub fn add_member(&self, member_id: Uuid) -> Result<OrganizationState, StateError> {
        match self {
            OrganizationState::Active {
                activated_at,
                units,
                members,
            } => {
                let mut new_members = members.clone();
                if !new_members.contains(&member_id) {
                    new_members.push(member_id);
                }

                Ok(OrganizationState::Active {
                    activated_at: *activated_at,
                    units: units.clone(),
                    members: new_members,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "add_member".to_string(),
                reason: "Can only add members to Active organizations".to_string(),
            }),
        }
    }

    /// Remove a unit from an active organization
    pub fn remove_unit(&self, unit_id: Uuid) -> Result<OrganizationState, StateError> {
        match self {
            OrganizationState::Active {
                activated_at,
                units,
                members,
            } => {
                let new_units: Vec<Uuid> = units.iter().filter(|&id| *id != unit_id).copied().collect();

                // Must have at least one unit or member
                if new_units.is_empty() && members.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Cannot remove last unit - organization must have units or members".to_string(),
                    ));
                }

                Ok(OrganizationState::Active {
                    activated_at: *activated_at,
                    units: new_units,
                    members: members.clone(),
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "remove_unit".to_string(),
                reason: "Can only remove units from Active organizations".to_string(),
            }),
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            OrganizationState::Draft { .. } => "Draft (not yet operational)",
            OrganizationState::Active { .. } => "Active (operational with units/members)",
            OrganizationState::Suspended { .. } => "Suspended (temporarily halted)",
            OrganizationState::Dissolved { .. } => "Dissolved (TERMINAL - permanently closed)",
        }
    }

    /// Get organizational units (if active)
    pub fn units(&self) -> Option<&[Uuid]> {
        match self {
            OrganizationState::Active { units, .. } => Some(units),
            _ => None,
        }
    }

    /// Get members (if active)
    pub fn members(&self) -> Option<&[Uuid]> {
        match self {
            OrganizationState::Active { members, .. } => Some(members),
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
