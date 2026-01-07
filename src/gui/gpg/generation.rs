// Copyright (c) 2025 - Cowboy AI, LLC.

//! GPG Key Message Definitions
//!
//! This module defines the message types for the GPG Keys bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Form Input**: User ID, key type, length, expiration
//! 3. **Key Generation**: Generate and list keys

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpg_message_variants() {
        let _ = GpgMessage::ToggleSection;
        let _ = GpgMessage::UserIdChanged("Test <test@example.com>".to_string());
        let _ = GpgMessage::KeyTypeSelected(GpgKeyType::Eddsa);
        let _ = GpgMessage::KeyLengthChanged("4096".to_string());
        let _ = GpgMessage::ExpiresDaysChanged("365".to_string());
        let _ = GpgMessage::Generate;
        let _ = GpgMessage::ListKeys;
        let _ = GpgMessage::KeysListed(Ok(vec![]));
    }
}
