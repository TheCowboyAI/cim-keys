// Copyright (c) 2025 - Cowboy AI, LLC.

//! YubiKey Bounded Context
//!
//! This module provides the YubiKey bounded context for cim-keys.
//! It includes YubiKey device management, PIV slots, and hardware token operations.
//!
//! ## Domain Types
//!
//! **YubiKey Hardware**:
//! - YubiKey devices with serial numbers
//! - PIV (Personal Identity Verification) slots
//! - Hardware-backed key storage
//!
//! **PIV Slots**:
//! - 9A: Authentication
//! - 9C: Digital Signature
//! - 9D: Key Management
//! - 9E: Card Authentication
//! - 82-95: Retired Key Management
//!
//! ## Bounded Context Separation
//!
//! This context is responsible for:
//! - YubiKey device inventory
//! - PIV slot allocation
//! - Hardware token provisioning
//! - Physical security of keys
//!
//! It does NOT handle:
//! - Organizational structure (see `organization` context)
//! - Certificate PKI (see `pki` context)
//! - NATS credentials (see `nats` context)
//!
//! ## YubiKey Role Mapping
//!
//! ```text
//! RootAuthority -> YubiKey with Root CA key (Slot 9C)
//! SecurityAdmin -> YubiKey with Intermediate CA (Slot 9C)
//! Developer -> YubiKey with personal signing key (Slot 9A, 9C)
//! Auditor -> YubiKey with authentication only (Slot 9A)
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export phantom-typed IDs for YubiKey
pub use super::ids::{
    YubiKeyDeviceId,
    YubiKeyMarker,
    SlotId,
    SlotMarker,
};

// Re-export YubiKey types from bootstrap
pub use super::bootstrap::{
    YubiKeyConfig,
    YubiKeyRole,
    PivConfig,
    PivAlgorithm,
    PgpConfig,
    FidoConfig,
    SslConfig,
};

/// PIV slot on a YubiKey
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PIVSlot {
    /// Authentication (9A) - Used for system login
    Authentication,
    /// Digital Signature (9C) - Used for signing
    Signature,
    /// Key Management (9D) - Used for encryption/decryption
    KeyManagement,
    /// Card Authentication (9E) - Used for physical access
    CardAuth,
    /// Retired Key Management slots (82-95)
    Retired(u8),
}

impl PIVSlot {
    /// Get the hex slot ID
    pub fn hex_id(&self) -> String {
        match self {
            PIVSlot::Authentication => "9A".to_string(),
            PIVSlot::Signature => "9C".to_string(),
            PIVSlot::KeyManagement => "9D".to_string(),
            PIVSlot::CardAuth => "9E".to_string(),
            PIVSlot::Retired(n) => format!("{:02X}", 0x82 + n),
        }
    }

    /// Get display name for the slot
    pub fn display_name(&self) -> &'static str {
        match self {
            PIVSlot::Authentication => "Authentication (9A)",
            PIVSlot::Signature => "Digital Signature (9C)",
            PIVSlot::KeyManagement => "Key Management (9D)",
            PIVSlot::CardAuth => "Card Authentication (9E)",
            PIVSlot::Retired(_) => "Retired Key Management",
        }
    }

    /// Check if this slot is typically used for signing
    pub fn is_signing_slot(&self) -> bool {
        matches!(self, PIVSlot::Signature)
    }

    /// Check if this slot is typically used for authentication
    pub fn is_auth_slot(&self) -> bool {
        matches!(self, PIVSlot::Authentication | PIVSlot::CardAuth)
    }
}

/// YubiKey device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyInfo {
    pub id: YubiKeyDeviceId,
    pub serial: String,
    pub name: String,
    pub owner_person_id: Option<Uuid>,
    pub role: YubiKeyRole,
    pub firmware_version: Option<String>,
    pub form_factor: YubiKeyFormFactor,
    pub slots: Vec<SlotInfo>,
}

impl YubiKeyInfo {
    /// Create new YubiKey info
    pub fn new(serial: String, name: String, role: YubiKeyRole) -> Self {
        Self {
            id: YubiKeyDeviceId::new(),
            serial,
            name,
            owner_person_id: None,
            role,
            firmware_version: None,
            form_factor: YubiKeyFormFactor::UsbA,
            slots: Vec::new(),
        }
    }

    /// Associate with an owner
    pub fn with_owner(mut self, person_id: Uuid) -> Self {
        self.owner_person_id = Some(person_id);
        self
    }

    /// Get a slot by PIV slot type
    pub fn get_slot(&self, slot: PIVSlot) -> Option<&SlotInfo> {
        self.slots.iter().find(|s| s.piv_slot == slot)
    }

    /// Check if a slot is available for new key
    pub fn is_slot_available(&self, slot: PIVSlot) -> bool {
        self.get_slot(slot).map(|s| !s.has_key).unwrap_or(true)
    }
}

/// YubiKey form factor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YubiKeyFormFactor {
    /// USB-A connector
    UsbA,
    /// USB-C connector
    UsbC,
    /// Nano form factor (USB-A)
    NanoA,
    /// Nano form factor (USB-C)
    NanoC,
    /// Bio series with fingerprint
    Bio,
    /// Security Key series
    SecurityKey,
}

/// Information about a PIV slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub id: SlotId,
    pub piv_slot: PIVSlot,
    pub has_key: bool,
    pub key_id: Option<Uuid>,
    pub certificate_id: Option<Uuid>,
    pub algorithm: Option<String>,
    pub touch_policy: TouchPolicy,
    pub pin_policy: PinPolicy,
}

