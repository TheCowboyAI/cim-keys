//! Event-sourced key management events
//!
//! All key operations are modeled as immutable events following CIM's FRP principles.
//! No mutable state - only event streams that represent facts about key operations.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Import domain types
use crate::domain::KeyOwnership;

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

    /// JWKS (JSON Web Key Set) exported for OIDC/OAuth2
    JwksExported(JwksExportedEvent),

    /// Key rotation initiated
    KeyRotationInitiated(KeyRotationInitiatedEvent),

    /// Key rotation completed
    KeyRotationCompleted(KeyRotationCompletedEvent),

    /// TOTP secret generated
    TotpSecretGenerated(TotpSecretGeneratedEvent),

    /// Service account created with accountability
    ServiceAccountCreated(ServiceAccountCreatedEvent),

    /// Agent created with accountability
    AgentCreated(AgentCreatedEvent),

    /// Accountability validated for automated identity
    AccountabilityValidated(AccountabilityValidatedEvent),

    /// Accountability violation detected
    AccountabilityViolated(AccountabilityViolatedEvent),

    /// Certificate exported to storage
    CertificateExported(CertificateExportedEvent),

    /// Export manifest created
    ManifestCreated(ManifestCreatedEvent),
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Rsa { bits: u32 },
    Ecdsa { curve: String },
    Ed25519,
    Secp256k1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    Signing,
    Encryption,
    Authentication,
    KeyAgreement,
    CertificateAuthority,
    /// JWT/JWS signing for OIDC/OAuth2
    JwtSigning,
    /// Token encryption (for encrypted JWTs)
    JwtEncryption,
    /// TOTP/OATH shared secrets
    TotpSecret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub label: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
    /// JWT Key ID (kid) for JWKS
    pub jwt_kid: Option<String>,
    /// JWT algorithm (RS256, ES256, EdDSA, etc.)
    pub jwt_alg: Option<String>,
    /// JWT key use (sig, enc)
    pub jwt_use: Option<JwtKeyUse>,
}

/// JWT key use per RFC 7517
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JwtKeyUse {
    /// Signature verification
    #[serde(rename = "sig")]
    Signature,
    /// Encryption
    #[serde(rename = "enc")]
    Encryption,
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
/// JWKS exported for OIDC/OAuth2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksExportedEvent {
    pub export_id: Uuid,
    pub key_ids: Vec<Uuid>,
    pub issuer: String,
    pub export_path: String,
    pub exported_at: DateTime<Utc>,
}

/// Key rotation initiated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationInitiatedEvent {
    pub rotation_id: Uuid,
    pub old_key_id: Uuid,
    pub new_key_id: Uuid,
    pub rotation_reason: String,
    pub initiated_at: DateTime<Utc>,
    pub initiated_by: String,
}

/// Key rotation completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationCompletedEvent {
    pub rotation_id: Uuid,
    pub old_key_id: Uuid,
    pub new_key_id: Uuid,
    pub completed_at: DateTime<Utc>,
    pub transition_period_ends: DateTime<Utc>,
}

/// TOTP secret generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecretGeneratedEvent {
    pub secret_id: Uuid,
    pub person_id: Uuid,
    pub algorithm: String,  // SHA1, SHA256, SHA512
    pub digits: u8,         // Usually 6 or 8
    pub period: u32,        // Usually 30 seconds
    pub generated_at: DateTime<Utc>,
    /// YubiKey serial if provisioned to hardware
    pub yubikey_serial: Option<String>,
    /// OATH slot if on YubiKey
    pub oath_slot: Option<u8>,
}

/// Service account created with required accountability
///
/// CRITICAL: ServiceAccounts MUST have a responsible_person_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountCreatedEvent {
    pub service_account_id: Uuid,
    pub name: String,
    pub purpose: String,
    pub owning_unit_id: Uuid,
    /// REQUIRED: Person responsible for this service account
    pub responsible_person_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Agent created with required accountability
///
/// CRITICAL: Agents MUST have a responsible_person_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCreatedEvent {
    pub agent_id: Uuid,
    pub name: String,
    pub agent_type: String,
    /// REQUIRED: Person responsible for this agent
    pub responsible_person_id: Uuid,
    pub organization_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Accountability validated for an automated identity
///
/// Confirms that an Agent or ServiceAccount has proper human accountability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountabilityValidatedEvent {
    pub validation_id: Uuid,
    pub identity_id: Uuid,
    pub identity_type: String,  // "Agent" or "ServiceAccount"
    pub identity_name: String,
    pub responsible_person_id: Uuid,
    pub responsible_person_name: String,
    pub validated_at: DateTime<Utc>,
    pub validation_result: String,  // "PASSED" or details
}

/// Accountability violation detected
///
/// An Agent or ServiceAccount was found without proper human accountability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountabilityViolatedEvent {
    pub violation_id: Uuid,
    pub identity_id: Uuid,
    pub identity_type: String,  // "Agent" or "ServiceAccount"
    pub identity_name: String,
    pub violation_reason: String,
    pub detected_at: DateTime<Utc>,
    /// Required action to remediate
    pub required_action: String,
    /// Severity: "CRITICAL", "HIGH", "MEDIUM"
    pub severity: String,
}

/// Certificate exported to storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateExportedEvent {
    pub export_id: Uuid,
    pub cert_id: Uuid,
    pub export_format: String,
    pub destination_path: String,
    pub exported_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Export manifest created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestCreatedEvent {
    pub manifest_id: Uuid,
    pub manifest_path: String,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub keys_count: usize,
    pub certificates_count: usize,
    pub nats_configs_count: usize,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
}

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
            KeyEvent::JwksExported(e) => e.export_id,
            KeyEvent::KeyRotationInitiated(e) => e.rotation_id,
            KeyEvent::KeyRotationCompleted(e) => e.rotation_id,
            KeyEvent::TotpSecretGenerated(e) => e.secret_id,
            KeyEvent::ServiceAccountCreated(e) => e.service_account_id,
            KeyEvent::AgentCreated(e) => e.agent_id,
            KeyEvent::AccountabilityValidated(e) => e.validation_id,
            KeyEvent::AccountabilityViolated(e) => e.violation_id,
            KeyEvent::CertificateExported(e) => e.export_id,
            KeyEvent::ManifestCreated(e) => e.manifest_id,
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
            KeyEvent::JwksExported(_) => "JwksExported",
            KeyEvent::KeyRotationInitiated(_) => "KeyRotationInitiated",
            KeyEvent::KeyRotationCompleted(_) => "KeyRotationCompleted",
            KeyEvent::TotpSecretGenerated(_) => "TotpSecretGenerated",
            KeyEvent::ServiceAccountCreated(_) => "ServiceAccountCreated",
            KeyEvent::AgentCreated(_) => "AgentCreated",
            KeyEvent::AccountabilityValidated(_) => "AccountabilityValidated",
            KeyEvent::AccountabilityViolated(_) => "AccountabilityViolated",
            KeyEvent::CertificateExported(_) => "CertificateExported",
            KeyEvent::ManifestCreated(_) => "ManifestCreated",
        }
    }
}