// Copyright (c) 2025 - Cowboy AI, LLC.

//! YubiKey Management Module
//!
//! Handles all YubiKey-related operations including:
//! - Device detection and enumeration
//! - Person assignment
//! - PIV slot management (9A, 9C, 9D, 9E)
//! - PIN and management key operations
//! - Domain registration and lifecycle
//! - Attestation retrieval
//! - Key generation in slots

use iced::Task;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::YubiKeyConfig;
use crate::ports::yubikey::{PivSlot, YubiKeyDevice};
use crate::projections::{PersonEntry, YubiKeyEntry};

use super::super::{graph, Message, SlotInfo};

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

/// YubiKey domain state
///
/// Holds all state relevant to the YubiKey bounded context.
/// Field names match CimKeysApp for easy state synchronization.
#[derive(Debug, Clone, Default)]
pub struct YubiKeyState {
    // Device detection
    pub yubikey_serial: String,
    pub detected_yubikeys: Vec<YubiKeyDevice>,
    pub yubikey_detection_status: String,

    // Configuration and assignments
    pub yubikey_configs: Vec<YubiKeyConfig>,
    pub yubikey_assignments: HashMap<String, Uuid>,
    pub yubikey_provisioning_status: HashMap<String, String>,
    pub selected_yubikey_for_assignment: Option<String>,

    // Section visibility
    pub yubikey_section_collapsed: bool,
    pub yubikey_slot_section_collapsed: bool,

    // Slot management
    pub selected_yubikey_for_management: Option<String>,
    pub selected_piv_slot: Option<PivSlot>,
    pub yubikey_slot_info: HashMap<String, Vec<SlotInfo>>,
    pub yubikey_slot_operation_status: Option<String>,
    pub yubikey_attestation_result: Option<String>,

    // PIN management
    pub yubikey_pin_input: String,
    pub yubikey_new_pin: String,
    pub yubikey_pin_confirm: String,

    // Management key
    pub yubikey_management_key: String,
    pub yubikey_new_management_key: String,

    // Domain registration
    pub yubikey_registration_name: String,
    pub registered_yubikeys: HashMap<String, Uuid>,

    // Filter
    pub filter_show_yubikey: bool,
}

impl YubiKeyState {
    /// Create new YubiKey state with default values
    pub fn new() -> Self {
        Self {
            yubikey_detection_status: "Click 'Detect YubiKeys' to scan for hardware".to_string(),
            filter_show_yubikey: true,
            ..Default::default()
        }
    }

    /// Check if PIN is valid (6-8 digits)
    pub fn is_pin_valid(&self) -> bool {
        let len = self.yubikey_new_pin.len();
        let all_digits = self.yubikey_new_pin.chars().all(|c| c.is_ascii_digit());
        let matches = self.yubikey_new_pin == self.yubikey_pin_confirm;
        len >= 6 && len <= 8 && all_digits && matches
    }

    /// Check if management key is valid (48 hex chars = 24 bytes)
    pub fn is_management_key_valid(&self) -> bool {
        let len = self.yubikey_new_management_key.len();
        let all_hex = self
            .yubikey_new_management_key
            .chars()
            .all(|c| c.is_ascii_hexdigit());
        len == 48 && all_hex
    }

    /// Get provisioning status for a serial
    pub fn get_provisioning_status(&self, serial: &str) -> Option<&String> {
        self.yubikey_provisioning_status.get(serial)
    }
}

