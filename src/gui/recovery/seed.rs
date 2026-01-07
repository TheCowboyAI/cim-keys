// Copyright (c) 2025 - Cowboy AI, LLC.

//! Key Recovery from Seed Bounded Context
//!
//! This module implements the Key Recovery domain with:
//! - Message enum for all recovery operations
//! - State struct for recovery-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Form Input**: Passphrase, confirmation, organization ID
//! 3. **Verification**: Seed derivation and verification
//! 4. **Recovery**: Key regeneration from verified seed

use iced::Task;

/// Recovery Message
///
/// Organized by sub-domain:
/// - UI State (1 message)
/// - Form Input (3 messages)
/// - Verification (2 messages)
/// - Recovery (2 messages)
#[derive(Debug, Clone)]
pub enum RecoveryMessage {
    // === UI State ===
    /// Toggle recovery section visibility
    ToggleSection,

    // === Form Input ===
    /// Recovery passphrase changed
    PassphraseChanged(String),
    /// Recovery passphrase confirmation changed
    PassphraseConfirmChanged(String),
    /// Organization ID changed (used for seed derivation)
    OrganizationIdChanged(String),

    // === Verification ===
    /// Verify the recovery seed can be derived
    VerifySeed,
    /// Seed verification result (Ok = fingerprint, Err = error message)
    SeedVerified(Result<String, String>),

    // === Recovery ===
    /// Recover keys from the verified seed
    RecoverKeys,
    /// Key recovery result (Ok = number of keys recovered, Err = error message)
    KeysRecovered(Result<usize, String>),
}

/// Recovery State
///
/// Contains all state related to key recovery from seed.
#[derive(Debug, Clone, Default)]
pub struct RecoveryState {
    // === UI State ===
    /// Whether the recovery section is collapsed
    pub section_collapsed: bool,

    // === Form Input ===
    /// Recovery passphrase (BIP-39 mnemonic or custom passphrase)
    pub passphrase: String,
    /// Passphrase confirmation (must match passphrase)
    pub passphrase_confirm: String,
    /// Organization ID for scoped recovery
    pub organization_id: String,

    // === Verification Status ===
    /// Current recovery status message
    pub status: Option<String>,
    /// Whether the seed has been successfully verified
    pub seed_verified: bool,
}

impl RecoveryState {
    /// Create a new RecoveryState with sensible defaults
    pub fn new() -> Self {
        Self {
            section_collapsed: true,
            passphrase: String::new(),
            passphrase_confirm: String::new(),
            organization_id: String::new(),
            status: None,
            seed_verified: false,
        }
    }

    /// Check if passphrases match
    pub fn passphrases_match(&self) -> bool {
        !self.passphrase.is_empty() && self.passphrase == self.passphrase_confirm
    }

    /// Check if the form has minimum required fields for verification
    pub fn is_verification_ready(&self) -> bool {
        !self.passphrase.is_empty()
            && self.passphrases_match()
            && !self.organization_id.is_empty()
    }

    /// Check if recovery can proceed (seed must be verified first)
    pub fn is_recovery_ready(&self) -> bool {
        self.is_verification_ready() && self.seed_verified
    }

    /// Get validation error message if form is invalid
    pub fn validation_error(&self) -> Option<String> {
        if self.passphrase.is_empty() {
            return Some("Recovery passphrase is required".to_string());
        }
        if !self.passphrases_match() {
            return Some("Passphrases do not match".to_string());
        }
        if self.organization_id.is_empty() {
            return Some("Organization ID is required for recovery".to_string());
        }
        None
    }

    /// Clear the form fields
    pub fn clear_form(&mut self) {
        self.passphrase.clear();
        self.passphrase_confirm.clear();
        self.organization_id.clear();
        self.status = None;
        self.seed_verified = false;
    }

    /// Reset verification state (called when inputs change)
    pub fn reset_verification(&mut self) {
        self.seed_verified = false;
        self.status = None;
    }

    /// Set status to "deriving seed"
    pub fn set_status_deriving(&mut self) {
        self.status = Some("⏳ Deriving seed...".to_string());
    }

