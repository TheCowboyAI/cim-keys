//! YubiKey Aggregate State Machine
//!
//! This module defines the lifecycle state machine for YubiKey devices.
//! YubiKeys transition through 6 states from detection to retirement.
//!
//! State Transitions:
//! - Detected → Provisioned (PIV slots configured and keys loaded)
//! - Provisioned → Active (device in use)
//! - Active → Locked (PIN retry limit exceeded)
//! - Locked → Active (unlocked with PUK)
//! - Active → Lost (device reported lost/stolen)
//! - Active/Locked/Lost → Retired (terminal)
//!
//! Invariants:
//! - Can't use for crypto unless Active
//! - Can't provision if already Provisioned or Active
//! - Retired is terminal state
//! - Lost devices should be revoked and replaced

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Lifecycle state machine for YubiKey devices
///
/// Enforces YubiKey lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum YubiKeyState {
    /// YubiKey detected but not yet provisioned
    Detected {
        serial: String,
        firmware: String,
        detected_at: DateTime<Utc>,  // Derived from YubiKey entry UUID v7 ID - convenience field
        detected_by: Uuid, // Person ID (also UUID v7)
    },

    /// YubiKey provisioned with PIV configuration
    Provisioned {
        provisioned_at: DateTime<Utc>,  // Actual provisioning time (operation timestamp)
        provisioned_by: Uuid, // Person ID (also UUID v7)
        slots: HashMap<PivSlot, Uuid>, // Slot -> Key ID mapping (keys also UUID v7)
        pin_changed: bool,
        puk_changed: bool,
    },

    /// YubiKey is active and in use
    Active {
        assigned_to: Uuid, // Person ID
        activated_at: DateTime<Utc>,
        last_used: Option<DateTime<Utc>>,
        usage_count: u64,
    },

    /// YubiKey is locked (PIN retry limit exceeded)
    Locked {
        locked_at: DateTime<Utc>,
        pin_retries: u8,
        can_unlock: bool, // True if PUK hasn't been exhausted
    },

    /// YubiKey reported as lost or stolen
    Lost {
        reported_at: DateTime<Utc>,
        reported_by: Uuid, // Person ID
        last_known_location: Option<Uuid>, // Location ID
    },

    /// YubiKey retired from service (TERMINAL STATE)
    Retired {
        retired_at: DateTime<Utc>,
        retired_by: Uuid, // Person ID
        reason: RetirementReason,
        replacement_yubikey_id: Option<Uuid>,
    },
}

