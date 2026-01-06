//! Key Aggregate Events
//!
//! Events related to the Key aggregate root.
//! Keys represent cryptographic key material (public/private keypairs).

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::types::{
    KeyAlgorithm, KeyPurpose, KeyMetadata,
    ImportSource, KeyFormat, ExportDestination, RevocationReason,
};
use crate::domain::KeyOwnership;
use crate::value_objects::ActorId;

/// Events for the Key aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum KeyEvents {
    /// A new key was generated
    KeyGenerated(KeyGeneratedEvent),

    /// A key was imported from external source
    KeyImported(KeyImportedEvent),

    /// A key was exported
    KeyExported(KeyExportedEvent),

    /// A key was stored in offline partition
    KeyStoredOffline(KeyStoredOfflineEvent),

    /// A key was revoked
    KeyRevoked(KeyRevokedEvent),

    /// Key rotation was initiated
    KeyRotationInitiated(KeyRotationInitiatedEvent),

    /// Key rotation was completed
    KeyRotationCompleted(KeyRotationCompletedEvent),

    /// SSH key was generated
    SshKeyGenerated(SshKeyGeneratedEvent),

    /// GPG key was generated
    GpgKeyGenerated(GpgKeyGeneratedEvent),

    /// TOTP secret was generated
    TotpSecretGenerated(TotpSecretGeneratedEvent),
}

/// A new key was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGeneratedEvent {
    pub key_id: Uuid,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub generated_at: DateTime<Utc>,
    pub generated_by: ActorId,
    pub hardware_backed: bool,
    pub metadata: KeyMetadata,
    pub ownership: Option<KeyOwnership>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A key was imported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyImportedEvent {
    pub key_id: Uuid,
    pub source: ImportSource,
    pub format: KeyFormat,
    pub imported_at: DateTime<Utc>,
    pub imported_by: ActorId,
    pub metadata: KeyMetadata,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A key was exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExportedEvent {
    pub key_id: Uuid,
    pub format: KeyFormat,
    pub include_private: bool,
    pub exported_at: DateTime<Utc>,
    pub exported_by: ActorId,
    pub destination: ExportDestination,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Key stored in offline partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStoredOfflineEvent {
    pub key_id: Uuid,
    pub partition_id: Uuid,
    pub encrypted: bool,
    pub stored_at: DateTime<Utc>,
    pub checksum: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// A key was revoked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRevokedEvent {
    pub key_id: Uuid,
    pub reason: RevocationReason,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: ActorId,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Key rotation was initiated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationInitiatedEvent {
    pub rotation_id: Uuid,
    pub old_key_id: Uuid,
    pub new_key_id: Uuid,
    pub rotation_reason: String,
    pub initiated_at: DateTime<Utc>,
    pub initiated_by: ActorId,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Key rotation was completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationCompletedEvent {
    pub rotation_id: Uuid,
    pub old_key_id: Uuid,
    pub new_key_id: Uuid,
    pub completed_at: DateTime<Utc>,
    pub transition_period_ends: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// SSH key was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyGeneratedEvent {
    pub key_id: Uuid,
    pub key_type: String,
    pub comment: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: ActorId,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// GPG key was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeyGeneratedEvent {
    pub key_id: Uuid,
    pub fingerprint: String,
    pub user_id: String,
    pub key_type: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: ActorId,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// TOTP secret was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecretGeneratedEvent {
    pub secret_id: Uuid,
    pub person_id: Uuid,
    pub algorithm: String,
    pub digits: u8,
    pub period: u32,
    pub generated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for KeyEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            KeyEvents::KeyGenerated(e) => e.key_id,
            KeyEvents::KeyImported(e) => e.key_id,
            KeyEvents::KeyExported(e) => e.key_id,
            KeyEvents::KeyStoredOffline(e) => e.key_id,
            KeyEvents::KeyRevoked(e) => e.key_id,
            KeyEvents::KeyRotationInitiated(e) => e.rotation_id,
            KeyEvents::KeyRotationCompleted(e) => e.rotation_id,
            KeyEvents::SshKeyGenerated(e) => e.key_id,
            KeyEvents::GpgKeyGenerated(e) => e.key_id,
            KeyEvents::TotpSecretGenerated(e) => e.secret_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            KeyEvents::KeyGenerated(_) => "KeyGenerated",
            KeyEvents::KeyImported(_) => "KeyImported",
            KeyEvents::KeyExported(_) => "KeyExported",
            KeyEvents::KeyStoredOffline(_) => "KeyStoredOffline",
            KeyEvents::KeyRevoked(_) => "KeyRevoked",
            KeyEvents::KeyRotationInitiated(_) => "KeyRotationInitiated",
            KeyEvents::KeyRotationCompleted(_) => "KeyRotationCompleted",
            KeyEvents::SshKeyGenerated(_) => "SshKeyGenerated",
            KeyEvents::GpgKeyGenerated(_) => "GpgKeyGenerated",
            KeyEvents::TotpSecretGenerated(_) => "TotpSecretGenerated",
        }
    }
}
