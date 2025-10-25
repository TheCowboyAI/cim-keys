//! Event-sourced key management events
//!
//! All key operations are modeled as immutable events following CIM's FRP principles.
//! No mutable state - only event streams that represent facts about key operations.

use cim_domain::{DomainEvent, CorrelationId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import domain types
use crate::domain::{KeyOwnership, KeyStorageLocation};

/// Base event for all key operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum KeyEvent {
    /// A new key was generated
    KeyGenerated(KeyGeneratedEvent),

    /// A key was imported from external source
    KeyImported(KeyImportedEvent),

    /// A certificate was generated
    CertificateGenerated(CertificateGeneratedEvent),

    /// A certificate was signed
    CertificateSigned(CertificateSignedEvent),

    /// A key was exported
    KeyExported(KeyExportedEvent),

    /// A key was stored in offline partition
    KeyStoredOffline(KeyStoredOfflineEvent),

    /// A YubiKey was provisioned
    YubiKeyProvisioned(YubiKeyProvisionedEvent),

    /// SSH key was created
    SshKeyGenerated(SshKeyGeneratedEvent),

    /// GPG key was created
    GpgKeyGenerated(GpgKeyGeneratedEvent),

    /// A key was revoked
    KeyRevoked(KeyRevokedEvent),

    /// Trust relationship established
    TrustEstablished(TrustEstablishedEvent),

    /// PKI hierarchy created
    PkiHierarchyCreated(PkiHierarchyCreatedEvent),

    /// NATS operator created
    NatsOperatorCreated(NatsOperatorCreatedEvent),

    /// NATS account created
    NatsAccountCreated(NatsAccountCreatedEvent),

    /// NATS user created
    NatsUserCreated(NatsUserCreatedEvent),

    /// NATS signing key generated
    NatsSigningKeyGenerated(NatsSigningKeyGeneratedEvent),

    /// NATS permissions set
    NatsPermissionsSet(NatsPermissionsSetEvent),

    /// NATS configuration exported
    NatsConfigExported(NatsConfigExportedEvent),
}

/// Key was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGeneratedEvent {
    pub key_id: Uuid,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub hardware_backed: bool,
    pub metadata: KeyMetadata,
    /// Domain context - who owns this key
    pub ownership: Option<KeyOwnership>,
    /// Domain context - where this key is stored
    pub storage_location: Option<KeyStorageLocation>,
}

/// Certificate was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateGeneratedEvent {
    pub cert_id: Uuid,
    pub key_id: Uuid,
    pub subject: String,
    pub issuer: Option<Uuid>, // None for self-signed
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub is_ca: bool,
    pub san: Vec<String>,
    pub key_usage: Vec<String>,
    pub extended_key_usage: Vec<String>,
}

/// Certificate was signed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSignedEvent {
    pub cert_id: Uuid,
    pub signed_by: Uuid, // CA cert ID
    pub signature_algorithm: String,
    pub signed_at: DateTime<Utc>,
}

/// Key was imported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyImportedEvent {
    pub key_id: Uuid,
    pub source: ImportSource,
    pub format: KeyFormat,
    pub imported_at: DateTime<Utc>,
    pub imported_by: String,
    pub metadata: KeyMetadata,
}

/// Key was exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExportedEvent {
    pub key_id: Uuid,
    pub format: KeyFormat,
    pub include_private: bool,
    pub exported_at: DateTime<Utc>,
    pub exported_by: String,
    pub destination: ExportDestination,
}

/// Key stored in offline partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStoredOfflineEvent {
    pub key_id: Uuid,
    pub partition_id: Uuid,
    pub encrypted: bool,
    pub stored_at: DateTime<Utc>,
    pub checksum: String,
}

/// YubiKey was provisioned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyProvisionedEvent {
    pub event_id: Uuid,  // Unique event ID
    pub yubikey_serial: String,
    pub slots_configured: Vec<YubiKeySlot>,
    pub provisioned_at: DateTime<Utc>,
    pub provisioned_by: String,
}

/// SSH key generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyGeneratedEvent {
    pub key_id: Uuid,
    pub key_type: SshKeyType,
    pub comment: String,
    pub generated_at: DateTime<Utc>,
}

/// GPG key generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeyGeneratedEvent {
    pub key_id: Uuid,
    pub user_id: String,
    pub key_type: GpgKeyType,
    pub capabilities: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

/// Key was revoked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRevokedEvent {
    pub key_id: Uuid,
    pub reason: RevocationReason,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: String,
}

/// Trust relationship established
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEstablishedEvent {
    pub trustor_id: Uuid,
    pub trustee_id: Uuid,
    pub trust_level: TrustLevel,
    pub established_at: DateTime<Utc>,
}

/// PKI hierarchy created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkiHierarchyCreatedEvent {
    pub root_ca_id: Uuid,
    pub intermediate_ca_ids: Vec<Uuid>,
    pub hierarchy_name: String,
    pub created_at: DateTime<Utc>,
}

/// NATS operator created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorCreatedEvent {
    pub operator_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    /// Links operator to organization
    pub organization_id: Option<Uuid>,
}