    /// Set status to "recovering keys"
    pub fn set_status_recovering(&mut self) {
        self.status = Some("⏳ Recovering keys...".to_string());
    }

    /// Set status to verification success
    pub fn set_status_verified(&mut self, fingerprint: &str) {
        self.status = Some(format!("✅ Seed verified! Fingerprint: {}", fingerprint));
        self.seed_verified = true;
    }

    /// Set status to recovery success
    pub fn set_status_recovered(&mut self, count: usize) {
        self.status = Some(format!(
            "✅ Recovery complete! {} key types can be regenerated",
            count
        ));
    }

    /// Set status to failure
    pub fn set_status_failure(&mut self, error: &str) {
        self.status = Some(format!("❌ Failed: {}", error));
        self.seed_verified = false;
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update recovery state based on message
///
/// This function handles recovery domain messages. Note that VerifySeed,
/// RecoverKeys, and result messages require crypto operations and will
/// be delegated to the main update function.
pub fn update(state: &mut RecoveryState, message: RecoveryMessage) -> Task<Message> {
    use RecoveryMessage::*;

    match message {
        // === UI State ===
        ToggleSection => {
            state.section_collapsed = !state.section_collapsed;
            Task::none()
        }

        // === Form Input ===
        PassphraseChanged(passphrase) => {
            state.passphrase = passphrase;
            state.reset_verification();
            Task::none()
        }

        PassphraseConfirmChanged(passphrase) => {
            state.passphrase_confirm = passphrase;
            state.reset_verification();
            Task::none()
        }

        OrganizationIdChanged(org_id) => {
            state.organization_id = org_id;
            state.reset_verification();
            Task::none()
        }

        // === Verification (delegated to main for crypto access) ===
        VerifySeed => {
            // Actual verification requires crypto operations
            // Delegated to main update function
            Task::none()
        }

        SeedVerified(_result) => {
            // Result handling requires status/error messaging
            // Delegated to main update function
            Task::none()
        }

        // === Recovery (delegated to main for crypto access) ===
        RecoverKeys => {
            // Key recovery requires crypto operations
            // Delegated to main update function
            Task::none()
        }

        KeysRecovered(_result) => {
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
    fn test_recovery_state_default() {
        let state = RecoveryState::default();
        assert!(state.passphrase.is_empty());
        assert!(state.passphrase_confirm.is_empty());
        assert!(state.organization_id.is_empty());
        assert!(state.status.is_none());
        assert!(!state.seed_verified);
        assert!(!state.section_collapsed);
    }

    #[test]
    fn test_recovery_state_new() {
        let state = RecoveryState::new();
        assert!(state.passphrase.is_empty());
        assert!(state.passphrase_confirm.is_empty());
        assert!(state.organization_id.is_empty());
        assert!(state.section_collapsed);
        assert!(!state.seed_verified);
    }

    #[test]
    fn test_toggle_section() {
        let mut state = RecoveryState::new();
        assert!(state.section_collapsed);

        let _ = update(&mut state, RecoveryMessage::ToggleSection);
        assert!(!state.section_collapsed);

        let _ = update(&mut state, RecoveryMessage::ToggleSection);
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_passphrase_changed() {
        let mut state = RecoveryState::new();
        state.seed_verified = true;
        state.status = Some("Previous status".to_string());

        let _ = update(
            &mut state,
            RecoveryMessage::PassphraseChanged("my secret phrase".to_string()),
        );

        assert_eq!(state.passphrase, "my secret phrase");
        assert!(!state.seed_verified); // Reset on change
        assert!(state.status.is_none()); // Reset on change
    }

    #[test]
    fn test_passphrase_confirm_changed() {
        let mut state = RecoveryState::new();
        state.seed_verified = true;

        let _ = update(
            &mut state,
            RecoveryMessage::PassphraseConfirmChanged("my secret phrase".to_string()),
        );

        assert_eq!(state.passphrase_confirm, "my secret phrase");
        assert!(!state.seed_verified); // Reset on change
    }

    #[test]
    fn test_organization_id_changed() {
        let mut state = RecoveryState::new();
        state.seed_verified = true;

        let _ = update(
            &mut state,
            RecoveryMessage::OrganizationIdChanged("org-123".to_string()),
        );

        assert_eq!(state.organization_id, "org-123");
        assert!(!state.seed_verified); // Reset on change
    }

    #[test]
    fn test_passphrases_match() {
        let mut state = RecoveryState::new();
        assert!(!state.passphrases_match());

        state.passphrase = "secret".to_string();
        assert!(!state.passphrases_match());

        state.passphrase_confirm = "different".to_string();
        assert!(!state.passphrases_match());

        state.passphrase_confirm = "secret".to_string();
        assert!(state.passphrases_match());
    }

    #[test]
    fn test_is_verification_ready() {
        let mut state = RecoveryState::new();
        assert!(!state.is_verification_ready());

        state.passphrase = "secret".to_string();
        assert!(!state.is_verification_ready());

        state.passphrase_confirm = "secret".to_string();
        assert!(!state.is_verification_ready());

        state.organization_id = "org-123".to_string();
        assert!(state.is_verification_ready());
    }

    #[test]
    fn test_is_recovery_ready() {
        let mut state = RecoveryState::new();
        state.passphrase = "secret".to_string();
        state.passphrase_confirm = "secret".to_string();
        state.organization_id = "org-123".to_string();

        assert!(!state.is_recovery_ready()); // Not verified yet

        state.seed_verified = true;
        assert!(state.is_recovery_ready());
    }

    #[test]
    fn test_validation_error_passphrase_required() {
        let state = RecoveryState::new();
        assert_eq!(
            state.validation_error(),
            Some("Recovery passphrase is required".to_string())
        );
    }

    #[test]
    fn test_validation_error_passphrases_mismatch() {
        let mut state = RecoveryState::new();
        state.passphrase = "secret".to_string();
        state.passphrase_confirm = "different".to_string();

        assert_eq!(
            state.validation_error(),
            Some("Passphrases do not match".to_string())
        );
    }

    #[test]
    fn test_validation_error_organization_required() {
        let mut state = RecoveryState::new();
        state.passphrase = "secret".to_string();
        state.passphrase_confirm = "secret".to_string();

        assert_eq!(
            state.validation_error(),
            Some("Organization ID is required for recovery".to_string())
        );
    }

    #[test]
    fn test_validation_no_error() {
        let mut state = RecoveryState::new();
        state.passphrase = "secret".to_string();
        state.passphrase_confirm = "secret".to_string();
        state.organization_id = "org-123".to_string();

        assert!(state.validation_error().is_none());
    }

    #[test]
    fn test_clear_form() {
        let mut state = RecoveryState::new();
        state.passphrase = "secret".to_string();
        state.passphrase_confirm = "secret".to_string();
        state.organization_id = "org-123".to_string();
        state.status = Some("Some status".to_string());
        state.seed_verified = true;

        state.clear_form();

        assert!(state.passphrase.is_empty());
        assert!(state.passphrase_confirm.is_empty());
        assert!(state.organization_id.is_empty());
        assert!(state.status.is_none());
        assert!(!state.seed_verified);
    }

    #[test]
    fn test_reset_verification() {
        let mut state = RecoveryState::new();
        state.seed_verified = true;
        state.status = Some("Verified".to_string());

        state.reset_verification();

        assert!(!state.seed_verified);
        assert!(state.status.is_none());
    }

    #[test]
    fn test_status_helpers() {
        let mut state = RecoveryState::new();

        state.set_status_deriving();
        assert_eq!(state.status, Some("⏳ Deriving seed...".to_string()));

        state.set_status_recovering();
        assert_eq!(state.status, Some("⏳ Recovering keys...".to_string()));

        state.set_status_verified("ABC:123");
        assert_eq!(
            state.status,
            Some("✅ Seed verified! Fingerprint: ABC:123".to_string())
        );
        assert!(state.seed_verified);

        state.set_status_recovered(3);
        assert_eq!(
            state.status,
            Some("✅ Recovery complete! 3 key types can be regenerated".to_string())
        );

        state.set_status_failure("Connection failed");
        assert_eq!(
            state.status,
            Some("❌ Failed: Connection failed".to_string())
        );
        assert!(!state.seed_verified);
    }
}
