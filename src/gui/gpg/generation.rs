// Copyright (c) 2025 - Cowboy AI, LLC.

//! GPG Key Generation Bounded Context
//!
//! This module implements the GPG Keys domain with:
//! - Message enum for all GPG key operations
//! - State struct for GPG-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Form Input**: User ID, key type, length, expiration
//! 3. **Key Generation**: Generate and list keys
//! 4. **Generated Keys**: Track generated key pairs

use iced::Task;

use crate::ports::gpg::{GpgKeyInfo, GpgKeyType, GpgKeypair};

/// GPG Message
///
/// Organized by sub-domain:
/// - UI State (1 message)
/// - Form Input (4 messages)
/// - Key Generation (4 messages)
#[derive(Debug, Clone)]
pub enum GpgMessage {
    // === UI State ===
    /// Toggle GPG section visibility
    ToggleSection,

    // === Form Input ===
    /// User ID changed (e.g., "Name <email@example.com>")
    UserIdChanged(String),
    /// Key type selected (EdDSA, ECDSA, RSA, DSA)
    KeyTypeSelected(GpgKeyType),
    /// Key length changed (in bits)
    KeyLengthChanged(String),
    /// Expiration days changed
    ExpiresDaysChanged(String),

    // === Key Generation ===
    /// Generate a new GPG key pair
    Generate,
    /// Key generation result
    Generated(Result<GpgKeypair, String>),
    /// List existing GPG keys
    ListKeys,
    /// Key listing result
    KeysListed(Result<Vec<GpgKeyInfo>, String>),
}

/// GPG State
///
/// Contains all state related to GPG key management.
#[derive(Debug, Clone)]
pub struct GpgState {
    // === UI State ===
    /// Whether the GPG section is collapsed
    pub section_collapsed: bool,

    // === Form Input ===
    /// User ID for the key (e.g., "Name <email@example.com>")
    pub user_id: String,
    /// Selected key type
    pub key_type: Option<GpgKeyType>,
    /// Key length in bits (as string for text input)
    pub key_length: String,
    /// Days until expiration (as string for text input)
    pub expires_days: String,

    // === Generation Status ===
    /// Current generation status message
    pub generation_status: Option<String>,

    // === Generated Keys ===
    /// List of generated GPG keys
    pub generated_keys: Vec<GpgKeyInfo>,
}

impl Default for GpgState {
    fn default() -> Self {
        Self {
            section_collapsed: false,
            user_id: String::new(),
            key_type: None,
            key_length: String::new(),
            expires_days: String::new(),
            generation_status: None,
            generated_keys: Vec::new(),
        }
    }
}

impl GpgState {
    /// Create a new GpgState with sensible defaults
    pub fn new() -> Self {
        Self {
            section_collapsed: true,
            user_id: String::new(),
            key_type: None,
            key_length: String::from("4096"),
            expires_days: String::from("365"),
            generation_status: None,
            generated_keys: Vec::new(),
        }
    }

    /// Check if the form has minimum required fields
    pub fn is_form_valid(&self) -> bool {
        !self.user_id.is_empty()
            && self.key_type.is_some()
            && self.parse_key_length().is_some()
    }

    /// Parse key length from string
    pub fn parse_key_length(&self) -> Option<u32> {
        self.key_length.parse::<u32>().ok().filter(|&len| len >= 1024)
    }

    /// Parse expiration days from string (None = never expires)
    pub fn parse_expires_days(&self) -> Option<u32> {
        if self.expires_days.is_empty() {
            None
        } else {
            self.expires_days.parse::<u32>().ok()
        }
    }

    /// Get validation error message if form is invalid
    pub fn validation_error(&self) -> Option<String> {
        if self.user_id.is_empty() {
            return Some("User ID is required (e.g., 'Name <email@example.com>')".to_string());
        }
        if self.key_type.is_none() {
            return Some("Please select a key type".to_string());
        }
        if self.parse_key_length().is_none() {
            return Some("Key length must be at least 1024 bits".to_string());
        }
        if !self.expires_days.is_empty() && self.parse_expires_days().is_none() {
            return Some("Invalid expiration days".to_string());
        }
        None
    }

    /// Clear the form fields after successful generation
    pub fn clear_form(&mut self) {
        self.user_id.clear();
        self.generation_status = None;
    }

    /// Get count of generated keys
    pub fn key_count(&self) -> usize {
        self.generated_keys.len()
    }

    /// Find a key by fingerprint
    pub fn find_key_by_fingerprint(&self, fingerprint: &str) -> Option<&GpgKeyInfo> {
        self.generated_keys
            .iter()
            .find(|k| k.fingerprint == fingerprint)
    }

