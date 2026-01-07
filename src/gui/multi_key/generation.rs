// Copyright (c) 2025 - Cowboy AI, LLC.

//! Multi-Purpose Key Message Definitions
//!
//! This module defines the message types for the Multi-Purpose Key bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Selection**: Person and purpose selection
//! 3. **Generation**: Key generation lifecycle

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

/// Get all available purposes for multi-purpose key generation
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_purposes() {
        let purposes = available_purposes();
        assert_eq!(purposes.len(), 4);
    }

    #[test]
    fn test_purpose_display_names() {
        assert_eq!(purpose_display_name(InvariantKeyPurpose::Authentication), "Authentication");
        assert_eq!(purpose_display_name(InvariantKeyPurpose::Signing), "Digital Signing");
        assert_eq!(purpose_display_name(InvariantKeyPurpose::Encryption), "Encryption");
        assert_eq!(purpose_display_name(InvariantKeyPurpose::KeyAgreement), "Key Agreement");
    }

    #[test]
    fn test_generation_result() {
        let result = GenerationResult {
            person_id: Uuid::nil(),
            key_fingerprints: vec!["fp1".to_string(), "fp2".to_string()],
        };
        assert_eq!(result.key_fingerprints.len(), 2);
    }
}