/// Update function for YubiKey messages
///
/// Handles simple state mutations. Complex operations that need
/// port access return Task::none() and are handled by the main
/// update() in gui.rs.
pub fn update(state: &mut YubiKeyState, message: YubiKeyMessage) -> Task<Message> {
    match message {
        // ========================================================================
        // Device Detection
        // ========================================================================
        YubiKeyMessage::DetectYubiKeys => {
            state.yubikey_detection_status = "Detecting YubiKeys...".to_string();
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::YubiKeysDetected(result) => {
            match result {
                Ok(devices) => {
                    state.yubikey_detection_status =
                        format!("Found {} YubiKey(s)", devices.len());
                    state.detected_yubikeys = devices;
                }
                Err(e) => {
                    state.yubikey_detection_status = format!("Detection error: {}", e);
                    state.detected_yubikeys.clear();
                }
            }
            Task::none()
        }
        YubiKeyMessage::YubiKeySerialChanged(serial) => {
            state.yubikey_serial = serial;
            Task::none()
        }

        // ========================================================================
        // Person Assignment
        // ========================================================================
        YubiKeyMessage::SelectYubiKeyForAssignment(serial) => {
            state.selected_yubikey_for_assignment = Some(serial);
            Task::none()
        }
        YubiKeyMessage::AssignYubiKeyToPerson { serial, person_id } => {
            state.yubikey_assignments.insert(serial, person_id);
            Task::none()
        }

        // ========================================================================
        // Provisioning
        // ========================================================================
        YubiKeyMessage::ProvisionYubiKey => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::ProvisionSingleYubiKey { .. } => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::YubiKeyProvisioned(result) => {
            match result {
                Ok(serial) => {
                    state
                        .yubikey_provisioning_status
                        .insert(serial.clone(), "Provisioned successfully".to_string());
                }
                Err(e) => {
                    state.yubikey_slot_operation_status =
                        Some(format!("Provisioning failed: {}", e));
                }
            }
            Task::none()
        }
        YubiKeyMessage::SingleYubiKeyProvisioned(result) => {
            match result {
                Ok((serial, status)) => {
                    state.yubikey_provisioning_status.insert(serial, status);
                }
                Err((serial, error)) => {
                    state
                        .yubikey_provisioning_status
                        .insert(serial, format!("Error: {}", error));
                }
            }
            Task::none()
        }

        // ========================================================================
        // Section Toggles
        // ========================================================================
        YubiKeyMessage::ToggleYubiKeySection => {
            state.yubikey_section_collapsed = !state.yubikey_section_collapsed;
            Task::none()
        }
        YubiKeyMessage::ToggleYubiKeySlotSection => {
            state.yubikey_slot_section_collapsed = !state.yubikey_slot_section_collapsed;
            Task::none()
        }

        // ========================================================================
        // Slot Management
        // ========================================================================
        YubiKeyMessage::SelectYubiKeyForManagement(serial) => {
            state.selected_yubikey_for_management = Some(serial);
            Task::none()
        }
        YubiKeyMessage::SelectPivSlot(slot) => {
            state.selected_piv_slot = Some(slot);
            Task::none()
        }
        YubiKeyMessage::QueryYubiKeySlots(_) => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::YubiKeySlotsQueried(result) => {
            match result {
                Ok((serial, slots)) => {
                    state.yubikey_slot_info.insert(serial, slots);
                    state.yubikey_slot_operation_status = Some("Slots queried successfully".to_string());
                }
                Err(e) => {
                    state.yubikey_slot_operation_status = Some(format!("Query failed: {}", e));
                }
            }
            Task::none()
        }
        YubiKeyMessage::ClearYubiKeySlot { .. } => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::YubiKeySlotCleared(result) => {
            match result {
                Ok((serial, slot)) => {
                    state.yubikey_slot_operation_status =
                        Some(format!("Slot {:?} cleared on {}", slot, serial));
                }
                Err(e) => {
                    state.yubikey_slot_operation_status = Some(format!("Clear failed: {}", e));
                }
            }
            Task::none()
        }
        YubiKeyMessage::GenerateKeyInSlot { .. } => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::KeyInSlotGenerated(result) => {
            match result {
                Ok((serial, slot, pub_key)) => {
                    state.yubikey_slot_operation_status = Some(format!(
                        "Key generated in {:?} on {}: {}",
                        slot,
                        serial,
                        &pub_key[..20.min(pub_key.len())]
                    ));
                }
                Err(e) => {
                    state.yubikey_slot_operation_status =
                        Some(format!("Key generation failed: {}", e));
                }
            }
            Task::none()
        }

        // ========================================================================
        // PIN Management
        // ========================================================================
        YubiKeyMessage::YubiKeyPinInputChanged(pin) => {
            state.yubikey_pin_input = pin;
            Task::none()
        }
        YubiKeyMessage::YubiKeyNewPinChanged(pin) => {
            state.yubikey_new_pin = pin;
            Task::none()
        }
        YubiKeyMessage::YubiKeyPinConfirmChanged(pin) => {
            state.yubikey_pin_confirm = pin;
            Task::none()
        }
        YubiKeyMessage::VerifyYubiKeyPin(_) => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::YubiKeyPinVerified(result) => {
            match result {
                Ok((serial, valid)) => {
                    state.yubikey_slot_operation_status = Some(format!(
                        "PIN {} for {}",
                        if valid { "verified" } else { "invalid" },
                        serial
                    ));
                }
                Err(e) => {
                    state.yubikey_slot_operation_status =
                        Some(format!("PIN verification failed: {}", e));
                }
            }
            Task::none()
        }

        // ========================================================================
        // Management Key Operations
        // ========================================================================
        YubiKeyMessage::YubiKeyManagementKeyChanged(key) => {
            state.yubikey_management_key = key;
            Task::none()
        }
        YubiKeyMessage::YubiKeyNewManagementKeyChanged(key) => {
            state.yubikey_new_management_key = key;
            Task::none()
        }
        YubiKeyMessage::ChangeYubiKeyManagementKey(_) => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::YubiKeyManagementKeyChanged2(result) => {
            match result {
                Ok(serial) => {
                    state.yubikey_slot_operation_status =
                        Some(format!("Management key changed for {}", serial));
                    state.yubikey_management_key.clear();
                    state.yubikey_new_management_key.clear();
                }
                Err(e) => {
                    state.yubikey_slot_operation_status =
                        Some(format!("Management key change failed: {}", e));
                }
            }
            Task::none()
        }

        // ========================================================================
        // PIV Reset
        // ========================================================================
        YubiKeyMessage::ResetYubiKeyPiv(_) => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::YubiKeyPivReset(result) => {
            match result {
                Ok(serial) => {
                    state.yubikey_slot_operation_status =
                        Some(format!("PIV reset for {} - all slots cleared", serial));
                    // Clear slot info for this device
                    state.yubikey_slot_info.remove(&serial);
                }
                Err(e) => {
                    state.yubikey_slot_operation_status = Some(format!("PIV reset failed: {}", e));
                }
            }
            Task::none()
        }

        // ========================================================================
        // Attestation
        // ========================================================================
        YubiKeyMessage::GetYubiKeyAttestation { .. } => {
            // Handled by main update - needs port access
            Task::none()
        }
        YubiKeyMessage::YubiKeyAttestationReceived(result) => {
            match result {
                Ok((_serial, attestation)) => {
                    state.yubikey_attestation_result = Some(attestation);
                }
                Err(e) => {
                    state.yubikey_attestation_result = Some(format!("Attestation failed: {}", e));
                }
            }
            Task::none()
        }

        // ========================================================================
        // Domain Registration and Lifecycle
        // ========================================================================
        YubiKeyMessage::RegisterYubiKeyInDomain { .. } => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        YubiKeyMessage::YubiKeyRegistered(result) => {
            match result {
                Ok((serial, id)) => {
                    state.registered_yubikeys.insert(serial, id);
                    state.yubikey_registration_name.clear();
                }
                Err(_) => {}
            }
            Task::none()
        }
        YubiKeyMessage::TransferYubiKey { .. } => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        YubiKeyMessage::YubiKeyTransferred(result) => {
            if let Ok((serial, _from, to)) = result {
                state.yubikey_assignments.insert(serial, to);
            }
            Task::none()
        }
        YubiKeyMessage::RevokeYubiKeyAssignment { .. } => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        YubiKeyMessage::YubiKeyAssignmentRevoked(result) => {
            if let Ok(serial) = result {
                state.yubikey_assignments.remove(&serial);
            }
            Task::none()
        }
        YubiKeyMessage::YubiKeyRegistrationNameChanged(name) => {
            state.yubikey_registration_name = name;
            Task::none()
        }

        // ========================================================================
        // Graph-Based Operations
        // ========================================================================
        YubiKeyMessage::YubiKeyDataLoaded(_, _) => {
            // Handled by main update - needs graph access
            Task::none()
        }
        YubiKeyMessage::ProvisionYubiKeysFromGraph => {
            // Handled by main update - needs graph and port access
            Task::none()
        }
        YubiKeyMessage::YubiKeysProvisioned(_) => {
            // Handled by main update - needs graph access
            Task::none()
        }

        // ========================================================================
        // Filter Toggle
        // ========================================================================
        YubiKeyMessage::ToggleFilterYubiKey => {
            state.filter_show_yubikey = !state.filter_show_yubikey;
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yubikey_state_default() {
        let state = YubiKeyState::default();
        assert!(state.detected_yubikeys.is_empty());
        assert!(state.yubikey_assignments.is_empty());
        assert!(!state.yubikey_section_collapsed);
    }

    #[test]
    fn test_yubikey_state_new() {
        let state = YubiKeyState::new();
        assert!(state.yubikey_detection_status.contains("Detect"));
        assert!(state.filter_show_yubikey);
    }

    #[test]
    fn test_pin_validation() {
        let mut state = YubiKeyState::default();

        // Too short
        state.yubikey_new_pin = "12345".to_string();
        state.yubikey_pin_confirm = "12345".to_string();
        assert!(!state.is_pin_valid());

        // Too long
        state.yubikey_new_pin = "123456789".to_string();
        state.yubikey_pin_confirm = "123456789".to_string();
        assert!(!state.is_pin_valid());

        // Non-digits
        state.yubikey_new_pin = "12345a".to_string();
        state.yubikey_pin_confirm = "12345a".to_string();
        assert!(!state.is_pin_valid());

        // Valid (6 digits)
        state.yubikey_new_pin = "123456".to_string();
        state.yubikey_pin_confirm = "123456".to_string();
        assert!(state.is_pin_valid());

        // Valid (8 digits)
        state.yubikey_new_pin = "12345678".to_string();
        state.yubikey_pin_confirm = "12345678".to_string();
        assert!(state.is_pin_valid());

        // Mismatch
        state.yubikey_pin_confirm = "87654321".to_string();
        assert!(!state.is_pin_valid());
    }

    #[test]
    fn test_management_key_validation() {
        let mut state = YubiKeyState::default();

        // Too short
        state.yubikey_new_management_key = "0102030405".to_string();
        assert!(!state.is_management_key_valid());

        // Non-hex
        state.yubikey_new_management_key = "0102030405060708091011121314151617181920212223GGGG".to_string();
        assert!(!state.is_management_key_valid());

        // Valid (48 hex chars = 24 bytes)
        state.yubikey_new_management_key =
            "010203040506070809101112131415161718192021222324".to_string();
        assert!(state.is_management_key_valid());
    }

    #[test]
    fn test_toggle_sections() {
        let mut state = YubiKeyState::new();

        // Verify defaults
        assert!(!state.yubikey_section_collapsed);
        assert!(!state.yubikey_slot_section_collapsed);
        assert!(state.filter_show_yubikey); // Default is true (showing)

        let _ = update(&mut state, YubiKeyMessage::ToggleYubiKeySection);
        assert!(state.yubikey_section_collapsed);

        let _ = update(&mut state, YubiKeyMessage::ToggleYubiKeySlotSection);
        assert!(state.yubikey_slot_section_collapsed);

        let _ = update(&mut state, YubiKeyMessage::ToggleFilterYubiKey);
        assert!(!state.filter_show_yubikey); // Toggled to false
    }

    #[test]
    fn test_assignment() {
        let mut state = YubiKeyState::default();
        let person_id = Uuid::now_v7();

        let _ = update(
            &mut state,
            YubiKeyMessage::AssignYubiKeyToPerson {
                serial: "12345678".to_string(),
                person_id,
            },
        );

        assert_eq!(
            state.yubikey_assignments.get("12345678"),
            Some(&person_id)
        );
    }

    #[test]
    fn test_detection_status_update() {
        let mut state = YubiKeyState::default();

        let _ = update(&mut state, YubiKeyMessage::DetectYubiKeys);
        assert!(state.yubikey_detection_status.contains("Detecting"));

        let devices = vec![YubiKeyDevice {
            serial: "12345678".to_string(),
            version: "5.4.3".to_string(),
            model: "YubiKey 5".to_string(),
            piv_enabled: true,
        }];
        let _ = update(
            &mut state,
            YubiKeyMessage::YubiKeysDetected(Ok(devices.clone())),
        );
        assert!(state.yubikey_detection_status.contains("Found 1"));
        assert_eq!(state.detected_yubikeys.len(), 1);
    }
}