    /// Get count of valid (not expired, not revoked) keys
    pub fn valid_key_count(&self) -> usize {
        self.generated_keys
            .iter()
            .filter(|k| !k.is_expired && !k.is_revoked)
            .count()
    }

    /// Set generation status with emoji prefix
    pub fn set_status_generating(&mut self) {
        self.generation_status = Some("⏳ Generating GPG key...".to_string());
    }

    /// Set generation status to success
    pub fn set_status_success(&mut self, fingerprint: &str) {
        self.generation_status = Some(format!("✅ Key generated: {}", fingerprint));
    }

    /// Set generation status to failure
    pub fn set_status_failure(&mut self, error: &str) {
        self.generation_status = Some(format!("❌ Failed: {}", error));
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update GPG state based on message
///
/// This function handles GPG domain messages. Note that Generate and ListKeys
/// require port access and will be delegated to the main update function.
pub fn update(state: &mut GpgState, message: GpgMessage) -> Task<Message> {
    use GpgMessage::*;

    match message {
        // === UI State ===
        ToggleSection => {
            state.section_collapsed = !state.section_collapsed;
            Task::none()
        }

        // === Form Input ===
        UserIdChanged(user_id) => {
            state.user_id = user_id;
            Task::none()
        }

        KeyTypeSelected(key_type) => {
            state.key_type = Some(key_type);
            Task::none()
        }

        KeyLengthChanged(length) => {
            state.key_length = length;
            Task::none()
        }

        ExpiresDaysChanged(days) => {
            state.expires_days = days;
            Task::none()
        }

        // === Key Generation (delegated to main for port access) ===
        Generate => {
            // Actual generation requires GPG port access
            // Delegated to main update function
            Task::none()
        }

        Generated(_result) => {
            // Result handling requires status/error messaging
            // Delegated to main update function
            Task::none()
        }

        ListKeys => {
            // Key listing requires GPG port access
            // Delegated to main update function
            Task::none()
        }

        KeysListed(_result) => {
            // Result handling requires status/error messaging
            // Delegated to main update function
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::gpg::GpgKeyId;

    #[test]
    fn test_gpg_state_default() {
        let state = GpgState::default();
        assert!(state.user_id.is_empty());
        assert!(state.key_type.is_none());
        assert!(state.key_length.is_empty());
        assert!(state.expires_days.is_empty());
        assert!(state.generation_status.is_none());
        assert!(state.generated_keys.is_empty());
        assert!(!state.section_collapsed);
    }

    #[test]
    fn test_gpg_state_new() {
        let state = GpgState::new();
        assert!(state.user_id.is_empty());
        assert!(state.key_type.is_none());
        assert_eq!(state.key_length, "4096");
        assert_eq!(state.expires_days, "365");
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_toggle_section() {
        let mut state = GpgState::new();
        assert!(state.section_collapsed);

        let _ = update(&mut state, GpgMessage::ToggleSection);
        assert!(!state.section_collapsed);

        let _ = update(&mut state, GpgMessage::ToggleSection);
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_user_id_changed() {
        let mut state = GpgState::new();
        let _ = update(
            &mut state,
            GpgMessage::UserIdChanged("John Doe <john@example.com>".to_string()),
        );
        assert_eq!(state.user_id, "John Doe <john@example.com>");
    }

    #[test]
    fn test_key_type_selected() {
        let mut state = GpgState::new();
        let _ = update(&mut state, GpgMessage::KeyTypeSelected(GpgKeyType::Eddsa));
        assert_eq!(state.key_type, Some(GpgKeyType::Eddsa));
    }

    #[test]
    fn test_key_length_changed() {
        let mut state = GpgState::new();
        let _ = update(&mut state, GpgMessage::KeyLengthChanged("2048".to_string()));
        assert_eq!(state.key_length, "2048");
    }

    #[test]
    fn test_expires_days_changed() {
        let mut state = GpgState::new();
        let _ = update(
            &mut state,
            GpgMessage::ExpiresDaysChanged("730".to_string()),
        );
        assert_eq!(state.expires_days, "730");
    }

    #[test]
    fn test_is_form_valid() {
        let mut state = GpgState::new();
        assert!(!state.is_form_valid());

        state.user_id = "John Doe <john@example.com>".to_string();
        assert!(!state.is_form_valid());

        state.key_type = Some(GpgKeyType::Rsa);
        assert!(state.is_form_valid()); // key_length is "4096" by default
    }

    #[test]
    fn test_parse_key_length() {
        let mut state = GpgState::new();

        state.key_length = "4096".to_string();
        assert_eq!(state.parse_key_length(), Some(4096));

        state.key_length = "512".to_string();
        assert_eq!(state.parse_key_length(), None); // Too small

        state.key_length = "invalid".to_string();
        assert_eq!(state.parse_key_length(), None);
    }

    #[test]
    fn test_parse_expires_days() {
        let mut state = GpgState::new();

        state.expires_days = "365".to_string();
        assert_eq!(state.parse_expires_days(), Some(365));

        state.expires_days = String::new();
        assert_eq!(state.parse_expires_days(), None); // Never expires

        state.expires_days = "invalid".to_string();
        assert_eq!(state.parse_expires_days(), None);
    }

    #[test]
    fn test_validation_error_user_id() {
        let state = GpgState::new();
        assert_eq!(
            state.validation_error(),
            Some("User ID is required (e.g., 'Name <email@example.com>')".to_string())
        );
    }

    #[test]
    fn test_validation_error_key_type() {
        let mut state = GpgState::new();
        state.user_id = "John Doe <john@example.com>".to_string();
        assert_eq!(
            state.validation_error(),
            Some("Please select a key type".to_string())
        );
    }

    #[test]
    fn test_validation_error_key_length() {
        let mut state = GpgState::new();
        state.user_id = "John Doe <john@example.com>".to_string();
        state.key_type = Some(GpgKeyType::Rsa);
        state.key_length = "512".to_string(); // Too small
        assert_eq!(
            state.validation_error(),
            Some("Key length must be at least 1024 bits".to_string())
        );
    }

    #[test]
    fn test_validation_error_expires_days() {
        let mut state = GpgState::new();
        state.user_id = "John Doe <john@example.com>".to_string();
        state.key_type = Some(GpgKeyType::Rsa);
        state.expires_days = "invalid".to_string();
        assert_eq!(
            state.validation_error(),
            Some("Invalid expiration days".to_string())
        );
    }

    #[test]
    fn test_clear_form() {
        let mut state = GpgState::new();
        state.user_id = "John Doe <john@example.com>".to_string();
        state.generation_status = Some("✅ Success".to_string());

        state.clear_form();

        assert!(state.user_id.is_empty());
        assert!(state.generation_status.is_none());
    }

    #[test]
    fn test_key_count() {
        let mut state = GpgState::new();
        assert_eq!(state.key_count(), 0);

        state.generated_keys.push(GpgKeyInfo {
            key_id: GpgKeyId("ABC123".to_string()),
            fingerprint: "FINGERPRINT1".to_string(),
            user_ids: vec!["John Doe <john@example.com>".to_string()],
            creation_time: 0,
            expiration_time: None,
            is_revoked: false,
            is_expired: false,
        });
        assert_eq!(state.key_count(), 1);
    }

    #[test]
    fn test_valid_key_count() {
        let mut state = GpgState::new();

        state.generated_keys.push(GpgKeyInfo {
            key_id: GpgKeyId("KEY1".to_string()),
            fingerprint: "FP1".to_string(),
            user_ids: vec![],
            creation_time: 0,
            expiration_time: None,
            is_revoked: false,
            is_expired: false,
        });

        state.generated_keys.push(GpgKeyInfo {
            key_id: GpgKeyId("KEY2".to_string()),
            fingerprint: "FP2".to_string(),
            user_ids: vec![],
            creation_time: 0,
            expiration_time: None,
            is_revoked: true, // Revoked
            is_expired: false,
        });

        state.generated_keys.push(GpgKeyInfo {
            key_id: GpgKeyId("KEY3".to_string()),
            fingerprint: "FP3".to_string(),
            user_ids: vec![],
            creation_time: 0,
            expiration_time: None,
            is_revoked: false,
            is_expired: true, // Expired
        });

        assert_eq!(state.key_count(), 3);
        assert_eq!(state.valid_key_count(), 1);
    }

    #[test]
    fn test_find_key_by_fingerprint() {
        let mut state = GpgState::new();
        state.generated_keys.push(GpgKeyInfo {
            key_id: GpgKeyId("KEY1".to_string()),
            fingerprint: "ABC123DEF456".to_string(),
            user_ids: vec!["User 1".to_string()],
            creation_time: 0,
            expiration_time: None,
            is_revoked: false,
            is_expired: false,
        });

        assert!(state.find_key_by_fingerprint("ABC123DEF456").is_some());
        assert!(state.find_key_by_fingerprint("NOT_FOUND").is_none());
    }

    #[test]
    fn test_status_helpers() {
        let mut state = GpgState::new();

        state.set_status_generating();
        assert_eq!(
            state.generation_status,
            Some("⏳ Generating GPG key...".to_string())
        );

        state.set_status_success("ABC123");
        assert_eq!(
            state.generation_status,
            Some("✅ Key generated: ABC123".to_string())
        );

        state.set_status_failure("Connection failed");
        assert_eq!(
            state.generation_status,
            Some("❌ Failed: Connection failed".to_string())
        );
    }
}
