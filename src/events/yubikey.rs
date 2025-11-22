//! YubiKey Aggregate Events
//!
//! Events related to the YubiKey aggregate root.
//! YubiKeys are hardware security modules used for key storage and operations.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import shared types from legacy module
use crate::events_legacy::{YubiKeySlot, KeyAlgorithm, KeyPurpose};

/// Events for the YubiKey aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum YubiKeyEvents {
    /// A YubiKey was detected
    YubiKeyDetected(YubiKeyDetectedEvent),

    /// A YubiKey was provisioned
    YubiKeyProvisioned(YubiKeyProvisionedEvent),

    /// YubiKey PIN was configured
    PinConfigured(PinConfiguredEvent),

    /// YubiKey PUK was configured
    PukConfigured(PukConfiguredEvent),

    /// YubiKey management key was rotated
    ManagementKeyRotated(ManagementKeyRotatedEvent),

    /// YubiKey slot allocation was planned
    SlotAllocationPlanned(SlotAllocationPlannedEvent),

    /// Key was generated in YubiKey slot
    KeyGeneratedInSlot(KeyGeneratedInSlotEvent),

    /// Certificate was imported to YubiKey slot
    CertificateImportedToSlot(CertificateImportedToSlotEvent),
}

/// A YubiKey was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyDetectedEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub firmware_version: String,
    pub detected_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A YubiKey was provisioned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyProvisionedEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub slots_configured: Vec<YubiKeySlot>,
    pub provisioned_at: DateTime<Utc>,
    pub provisioned_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// YubiKey PIN was configured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinConfiguredEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub pin_hash: String,
    pub retry_count: u8,
    pub configured_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// YubiKey PUK was configured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PukConfiguredEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub puk_hash: String,
    pub retry_count: u8,
    pub configured_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// YubiKey management key was rotated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementKeyRotatedEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub algorithm: String,
    pub rotated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// YubiKey slot allocation was planned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotAllocationPlannedEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub slot: String,
    pub purpose: KeyPurpose,
    pub person_id: Uuid,
    pub planned_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Key was generated in YubiKey slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGeneratedInSlotEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub slot: String,
    pub key_id: Uuid,
    pub algorithm: KeyAlgorithm,
    pub public_key: Vec<u8>,
    pub generated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Certificate was imported to YubiKey slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateImportedToSlotEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub slot: String,
    pub cert_id: Uuid,
    pub imported_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for YubiKeyEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            YubiKeyEvents::YubiKeyDetected(e) => e.event_id,
            YubiKeyEvents::YubiKeyProvisioned(e) => e.event_id,
            YubiKeyEvents::PinConfigured(e) => e.event_id,
            YubiKeyEvents::PukConfigured(e) => e.event_id,
            YubiKeyEvents::ManagementKeyRotated(e) => e.event_id,
            YubiKeyEvents::SlotAllocationPlanned(e) => e.event_id,
            YubiKeyEvents::KeyGeneratedInSlot(e) => e.event_id,
            YubiKeyEvents::CertificateImportedToSlot(e) => e.event_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            YubiKeyEvents::YubiKeyDetected(_) => "YubiKeyDetected",
            YubiKeyEvents::YubiKeyProvisioned(_) => "YubiKeyProvisioned",
            YubiKeyEvents::PinConfigured(_) => "PinConfigured",
            YubiKeyEvents::PukConfigured(_) => "PukConfigured",
            YubiKeyEvents::ManagementKeyRotated(_) => "ManagementKeyRotated",
            YubiKeyEvents::SlotAllocationPlanned(_) => "SlotAllocationPlanned",
            YubiKeyEvents::KeyGeneratedInSlot(_) => "KeyGeneratedInSlot",
            YubiKeyEvents::CertificateImportedToSlot(_) => "CertificateImportedToSlot",
        }
    }
}
