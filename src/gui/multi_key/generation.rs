// Copyright (c) 2025 - Cowboy AI, LLC.

//! Multi-Purpose Key Generation Bounded Context
//!
//! This module implements the Multi-Purpose Key domain with:
//! - Message enum for all multi-key operations
//! - State struct for multi-key generation fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Selection**: Person and purpose selection
//! 3. **Generation**: Key generation lifecycle

use iced::Task;
use std::collections::HashSet;
use uuid::Uuid;

use crate::domain::InvariantKeyPurpose;

/// Multi-Purpose Key Message
///
/// Organized by sub-domain:
/// - UI State (1 message)
/// - Selection (2 messages)
/// - Generation (2 messages)
#[derive(Debug, Clone)]
pub enum MultiKeyMessage {
    // === UI State ===
    /// Toggle multi-purpose key section visibility
    ToggleSection,

    // === Selection ===
    /// Person selected for key generation
    PersonSelected(Uuid),
    /// Toggle a key purpose (add/remove from selection)
    TogglePurpose(InvariantKeyPurpose),

    // === Generation ===
    /// Generate keys for selected person with selected purposes
    Generate,
    /// Key generation completed
    Generated(Result<GenerationResult, String>),
}

/// Result of multi-purpose key generation
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Person ID that keys were generated for
    pub person_id: Uuid,
    /// Fingerprints of generated keys
    pub key_fingerprints: Vec<String>,
}

/// Multi-Purpose Key State
///
/// Contains all state related to multi-purpose key generation.
#[derive(Debug, Clone, Default)]
pub struct MultiKeyState {
    // === UI State ===
    /// Whether the multi-purpose key section is collapsed
    pub section_collapsed: bool,

    // === Selection ===
    /// Currently selected person for key generation
    pub selected_person: Option<Uuid>,
    /// Selected key purposes
    pub selected_purposes: HashSet<InvariantKeyPurpose>,

    // === Status ===
    /// Status message for user feedback
    pub status: Option<String>,
}

impl MultiKeyState {
    /// Create a new MultiKeyState with sensible defaults
    pub fn new() -> Self {
        Self {
            section_collapsed: true,
            selected_person: None,
            selected_purposes: HashSet::new(),
            status: None,
        }
    }

    /// Check if ready to generate keys
    pub fn is_ready_to_generate(&self) -> bool {
        self.selected_person.is_some() && !self.selected_purposes.is_empty()
    }

    /// Get validation error message if not ready
    pub fn validation_error(&self) -> Option<String> {
        if self.selected_person.is_none() {
            return Some("Please select a person for key generation".to_string());
        }
        if self.selected_purposes.is_empty() {
            return Some("Please select at least one key purpose".to_string());
        }
        None
    }

    /// Clear selection after successful generation
    pub fn clear_selection(&mut self) {
        self.selected_purposes.clear();
        // Keep person selected for convenience
    }

    /// Reset all state
    pub fn reset(&mut self) {
        self.selected_person = None;
        self.selected_purposes.clear();
        self.status = None;
    }

    /// Get count of selected purposes
    pub fn purpose_count(&self) -> usize {
        self.selected_purposes.len()
    }

    /// Check if a purpose is selected
    pub fn is_purpose_selected(&self, purpose: InvariantKeyPurpose) -> bool {
        self.selected_purposes.contains(&purpose)
    }

    /// Get all available purposes
    pub fn available_purposes() -> Vec<InvariantKeyPurpose> {
        vec![
            InvariantKeyPurpose::Authentication,
            InvariantKeyPurpose::Signing,
            InvariantKeyPurpose::Encryption,
            InvariantKeyPurpose::KeyAgreement,
        ]
    }

