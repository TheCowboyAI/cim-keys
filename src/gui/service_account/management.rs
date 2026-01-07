// Copyright (c) 2025 - Cowboy AI, LLC.

//! Service Account Management Bounded Context
//!
//! This module implements the Service Account domain with:
//! - Message enum for all service account operations
//! - State struct for service account-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Form Input**: Name, purpose, ownership fields
//! 3. **Lifecycle**: Create, deactivate, remove
//! 4. **Key Generation**: Generate keys for service accounts

use iced::Task;
use uuid::Uuid;

use crate::domain::{KeyOwnerRole, ServiceAccount};

/// Service Account Message
///
/// Organized by sub-domain:
/// - UI State (1 message)
/// - Form Input (4 messages)
/// - Lifecycle (4 messages)
/// - Key Generation (2 messages)
#[derive(Debug, Clone)]
pub enum ServiceAccountMessage {
    // === UI State ===
    /// Toggle service account section visibility
    ToggleSection,

    // === Form Input ===
    /// Service account name changed
    NameChanged(String),
    /// Service account purpose changed
    PurposeChanged(String),
    /// Owning organizational unit selected
    OwningUnitSelected(Uuid),
    /// Responsible person selected
    ResponsiblePersonSelected(Uuid),

    // === Lifecycle ===
    /// Create a new service account
    Create,
    /// Service account creation result
    Created(Result<ServiceAccount, String>),
    /// Deactivate a service account (mark inactive but keep record)
    Deactivate(Uuid),
    /// Remove a service account completely
    Remove(Uuid),

    // === Key Generation ===
    /// Generate a key for a service account
    GenerateKey { service_account_id: Uuid },
    /// Key generation result
    KeyGenerated(Result<(Uuid, KeyOwnerRole), String>),
}

/// Service Account State
///
/// Contains all state related to service account management.
#[derive(Debug, Clone, Default)]
pub struct ServiceAccountState {
    // === UI State ===
    /// Whether the service account section is collapsed
    pub section_collapsed: bool,

    // === Form Input ===
    /// Name of the new service account
    pub new_name: String,
    /// Purpose/description of the new service account
    pub new_purpose: String,
    /// Selected owning organizational unit
    pub new_owning_unit: Option<Uuid>,
    /// Selected responsible person
    pub new_responsible_person: Option<Uuid>,

    // === Loaded Data ===
    /// Created service accounts
    pub created_service_accounts: Vec<ServiceAccount>,
}

impl ServiceAccountState {
    /// Create a new ServiceAccountState with sensible defaults
    pub fn new() -> Self {
        Self {
            section_collapsed: true,
            new_name: String::new(),
            new_purpose: String::new(),
            new_owning_unit: None,
            new_responsible_person: None,
            created_service_accounts: Vec::new(),
        }
    }

    /// Check if the form has minimum required fields
    pub fn is_form_valid(&self) -> bool {
        !self.new_name.is_empty()
            && self.new_owning_unit.is_some()
            && self.new_responsible_person.is_some()
    }

    /// Get validation error message if form is invalid
    pub fn validation_error(&self) -> Option<String> {
        if self.new_name.is_empty() {
            return Some("Service account name is required".to_string());
        }
        if self.new_owning_unit.is_none() {
            return Some("Owning organizational unit is required".to_string());
        }
        if self.new_responsible_person.is_none() {
            return Some("Responsible person is required".to_string());
        }
        None
    }

    /// Clear the form fields after successful creation
    pub fn clear_form(&mut self) {
        self.new_name.clear();
        self.new_purpose.clear();
        self.new_owning_unit = None;
        self.new_responsible_person = None;
    }

    /// Get count of service accounts
    pub fn service_account_count(&self) -> usize {
        self.created_service_accounts.len()
    }

    /// Get count of active service accounts
    pub fn active_count(&self) -> usize {
        self.created_service_accounts
            .iter()
            .filter(|sa| sa.active)
            .count()
    }

    /// Find a service account by ID
    pub fn find_service_account(&self, id: Uuid) -> Option<&ServiceAccount> {
        self.created_service_accounts.iter().find(|sa| sa.id == id)
    }

    /// Find a service account by ID (mutable)
    pub fn find_service_account_mut(&mut self, id: Uuid) -> Option<&mut ServiceAccount> {
        self.created_service_accounts
            .iter_mut()
            .find(|sa| sa.id == id)
    }

