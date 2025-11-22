//! Location Aggregate State Machine
//!
//! This module defines the lifecycle state machine for storage locations.
//! Locations transition through 4 states from planning to archival.
//!
//! State Transitions:
//! - Planned → Active (AccessGranted)
//! - Active → Decommissioned (LocationDeactivated)
//! - Decommissioned → Archived (after assets removed)
//!
//! Invariants:
//! - Can't store keys unless Active
//! - Can't grant access unless Active
//! - Must remove all assets before archival
//! - Archived is terminal state

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state machine for storage locations
///
/// Enforces location lifecycle invariants and valid state transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LocationState {
    /// Location planned but not yet operational
    Planned {
        planned_at: DateTime<Utc>,
        planned_by: Uuid, // Person ID
        location_type: LocationType,
    },

    /// Location is active and can store assets
    Active {
        activated_at: DateTime<Utc>,
        access_grants: Vec<AccessGrant>,
        assets_stored: u64, // Count of assets (keys, certs, etc.)
        last_accessed: Option<DateTime<Utc>>,
    },

    /// Location decommissioned (no new assets, but existing assets remain)
    Decommissioned {
        reason: String,
        decommissioned_at: DateTime<Utc>,
        decommissioned_by: Uuid, // Admin who decommissioned
        remaining_assets: u64,    // Assets that need to be moved
    },

    /// Location archived (TERMINAL STATE - all assets removed)
    Archived {
        archived_at: DateTime<Utc>,
        archived_by: Uuid, // Admin who archived
        final_audit_id: Option<Uuid>,
    },
}

impl LocationState {
    // ========================================================================
    // State Query Methods
    // ========================================================================

    /// Is the location active?
    pub fn is_active(&self) -> bool {
        matches!(self, LocationState::Active { .. })
    }

    /// Can assets be stored at this location?
    pub fn can_store_assets(&self) -> bool {
        matches!(self, LocationState::Active { .. })
    }

    /// Can access be granted to this location?
    pub fn can_grant_access(&self) -> bool {
        matches!(self, LocationState::Active { .. })
    }

