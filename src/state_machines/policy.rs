//! Policy Aggregate State Machine
//!
//! This module defines the lifecycle state machine for authorization policies.
//! Policies transition through 5 states from draft to revocation.
//!
//! State Transitions:
//! - Draft → Active (PolicyCreated with valid claims)
//! - Active → Modified (PolicyUpdated)
//! - Modified → Active (changes applied)
//! - Active ↔ Suspended (administrative action)
//! - Any → Revoked (PolicyRevoked - terminal)
//!
//! Invariants:
//! - Can't enforce policy unless Active
//! - Can't modify if Revoked (terminal state)
//! - Suspended policies don't grant permissions
//! - Must have at least one claim to be Active
//! - Conditions must be valid before activation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state machine for authorization policies
///
/// Enforces policy lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyState {
    /// Policy created but not yet activated (under review)
    Draft {
        author_id: Uuid, // Person who created the policy
        review_status: ReviewStatus,
    },

    /// Policy is active and enforced
    Active {
        activated_at: DateTime<Utc>,
        claims: Vec<PolicyClaim>,
        conditions: Vec<PolicyCondition>,
        enforcement_count: u64,
        last_enforced: Option<DateTime<Utc>>,
    },

    /// Policy has been modified and awaiting activation
    Modified {
        modified_at: DateTime<Utc>,
        modified_by: Uuid, // Person who modified the policy
        previous_version: Uuid,
        changes: Vec<PolicyChange>,
    },

    /// Policy temporarily suspended (not enforced)
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid, // Person who suspended the policy
    },

    /// Policy permanently revoked (TERMINAL STATE)
    Revoked {
        reason: String,
        revoked_at: DateTime<Utc>,
        revoked_by: Uuid, // Person who revoked the policy
        replacement_policy: Option<Uuid>,
    },
}