/// NATS account created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountCreatedEvent {
    pub account_id: Uuid,
    pub operator_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    /// Links account to organizational unit
    pub organization_unit_id: Option<Uuid>,
}

/// NATS user created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserCreatedEvent {
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    /// Links user to person in organization
    pub person_id: Option<Uuid>,
}

/// NATS signing key generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSigningKeyGeneratedEvent {
    pub key_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: NatsEntityType,
    pub public_key: String,
    pub generated_at: DateTime<Utc>,
}

/// NATS permissions set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsPermissionsSetEvent {
    pub entity_id: Uuid,
    pub entity_type: NatsEntityType,
    pub permissions: NatsPermissions,
    pub set_at: DateTime<Utc>,
    pub set_by: String,
}

/// NATS configuration exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfigExportedEvent {
    pub export_id: Uuid,
    pub operator_id: Uuid,
    pub format: NatsExportFormat,
    pub exported_at: DateTime<Utc>,
    pub exported_by: String,
}

/// NATS entity type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NatsEntityType {
    Operator,
    Account,
    User,
}

/// NATS permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsPermissions {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
    pub allow_responses: bool,
    pub max_payload: Option<i64>,
}

/// NATS export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NatsExportFormat {
    NscStore,
    ServerConfig,
    Credentials,
}

// Supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Rsa { bits: u32 },
    Ecdsa { curve: String },
    Ed25519,
    Secp256k1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyPurpose {
    Signing,
    Encryption,
    Authentication,
    KeyAgreement,
    CertificateAuthority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub label: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportSource {
    File { path: String },
    YubiKey { serial: String },
    Hsm { identifier: String },
    Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyFormat {
    Der,
    Pem,
    Pkcs8,
    Pkcs12,
    Jwk,
    SshPublicKey,
    GpgAsciiArmor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportDestination {
    File { path: String },
    Memory,
    YubiKey { serial: String },
    Partition { id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeySlot {
    pub slot_id: String,
    pub key_id: Uuid,
    pub purpose: KeyPurpose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SshKeyType {
    Rsa,
    Ed25519,
    Ecdsa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpgKeyType {
    Master,
    Subkey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevocationReason {
    KeyCompromise,
    CaCompromise,
    AffiliationChanged,
    Superseded,
    CessationOfOperation,
    Unspecified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Unknown,
    Never,
    Marginal,
    Full,
    Ultimate,
}

// Implement DomainEvent for KeyEvent
impl DomainEvent for KeyEvent {
    fn aggregate_id(&self) -> Uuid {
        match self {
            KeyEvent::KeyGenerated(e) => e.key_id,
            KeyEvent::KeyImported(e) => e.key_id,
            KeyEvent::CertificateGenerated(e) => e.cert_id,
            KeyEvent::CertificateSigned(e) => e.cert_id,
            KeyEvent::KeyExported(e) => e.key_id,
            KeyEvent::KeyStoredOffline(e) => e.key_id,
            KeyEvent::YubiKeyProvisioned(e) => e.event_id,
            KeyEvent::SshKeyGenerated(e) => e.key_id,
            KeyEvent::GpgKeyGenerated(e) => e.key_id,
            KeyEvent::KeyRevoked(e) => e.key_id,
            KeyEvent::TrustEstablished(e) => e.trustor_id,
            KeyEvent::PkiHierarchyCreated(e) => e.root_ca_id,
            KeyEvent::NatsOperatorCreated(e) => e.operator_id,
            KeyEvent::NatsAccountCreated(e) => e.account_id,
            KeyEvent::NatsUserCreated(e) => e.user_id,
            KeyEvent::NatsSigningKeyGenerated(e) => e.key_id,
            KeyEvent::NatsPermissionsSet(e) => e.entity_id,
            KeyEvent::NatsConfigExported(e) => e.export_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            KeyEvent::KeyGenerated(_) => "KeyGenerated",
            KeyEvent::KeyImported(_) => "KeyImported",
            KeyEvent::CertificateGenerated(_) => "CertificateGenerated",
            KeyEvent::CertificateSigned(_) => "CertificateSigned",
            KeyEvent::KeyExported(_) => "KeyExported",
            KeyEvent::KeyStoredOffline(_) => "KeyStoredOffline",
            KeyEvent::YubiKeyProvisioned(_) => "YubiKeyProvisioned",
            KeyEvent::SshKeyGenerated(_) => "SshKeyGenerated",
            KeyEvent::GpgKeyGenerated(_) => "GpgKeyGenerated",
            KeyEvent::KeyRevoked(_) => "KeyRevoked",
            KeyEvent::TrustEstablished(_) => "TrustEstablished",
            KeyEvent::PkiHierarchyCreated(_) => "PkiHierarchyCreated",
            KeyEvent::NatsOperatorCreated(_) => "NatsOperatorCreated",
            KeyEvent::NatsAccountCreated(_) => "NatsAccountCreated",
            KeyEvent::NatsUserCreated(_) => "NatsUserCreated",
            KeyEvent::NatsSigningKeyGenerated(_) => "NatsSigningKeyGenerated",
            KeyEvent::NatsPermissionsSet(_) => "NatsPermissionsSet",
            KeyEvent::NatsConfigExported(_) => "NatsConfigExported",
        }
    }
}