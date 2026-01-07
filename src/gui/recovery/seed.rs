// Copyright (c) 2025 - Cowboy AI, LLC.

//! Key Recovery Message Definitions
//!
//! This module defines the message types for the Key Recovery bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Form Input**: Passphrase, confirmation, organization ID
//! 3. **Verification**: Seed derivation and verification
//! 4. **Recovery**: Key regeneration from verified seed

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_message_variants() {
        let _ = RecoveryMessage::ToggleSection;
        let _ = RecoveryMessage::PassphraseChanged("test".to_string());
        let _ = RecoveryMessage::PassphraseConfirmChanged("test".to_string());
        let _ = RecoveryMessage::OrganizationIdChanged("org-123".to_string());
        let _ = RecoveryMessage::VerifySeed;
        let _ = RecoveryMessage::SeedVerified(Ok("fingerprint".to_string()));
        let _ = RecoveryMessage::RecoverKeys;
        let _ = RecoveryMessage::KeysRecovered(Ok(5));
    }
}
