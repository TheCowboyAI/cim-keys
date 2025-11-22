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

    // Certificate Lifecycle State Transitions (Phase 11)
    /// Certificate activated
    CertificateActivated(CertificateActivatedEvent),

    /// Certificate suspended
    CertificateSuspended(CertificateSuspendedEvent),

    /// Certificate revoked (terminal)
    CertificateRevoked(CertificateRevokedEvent),

    /// Certificate expired (terminal)
    CertificateExpired(CertificateExpiredEvent),

    /// Certificate renewed
    CertificateRenewed(CertificateRenewedEvent),

    // Person Lifecycle State Transitions (Phase 12)
    /// Person activated
    PersonActivated(PersonActivatedEvent),

    /// Person suspended
    PersonSuspended(PersonSuspendedEvent),

    /// Person reactivated
    PersonReactivated(PersonReactivatedEvent),

    /// Person archived (terminal)
    PersonArchived(PersonArchivedEvent),

    // Location Lifecycle State Transitions (Phase 12)
    /// Location activated
    LocationActivated(LocationActivatedEvent),

    /// Location suspended
    LocationSuspended(LocationSuspendedEvent),

    /// Location reactivated
    LocationReactivated(LocationReactivatedEvent),

    /// Location decommissioned (terminal)
    LocationDecommissioned(LocationDecommissionedEvent),

    /// A key was exported
    KeyExported(KeyExportedEvent),

    /// A key was stored in offline partition
    KeyStoredOffline(KeyStoredOfflineEvent),

    /// A YubiKey was provisioned
    YubiKeyProvisioned(YubiKeyProvisionedEvent),

    /// YubiKey PIN was configured
    PinConfigured(PinConfiguredEvent),

    /// YubiKey PUK was configured
    PukConfigured(PukConfiguredEvent),

    /// YubiKey management key was rotated
    ManagementKeyRotated(ManagementKeyRotatedEvent),

    /// YubiKey was detected
    YubiKeyDetected(YubiKeyDetectedEvent),

    /// Key was generated in YubiKey slot
    KeyGeneratedInSlot(KeyGeneratedInSlotEvent),

    /// Certificate was imported to YubiKey slot
    CertificateImportedToSlot(CertificateImportedToSlotEvent),

    /// YubiKey slot allocation was planned
    SlotAllocationPlanned(SlotAllocationPlannedEvent),

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

    // NATS Operator State Transitions
    /// NATS operator suspended
    NatsOperatorSuspended(NatsOperatorSuspendedEvent),

    /// NATS operator reactivated
    NatsOperatorReactivated(NatsOperatorReactivatedEvent),

    /// NATS operator revoked (terminal)
    NatsOperatorRevoked(NatsOperatorRevokedEvent),

    // NATS Account State Transitions
    /// NATS account activated
    NatsAccountActivated(NatsAccountActivatedEvent),

    /// NATS account suspended
    NatsAccountSuspended(NatsAccountSuspendedEvent),

    /// NATS account reactivated
    NatsAccountReactivated(NatsAccountReactivatedEvent),

    /// NATS account deleted (terminal)
    NatsAccountDeleted(NatsAccountDeletedEvent),

    // NATS User State Transitions
    /// NATS user activated
    NatsUserActivated(NatsUserActivatedEvent),

    /// NATS user suspended
    NatsUserSuspended(NatsUserSuspendedEvent),

    /// NATS user reactivated
    NatsUserReactivated(NatsUserReactivatedEvent),

    /// NATS user deleted (terminal)
    NatsUserDeleted(NatsUserDeletedEvent),

    /// NATS signing key generated
    NatsSigningKeyGenerated(NatsSigningKeyGeneratedEvent),

    /// NKey generated (US-021: projection step event)
    NKeyGenerated(NKeyGeneratedEvent),

    /// JWT claims created (US-021: projection step event)
    JwtClaimsCreated(JwtClaimsCreatedEvent),

    /// JWT signed (US-021: projection step event)
    JwtSigned(JwtSignedEvent),

    /// Projection applied (US-021: projection step event)
    ProjectionApplied(ProjectionAppliedEvent),

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

    // Organization Domain Events
    /// Person created in organization
    PersonCreated(PersonCreatedEvent),

    /// Location created
    LocationCreated(LocationCreatedEvent),

    /// Organization created
    OrganizationCreated(OrganizationCreatedEvent),

    /// Organizational unit created
    OrganizationalUnitCreated(OrganizationalUnitCreatedEvent),

    /// Role created
    RoleCreated(RoleCreatedEvent),

    /// Policy created
    PolicyCreated(PolicyCreatedEvent),

    /// Relationship established between entities
    RelationshipEstablished(RelationshipEstablishedEvent),
}

