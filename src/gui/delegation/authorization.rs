// Copyright (c) 2025 - Cowboy AI, LLC.

//! Delegation Authorization Bounded Context
//!
//! This module implements the Delegation domain with:
//! - Message enum for all delegation operations
//! - State struct for delegation-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **Person Selection**: Grantor (from) and grantee (to)
//! 2. **Permission Management**: Permission set toggles
//! 3. **Expiration**: Time-limited delegations
//! 4. **Lifecycle**: Create, revoke, track delegations

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use iced::Task;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::KeyPermission;

/// Entry for displaying active delegations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationEntry {
    pub id: Uuid,
    pub from_person_id: Uuid,
    pub from_person_name: String,
    pub to_person_id: Uuid,
    pub to_person_name: String,
    pub permissions: Vec<KeyPermission>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl DelegationEntry {
    /// Check if delegation has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if delegation is currently valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    /// Get days until expiration (None if no expiration)
    pub fn days_until_expiration(&self) -> Option<i64> {
        self.expires_at.map(|exp| {
            let duration = exp - Utc::now();
            duration.num_days()
        })
    }
}

/// Delegation Message
///
/// Organized by sub-domain:
/// - Section Toggle (1 message)
/// - Person Selection (2 messages)
/// - Permission Management (1 message)
/// - Expiration (1 message)
/// - Lifecycle (4 messages)
#[derive(Debug, Clone)]
pub enum DelegationMessage {
    // === Section Toggle ===
    /// Toggle delegation section visibility
    ToggleDelegationSection,

    // === Person Selection ===
    /// Select person to delegate from (grantor)
    DelegationFromPersonSelected(Uuid),
    /// Select person to delegate to (grantee)
    DelegationToPersonSelected(Uuid),

    // === Permission Management ===
    /// Toggle a specific permission in the delegation set
    ToggleDelegationPermission(KeyPermission),

    // === Expiration ===
    /// Change expiration days (empty = no expiration)
    DelegationExpiresDaysChanged(String),

    // === Lifecycle ===
    /// Create a new delegation
    CreateDelegation,
    /// Delegation creation completed
    DelegationCreated(Result<DelegationEntry, String>),
    /// Revoke an existing delegation
    RevokeDelegation(Uuid),
    /// Delegation revocation completed
    DelegationRevoked(Result<Uuid, String>),
}

/// Delegation State
///
/// Contains all state related to delegation management.
#[derive(Debug, Clone, Default)]
pub struct DelegationState {
    // === UI State ===
    /// Whether delegation section is collapsed
    pub delegation_section_collapsed: bool,

    // === Person Selection ===
    /// Person delegating (grantor)
    pub delegation_from_person: Option<Uuid>,
    /// Person receiving delegation (grantee)
    pub delegation_to_person: Option<Uuid>,

    // === Permission Management ===
    /// Set of permissions being delegated
    pub delegation_permissions: HashSet<KeyPermission>,

    // === Expiration ===
    /// Expiration in days (empty = no expiration)
    pub delegation_expires_days: String,

    // === Active Delegations ===
    /// List of active delegations
    pub active_delegations: Vec<DelegationEntry>,
}

impl DelegationState {
    /// Create a new DelegationState with sensible defaults
    pub fn new() -> Self {
        Self {
            delegation_section_collapsed: true,
            delegation_from_person: None,
            delegation_to_person: None,
            delegation_permissions: HashSet::new(),
            delegation_expires_days: String::new(),
            active_delegations: Vec::new(),
        }
    }

    /// Check if ready to create delegation (has from, to, and permissions)
    pub fn is_ready_to_create(&self) -> bool {
        self.delegation_from_person.is_some()
            && self.delegation_to_person.is_some()
            && !self.delegation_permissions.is_empty()
    }

    /// Check if expiration days is valid (empty or positive integer)
    pub fn is_expiration_valid(&self) -> bool {
        self.delegation_expires_days.is_empty()
            || self.delegation_expires_days.parse::<i64>().map(|d| d > 0).unwrap_or(false)
    }

    /// Get count of active (not revoked) delegations
    pub fn active_delegation_count(&self) -> usize {
        self.active_delegations.iter().filter(|d| d.is_active).count()
    }

    /// Get count of valid (active and not expired) delegations
    pub fn valid_delegation_count(&self) -> usize {
        self.active_delegations.iter().filter(|d| d.is_valid()).count()
    }