impl PolicyState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the policy active and enforced?
    pub fn is_active(&self) -> bool {
        matches!(self, PolicyState::Active { .. })
    }

    /// Can the policy be enforced for authorization decisions?
    pub fn can_enforce(&self) -> bool {
        matches!(self, PolicyState::Active { .. })
    }

    /// Can the policy be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, PolicyState::Revoked { .. })
    }

    /// Is the policy suspended?
    pub fn is_suspended(&self) -> bool {
        matches!(self, PolicyState::Suspended { .. })
    }

    /// Is the policy in draft state?
    pub fn is_draft(&self) -> bool {
        matches!(self, PolicyState::Draft { .. })
    }

    /// Is the policy modified and awaiting activation?
    pub fn is_modified(&self) -> bool {
        matches!(self, PolicyState::Modified { .. })
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &PolicyState) -> bool {
        match (self, target) {
            // Draft → Active
            (PolicyState::Draft { .. }, PolicyState::Active { .. }) => true,

            // Active → Modified
            (PolicyState::Active { .. }, PolicyState::Modified { .. }) => true,

            // Modified → Active
            (PolicyState::Modified { .. }, PolicyState::Active { .. }) => true,

            // Active → Suspended
            (PolicyState::Active { .. }, PolicyState::Suspended { .. }) => true,

            // Suspended → Active (reactivation)
            (PolicyState::Suspended { .. }, PolicyState::Active { .. }) => true,

            // Any non-terminal → Revoked
            (_, PolicyState::Revoked { .. }) if !self.is_terminal() => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    /// Validate that a policy change is allowed in the current state
    pub fn validate_modification(&self) -> Result<(), StateError> {
        if self.is_terminal() {
            return Err(StateError::TerminalState(
                "Cannot modify a revoked policy".to_string(),
            ));
        }

        match self {
            PolicyState::Draft { .. } => Ok(()),
            PolicyState::Active { .. } => Ok(()),
            PolicyState::Modified { .. } => Ok(()),
            PolicyState::Suspended { .. } => {
                Err(StateError::ValidationFailed(
                    "Cannot modify a suspended policy - reactivate first".to_string(),
                ))
            }
            PolicyState::Revoked { .. } => Err(StateError::TerminalState(
                "Cannot modify a revoked policy".to_string(),
            )),
        }
    }

    /// Validate that claims are present and well-formed
    pub fn validate_claims(claims: &[PolicyClaim]) -> Result<(), StateError> {
        if claims.is_empty() {
            return Err(StateError::ValidationFailed(
                "Policy must have at least one claim".to_string(),
            ));
        }

        // Validate each claim
        for claim in claims {
            claim.validate()?;
        }

        Ok(())
    }

    /// Validate that conditions are well-formed
    pub fn validate_conditions(conditions: &[PolicyCondition]) -> Result<(), StateError> {
        for condition in conditions {
            condition.validate()?;
        }

        Ok(())
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Transition from Draft to Active
    pub fn activate(
        &self,
        activated_at: DateTime<Utc>,
        claims: Vec<PolicyClaim>,
        conditions: Vec<PolicyCondition>,
    ) -> Result<PolicyState, StateError> {
        match self {
            PolicyState::Draft { .. } => {
                // Validate claims and conditions
                Self::validate_claims(&claims)?;
                Self::validate_conditions(&conditions)?;

                Ok(PolicyState::Active {
                    activated_at,
                    claims,
                    conditions,
                    enforcement_count: 0,
                    last_enforced: None,
                })
            }
            PolicyState::Modified { .. } => {
                // Reactivating after modification
                Self::validate_claims(&claims)?;
                Self::validate_conditions(&conditions)?;

                Ok(PolicyState::Active {
                    activated_at,
                    claims,
                    conditions,
                    enforcement_count: 0,
                    last_enforced: None,
                })
            }
            PolicyState::Suspended { .. } => {
                // Reactivating from suspension
                Self::validate_claims(&claims)?;
                Self::validate_conditions(&conditions)?;

                Ok(PolicyState::Active {
                    activated_at,
                    claims,
                    conditions,
                    enforcement_count: 0,
                    last_enforced: None,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "activate".to_string(),
                reason: "Can only activate from Draft, Modified, or Suspended state".to_string(),
            }),
        }
    }

    /// Record policy enforcement
    pub fn record_enforcement(&self, enforced_at: DateTime<Utc>) -> Result<PolicyState, StateError> {
        match self {
            PolicyState::Active {
                activated_at,
                claims,
                conditions,
                enforcement_count,
                ..
            } => Ok(PolicyState::Active {
                activated_at: *activated_at,
                claims: claims.clone(),
                conditions: conditions.clone(),
                enforcement_count: enforcement_count + 1,
                last_enforced: Some(enforced_at),
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "record_enforcement".to_string(),
                reason: "Can only enforce Active policies".to_string(),
            }),
        }
    }

    /// Suspend an active policy
    pub fn suspend(
        &self,
        reason: String,
        suspended_at: DateTime<Utc>,
        suspended_by: Uuid,
    ) -> Result<PolicyState, StateError> {
        match self {
            PolicyState::Active { .. } => Ok(PolicyState::Suspended {
                reason,
                suspended_at,
                suspended_by,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "suspend".to_string(),
                reason: "Can only suspend Active policies".to_string(),
            }),
        }
    }

    /// Revoke a policy (terminal)
    pub fn revoke(
        &self,
        reason: String,
        revoked_at: DateTime<Utc>,
        revoked_by: Uuid,
        replacement_policy: Option<Uuid>,
    ) -> Result<PolicyState, StateError> {
        if self.is_terminal() {
            return Err(StateError::TerminalState(
                "Policy already revoked".to_string(),
            ));
        }

        Ok(PolicyState::Revoked {
            reason,
            revoked_at,
            revoked_by,
            replacement_policy,
        })
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            PolicyState::Draft { .. } => "Draft (under review, not enforced)",
            PolicyState::Active { .. } => "Active (enforced for authorization)",
            PolicyState::Modified { .. } => "Modified (awaiting activation)",
            PolicyState::Suspended { .. } => "Suspended (temporarily not enforced)",
            PolicyState::Revoked { .. } => "Revoked (TERMINAL - permanently disabled)",
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Review status for draft policies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    /// Awaiting review
    Pending,
    /// Under review
    InReview { reviewer_id: Uuid },
    /// Approved for activation
    Approved { approver_id: Uuid },
    /// Rejected
    Rejected { reason: String },
}

/// Policy claim (permission grant)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyClaim {
    /// Resource being accessed (e.g., "key", "certificate", "yubikey")
    pub resource: String,
    /// Action being performed (e.g., "read", "write", "sign", "revoke")
    pub action: String,
    /// Optional scope limitation (e.g., "organization:123", "person:456")
    pub scope: Option<String>,
}

impl PolicyClaim {
    /// Validate that a claim is well-formed
    pub fn validate(&self) -> Result<(), StateError> {
        if self.resource.is_empty() {
            return Err(StateError::ValidationFailed(
                "Policy claim must have a resource".to_string(),
            ));
        }

        if self.action.is_empty() {
            return Err(StateError::ValidationFailed(
                "Policy claim must have an action".to_string(),
            ));
        }

        Ok(())
    }
}

/// Policy condition (contextual requirements)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyCondition {
    /// Condition type (e.g., "time_of_day", "location", "ip_address")
    pub condition_type: String,
    /// Operator (e.g., "equals", "not_equals", "in_range")
    pub operator: String,
    /// Value to compare against
    pub value: String,
}

impl PolicyCondition {
    /// Validate that a condition is well-formed
    pub fn validate(&self) -> Result<(), StateError> {
        if self.condition_type.is_empty() {
            return Err(StateError::ValidationFailed(
                "Policy condition must have a type".to_string(),
            ));
        }

        if self.operator.is_empty() {
            return Err(StateError::ValidationFailed(
                "Policy condition must have an operator".to_string(),
            ));
        }

        Ok(())
    }
}

/// Description of a policy change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyChange {
    /// Claim added
    ClaimAdded(PolicyClaim),
    /// Claim removed
    ClaimRemoved(PolicyClaim),
    /// Condition added
    ConditionAdded(PolicyCondition),
    /// Condition removed
    ConditionRemoved(PolicyCondition),
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
