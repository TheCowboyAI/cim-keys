// Command Module
//
// Command handlers for all key operations organized by DDD aggregate roots.
// Each handler validates input, executes domain logic, and emits events.
//
// Commands follow the pattern:
// 1. Validate input
// 2. Execute domain logic (projections, aggregates)
// 3. Emit events
// 4. Return result
//
// All commands include correlation_id and causation_id for event tracing.

// Legacy command modules (to be refactored)
pub mod organization;
pub mod nats_identity;
pub mod yubikey;
pub mod pki;
pub mod export;

// DDD-organized command modules (aggregate-aligned)
pub mod person;
pub mod location;
pub mod certificate;
pub mod key;
pub mod nats_operator;
pub mod nats_account;
pub mod nats_user;
pub mod relationship;
pub mod manifest;

// Re-export command types
pub use nats_identity::{
    BootstrapNatsInfrastructure, CreateNatsAccount, CreateNatsOperator, CreateNatsUser,
    NatsAccountCreated, NatsInfrastructureBootstrapped, NatsOperatorCreated, NatsUserCreated,
};

pub use yubikey::{
    ConfigureYubiKeySecurity, ProvisionYubiKeySlot, YubiKeySecurityConfigured,
    YubiKeySlotProvisioned,
};

pub use pki::{
    GenerateCertificate, GenerateKeyPair, GenerateRootCA, CertificateGenerated, KeyPairGenerated,
    RootCAGenerated,
};

pub use export::{ExportToEncryptedStorage, ExportCompleted};

// Legacy command wrapper for backward compatibility with GUI and tests
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum KeyCommand {
    // Key/Certificate operations
    GenerateRootCA(GenerateRootCA),
    GenerateCertificate(GenerateCertificateCommand),
    GenerateSshKey(GenerateSshKeyCommand),
    ProvisionYubiKey(ProvisionYubiKeySlot),
    ExportKeys(ExportToEncryptedStorage),

    // Organizational domain operations
    CreateOrganization(organization::CreateOrganization),
    CreatePerson(organization::CreatePerson),
    CreateLocation(organization::CreateLocation),
}

// Legacy command structures for backward compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerateCertificateCommand {
    pub command_id: cim_domain::EntityId<Self>,
    pub key_id: uuid::Uuid,
    pub subject: CertificateSubject,
    pub validity_days: u32,
    pub is_ca: bool,
    pub san: Vec<String>,
    pub key_usage: Vec<String>,
    pub extended_key_usage: Vec<String>,
    pub requestor: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CertificateSubject {
    pub common_name: String,
    pub organization: Option<String>,
    pub country: Option<String>,
    pub organizational_unit: Option<String>,
    pub locality: Option<String>,
    pub state_or_province: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerateSshKeyCommand {
    pub command_id: cim_domain::EntityId<Self>,
    pub person_id: uuid::Uuid,
    pub key_type: String,
    pub requestor: String,
    pub comment: Option<String>,
}