impl SlotInfo {
    /// Create new empty slot info
    pub fn new(piv_slot: PIVSlot) -> Self {
        Self {
            id: SlotId::new(),
            piv_slot,
            has_key: false,
            key_id: None,
            certificate_id: None,
            algorithm: None,
            touch_policy: TouchPolicy::Default,
            pin_policy: PinPolicy::Default,
        }
    }

    /// Create slot info with key
    pub fn with_key(mut self, key_id: Uuid, algorithm: String) -> Self {
        self.has_key = true;
        self.key_id = Some(key_id);
        self.algorithm = Some(algorithm);
        self
    }
}

/// YubiKey touch policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TouchPolicy {
    /// Use default behavior
    #[default]
    Default,
    /// Never require touch
    Never,
    /// Always require touch
    Always,
    /// Cache touch for 15 seconds
    Cached,
}

/// YubiKey PIN policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PinPolicy {
    /// Use default behavior
    #[default]
    Default,
    /// Never require PIN
    Never,
    /// Require PIN once per session
    Once,
    /// Always require PIN
    Always,
}

impl std::fmt::Display for PIVSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::fmt::Display for YubiKeyFormFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            YubiKeyFormFactor::UsbA => write!(f, "USB-A"),
            YubiKeyFormFactor::UsbC => write!(f, "USB-C"),
            YubiKeyFormFactor::NanoA => write!(f, "Nano USB-A"),
            YubiKeyFormFactor::NanoC => write!(f, "Nano USB-C"),
            YubiKeyFormFactor::Bio => write!(f, "Bio"),
            YubiKeyFormFactor::SecurityKey => write!(f, "Security Key"),
        }
    }
}

impl std::fmt::Display for TouchPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TouchPolicy::Default => write!(f, "Default"),
            TouchPolicy::Never => write!(f, "Never"),
            TouchPolicy::Always => write!(f, "Always"),
            TouchPolicy::Cached => write!(f, "Cached"),
        }
    }
}

impl std::fmt::Display for PinPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PinPolicy::Default => write!(f, "Default"),
            PinPolicy::Never => write!(f, "Never"),
            PinPolicy::Once => write!(f, "Once"),
            PinPolicy::Always => write!(f, "Always"),
        }
    }
}

// ============================================================================
// GRAPH NODE TYPES (for DomainNode visualization)
// ============================================================================

use chrono::{DateTime, Utc};

/// YubiKey device node for graph visualization
///
/// This type is used in `DomainNodeData` for rendering YubiKeys in the graph.
/// It uses phantom-typed `YubiKeyDeviceId` for compile-time safety.
#[derive(Debug, Clone)]
pub struct YubiKeyNode {
    pub id: YubiKeyDeviceId,
    pub serial: String,
    pub version: String,
    pub provisioned_at: Option<DateTime<Utc>>,
    pub slots_used: Vec<String>,
}

impl YubiKeyNode {
    /// Create a new YubiKey node
    pub fn new(
        id: YubiKeyDeviceId,
        serial: String,
        version: String,
        provisioned_at: Option<DateTime<Utc>>,
        slots_used: Vec<String>,
    ) -> Self {
        Self { id, serial, version, provisioned_at, slots_used }
    }

    /// Check if YubiKey has been provisioned
    pub fn is_provisioned(&self) -> bool {
        self.provisioned_at.is_some()
    }

    /// Get the number of slots in use
    pub fn slots_in_use(&self) -> usize {
        self.slots_used.len()
    }
}

/// PIV slot node for graph visualization
///
/// This type is used in `DomainNodeData` for rendering PIV slots in the graph.
/// It uses phantom-typed `SlotId` for compile-time safety.
#[derive(Debug, Clone)]
pub struct PivSlotNode {
    pub id: SlotId,
    pub slot_name: String,
    pub yubikey_serial: String,
    pub has_key: bool,
    pub certificate_subject: Option<String>,
}

impl PivSlotNode {
    /// Create a new PIV slot node
    pub fn new(
        id: SlotId,
        slot_name: String,
        yubikey_serial: String,
        has_key: bool,
        certificate_subject: Option<String>,
    ) -> Self {
        Self { id, slot_name, yubikey_serial, has_key, certificate_subject }
    }

    /// Check if slot is empty
    pub fn is_empty(&self) -> bool {
        !self.has_key
    }
}

/// YubiKey provisioning status node for graph visualization
///
/// Shows the provisioning status of a YubiKey for a specific person.
#[derive(Debug, Clone)]
pub struct YubiKeyStatusNode {
    pub person_id: Uuid,
    pub yubikey_serial: Option<String>,
    pub slots_provisioned: Vec<PIVSlot>,
    pub slots_needed: Vec<PIVSlot>,
}

impl YubiKeyStatusNode {
    /// Create a new YubiKey status node
    pub fn new(
        person_id: Uuid,
        yubikey_serial: Option<String>,
        slots_provisioned: Vec<PIVSlot>,
        slots_needed: Vec<PIVSlot>,
    ) -> Self {
        Self { person_id, yubikey_serial, slots_provisioned, slots_needed }
    }

    /// Check if all needed slots are provisioned
    pub fn is_fully_provisioned(&self) -> bool {
        self.slots_needed.is_empty()
    }

    /// Get the number of slots still needed
    pub fn pending_slots(&self) -> usize {
        self.slots_needed.len()
    }

    /// Check if person has a YubiKey assigned
    pub fn has_yubikey(&self) -> bool {
        self.yubikey_serial.is_some()
    }
}