    /// Deactivate a service account by ID
    pub fn deactivate(&mut self, id: Uuid) -> Option<String> {
        if let Some(sa) = self.find_service_account_mut(id) {
            sa.active = false;
            Some(sa.name.clone())
        } else {
            None
        }
    }

    /// Remove a service account by ID
    pub fn remove(&mut self, id: Uuid) -> Option<String> {
        if let Some(pos) = self
            .created_service_accounts
            .iter()
            .position(|sa| sa.id == id)
        {
            let removed = self.created_service_accounts.remove(pos);
            Some(removed.name)
        } else {
            None
        }
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update service account state based on message
///
/// This function handles service account domain messages. Note that Create,
/// GenerateKey, and result messages require additional context (projection,
/// error/status messaging) and will be delegated to the main update function.
pub fn update(state: &mut ServiceAccountState, message: ServiceAccountMessage) -> Task<Message> {
    use ServiceAccountMessage::*;

    match message {
        // === UI State ===
        ToggleSection => {
            state.section_collapsed = !state.section_collapsed;
            Task::none()
        }

        // === Form Input ===
        NameChanged(name) => {
            state.new_name = name;
            Task::none()
        }

        PurposeChanged(purpose) => {
            state.new_purpose = purpose;
            Task::none()
        }

        OwningUnitSelected(unit_id) => {
            state.new_owning_unit = Some(unit_id);
            Task::none()
        }

        ResponsiblePersonSelected(person_id) => {
            state.new_responsible_person = Some(person_id);
            Task::none()
        }

        // === Lifecycle (delegated to main for validation/projection access) ===
        Create => {
            // Actual creation requires validation and projection access
            // Delegated to main update function
            Task::none()
        }

        Created(_result) => {
            // Result handling requires status/error messaging
            // Delegated to main update function
            Task::none()
        }

        Deactivate(sa_id) => {
            state.deactivate(sa_id);
            Task::none()
        }

        Remove(sa_id) => {
            state.remove(sa_id);
            Task::none()
        }

        // === Key Generation (delegated to main) ===
        GenerateKey { .. } => {
            // Key generation requires workflow access
            // Delegated to main update function
            Task::none()
        }

        KeyGenerated(_result) => {
            // Result handling requires status/error messaging
            // Delegated to main update function
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_account_state_default() {
        let state = ServiceAccountState::default();
        assert!(state.new_name.is_empty());
        assert!(state.new_purpose.is_empty());
        assert!(state.new_owning_unit.is_none());
        assert!(state.new_responsible_person.is_none());
        assert!(state.created_service_accounts.is_empty());
        // Default doesn't set section_collapsed, so it's false
        assert!(!state.section_collapsed);
    }

    #[test]
    fn test_service_account_state_new() {
        let state = ServiceAccountState::new();
        assert!(state.new_name.is_empty());
        assert!(state.new_purpose.is_empty());
        assert!(state.new_owning_unit.is_none());
        assert!(state.new_responsible_person.is_none());
        assert!(state.created_service_accounts.is_empty());
        // new() sets section_collapsed to true
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_toggle_section() {
        let mut state = ServiceAccountState::new();
        assert!(state.section_collapsed);

        let _ = update(&mut state, ServiceAccountMessage::ToggleSection);
        assert!(!state.section_collapsed);

        let _ = update(&mut state, ServiceAccountMessage::ToggleSection);
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_name_changed() {
        let mut state = ServiceAccountState::new();
        let _ = update(
            &mut state,
            ServiceAccountMessage::NameChanged("ci-runner".to_string()),
        );
        assert_eq!(state.new_name, "ci-runner");
    }

    #[test]
    fn test_purpose_changed() {
        let mut state = ServiceAccountState::new();
        let _ = update(
            &mut state,
            ServiceAccountMessage::PurposeChanged("Continuous integration runner".to_string()),
        );
        assert_eq!(state.new_purpose, "Continuous integration runner");
    }

    #[test]
    fn test_owning_unit_selected() {
        let mut state = ServiceAccountState::new();
        let unit_id = Uuid::now_v7();

        let _ = update(
            &mut state,
            ServiceAccountMessage::OwningUnitSelected(unit_id),
        );
        assert_eq!(state.new_owning_unit, Some(unit_id));
    }

    #[test]
    fn test_responsible_person_selected() {
        let mut state = ServiceAccountState::new();
        let person_id = Uuid::now_v7();

        let _ = update(
            &mut state,
            ServiceAccountMessage::ResponsiblePersonSelected(person_id),
        );
        assert_eq!(state.new_responsible_person, Some(person_id));
    }

    #[test]
    fn test_is_form_valid() {
        let mut state = ServiceAccountState::new();
        assert!(!state.is_form_valid());

        state.new_name = "test-sa".to_string();
        assert!(!state.is_form_valid());

        state.new_owning_unit = Some(Uuid::now_v7());
        assert!(!state.is_form_valid());

        state.new_responsible_person = Some(Uuid::now_v7());
        assert!(state.is_form_valid());
    }

    #[test]
    fn test_validation_error_name_required() {
        let state = ServiceAccountState::new();
        assert_eq!(
            state.validation_error(),
            Some("Service account name is required".to_string())
        );
    }

    #[test]
    fn test_validation_error_owning_unit_required() {
        let mut state = ServiceAccountState::new();
        state.new_name = "test-sa".to_string();

        assert_eq!(
            state.validation_error(),
            Some("Owning organizational unit is required".to_string())
        );
    }

    #[test]
    fn test_validation_error_responsible_person_required() {
        let mut state = ServiceAccountState::new();
        state.new_name = "test-sa".to_string();
        state.new_owning_unit = Some(Uuid::now_v7());

        assert_eq!(
            state.validation_error(),
            Some("Responsible person is required".to_string())
        );
    }

    #[test]
    fn test_clear_form() {
        let mut state = ServiceAccountState::new();
        state.new_name = "test-sa".to_string();
        state.new_purpose = "Test purpose".to_string();
        state.new_owning_unit = Some(Uuid::now_v7());
        state.new_responsible_person = Some(Uuid::now_v7());

        state.clear_form();

        assert!(state.new_name.is_empty());
        assert!(state.new_purpose.is_empty());
        assert!(state.new_owning_unit.is_none());
        assert!(state.new_responsible_person.is_none());
    }

    #[test]
    fn test_service_account_count() {
        let mut state = ServiceAccountState::new();
        assert_eq!(state.service_account_count(), 0);

        state.created_service_accounts.push(ServiceAccount::new(
            "sa-1".to_string(),
            "Purpose 1".to_string(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        ));
        assert_eq!(state.service_account_count(), 1);

        state.created_service_accounts.push(ServiceAccount::new(
            "sa-2".to_string(),
            "Purpose 2".to_string(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        ));
        assert_eq!(state.service_account_count(), 2);
    }

    #[test]
    fn test_active_count() {
        let mut state = ServiceAccountState::new();

        let mut sa1 = ServiceAccount::new(
            "sa-1".to_string(),
            "Purpose 1".to_string(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let sa2 = ServiceAccount::new(
            "sa-2".to_string(),
            "Purpose 2".to_string(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );

        sa1.active = false;

        state.created_service_accounts.push(sa1);
        state.created_service_accounts.push(sa2);

        assert_eq!(state.active_count(), 1);
    }

    #[test]
    fn test_find_service_account() {
        let mut state = ServiceAccountState::new();
        let sa = ServiceAccount::new(
            "sa-1".to_string(),
            "Purpose 1".to_string(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let sa_id = sa.id;

        state.created_service_accounts.push(sa);

        assert!(state.find_service_account(sa_id).is_some());
        assert!(state.find_service_account(Uuid::now_v7()).is_none());
    }

    #[test]
    fn test_deactivate() {
        let mut state = ServiceAccountState::new();
        let sa = ServiceAccount::new(
            "sa-1".to_string(),
            "Purpose 1".to_string(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let sa_id = sa.id;

        state.created_service_accounts.push(sa);
        assert!(state.created_service_accounts[0].active);

        let _ = update(&mut state, ServiceAccountMessage::Deactivate(sa_id));
        assert!(!state.created_service_accounts[0].active);
    }

    #[test]
    fn test_remove() {
        let mut state = ServiceAccountState::new();
        let sa = ServiceAccount::new(
            "sa-1".to_string(),
            "Purpose 1".to_string(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let sa_id = sa.id;

        state.created_service_accounts.push(sa);
        assert_eq!(state.service_account_count(), 1);

        let _ = update(&mut state, ServiceAccountMessage::Remove(sa_id));
        assert_eq!(state.service_account_count(), 0);
    }
}