/// Key was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGeneratedEvent {
    pub key_id: Uuid,  // UUID v7 - contains embedded timestamp
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub generated_at: DateTime<Utc>,  // Derived from key_id (UUID v7 timestamp) for convenience
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
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Certificate was signed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSignedEvent {
    pub cert_id: Uuid,
    pub signed_by: Uuid, // CA cert ID
    pub signature_algorithm: String,
    pub signed_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Certificate Lifecycle State Transitions (Phase 11)
// ============================================================================

/// Certificate activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateActivatedEvent {
    pub cert_id: Uuid,
    pub activated_at: DateTime<Utc>,
    pub activated_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Certificate suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSuspendedEvent {
    pub cert_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Certificate revoked (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRevokedEvent {
    pub cert_id: Uuid,
    pub reason: String,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Certificate expired (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateExpiredEvent {
    pub cert_id: Uuid,
    pub expired_at: DateTime<Utc>,
    pub not_after: DateTime<Utc>, // Original expiry date
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Certificate renewed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRenewedEvent {
    pub old_cert_id: Uuid,
    pub new_cert_id: Uuid,
    pub renewed_at: DateTime<Utc>,
    pub renewed_by: Uuid, // Person ID
    pub new_not_after: DateTime<Utc>, // New expiry date
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Person Lifecycle State Transitions (Phase 12)
// ============================================================================

/// Person activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonActivatedEvent {
    pub person_id: Uuid,
    pub activated_at: DateTime<Utc>,
    pub activated_by: Uuid, // Person ID who performed activation
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Person suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSuspendedEvent {
    pub person_id: Uuid,
    pub reason: String, // e.g., "On leave", "Security review"
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid, // Person ID who performed suspension
    pub expected_return: Option<DateTime<Utc>>, // Optional return date
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Person reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonReactivatedEvent {
    pub person_id: Uuid,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: Uuid, // Person ID who performed reactivation
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Person archived (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonArchivedEvent {
    pub person_id: Uuid,
    pub reason: String, // e.g., "Left organization", "Retired"
    pub archived_at: DateTime<Utc>,
    pub archived_by: Uuid, // Person ID who performed archival
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Location Lifecycle State Transitions (Phase 12)
// ============================================================================

/// Location activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationActivatedEvent {
    pub location_id: Uuid,
    pub activated_at: DateTime<Utc>,
    pub activated_by: Uuid, // Person ID who performed activation
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSuspendedEvent {
    pub location_id: Uuid,
    pub reason: String, // e.g., "Maintenance", "Security incident"
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid, // Person ID who performed suspension
    pub expected_restoration: Option<DateTime<Utc>>, // Optional restoration date
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReactivatedEvent {
    pub location_id: Uuid,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: Uuid, // Person ID who performed reactivation
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location decommissioned (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationDecommissionedEvent {
    pub location_id: Uuid,
    pub reason: String, // e.g., "Facility closed", "Moved to new location"
    pub decommissioned_at: DateTime<Utc>,
    pub decommissioned_by: Uuid, // Person ID who performed decommissioning
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Key was imported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyImportedEvent {
    pub key_id: Uuid,  // UUID v7 - contains embedded timestamp
    pub source: ImportSource,
    pub format: KeyFormat,
    pub imported_at: DateTime<Utc>,  // Derived from key_id (UUID v7 timestamp) for convenience
    pub imported_by: String,
    pub metadata: KeyMetadata,
}

/// Key was exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExportedEvent {
    pub key_id: Uuid,  // References the key (not UUID v7 - key already exists)
    pub format: KeyFormat,
    pub include_private: bool,
    pub exported_at: DateTime<Utc>,  // Actual export timestamp (operation time, not key creation)
    pub exported_by: String,
    pub destination: ExportDestination,
}

/// Key stored in offline partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStoredOfflineEvent {
    pub key_id: Uuid,  // References the key (not UUID v7 - key already exists)
    pub partition_id: Uuid,  // UUID v7 - contains embedded timestamp
    pub encrypted: bool,
    pub stored_at: DateTime<Utc>,  // Actual storage timestamp (operation time, not key creation)
    pub checksum: String,
}

/// YubiKey was provisioned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyProvisionedEvent {
    pub event_id: Uuid,  // UUID v7 - contains embedded timestamp
    pub yubikey_serial: String,
    pub slots_configured: Vec<YubiKeySlot>,
    pub provisioned_at: DateTime<Utc>,  // Derived from event_id (UUID v7 timestamp) for convenience
    pub provisioned_by: String,
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

/// YubiKey was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyDetectedEvent {
    pub event_id: Uuid,
    pub yubikey_serial: String,
    pub firmware_version: String,
    pub detected_at: DateTime<Utc>,
    pub correlation_id: Uuid,
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
    /// Event sourcing: correlation ID links related events
    pub correlation_id: Uuid,
    /// Event sourcing: causation ID points to event that caused this
    pub causation_id: Option<Uuid>,
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
    /// Event sourcing: correlation ID links related events
    pub correlation_id: Uuid,
    /// Event sourcing: causation ID points to event that caused this
    pub causation_id: Option<Uuid>,
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
    /// Event sourcing: correlation ID links related events
    pub correlation_id: Uuid,
    /// Event sourcing: causation ID points to event that caused this
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// NATS Operator State Transitions
// ============================================================================

/// NATS operator suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorSuspendedEvent {
    pub operator_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS operator reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorReactivatedEvent {
    pub operator_id: Uuid,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS operator revoked (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorRevokedEvent {
    pub operator_id: Uuid,
    pub reason: String,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// NATS Account State Transitions
// ============================================================================

/// NATS account activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountActivatedEvent {
    pub account_id: Uuid,
    pub permissions: NatsPermissions,
    pub activated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS account suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountSuspendedEvent {
    pub account_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS account reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountReactivatedEvent {
    pub account_id: Uuid,
    pub permissions: NatsPermissions,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS account deleted (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountDeletedEvent {
    pub account_id: Uuid,
    pub reason: String,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// NATS User State Transitions
// ============================================================================

/// NATS user activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserActivatedEvent {
    pub user_id: Uuid,
    pub permissions: NatsUserPermissions,
    pub activated_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS user suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserSuspendedEvent {
    pub user_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS user reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserReactivatedEvent {
    pub user_id: Uuid,
    pub permissions: NatsUserPermissions,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NATS user deleted (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserDeletedEvent {
    pub user_id: Uuid,
    pub reason: String,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: Uuid, // Person ID
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
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

/// NATS permissions (for operators and accounts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsPermissions {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
    pub allow_responses: bool,
    pub max_payload: Option<i64>,
}

/// NATS user permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserPermissions {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
    pub allow_responses: bool,
    pub max_payload: Option<u64>,
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

// ============================================================================
// Organization Domain Events
// ============================================================================

/// Person created in organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreatedEvent {
    pub person_id: Uuid,
    pub name: String,
    pub email: String,
    pub title: Option<String>,
    pub department: Option<String>,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCreatedEvent {
    pub location_id: Uuid,
    pub name: String,
    pub location_type: String,
    pub address: Option<String>,
    pub coordinates: Option<(f64, f64)>,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Organization created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCreatedEvent {
    pub organization_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Organizational unit created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalUnitCreatedEvent {
    pub unit_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub organization_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Role created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCreatedEvent {
    pub role_id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Policy created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCreatedEvent {
    pub policy_id: Uuid,
    pub name: String,
    pub description: String,
    pub claims: Vec<crate::policy_types::PolicyClaim>,
    pub conditions: Vec<crate::policy_types::PolicyCondition>,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Relationship established between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEstablishedEvent {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub relationship_type: crate::commands::organization::RelationshipType,
    pub established_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// US-021: Projection Step Events (Audit Trail)
// ============================================================================

/// NKey generated (US-021: projection step event)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NKeyGeneratedEvent {
    pub nkey_id: Uuid,
    pub key_type: String, // Operator, Account, User
    pub public_key: String,
    pub purpose: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub generated_at: DateTime<Utc>,
    /// Event sourcing: correlation ID links related events
    pub correlation_id: Uuid,
    /// Event sourcing: causation ID points to event that caused this
    pub causation_id: Option<Uuid>,
}

/// JWT claims created (US-021: projection step event)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaimsCreatedEvent {
    pub claims_id: Uuid,
    pub issuer: String,
    pub subject: String,
    pub audience: Option<Vec<String>>,
    pub permissions: String, // JSON string of permissions
    pub not_before: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// Event sourcing: correlation ID links related events
    pub correlation_id: Uuid,
    /// Event sourcing: causation ID points to event that caused this
    pub causation_id: Option<Uuid>,
}

/// JWT signed (US-021: projection step event)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSignedEvent {
    pub jwt_id: Uuid,
    pub signer_public_key: String,
    pub signature_algorithm: String,
    pub jwt_token: String,
    pub signature_verification_data: String, // Hex-encoded signature for verification
    pub signed_at: DateTime<Utc>,
    /// Event sourcing: correlation ID links related events
    pub correlation_id: Uuid,
    /// Event sourcing: causation ID points to event that caused this
    pub causation_id: Option<Uuid>,
}

/// Projection applied (US-021: projection step event)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionAppliedEvent {
    pub projection_id: Uuid,
    pub projection_type: String, // e.g., "NatsOperatorProjection", "NatsAccountProjection"
    pub input_checksum: String, // SHA-256 of input data
    pub output_checksum: String, // SHA-256 of output data
    pub applied_at: DateTime<Utc>,
    /// Event sourcing: correlation ID links related events
    pub correlation_id: Uuid,
    /// Event sourcing: causation ID points to event that caused this
    pub causation_id: Option<Uuid>,
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
            KeyEvent::PinConfigured(e) => e.event_id,
            KeyEvent::PukConfigured(e) => e.event_id,
            KeyEvent::ManagementKeyRotated(e) => e.event_id,
            KeyEvent::YubiKeyDetected(e) => e.event_id,
            KeyEvent::KeyGeneratedInSlot(e) => e.event_id,
            KeyEvent::CertificateImportedToSlot(e) => e.event_id,
            KeyEvent::SlotAllocationPlanned(e) => e.event_id,
            KeyEvent::SshKeyGenerated(e) => e.key_id,
            KeyEvent::GpgKeyGenerated(e) => e.key_id,
            KeyEvent::KeyRevoked(e) => e.key_id,
            KeyEvent::TrustEstablished(e) => e.trustor_id,
            KeyEvent::PkiHierarchyCreated(e) => e.root_ca_id,
            KeyEvent::NatsOperatorCreated(e) => e.operator_id,
            KeyEvent::NatsAccountCreated(e) => e.account_id,
            KeyEvent::NatsUserCreated(e) => e.user_id,
            // NATS Operator State Transitions
            KeyEvent::NatsOperatorSuspended(e) => e.operator_id,
            KeyEvent::NatsOperatorReactivated(e) => e.operator_id,
            KeyEvent::NatsOperatorRevoked(e) => e.operator_id,
            // NATS Account State Transitions
            KeyEvent::NatsAccountActivated(e) => e.account_id,
            KeyEvent::NatsAccountSuspended(e) => e.account_id,
            KeyEvent::NatsAccountReactivated(e) => e.account_id,
            KeyEvent::NatsAccountDeleted(e) => e.account_id,
            // NATS User State Transitions
            KeyEvent::NatsUserActivated(e) => e.user_id,
            KeyEvent::NatsUserSuspended(e) => e.user_id,
            KeyEvent::NatsUserReactivated(e) => e.user_id,
            KeyEvent::NatsUserDeleted(e) => e.user_id,
            // Certificate Lifecycle State Transitions (Phase 11)
            KeyEvent::CertificateActivated(e) => e.cert_id,
            KeyEvent::CertificateSuspended(e) => e.cert_id,
            KeyEvent::CertificateRevoked(e) => e.cert_id,
            KeyEvent::CertificateExpired(e) => e.cert_id,
            KeyEvent::CertificateRenewed(e) => e.new_cert_id,
            // Person Lifecycle State Transitions (Phase 12)
            KeyEvent::PersonActivated(e) => e.person_id,
            KeyEvent::PersonSuspended(e) => e.person_id,
            KeyEvent::PersonReactivated(e) => e.person_id,
            KeyEvent::PersonArchived(e) => e.person_id,
            // Location Lifecycle State Transitions (Phase 12)
            KeyEvent::LocationActivated(e) => e.location_id,
            KeyEvent::LocationSuspended(e) => e.location_id,
            KeyEvent::LocationReactivated(e) => e.location_id,
            KeyEvent::LocationDecommissioned(e) => e.location_id,
            KeyEvent::NatsSigningKeyGenerated(e) => e.key_id,
            KeyEvent::NKeyGenerated(e) => e.nkey_id,
            KeyEvent::JwtClaimsCreated(e) => e.claims_id,
            KeyEvent::JwtSigned(e) => e.jwt_id,
            KeyEvent::ProjectionApplied(e) => e.projection_id,
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
            KeyEvent::PersonCreated(e) => e.person_id,
            KeyEvent::LocationCreated(e) => e.location_id,
            KeyEvent::OrganizationCreated(e) => e.organization_id,
            KeyEvent::OrganizationalUnitCreated(e) => e.unit_id,
            KeyEvent::RoleCreated(e) => e.role_id,
            KeyEvent::PolicyCreated(e) => e.policy_id,
            KeyEvent::RelationshipEstablished(e) => e.from_id,
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
            KeyEvent::PinConfigured(_) => "PinConfigured",
            KeyEvent::PukConfigured(_) => "PukConfigured",
            KeyEvent::ManagementKeyRotated(_) => "ManagementKeyRotated",
            KeyEvent::YubiKeyDetected(_) => "YubiKeyDetected",
            KeyEvent::KeyGeneratedInSlot(_) => "KeyGeneratedInSlot",
            KeyEvent::CertificateImportedToSlot(_) => "CertificateImportedToSlot",
            KeyEvent::SlotAllocationPlanned(_) => "SlotAllocationPlanned",
            KeyEvent::SshKeyGenerated(_) => "SshKeyGenerated",
            KeyEvent::GpgKeyGenerated(_) => "GpgKeyGenerated",
            KeyEvent::KeyRevoked(_) => "KeyRevoked",
            KeyEvent::TrustEstablished(_) => "TrustEstablished",
            KeyEvent::PkiHierarchyCreated(_) => "PkiHierarchyCreated",
            KeyEvent::NatsOperatorCreated(_) => "NatsOperatorCreated",
            KeyEvent::NatsAccountCreated(_) => "NatsAccountCreated",
            KeyEvent::NatsUserCreated(_) => "NatsUserCreated",
            // NATS Operator State Transitions
            KeyEvent::NatsOperatorSuspended(_) => "NatsOperatorSuspended",
            KeyEvent::NatsOperatorReactivated(_) => "NatsOperatorReactivated",
            KeyEvent::NatsOperatorRevoked(_) => "NatsOperatorRevoked",
            // NATS Account State Transitions
            KeyEvent::NatsAccountActivated(_) => "NatsAccountActivated",
            KeyEvent::NatsAccountSuspended(_) => "NatsAccountSuspended",
            KeyEvent::NatsAccountReactivated(_) => "NatsAccountReactivated",
            KeyEvent::NatsAccountDeleted(_) => "NatsAccountDeleted",
            // NATS User State Transitions
            KeyEvent::NatsUserActivated(_) => "NatsUserActivated",
            KeyEvent::NatsUserSuspended(_) => "NatsUserSuspended",
            KeyEvent::NatsUserReactivated(_) => "NatsUserReactivated",
            KeyEvent::NatsUserDeleted(_) => "NatsUserDeleted",
            // Certificate Lifecycle State Transitions (Phase 11)
            KeyEvent::CertificateActivated(_) => "CertificateActivated",
            KeyEvent::CertificateSuspended(_) => "CertificateSuspended",
            KeyEvent::CertificateRevoked(_) => "CertificateRevoked",
            KeyEvent::CertificateExpired(_) => "CertificateExpired",
            KeyEvent::CertificateRenewed(_) => "CertificateRenewed",
            // Person Lifecycle State Transitions (Phase 12)
            KeyEvent::PersonActivated(_) => "PersonActivated",
            KeyEvent::PersonSuspended(_) => "PersonSuspended",
            KeyEvent::PersonReactivated(_) => "PersonReactivated",
            KeyEvent::PersonArchived(_) => "PersonArchived",
            // Location Lifecycle State Transitions (Phase 12)
            KeyEvent::LocationActivated(_) => "LocationActivated",
            KeyEvent::LocationSuspended(_) => "LocationSuspended",
            KeyEvent::LocationReactivated(_) => "LocationReactivated",
            KeyEvent::LocationDecommissioned(_) => "LocationDecommissioned",
            KeyEvent::NatsSigningKeyGenerated(_) => "NatsSigningKeyGenerated",
            KeyEvent::NKeyGenerated(_) => "NKeyGenerated",
            KeyEvent::JwtClaimsCreated(_) => "JwtClaimsCreated",
            KeyEvent::JwtSigned(_) => "JwtSigned",
            KeyEvent::ProjectionApplied(_) => "ProjectionApplied",
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
            KeyEvent::PersonCreated(_) => "PersonCreated",
            KeyEvent::LocationCreated(_) => "LocationCreated",
            KeyEvent::OrganizationCreated(_) => "OrganizationCreated",
            KeyEvent::OrganizationalUnitCreated(_) => "OrganizationalUnitCreated",
            KeyEvent::RoleCreated(_) => "RoleCreated",
            KeyEvent::PolicyCreated(_) => "PolicyCreated",
            KeyEvent::RelationshipEstablished(_) => "RelationshipEstablished",
        }
    }
}