    /// Clear the form after successful creation
    pub fn clear_form(&mut self) {
        self.delegation_from_person = None;
        self.delegation_to_person = None;
        self.delegation_permissions.clear();
        self.delegation_expires_days = String::new();
    }

    /// Find delegation by ID
    pub fn find_delegation(&self, id: Uuid) -> Option<&DelegationEntry> {
        self.active_delegations.iter().find(|d| d.id == id)
    }

    /// Find mutable delegation by ID
    pub fn find_delegation_mut(&mut self, id: Uuid) -> Option<&mut DelegationEntry> {
        self.active_delegations.iter_mut().find(|d| d.id == id)
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update delegation state based on message
///
/// This function handles delegation domain messages. Note that some messages
/// require access to loaded_people for name lookup and will set status in main.
pub fn update(state: &mut DelegationState, message: DelegationMessage) -> Task<Message> {
    use DelegationMessage::*;

    match message {
        // === Section Toggle ===
        ToggleDelegationSection => {
            state.delegation_section_collapsed = !state.delegation_section_collapsed;
            Task::none()
        }

        // === Person Selection ===
        DelegationFromPersonSelected(person_id) => {
            state.delegation_from_person = Some(person_id);
            // Can't delegate to yourself
            if state.delegation_to_person == Some(person_id) {
                state.delegation_to_person = None;
            }
            Task::none()
        }

        DelegationToPersonSelected(person_id) => {
            state.delegation_to_person = Some(person_id);
            // Can't delegate to yourself
            if state.delegation_from_person == Some(person_id) {
                state.delegation_from_person = None;
            }
            Task::none()
        }

        // === Permission Management ===
        ToggleDelegationPermission(permission) => {
            if state.delegation_permissions.contains(&permission) {
                state.delegation_permissions.remove(&permission);
            } else {
                state.delegation_permissions.insert(permission);
            }
            Task::none()
        }

        // === Expiration ===
        DelegationExpiresDaysChanged(days) => {
            state.delegation_expires_days = days;
            Task::none()
        }

        // === Lifecycle (delegated to main for loaded_people access) ===
        CreateDelegation => {
            // Validation and creation happens in main update
            // because we need access to loaded_people for name lookup
            Task::none()
        }

        DelegationCreated(result) => {
            match result {
                Ok(entry) => {
                    state.active_delegations.push(entry);
                    state.clear_form();
                }
                Err(_) => {
                    // Error handling done in main app
                }
            }
            Task::none()
        }

        RevokeDelegation(delegation_id) => {
            // Find and deactivate the delegation
            if let Some(delegation) = state.find_delegation_mut(delegation_id) {
                delegation.is_active = false;
            }
            Task::none()
        }

        DelegationRevoked(_result) => {
            // Status handling done in main app
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_state_default() {
        let state = DelegationState::default();
        // Default derive sets bools to false
        assert!(!state.delegation_section_collapsed);
        assert!(state.delegation_from_person.is_none());
        assert!(state.delegation_to_person.is_none());
        assert!(state.delegation_permissions.is_empty());
        assert!(state.delegation_expires_days.is_empty());
        assert!(state.active_delegations.is_empty());
    }

    #[test]
    fn test_delegation_state_new() {
        let state = DelegationState::new();
        assert!(state.delegation_section_collapsed);
        assert!(!state.is_ready_to_create());
    }

    #[test]
    fn test_is_ready_to_create() {
        let mut state = DelegationState::new();
        assert!(!state.is_ready_to_create());

        state.delegation_from_person = Some(Uuid::now_v7());
        assert!(!state.is_ready_to_create());

        state.delegation_to_person = Some(Uuid::now_v7());
        assert!(!state.is_ready_to_create());

        state.delegation_permissions.insert(KeyPermission::Sign);
        assert!(state.is_ready_to_create());
    }

    #[test]
    fn test_is_expiration_valid() {
        let mut state = DelegationState::new();
        assert!(state.is_expiration_valid()); // Empty is valid

        state.delegation_expires_days = "30".to_string();
        assert!(state.is_expiration_valid());

        state.delegation_expires_days = "0".to_string();
        assert!(!state.is_expiration_valid());

        state.delegation_expires_days = "-5".to_string();
        assert!(!state.is_expiration_valid());

        state.delegation_expires_days = "abc".to_string();
        assert!(!state.is_expiration_valid());
    }

    #[test]
    fn test_toggle_section() {
        let mut state = DelegationState::new();
        assert!(state.delegation_section_collapsed);

        let _ = update(&mut state, DelegationMessage::ToggleDelegationSection);
        assert!(!state.delegation_section_collapsed);

        let _ = update(&mut state, DelegationMessage::ToggleDelegationSection);
        assert!(state.delegation_section_collapsed);
    }

    #[test]
    fn test_self_delegation_prevention() {
        let mut state = DelegationState::new();
        let person_id = Uuid::now_v7();

        // Set from person
        let _ = update(&mut state, DelegationMessage::DelegationFromPersonSelected(person_id));
        assert_eq!(state.delegation_from_person, Some(person_id));

        // Set to same person - should clear from
        let _ = update(&mut state, DelegationMessage::DelegationToPersonSelected(person_id));
        assert_eq!(state.delegation_to_person, Some(person_id));
        assert_eq!(state.delegation_from_person, None); // Cleared

        // Set from same person again - should clear to
        let _ = update(&mut state, DelegationMessage::DelegationFromPersonSelected(person_id));
        assert_eq!(state.delegation_from_person, Some(person_id));
        assert_eq!(state.delegation_to_person, None); // Cleared
    }

    #[test]
    fn test_permission_toggle() {
        let mut state = DelegationState::new();

        // Add permission
        let _ = update(&mut state, DelegationMessage::ToggleDelegationPermission(KeyPermission::Sign));
        assert!(state.delegation_permissions.contains(&KeyPermission::Sign));

        // Add another
        let _ = update(&mut state, DelegationMessage::ToggleDelegationPermission(KeyPermission::Encrypt));
        assert!(state.delegation_permissions.contains(&KeyPermission::Encrypt));
        assert_eq!(state.delegation_permissions.len(), 2);

        // Remove first
        let _ = update(&mut state, DelegationMessage::ToggleDelegationPermission(KeyPermission::Sign));
        assert!(!state.delegation_permissions.contains(&KeyPermission::Sign));
        assert_eq!(state.delegation_permissions.len(), 1);
    }

    #[test]
    fn test_delegation_entry_validity() {
        let entry = DelegationEntry {
            id: Uuid::now_v7(),
            from_person_id: Uuid::now_v7(),
            from_person_name: "Alice".to_string(),
            to_person_id: Uuid::now_v7(),
            to_person_name: "Bob".to_string(),
            permissions: vec![KeyPermission::Sign],
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
        };

        assert!(entry.is_valid());
        assert!(!entry.is_expired());
        assert!(entry.days_until_expiration().is_none());
    }

    #[test]
    fn test_delegation_entry_expired() {
        let entry = DelegationEntry {
            id: Uuid::now_v7(),
            from_person_id: Uuid::now_v7(),
            from_person_name: "Alice".to_string(),
            to_person_id: Uuid::now_v7(),
            to_person_name: "Bob".to_string(),
            permissions: vec![KeyPermission::Sign],
            created_at: Utc::now() - chrono::Duration::days(10),
            expires_at: Some(Utc::now() - chrono::Duration::days(1)), // Expired yesterday
            is_active: true,
        };

        assert!(!entry.is_valid()); // Active but expired
        assert!(entry.is_expired());
    }

    #[test]
    fn test_revoke_delegation() {
        let mut state = DelegationState::new();
        let delegation_id = Uuid::now_v7();

        // Add a delegation
        state.active_delegations.push(DelegationEntry {
            id: delegation_id,
            from_person_id: Uuid::now_v7(),
            from_person_name: "Alice".to_string(),
            to_person_id: Uuid::now_v7(),
            to_person_name: "Bob".to_string(),
            permissions: vec![KeyPermission::Sign],
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
        });

        assert_eq!(state.active_delegation_count(), 1);

        // Revoke it
        let _ = update(&mut state, DelegationMessage::RevokeDelegation(delegation_id));
        assert_eq!(state.active_delegation_count(), 0);
        assert_eq!(state.active_delegations.len(), 1); // Still in list but inactive
    }

    #[test]
    fn test_clear_form() {
        let mut state = DelegationState::new();
        state.delegation_from_person = Some(Uuid::now_v7());
        state.delegation_to_person = Some(Uuid::now_v7());
        state.delegation_permissions.insert(KeyPermission::Sign);
        state.delegation_expires_days = "30".to_string();

        state.clear_form();

        assert!(state.delegation_from_person.is_none());
        assert!(state.delegation_to_person.is_none());
        assert!(state.delegation_permissions.is_empty());
        assert!(state.delegation_expires_days.is_empty());
    }
}
