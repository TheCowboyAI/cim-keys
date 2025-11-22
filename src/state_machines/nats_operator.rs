//! NATS Operator Aggregate State Machine
//!
//! This module defines the lifecycle state machine for NATS operators.
//! Operators transition through 5 states from creation to revocation.
//!
//! State Transitions:
//! - Created → KeysGenerated (signing keys generated)
//! - KeysGenerated → Active (operator JWT signed)
//! - Active → Suspended (administrative action)
//! - Suspended → Active (reactivated)
//! - Active/Suspended → Revoked (terminal)
//!
//! Invariants:
//! - Can't create accounts unless Active
//! - Can't sign JWTs unless Active
//! - Revoked is terminal state
//! - Must have signing keys to be Active

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state machine for NATS operators
///
/// Enforces NATS operator lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NatsOperatorState {
    /// Operator created but signing keys not yet generated
    Created {
        created_at: DateTime<Utc>,
        created_by: Uuid, // Person ID
        operator_name: String,
    },

    /// Signing keys generated for operator
    KeysGenerated {
        signing_key_id: Uuid,
        public_key: String,
        generated_at: DateTime<Utc>,
    },

    /// Operator is active and can sign account JWTs
    Active {
        activated_at: DateTime<Utc>,
        jwt_issued_at: DateTime<Utc>,
        accounts: Vec<Uuid>, // Account IDs under this operator
    },

    /// Operator temporarily suspended
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid, // Person ID
    },

    /// Operator permanently revoked (TERMINAL STATE)
    Revoked {
        revoked_at: DateTime<Utc>,
        revoked_by: Uuid, // Person ID
        reason: String,
        successor_operator_id: Option<Uuid>,
    },
}

impl NatsOperatorState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the operator active?
    pub fn is_active(&self) -> bool {
        matches!(self, NatsOperatorState::Active { .. })
    }

    /// Can the operator create accounts?
    pub fn can_create_accounts(&self) -> bool {
        matches!(self, NatsOperatorState::Active { .. })
    }

    /// Can the operator sign JWTs?
    pub fn can_sign_jwts(&self) -> bool {
        matches!(self, NatsOperatorState::Active { .. })
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, NatsOperatorState::Revoked { .. })
    }

    /// Is the operator suspended?
    pub fn is_suspended(&self) -> bool {
        matches!(self, NatsOperatorState::Suspended { .. })
    }

    /// Has the operator been revoked?
    pub fn is_revoked(&self) -> bool {
        matches!(self, NatsOperatorState::Revoked { .. })
    }

    /// Can the operator be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &NatsOperatorState) -> bool {
        match (self, target) {
            // Created → KeysGenerated
            (NatsOperatorState::Created { .. }, NatsOperatorState::KeysGenerated { .. }) => true,

            // KeysGenerated → Active
            (NatsOperatorState::KeysGenerated { .. }, NatsOperatorState::Active { .. }) => true,

            // Active → Suspended
            (NatsOperatorState::Active { .. }, NatsOperatorState::Suspended { .. }) => true,

            // Suspended → Active
            (NatsOperatorState::Suspended { .. }, NatsOperatorState::Active { .. }) => true,

            // Active/Suspended → Revoked
            (NatsOperatorState::Active { .. }, NatsOperatorState::Revoked { .. }) => true,
            (NatsOperatorState::Suspended { .. }, NatsOperatorState::Revoked { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Generate signing keys for operator
    pub fn generate_keys(
        &self,
        signing_key_id: Uuid,
        public_key: String,
        generated_at: DateTime<Utc>,
    ) -> Result<NatsOperatorState, StateError> {
        match self {
            NatsOperatorState::Created { .. } => {
                if public_key.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Public key cannot be empty".to_string(),
                    ));
                }

                Ok(NatsOperatorState::KeysGenerated {
                    signing_key_id,
                    public_key,
                    generated_at,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "generate_keys".to_string(),
                reason: "Can only generate keys from Created state".to_string(),
            }),
        }
    }

    /// Activate operator (sign JWT)
    pub fn activate(
        &self,
        activated_at: DateTime<Utc>,
        jwt_issued_at: DateTime<Utc>,
    ) -> Result<NatsOperatorState, StateError> {
        match self {
            NatsOperatorState::KeysGenerated { .. } => Ok(NatsOperatorState::Active {
                activated_at,
                jwt_issued_at,
                accounts: Vec::new(),
            }),
            NatsOperatorState::Suspended { .. } => {
                // Reactivation from suspension
                Ok(NatsOperatorState::Active {
                    activated_at,
                    jwt_issued_at,
                    accounts: Vec::new(),
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "activate".to_string(),
                reason: "Can only activate from KeysGenerated or Suspended state".to_string(),
            }),
        }
    }

    /// Suspend operator (temporary revocation)
    pub fn suspend(
        &self,
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid,
    ) -> Result<NatsOperatorState, StateError> {
        match self {
            NatsOperatorState::Active { .. } => Ok(NatsOperatorState::Suspended {
                reason,
                suspended_at,
                suspended_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "suspend".to_string(),
                reason: "Can only suspend Active operators".to_string(),
            }),
        }
    }

    /// Revoke operator (terminal state)
    pub fn revoke(
        &self,
        reason: String,
        revoked_at: DateTime<Utc>,
        revoked_by: Uuid,
        successor_operator_id: Option<Uuid>,
    ) -> Result<NatsOperatorState, StateError> {
        match self {
            NatsOperatorState::Active { .. } | NatsOperatorState::Suspended { .. } => {
                Ok(NatsOperatorState::Revoked {
                    revoked_at,
                    revoked_by,
                    reason,
                    successor_operator_id,
                })
            }
            NatsOperatorState::Revoked { .. } => Err(StateError::TerminalState(
                "Operator already revoked".to_string(),
            )),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "revoke".to_string(),
                reason: "Can only revoke Active or Suspended operators".to_string(),
            }),
        }
    }

    /// Add an account to active operator
    pub fn add_account(&self, account_id: Uuid) -> Result<NatsOperatorState, StateError> {
        match self {
            NatsOperatorState::Active {
                activated_at,
                jwt_issued_at,
                accounts,
            } => {
                let mut new_accounts = accounts.clone();
                if !new_accounts.contains(&account_id) {
                    new_accounts.push(account_id);
                }

                Ok(NatsOperatorState::Active {
                    activated_at: *activated_at,
                    jwt_issued_at: *jwt_issued_at,
                    accounts: new_accounts,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "add_account".to_string(),
                reason: "Can only add accounts to Active operators".to_string(),
            }),
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            NatsOperatorState::Created { .. } => "Created (awaiting key generation)",
            NatsOperatorState::KeysGenerated { .. } => "Keys Generated (ready for activation)",
            NatsOperatorState::Active { .. } => "Active (can sign account JWTs)",
            NatsOperatorState::Suspended { .. } => "Suspended (temporarily disabled)",
            NatsOperatorState::Revoked { .. } => "Revoked (TERMINAL - permanently disabled)",
        }
    }

    /// Get signing key ID (if keys generated or active)
    pub fn signing_key_id(&self) -> Option<&Uuid> {
        match self {
            NatsOperatorState::KeysGenerated { signing_key_id, .. } => Some(signing_key_id),
            _ => None,
        }
    }

    /// Get accounts (if active)
    pub fn accounts(&self) -> Option<&[Uuid]> {
        match self {
            NatsOperatorState::Active { accounts, .. } => Some(accounts),
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
