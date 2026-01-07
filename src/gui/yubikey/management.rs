// Copyright (c) 2025 - Cowboy AI, LLC.

//! YubiKey Message Definitions
//!
//! This module defines the message types for the YubiKey bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **Device Detection**: Detect and enumerate YubiKeys
//! 2. **Person Assignment**: Assign YubiKeys to persons
//! 3. **PIV Slot Management**: Manage PIV slots (9A, 9C, 9D, 9E)
//! 4. **PIN/Management Key**: PIN and key operations
//! 5. **Domain Registration**: Register in domain lifecycle

use uuid::Uuid;

use crate::ports::yubikey::{PivSlot, YubiKeyDevice};
use crate::projections::{PersonEntry, YubiKeyEntry};

use super::super::{graph, SlotInfo};

/// YubiKey domain messages
///
/// These messages handle all YubiKey-related UI interactions and
/// async operation results.
#[derive(Debug, Clone)]
pub enum YubiKeyMessage {
    // ============================================================================
    // Device Detection
    // ============================================================================
    /// Detect connected YubiKeys
    DetectYubiKeys,
    /// YubiKeys detected result
    YubiKeysDetected(Result<Vec<YubiKeyDevice>, String>),
    /// YubiKey serial number changed (manual input)
    YubiKeySerialChanged(String),

    // ============================================================================
    // Person Assignment
    // ============================================================================
    /// Select YubiKey for assignment (by serial)
    SelectYubiKeyForAssignment(String),
    /// Assign YubiKey to person
    AssignYubiKeyToPerson { serial: String, person_id: Uuid },

    // ============================================================================
    // Provisioning
    // ============================================================================
    /// Provision all detected YubiKeys
    ProvisionYubiKey,
    /// Provision a single YubiKey by serial
    ProvisionSingleYubiKey { serial: String },
    /// YubiKey provisioned result
    YubiKeyProvisioned(Result<String, String>),
    /// Single YubiKey provisioned result
    SingleYubiKeyProvisioned(Result<(String, String), (String, String)>),

    // ============================================================================
    // Section Toggles
    // ============================================================================
    /// Toggle YubiKey section visibility
    ToggleYubiKeySection,
    /// Toggle YubiKey slot management section
    ToggleYubiKeySlotSection,

    // ============================================================================
    // Slot Management
    // ============================================================================
    /// Select YubiKey for management (by serial)
    SelectYubiKeyForManagement(String),
    /// Select PIV slot
    SelectPivSlot(PivSlot),
    /// Query slots for a specific YubiKey
    QueryYubiKeySlots(String),
    /// Query slots result
    YubiKeySlotsQueried(Result<(String, Vec<SlotInfo>), String>),
    /// Clear a specific slot
    ClearYubiKeySlot { serial: String, slot: PivSlot },
    /// Slot cleared result
    YubiKeySlotCleared(Result<(String, PivSlot), String>),
    /// Generate key in a specific slot
    GenerateKeyInSlot { serial: String, slot: PivSlot },
    /// Key generated in slot result
    KeyInSlotGenerated(Result<(String, PivSlot, String), String>),

    // ============================================================================
    // PIN Management
    // ============================================================================
    /// PIN input changed
    YubiKeyPinInputChanged(String),
    /// New PIN changed
    YubiKeyNewPinChanged(String),
    /// PIN confirmation changed
    YubiKeyPinConfirmChanged(String),
    /// Verify PIN
    VerifyYubiKeyPin(String),
    /// PIN verified result
    YubiKeyPinVerified(Result<(String, bool), String>),

    // ============================================================================
    // Management Key Operations
    // ============================================================================
    /// Management key changed (current)
    YubiKeyManagementKeyChanged(String),
    /// New management key changed
    YubiKeyNewManagementKeyChanged(String),
    /// Change management key for YubiKey
    ChangeYubiKeyManagementKey(String),
    /// Management key changed result
    YubiKeyManagementKeyChanged2(Result<String, String>),

    // ============================================================================
    // PIV Reset
    // ============================================================================
    /// Factory reset PIV for YubiKey
    ResetYubiKeyPiv(String),
    /// PIV reset result
    YubiKeyPivReset(Result<String, String>),

    // ============================================================================
    // Attestation
    // ============================================================================
    /// Get attestation for a slot
    GetYubiKeyAttestation { serial: String, slot: PivSlot },
    /// Attestation received result
    YubiKeyAttestationReceived(Result<(String, String), String>),

    // ============================================================================
    // Domain Registration and Lifecycle
    // ============================================================================
    /// Register YubiKey in domain
    RegisterYubiKeyInDomain { serial: String, name: String },
    /// YubiKey registered result
    YubiKeyRegistered(Result<(String, Uuid), String>),
    /// Transfer YubiKey between persons
    TransferYubiKey {
        serial: String,
        from_person_id: Uuid,
        to_person_id: Uuid,
    },
    /// YubiKey transferred result
    YubiKeyTransferred(Result<(String, Uuid, Uuid), String>),
    /// Revoke YubiKey assignment
    RevokeYubiKeyAssignment { serial: String },
    /// Assignment revoked result
    YubiKeyAssignmentRevoked(Result<String, String>),
    /// Registration name changed
    YubiKeyRegistrationNameChanged(String),

    // ============================================================================
    // Graph-Based Operations
    // ============================================================================
    /// YubiKey data loaded from projection
    YubiKeyDataLoaded(Vec<YubiKeyEntry>, Vec<PersonEntry>),
    /// Provision YubiKeys from graph
    ProvisionYubiKeysFromGraph,
    /// YubiKeys provisioned from graph result
    YubiKeysProvisioned(
        Result<Vec<(graph::ConceptEntity, iced::Point, iced::Color, String, Uuid)>, String>,
    ),

    // ============================================================================
    // Filter Toggle
    // ============================================================================
    /// Toggle YubiKey filter in graph view
    ToggleFilterYubiKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yubikey_message_variants() {
        let _ = YubiKeyMessage::DetectYubiKeys;
        let _ = YubiKeyMessage::YubiKeysDetected(Ok(vec![]));
        let _ = YubiKeyMessage::YubiKeySerialChanged("12345678".to_string());
        let _ = YubiKeyMessage::SelectYubiKeyForAssignment("12345678".to_string());
        let _ = YubiKeyMessage::AssignYubiKeyToPerson { serial: "12345678".to_string(), person_id: Uuid::nil() };
        let _ = YubiKeyMessage::ProvisionYubiKey;
        let _ = YubiKeyMessage::ProvisionSingleYubiKey { serial: "12345678".to_string() };
        let _ = YubiKeyMessage::ToggleYubiKeySection;
        let _ = YubiKeyMessage::ToggleYubiKeySlotSection;
        let _ = YubiKeyMessage::SelectYubiKeyForManagement("12345678".to_string());
        let _ = YubiKeyMessage::SelectPivSlot(PivSlot::Authentication);
        let _ = YubiKeyMessage::YubiKeyPinInputChanged("123456".to_string());
        let _ = YubiKeyMessage::YubiKeyNewPinChanged("654321".to_string());
        let _ = YubiKeyMessage::YubiKeyPinConfirmChanged("654321".to_string());
        let _ = YubiKeyMessage::YubiKeyManagementKeyChanged("key".to_string());
        let _ = YubiKeyMessage::YubiKeyNewManagementKeyChanged("newkey".to_string());
        let _ = YubiKeyMessage::YubiKeyRegistrationNameChanged("My Key".to_string());
        let _ = YubiKeyMessage::ToggleFilterYubiKey;
    }
}