    /// Is this a terminal state (no further transitions allowed)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, LocationState::Archived { .. })
    }

    /// Is the location decommissioned?
    pub fn is_decommissioned(&self) -> bool {
        matches!(self, LocationState::Decommissioned { .. })
    }

    /// Can the location be modified?
    pub fn can_be_modified(&self) -> bool {
        !self.is_terminal()
    }

    /// How many assets are stored?
    pub fn asset_count(&self) -> u64 {
        match self {
            LocationState::Active { assets_stored, .. } => *assets_stored,
            LocationState::Decommissioned {
                remaining_assets, ..
            } => *remaining_assets,
            _ => 0,
        }
    }

    // ========================================================================
    // State Transition Validation
    // ========================================================================

    /// Can we transition from current state to target state?
    pub fn can_transition_to(&self, target: &LocationState) -> bool {
        match (self, target) {
            // Planned → Active (access granted)
            (LocationState::Planned { .. }, LocationState::Active { .. }) => true,

            // Active → Decommissioned
            (LocationState::Active { .. }, LocationState::Decommissioned { .. }) => true,

            // Decommissioned → Archived (must have 0 remaining assets)
            (
                LocationState::Decommissioned {
                    remaining_assets, ..
                },
                LocationState::Archived { .. },
            ) => *remaining_assets == 0,

            // All other transitions are invalid
            _ => false,
        }
    }

    // ========================================================================
    // State Transition Methods
    // ========================================================================

    /// Activate a planned location (grant initial access)
    pub fn activate(
        &self,
        activated_at: DateTime<Utc>,
        initial_access_grants: Vec<AccessGrant>,
    ) -> Result<LocationState, StateError> {
        match self {
            LocationState::Planned { .. } => {
                if initial_access_grants.is_empty() {
                    return Err(StateError::ValidationFailed(
                        "Cannot activate location without access grants".to_string(),
                    ));
                }

                Ok(LocationState::Active {
                    activated_at,
                    access_grants: initial_access_grants,
                    assets_stored: 0,
                    last_accessed: None,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "activate".to_string(),
                reason: "Can only activate from Planned state".to_string(),
            }),
        }
    }

    /// Decommission an active location (prepare for shutdown)
    pub fn decommission(
        &self,
        reason: String,
        decommissioned_at: DateTime<Utc>,
        decommissioned_by: Uuid,
    ) -> Result<LocationState, StateError> {
        match self {
            LocationState::Active { assets_stored, .. } => {
                Ok(LocationState::Decommissioned {
                    reason,
                    decommissioned_at,
                    decommissioned_by,
                    remaining_assets: *assets_stored,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "decommission".to_string(),
                reason: "Can only decommission Active locations".to_string(),
            }),
        }
    }

    /// Archive a decommissioned location (terminal state)
    pub fn archive(
        &self,
        archived_at: DateTime<Utc>,
        archived_by: Uuid,
        final_audit_id: Option<Uuid>,
    ) -> Result<LocationState, StateError> {
        match self {
            LocationState::Decommissioned {
                remaining_assets, ..
            } => {
                if *remaining_assets > 0 {
                    return Err(StateError::ValidationFailed(format!(
                        "Cannot archive location with {} remaining assets - must be removed first",
                        remaining_assets
                    )));
                }

                Ok(LocationState::Archived {
                    archived_at,
                    archived_by,
                    final_audit_id,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "archive".to_string(),
                reason: "Can only archive Decommissioned locations".to_string(),
            }),
        }
    }

    /// Grant access to an active location
    pub fn grant_access(&self, grant: AccessGrant) -> Result<LocationState, StateError> {
        match self {
            LocationState::Active {
                activated_at,
                access_grants,
                assets_stored,
                last_accessed,
            } => {
                let mut new_grants = access_grants.clone();

                // Check if person already has access
                if new_grants.iter().any(|g| g.person_id == grant.person_id) {
                    return Err(StateError::ValidationFailed(
                        "Person already has access to this location".to_string(),
                    ));
                }

                new_grants.push(grant);

                Ok(LocationState::Active {
                    activated_at: *activated_at,
                    access_grants: new_grants,
                    assets_stored: *assets_stored,
                    last_accessed: *last_accessed,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "grant_access".to_string(),
                reason: "Can only grant access to Active locations".to_string(),
            }),
        }
    }

    /// Revoke access from an active location
    pub fn revoke_access(&self, person_id: Uuid) -> Result<LocationState, StateError> {
        match self {
            LocationState::Active {
                activated_at,
                access_grants,
                assets_stored,
                last_accessed,
            } => {
                let new_grants: Vec<AccessGrant> = access_grants
                    .iter()
                    .filter(|g| g.person_id != person_id)
                    .cloned()
                    .collect();

                if new_grants.len() == access_grants.len() {
                    return Err(StateError::ValidationFailed(
                        "Person does not have access to this location".to_string(),
                    ));
                }

                Ok(LocationState::Active {
                    activated_at: *activated_at,
                    access_grants: new_grants,
                    assets_stored: *assets_stored,
                    last_accessed: *last_accessed,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "revoke_access".to_string(),
                reason: "Can only revoke access from Active locations".to_string(),
            }),
        }
    }

    /// Add an asset to an active location
    pub fn add_asset(&self) -> Result<LocationState, StateError> {
        match self {
            LocationState::Active {
                activated_at,
                access_grants,
                assets_stored,
                last_accessed,
            } => Ok(LocationState::Active {
                activated_at: *activated_at,
                access_grants: access_grants.clone(),
                assets_stored: assets_stored + 1,
                last_accessed: *last_accessed,
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "add_asset".to_string(),
                reason: "Can only add assets to Active locations".to_string(),
            }),
        }
    }

    /// Remove an asset from a location
    pub fn remove_asset(&self) -> Result<LocationState, StateError> {
        match self {
            LocationState::Active {
                activated_at,
                access_grants,
                assets_stored,
                last_accessed,
            } => {
                if *assets_stored == 0 {
                    return Err(StateError::ValidationFailed(
                        "No assets to remove from location".to_string(),
                    ));
                }

                Ok(LocationState::Active {
                    activated_at: *activated_at,
                    access_grants: access_grants.clone(),
                    assets_stored: assets_stored - 1,
                    last_accessed: *last_accessed,
                })
            }
            LocationState::Decommissioned {
                reason,
                decommissioned_at,
                decommissioned_by,
                remaining_assets,
            } => {
                if *remaining_assets == 0 {
                    return Err(StateError::ValidationFailed(
                        "No assets to remove from location".to_string(),
                    ));
                }

                Ok(LocationState::Decommissioned {
                    reason: reason.clone(),
                    decommissioned_at: *decommissioned_at,
                    decommissioned_by: *decommissioned_by,
                    remaining_assets: remaining_assets - 1,
                })
            }
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "remove_asset".to_string(),
                reason: "Can only remove assets from Active or Decommissioned locations".to_string(),
            }),
        }
    }

    /// Record access to the location
    pub fn record_access(&self, accessed_at: DateTime<Utc>) -> Result<LocationState, StateError> {
        match self {
            LocationState::Active {
                activated_at,
                access_grants,
                assets_stored,
                ..
            } => Ok(LocationState::Active {
                activated_at: *activated_at,
                access_grants: access_grants.clone(),
                assets_stored: *assets_stored,
                last_accessed: Some(accessed_at),
            }),
            _ => Err(StateError::InvalidTransition {
                current: self.description().to_string(),
                event: "record_access".to_string(),
                reason: "Can only record access for Active locations".to_string(),
            }),
        }
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get human-readable state description
    pub fn description(&self) -> &str {
        match self {
            LocationState::Planned { .. } => "Planned (not yet operational)",
            LocationState::Active { .. } => "Active (operational, can store assets)",
            LocationState::Decommissioned { .. } => {
                "Decommissioned (no new assets, existing assets need migration)"
            }
            LocationState::Archived { .. } => "Archived (TERMINAL - all assets removed)",
        }
    }

    /// Get access grants (if active)
    pub fn access_grants(&self) -> Option<&[AccessGrant]> {
        match self {
            LocationState::Active { access_grants, .. } => Some(access_grants),
            _ => None,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Type of location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LocationType {
    /// Physical location (office, data center, etc.)
    Physical,
    /// Virtual location (cloud storage, virtual machine, etc.)
    Virtual,
    /// Logical location (namespace, partition, etc.)
    Logical,
    /// Hybrid (combination of physical and virtual)
    Hybrid,
}

/// Access grant for a location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessGrant {
    pub person_id: Uuid,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Uuid, // Admin who granted access
    pub access_level: AccessLevel,
}

/// Level of access to a location
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessLevel {
    /// Read-only access (can view assets)
    ReadOnly,
    /// Read-write access (can add/remove assets)
    ReadWrite,
    /// Administrative access (can manage location and grants)
    Admin,
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
