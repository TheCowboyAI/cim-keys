//! Key Aggregate Events
//!
//! Events related to the Key aggregate root.
//! Keys represent cryptographic key material (public/private keypairs).
//!
//! ## Value Object Migration
//!
//! Events use dual-path fields for backward compatibility:
//! - Old string fields are kept for deserializing existing events
//! - New typed fields use Option<T> for gradual migration
//! - Accessor methods prefer typed fields, fall back to strings

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import shared domain ontologies
use crate::types::{
    KeyAlgorithm, KeyPurpose, KeyMetadata,
    ImportSource, KeyFormat, ExportDestination, RevocationReason,
};
// Import from domain
use crate::domain::KeyOwnership;
// Import value objects
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
///
/// This event uses dual-path fields for backward compatibility:
/// - Legacy string field (`generated_by`) for existing events
/// - Typed value object (`generated_by_actor`) for new events
///
/// Use the accessor method `generated_by_actor()` which prefers
/// typed field and falls back to parsing legacy string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGeneratedEvent {
    pub key_id: Uuid,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub generated_at: DateTime<Utc>,

    // ========================================================================
    // Legacy fields (deprecated, kept for backward compatibility)
    // ========================================================================

    /// Legacy: Actor who generated the key (use generated_by_actor instead)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use generated_by_actor field instead")]
    pub generated_by: String,

    // ========================================================================
    // Typed value object fields (preferred)
    // ========================================================================

    /// Typed: Actor who generated the key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_by_actor: Option<ActorId>,

    // ========================================================================
    // Other fields
    // ========================================================================

    pub hardware_backed: bool,
    pub metadata: KeyMetadata,
    pub ownership: Option<KeyOwnership>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl KeyGeneratedEvent {
    /// Create a new event using legacy string field (for backward compatibility)
    ///
    /// Use this when migrating existing code. New code should use `new_typed()`.
    #[allow(clippy::too_many_arguments)]
    pub fn new_legacy(
        key_id: Uuid,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        generated_at: DateTime<Utc>,
        generated_by: String,
        hardware_backed: bool,
        metadata: KeyMetadata,
        ownership: Option<KeyOwnership>,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
    ) -> Self {
        Self {
            key_id,
            algorithm,
            purpose,
            generated_at,
            generated_by,
            generated_by_actor: None,
            hardware_backed,
            metadata,
            ownership,
            correlation_id,
            causation_id,
        }
    }

    /// Create a new event using typed value objects (preferred for new code)
    #[allow(clippy::too_many_arguments)]
    pub fn new_typed(
        key_id: Uuid,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        generated_at: DateTime<Utc>,
        generated_by_actor: ActorId,
        hardware_backed: bool,
        metadata: KeyMetadata,
        ownership: Option<KeyOwnership>,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
    ) -> Self {
        Self {
            key_id,
            algorithm,
            purpose,
            generated_at,
            // Legacy field populated for backward compat serialization
            generated_by: generated_by_actor.to_legacy_string(),
            generated_by_actor: Some(generated_by_actor),
            hardware_backed,
            metadata,
            ownership,
            correlation_id,
            causation_id,
        }
    }

    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn generated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.generated_by_actor {
            return actor.clone();
        }
        // Fall back to parsing legacy string
        ActorId::parse(&self.generated_by)
    }

    /// Check if this event uses typed value objects (new format)
    pub fn uses_typed_fields(&self) -> bool {
        self.generated_by_actor.is_some()
    }
}

/// A key was imported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyImportedEvent {
    pub key_id: Uuid,
    pub source: ImportSource,
    pub format: KeyFormat,
    pub imported_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use imported_by_actor field instead")]
    pub imported_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imported_by_actor: Option<ActorId>,

    pub metadata: KeyMetadata,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl KeyImportedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn imported_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.imported_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.imported_by)
    }
}

/// A key was exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExportedEvent {
    pub key_id: Uuid,
    pub format: KeyFormat,
    pub include_private: bool,
    pub exported_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use exported_by_actor field instead")]
    pub exported_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exported_by_actor: Option<ActorId>,

    pub destination: ExportDestination,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl KeyExportedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn exported_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.exported_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.exported_by)
    }
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

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use revoked_by_actor field instead")]
    pub revoked_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl KeyRevokedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn revoked_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.revoked_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.revoked_by)
    }
}

/// Key rotation was initiated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationInitiatedEvent {
    pub rotation_id: Uuid,
    pub old_key_id: Uuid,
    pub new_key_id: Uuid,
    pub rotation_reason: String,
    pub initiated_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use initiated_by_actor field instead")]
    pub initiated_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiated_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl KeyRotationInitiatedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn initiated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.initiated_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.initiated_by)
    }
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
    pub key_type: String, // e.g., "rsa", "ed25519"
    pub comment: String,
    pub generated_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use generated_by_actor field instead")]
    pub generated_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl SshKeyGeneratedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn generated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.generated_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.generated_by)
    }
}

/// GPG key was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeyGeneratedEvent {
    pub key_id: Uuid,
    pub fingerprint: String,
    pub user_id: String,
    pub key_type: String,
    pub generated_at: DateTime<Utc>,

    // Legacy field (deprecated)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[deprecated(note = "Use generated_by_actor field instead")]
    pub generated_by: String,

    // Typed field (preferred)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_by_actor: Option<ActorId>,

    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

#[allow(deprecated)]
impl GpgKeyGeneratedEvent {
    /// Get ActorId, preferring typed field, falling back to parsing legacy string
    pub fn generated_by_value_object(&self) -> ActorId {
        if let Some(ref actor) = self.generated_by_actor {
            return actor.clone();
        }
        ActorId::parse(&self.generated_by)
    }
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
