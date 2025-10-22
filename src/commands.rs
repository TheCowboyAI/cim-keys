//! Command definitions for key management operations
//!
//! Commands represent intentions to change the system state.
//! They are processed by command handlers which emit events.

use cim_domain::{Command, CommandId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::events::{KeyAlgorithm, KeyPurpose, KeyFormat, ImportSource, ExportDestination};
use crate::domain::{KeyOwnership, KeyStorageLocation, KeyContext};

/// Base command for all key operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command_type")]
pub enum KeyCommand {
    /// Generate a new key
    GenerateKey(GenerateKeyCommand),

    /// Import an existing key
    ImportKey(ImportKeyCommand),

    /// Generate a certificate
    GenerateCertificate(GenerateCertificateCommand),

    /// Sign a certificate
    SignCertificate(SignCertificateCommand),

    /// Export a key
    ExportKey(ExportKeyCommand),

    /// Store key in offline partition
    StoreKeyOffline(StoreKeyOfflineCommand),

    /// Provision a YubiKey
    ProvisionYubiKey(ProvisionYubiKeyCommand),

    /// Generate SSH key
    GenerateSshKey(GenerateSshKeyCommand),

    /// Generate GPG key
    GenerateGpgKey(GenerateGpgKeyCommand),

    /// Revoke a key
    RevokeKey(RevokeKeyCommand),

    /// Establish trust
    EstablishTrust(EstablishTrustCommand),

    /// Create PKI hierarchy
    CreatePkiHierarchy(CreatePkiHierarchyCommand),

    /// Create NATS operator
    CreateNatsOperator(CreateNatsOperatorCommand),

    /// Create NATS account
    CreateNatsAccount(CreateNatsAccountCommand),

    /// Create NATS user
    CreateNatsUser(CreateNatsUserCommand),

    /// Generate NATS signing key
    GenerateNatsSigningKey(GenerateNatsSigningKeyCommand),

    /// Set NATS permissions
    SetNatsPermissions(SetNatsPermissionsCommand),

    /// Export NATS configuration
    ExportNatsConfig(ExportNatsConfigCommand),
}

/// Command to generate a new key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateKeyCommand {
    pub command_id: CommandId,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub label: String,
    pub hardware_backed: bool,
    pub requestor: String,
    /// Domain context for the key operation
    pub context: Option<KeyContext>,
}

/// Command to generate a certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCertificateCommand {
    pub command_id: CommandId,
    pub key_id: Uuid,
    pub subject: CertificateSubject,
    pub validity_days: u32,
    pub is_ca: bool,
    pub san: Vec<String>,
    pub key_usage: Vec<String>,
    pub extended_key_usage: Vec<String>,
    pub requestor: String,
}

/// Command to sign a certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignCertificateCommand {
    pub command_id: CommandId,
    pub cert_id: Uuid,
    pub ca_cert_id: Uuid,
    pub ca_key_id: Uuid,
    pub signature_algorithm: String,
    pub requestor: String,
}

/// Command to import a key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportKeyCommand {
    pub command_id: CommandId,
    pub source: ImportSource,
    pub format: KeyFormat,
    pub label: String,
    pub key_data: Vec<u8>,
    pub requestor: String,
}

/// Command to export a key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportKeyCommand {
    pub command_id: CommandId,
    pub key_id: Uuid,
    pub format: KeyFormat,
    pub include_private: bool,
    pub destination: ExportDestination,
    pub requestor: String,
}

/// Command to store key offline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreKeyOfflineCommand {
    pub command_id: CommandId,
    pub key_id: Uuid,
    pub partition_id: Uuid,
    pub encrypt: bool,
    pub encryption_key_id: Option<Uuid>,
    pub requestor: String,
}

/// Command to provision YubiKey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionYubiKeyCommand {
    pub command_id: CommandId,
    pub yubikey_serial: String,
    pub slots: Vec<YubiKeySlotConfig>,
    pub management_key: Option<Vec<u8>>,
    pub requestor: String,
}

/// Command to generate SSH key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateSshKeyCommand {
    pub command_id: CommandId,
    pub key_type: String,
    pub comment: String,
    pub requestor: String,
}

/// Command to generate GPG key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateGpgKeyCommand {
    pub command_id: CommandId,
    pub user_id: String,
    pub real_name: String,
    pub email: String,
    pub key_type: String,
    pub key_length: u32,
    pub expires_in_days: Option<u32>,
    pub requestor: String,
}

/// Command to revoke a key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeKeyCommand {
    pub command_id: CommandId,
    pub key_id: Uuid,
    pub reason: String,
    pub requestor: String,
}

/// Command to establish trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstablishTrustCommand {
    pub command_id: CommandId,
    pub trustor_id: Uuid,
    pub trustee_id: Uuid,
    pub trust_level: String,
    pub requestor: String,
}

/// Command to create PKI hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePkiHierarchyCommand {
    pub command_id: CommandId,
    pub hierarchy_name: String,
    pub root_ca_config: CaConfig,
    pub intermediate_ca_configs: Vec<CaConfig>,
    pub requestor: String,
}

/// Command to create NATS operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNatsOperatorCommand {
    pub command_id: CommandId,
    pub name: String,
    pub requestor: String,
    /// Organization this operator represents
    pub organization_id: Option<Uuid>,
}

/// Command to create NATS account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNatsAccountCommand {
    pub command_id: CommandId,
    pub operator_id: Uuid,
    pub name: String,
    pub is_system: bool,
    pub requestor: String,
    /// Organizational unit this account belongs to
    pub organization_unit_id: Option<Uuid>,
}

/// Command to create NATS user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNatsUserCommand {
    pub command_id: CommandId,
    pub account_id: Uuid,
    pub name: String,
    pub requestor: String,
    /// Person this user represents
    pub person_id: Option<Uuid>,
}

/// Command to generate NATS signing key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateNatsSigningKeyCommand {
    pub command_id: CommandId,
    pub entity_id: Uuid,
    pub entity_type: String,
    pub requestor: String,
}

/// Command to set NATS permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetNatsPermissionsCommand {
    pub command_id: CommandId,
    pub entity_id: Uuid,
    pub entity_type: String,
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
    pub allow_responses: bool,
    pub max_payload: Option<i64>,
    pub requestor: String,
}

/// Command to export NATS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportNatsConfigCommand {
    pub command_id: CommandId,
    pub operator_id: Uuid,
    pub format: String,
    pub output_dir: String,
    pub requestor: String,
}

// Supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSubject {
    pub common_name: String,
    pub organization: Option<String>,
    pub organizational_unit: Option<String>,
    pub country: Option<String>,
    pub state_or_province: Option<String>,
    pub locality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeySlotConfig {
    pub slot_id: String,
    pub key_algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub pin_policy: String,
    pub touch_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaConfig {
    pub name: String,
    pub subject: CertificateSubject,
    pub validity_years: u32,
    pub key_algorithm: KeyAlgorithm,
    pub path_len_constraint: Option<u32>,
}

// Implement Command trait
impl Command for KeyCommand {
    type Aggregate = crate::aggregate::KeyManagementAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        // For key management, we use a single aggregate per domain/organization
        // In a real system, this might return specific aggregate IDs based on the command
        None // Commands create new entities, so no specific aggregate ID
    }
}