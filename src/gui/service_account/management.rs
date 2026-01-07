// Copyright (c) 2025 - Cowboy AI, LLC.

//! Service Account Message Definitions
//!
//! This module defines the message types for the Service Account bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Form Input**: Name, purpose, ownership fields
//! 3. **Lifecycle**: Create, deactivate, remove
//! 4. **Key Generation**: Generate keys for service accounts

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_account_message_variants() {
        let _ = ServiceAccountMessage::ToggleSection;
        let _ = ServiceAccountMessage::NameChanged("test-svc".to_string());
        let _ = ServiceAccountMessage::PurposeChanged("Test service".to_string());
        let _ = ServiceAccountMessage::OwningUnitSelected(Uuid::nil());
        let _ = ServiceAccountMessage::ResponsiblePersonSelected(Uuid::nil());
        let _ = ServiceAccountMessage::Create;
        let _ = ServiceAccountMessage::Deactivate(Uuid::nil());
        let _ = ServiceAccountMessage::Remove(Uuid::nil());
        let _ = ServiceAccountMessage::GenerateKey { service_account_id: Uuid::nil() };
    }
}