    /// Get purpose display name
    pub fn purpose_display_name(purpose: InvariantKeyPurpose) -> &'static str {
        match purpose {
            InvariantKeyPurpose::Authentication => "Authentication",
            InvariantKeyPurpose::Signing => "Digital Signing",
            InvariantKeyPurpose::Encryption => "Encryption",
            InvariantKeyPurpose::KeyAgreement => "Key Agreement",
        }
    }

    /// Set status for generating
    pub fn set_status_generating(&mut self) {
        self.status = Some("🔑 Generating keys...".to_string());
    }

    /// Set status for success
    pub fn set_status_success(&mut self, count: usize) {
        self.status = Some(format!("✅ Generated {} key(s) successfully", count));
    }

    /// Set status for failure
    pub fn set_status_failure(&mut self, error: &str) {
        self.status = Some(format!("❌ Generation failed: {}", error));
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update multi-purpose key state based on message
///
/// This function handles multi-purpose key domain messages. Note that Generate
/// requires additional context and will be delegated to the main update function.
pub fn update(state: &mut MultiKeyState, message: MultiKeyMessage) -> Task<Message> {
    use MultiKeyMessage::*;

    match message {
        // === UI State ===
        ToggleSection => {
            state.section_collapsed = !state.section_collapsed;
            Task::none()
        }

        // === Selection ===
        PersonSelected(person_id) => {
            state.selected_person = Some(person_id);
            Task::none()
        }

        TogglePurpose(purpose) => {
            if state.selected_purposes.contains(&purpose) {
                state.selected_purposes.remove(&purpose);
            } else {
                state.selected_purposes.insert(purpose);
            }
            Task::none()
        }

        // === Generation (delegated to main for actual key generation) ===
        Generate => {
            // Actual generation requires crypto operations
            // Delegated to main update function
            Task::none()
        }

        Generated(_result) => {
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
    fn test_multi_key_state_default() {
        let state = MultiKeyState::default();
        assert!(!state.section_collapsed);
        assert!(state.selected_person.is_none());
        assert!(state.selected_purposes.is_empty());
        assert!(state.status.is_none());
    }

    #[test]
    fn test_multi_key_state_new() {
        let state = MultiKeyState::new();
        assert!(state.section_collapsed);
        assert!(state.selected_person.is_none());
        assert!(state.selected_purposes.is_empty());
    }

    #[test]
    fn test_toggle_section() {
        let mut state = MultiKeyState::new();
        assert!(state.section_collapsed);

        let _ = update(&mut state, MultiKeyMessage::ToggleSection);
        assert!(!state.section_collapsed);

        let _ = update(&mut state, MultiKeyMessage::ToggleSection);
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_person_selected() {
        let mut state = MultiKeyState::new();
        let person_id = Uuid::now_v7();

        let _ = update(&mut state, MultiKeyMessage::PersonSelected(person_id));
        assert_eq!(state.selected_person, Some(person_id));
    }

    #[test]
    fn test_toggle_purpose_add() {
        let mut state = MultiKeyState::new();

        let _ = update(
            &mut state,
            MultiKeyMessage::TogglePurpose(InvariantKeyPurpose::Authentication),
        );
        assert!(state.selected_purposes.contains(&InvariantKeyPurpose::Authentication));
        assert_eq!(state.purpose_count(), 1);
    }

    #[test]
    fn test_toggle_purpose_remove() {
        let mut state = MultiKeyState::new();
        state.selected_purposes.insert(InvariantKeyPurpose::Signing);

        let _ = update(
            &mut state,
            MultiKeyMessage::TogglePurpose(InvariantKeyPurpose::Signing),
        );
        assert!(!state.selected_purposes.contains(&InvariantKeyPurpose::Signing));
        assert_eq!(state.purpose_count(), 0);
    }

    #[test]
    fn test_toggle_purpose_multiple() {
        let mut state = MultiKeyState::new();

        let _ = update(
            &mut state,
            MultiKeyMessage::TogglePurpose(InvariantKeyPurpose::Authentication),
        );
        let _ = update(
            &mut state,
            MultiKeyMessage::TogglePurpose(InvariantKeyPurpose::Signing),
        );
        let _ = update(
            &mut state,
            MultiKeyMessage::TogglePurpose(InvariantKeyPurpose::Encryption),
        );

        assert_eq!(state.purpose_count(), 3);
        assert!(state.is_purpose_selected(InvariantKeyPurpose::Authentication));
        assert!(state.is_purpose_selected(InvariantKeyPurpose::Signing));
        assert!(state.is_purpose_selected(InvariantKeyPurpose::Encryption));
        assert!(!state.is_purpose_selected(InvariantKeyPurpose::KeyAgreement));
    }

    #[test]
    fn test_is_ready_to_generate_no_person() {
        let mut state = MultiKeyState::new();
        state.selected_purposes.insert(InvariantKeyPurpose::Signing);

        assert!(!state.is_ready_to_generate());
    }

    #[test]
    fn test_is_ready_to_generate_no_purposes() {
        let mut state = MultiKeyState::new();
        state.selected_person = Some(Uuid::now_v7());

        assert!(!state.is_ready_to_generate());
    }

    #[test]
    fn test_is_ready_to_generate_valid() {
        let mut state = MultiKeyState::new();
        state.selected_person = Some(Uuid::now_v7());
        state.selected_purposes.insert(InvariantKeyPurpose::Signing);

        assert!(state.is_ready_to_generate());
    }

    #[test]
    fn test_validation_error_no_person() {
        let state = MultiKeyState::new();
        assert_eq!(
            state.validation_error(),
            Some("Please select a person for key generation".to_string())
        );
    }

    #[test]
    fn test_validation_error_no_purposes() {
        let mut state = MultiKeyState::new();
        state.selected_person = Some(Uuid::now_v7());

        assert_eq!(
            state.validation_error(),
            Some("Please select at least one key purpose".to_string())
        );
    }

    #[test]
    fn test_validation_no_error() {
        let mut state = MultiKeyState::new();
        state.selected_person = Some(Uuid::now_v7());
        state.selected_purposes.insert(InvariantKeyPurpose::Signing);

        assert!(state.validation_error().is_none());
    }

    #[test]
    fn test_clear_selection() {
        let mut state = MultiKeyState::new();
        let person_id = Uuid::now_v7();
        state.selected_person = Some(person_id);
        state.selected_purposes.insert(InvariantKeyPurpose::Signing);
        state.selected_purposes.insert(InvariantKeyPurpose::Encryption);

        state.clear_selection();

        // Person should remain selected for convenience
        assert_eq!(state.selected_person, Some(person_id));
        // Purposes should be cleared
        assert!(state.selected_purposes.is_empty());
    }

    #[test]
    fn test_reset() {
        let mut state = MultiKeyState::new();
        state.selected_person = Some(Uuid::now_v7());
        state.selected_purposes.insert(InvariantKeyPurpose::Signing);
        state.status = Some("Test status".to_string());

        state.reset();

        assert!(state.selected_person.is_none());
        assert!(state.selected_purposes.is_empty());
        assert!(state.status.is_none());
    }

    #[test]
    fn test_available_purposes() {
        let purposes = MultiKeyState::available_purposes();
        assert_eq!(purposes.len(), 4);
        assert!(purposes.contains(&InvariantKeyPurpose::Authentication));
        assert!(purposes.contains(&InvariantKeyPurpose::Signing));
        assert!(purposes.contains(&InvariantKeyPurpose::Encryption));
        assert!(purposes.contains(&InvariantKeyPurpose::KeyAgreement));
    }

    #[test]
    fn test_purpose_display_name() {
        assert_eq!(
            MultiKeyState::purpose_display_name(InvariantKeyPurpose::Authentication),
            "Authentication"
        );
        assert_eq!(
            MultiKeyState::purpose_display_name(InvariantKeyPurpose::Signing),
            "Digital Signing"
        );
        assert_eq!(
            MultiKeyState::purpose_display_name(InvariantKeyPurpose::Encryption),
            "Encryption"
        );
        assert_eq!(
            MultiKeyState::purpose_display_name(InvariantKeyPurpose::KeyAgreement),
            "Key Agreement"
        );
    }

    #[test]
    fn test_status_helpers() {
        let mut state = MultiKeyState::new();

        state.set_status_generating();
        assert!(state.status.as_ref().unwrap().contains("Generating"));

        state.set_status_success(3);
        assert!(state.status.as_ref().unwrap().contains("3 key(s)"));

        state.set_status_failure("Test error");
        assert!(state.status.as_ref().unwrap().contains("Test error"));
    }
}