impl YubiKeyState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the YubiKey active?
    pub fn is_active(&self) -> bool {
        matches!(self, YubiKeyState::Active { .. })
    }

    /// Can the YubiKey be used for cryptographic operations?
    pub fn can_use_for_crypto(&self) -> bool {
        matches!(self, YubiKeyState::Active { .. })
    }

    /// Is the YubiKey locked?
    pub fn is_locked(&self) -> bool {
        matches!(self, YubiKeyState::Locked { .. })
    }

    /// Has the YubiKey been reported lost?
    pub fn is_lost(&self) -> bool {
        matches!(self, YubiKeyState::Lost { .. })
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, YubiKeyState::Retired { .. })
    }

    /// Has the YubiKey been provisioned?
    pub fn is_provisioned(&self) -> bool {
        matches!(
            self,
            YubiKeyState::Provisioned { .. } | YubiKeyState::Active { .. }
        )
    }

    /// Can the YubiKey be unlocked?
    pub fn can_unlock(&self) -> bool {
        matches!(
            self,
            YubiKeyState::Locked {
                can_unlock: true,
                ..
            }
        )
    }

    /// Can the YubiKey be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &YubiKeyState) -> bool {
        match (self, target) {
            // Detected → Provisioned
            (YubiKeyState::Detected { .. }, YubiKeyState::Provisioned { .. }) => true,

            // Provisioned → Active
            (YubiKeyState::Provisioned { .. }, YubiKeyState::Active { .. }) => true,

            // Active → Locked
            (YubiKeyState::Active { .. }, YubiKeyState::Locked { .. }) => true,

            // Locked → Active (unlocked with PUK)
            (
                YubiKeyState::Locked {
                    can_unlock: true, ..
                },
                YubiKeyState::Active { .. },
            ) => true,

            // Active → Lost
            (YubiKeyState::Active { .. }, YubiKeyState::Lost { .. }) => true,

            // Active/Locked/Lost → Retired
            (YubiKeyState::Active { .. }, YubiKeyState::Retired { .. }) => true,
            (YubiKeyState::Locked { .. }, YubiKeyState::Retired { .. }) => true,
            (YubiKeyState::Lost { .. }, YubiKeyState::Retired { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Provision YubiKey with PIV configuration
    pub fn provision(
        &self,
        provisioned_at: DateTime<Utc>,
        provisioned_by: Uuid,
        slots: HashMap<PivSlot, Uuid>,
        pin_changed: bool,
        puk_changed: bool,
    ) -> Result<YubiKeyState, StateError> {
        match self {
            YubiKeyState::Detected { .. } => {
                if slots.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Cannot provision YubiKey without slot assignments".to_string(),
                    ));
                }

                if !pin_changed || !puk_changed {
                    return Err(StateError::ValidationFailed(
                        "PIN and PUK must be changed from factory defaults".to_string(),
                    ));
                }

                Ok(YubiKeyState::Provisioned {
                    provisioned_at,
                    provisioned_by,
                    slots,
                    pin_changed,
                    puk_changed,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "provision".to_string(),
                reason: "Can only provision Detected YubiKeys".to_string(),
            }),
        }
    }

    /// Activate YubiKey (assign to person)
    pub fn activate(
        &self,
        assigned_to: Uuid,
        activated_at: DateTime<Utc>,
    ) -> Result<YubiKeyState, StateError> {
        match self {
            YubiKeyState::Provisioned { .. } => Ok(YubiKeyState::Active {
                assigned_to,
                activated_at,
                last_used: None,
                usage_count: 0,
            }),
            YubiKeyState::Locked {
                can_unlock: true, ..
            } => {
                // Unlocked with PUK
                Ok(YubiKeyState::Active {
                    assigned_to,
                    activated_at,
                    last_used: None,
                    usage_count: 0,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "activate".to_string(),
                reason: "Can only activate from Provisioned or unlockable Locked state".to_string(),
            }),
        }
    }

    /// Lock YubiKey (PIN retry limit exceeded)
    pub fn lock(
        &self,
        locked_at: DateTime<Utc>,
        pin_retries: u8,
        can_unlock: bool,
    ) -> Result<YubiKeyState, StateError> {
        match self {
            YubiKeyState::Active { .. } => Ok(YubiKeyState::Locked {
                locked_at,
                pin_retries,
                can_unlock,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "lock".to_string(),
                reason: "Can only lock Active YubiKeys".to_string(),
            }),
        }
    }

    /// Report YubiKey as lost or stolen
    pub fn report_lost(
        &self,
        reported_at: DateTime<Utc>,
        reported_by: Uuid,
        last_known_location: Option<Uuid>,
    ) -> Result<YubiKeyState, StateError> {
        match self {
            YubiKeyState::Active { .. } => Ok(YubiKeyState::Lost {
                reported_at,
                reported_by,
                last_known_location,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "report_lost".to_string(),
                reason: "Can only report Active YubiKeys as lost".to_string(),
            }),
        }
    }

    /// Retire YubiKey from service (terminal state)
    pub fn retire(
        &self,
        reason: RetirementReason,
        retired_at: DateTime<Utc>,
        retired_by: Uuid,
        replacement_yubikey_id: Option<Uuid>,
    ) -> Result<YubiKeyState, StateError> {
        match self {
            YubiKeyState::Active { .. }
            | YubiKeyState::Locked { .. }
            | YubiKeyState::Lost { .. } => Ok(YubiKeyState::Retired {
                retired_at,
                retired_by,
                reason,
                replacement_yubikey_id,
            }),
            YubiKeyState::Retired { .. } => Err(StateError::TerminalState(
                "YubiKey already retired".to_string(),
            )),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "retire".to_string(),
                reason: "Can only retire Active, Locked, or Lost YubiKeys".to_string(),
            }),
        }
    }

    /// Record usage for active YubiKey
    pub fn record_usage(&self, used_at: DateTime<Utc>) -> Result<YubiKeyState, StateError> {
        match self {
            YubiKeyState::Active {
                assigned_to,
                activated_at,
                usage_count,
                ..
            } => Ok(YubiKeyState::Active {
                assigned_to: *assigned_to,
                activated_at: *activated_at,
                last_used: Some(used_at),
                usage_count: usage_count + 1,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "record_usage".to_string(),
                reason: "Can only record usage for Active YubiKeys".to_string(),
            }),
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            YubiKeyState::Detected { .. } => "Detected (awaiting provisioning)",
            YubiKeyState::Provisioned { .. } => "Provisioned (ready for assignment)",
            YubiKeyState::Active { .. } => "Active (in use for crypto operations)",
            YubiKeyState::Locked { .. } => "Locked (PIN retry limit exceeded)",
            YubiKeyState::Lost { .. } => "Lost (reported lost/stolen)",
            YubiKeyState::Retired { .. } => "Retired (TERMINAL - removed from service)",
        }
    }

    /// Get assigned person (if active)
    pub fn assigned_to(&self) -> Option<&Uuid> {
        match self {
            YubiKeyState::Active { assigned_to, .. } => Some(assigned_to),
            _ => None,
        }
    }

    /// Get provisioned slots (if provisioned or active)
    pub fn slots(&self) -> Option<&HashMap<PivSlot, Uuid>> {
        match self {
            YubiKeyState::Provisioned { slots, .. } => Some(slots),
            _ => None,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// PIV slot on YubiKey
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PivSlot {
    /// 9a - PIV Authentication
    Authentication,
    /// 9c - Digital Signature
    Signature,
    /// 9d - Key Management
    KeyManagement,
    /// 9e - Card Authentication
    CardAuth,
    /// 82-95 - Retired slots
    Retired(u8),
}

/// Reason for YubiKey retirement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RetirementReason {
    /// Device lost or stolen
    LostOrStolen,
    /// Device damaged or malfunctioning
    Damaged,
    /// Firmware outdated or vulnerable
    FirmwareOutdated,
    /// Person left organization
    EmployeeTermination,
    /// Upgraded to newer device
    Upgraded,
    /// Administrative decision
    Administrative { reason: String },
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
