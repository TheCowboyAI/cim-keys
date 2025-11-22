//! Key Aggregate State Machine
//!
//! This module defines the lifecycle state machine for cryptographic keys.
//! Keys transition through 8 states from generation to archival.
//!
//! State Transitions:
//! - Generated/Imported → Active (KeyActivated)
//! - Active → RotationPending (KeyRotationInitiated)
//! - RotationPending → Rotated (KeyRotationCompleted)
//! - Any → Revoked (KeyRevoked - terminal)
//! - Active → Expired (time-based)
//! - Rotated/Revoked/Expired → Archived (terminal)
//!
//! Invariants:
//! - Can only sign/encrypt if Active
//! - Can't rotate if already in RotationPending
//! - Revoked keys can't be reactivated
//! - Archived is terminal state

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::KeyAlgorithm;

/// Lifecycle state machine for cryptographic keys
///
/// Enforces key lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyState {
    /// Key generated but not yet activated
    Generated {
        algorithm: KeyAlgorithm,
        generated_at: DateTime<Utc>,
        generated_by: Uuid, // Person ID
    },

    /// Key imported from external source
    Imported {
        source: ImportSource,
        imported_at: DateTime<Utc>,
        imported_by: Uuid, // Person ID
    },

    /// Key is active and can be used for cryptographic operations
    Active {
        activated_at: DateTime<Utc>,
        usage_count: u64,
        last_used: Option<DateTime<Utc>>,
    },

    /// Key rotation has been initiated, new key is being generated
    RotationPending {
        new_key_id: Uuid,
        initiated_at: DateTime<Utc>,
        initiated_by: Uuid, // Person ID
    },

    /// Key has been rotated, new key is now active
    Rotated {
        new_key_id: Uuid,
        rotated_at: DateTime<Utc>,
        rotated_by: Uuid, // Person ID
    },

    /// Key has been revoked (TERMINAL STATE)
    Revoked {
        reason: RevocationReason,
        revoked_at: DateTime<Utc>,
        revoked_by: Uuid, // Person ID
    },

    /// Key has expired based on time-based policy
    Expired {
        expired_at: DateTime<Utc>,
        expiry_reason: ExpiryReason,
    },

    /// Key has been archived (TERMINAL STATE)
    Archived {
        archived_at: DateTime<Utc>,
        archived_by: Uuid, // Person ID
        previous_state: ArchivedFromState,
    },
}

impl KeyState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the key active and usable?
    pub fn is_active(&self) -> bool {
        matches!(self, KeyState::Active { .. })
    }

    /// Can the key be used for cryptographic operations?
    pub fn can_use_for_crypto(&self) -> bool {
        matches!(self, KeyState::Active { .. })
    }

    /// Can the key be modified (metadata, permissions)?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            KeyState::Revoked { .. } | KeyState::Archived { .. }
        )
    }

    /// Is key rotation in progress?
    pub fn is_rotation_pending(&self) -> bool {
        matches!(self, KeyState::RotationPending { .. })
    }

    /// Has the key been rotated?
    pub fn is_rotated(&self) -> bool {
        matches!(self, KeyState::Rotated { .. })
    }

    /// Has the key expired?
    pub fn is_expired(&self) -> bool {
        matches!(self, KeyState::Expired { .. })
    }

    /// Has the key been revoked?
    pub fn is_revoked(&self) -> bool {
        matches!(self, KeyState::Revoked { .. })
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &KeyState) -> bool {
        match (self, target) {
            // Generated → Active
            (KeyState::Generated { .. }, KeyState::Active { .. }) => true,

            // Imported → Active
            (KeyState::Imported { .. }, KeyState::Active { .. }) => true,

            // Active → RotationPending
            (KeyState::Active { .. }, KeyState::RotationPending { .. }) => true,

            // RotationPending → Rotated
            (KeyState::RotationPending { .. }, KeyState::Rotated { .. }) => true,

            // Any non-terminal → Revoked
            (_, KeyState::Revoked { .. }) if !self.is_terminal() => true,

            // Active → Expired
            (KeyState::Active { .. }, KeyState::Expired { .. }) => true,

            // Rotated → Archived
            (KeyState::Rotated { .. }, KeyState::Archived { .. }) => true,

            // Revoked → Archived
            (KeyState::Revoked { .. }, KeyState::Archived { .. }) => true,

            // Expired → Archived
            (KeyState::Expired { .. }, KeyState::Archived { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // TODO: Event validation and application will be implemented in Phase 4
    // when wiring state machines to aggregate event handlers.
    // For now, state transitions are managed through explicit transition methods.

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            KeyState::Generated { .. } => "Generated (awaiting activation)",
            KeyState::Imported { .. } => "Imported (awaiting activation)",
            KeyState::Active { .. } => "Active (usable for cryptographic operations)",
            KeyState::RotationPending { .. } => "Rotation Pending (new key being generated)",
            KeyState::Rotated { .. } => "Rotated (superseded by new key)",
            KeyState::Revoked { .. } => "Revoked (TERMINAL - cannot be reactivated)",
            KeyState::Expired { .. } => "Expired (time-based expiration)",
            KeyState::Archived { .. } => "Archived (TERMINAL - long-term storage)",
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Source of an imported key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportSource {
    /// Imported from a file
    File { path: String },
    /// Imported from a YubiKey
    YubiKey { serial: String, slot: String },
    /// Imported from another CIM
    CIM { cim_id: Uuid },
    /// Imported from external PKI
    ExternalPKI { authority: String },
}

/// Reason for key revocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RevocationReason {
    /// Key compromised or suspected compromise
    Compromised,
    /// Person left organization
    EmployeeTermination,
    /// Superseded by new key
    Superseded,
    /// No longer needed
    CessationOfOperation,
    /// Administrative revocation
    Administrative { reason: String },
}

/// Reason for key expiration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpiryReason {
    /// Time-based expiration (validity period ended)
    TimeBasedExpiry,
    /// Usage count exceeded
    UsageLimitExceeded,
    /// Policy-based expiration
    PolicyExpiration { policy_id: Uuid },
}

/// State from which key was archived
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ArchivedFromState {
    Rotated,
    Revoked,
    Expired,
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
