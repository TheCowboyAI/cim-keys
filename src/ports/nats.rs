//! NATS key management port
//!
//! This defines the interface for NATS key operations that our domain needs.
//! The actual implementation (NSC adapter) is separate from this interface.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Port for NATS key operations
///
/// This is the interface that our domain uses. The actual implementation
/// could be NSC, a mock, or any other NATS key provider.
#[async_trait]
pub trait NatsKeyPort: Send + Sync {
    /// Generate an operator keypair
    async fn generate_operator(&self, name: &str) -> Result<NatsOperatorKeys, NatsKeyError>;

    /// Generate an account keypair
    async fn generate_account(&self, operator_id: &str, name: &str) -> Result<NatsAccountKeys, NatsKeyError>;

    /// Generate a user keypair
    async fn generate_user(&self, account_id: &str, name: &str) -> Result<NatsUserKeys, NatsKeyError>;

    /// Generate a signing key
    async fn generate_signing_key(&self, entity_id: &str) -> Result<NatsSigningKey, NatsKeyError>;

    /// Create a JWT token
    async fn create_jwt(&self, claims: &JwtClaims, signing_key: &str) -> Result<String, NatsKeyError>;

    /// Export keys in NATS format
    async fn export_keys(&self, keys: &NatsKeys) -> Result<NatsKeyExport, NatsKeyError>;

    /// Validate a key
    async fn validate_key(&self, key: &str) -> Result<bool, NatsKeyError>;
}

/// Operations that can be performed on NATS keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsKeyOperations {
    /// Generate operator hierarchy
    pub generate_hierarchy: bool,

    /// Create system account
    pub create_system_account: bool,

    /// Create user accounts
    pub create_user_accounts: Vec<String>,

    /// Set permissions
    pub set_permissions: HashMap<String, NatsPermissions>,

    /// Export configuration
    pub export_config: bool,
}

/// NATS Operator keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsOperatorKeys {
    pub id: Uuid,
    pub name: String,
    pub public_key: String,
    pub seed: String,  // Encrypted
    pub jwt: Option<String>,
}

/// NATS Account keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountKeys {
    pub id: Uuid,
    pub operator_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub seed: String,  // Encrypted
    pub jwt: Option<String>,
    pub is_system: bool,
}

/// NATS User keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserKeys {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub seed: String,  // Encrypted
    pub jwt: Option<String>,
}

/// NATS Signing key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSigningKey {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub public_key: String,
    pub seed: String,  // Encrypted
}

/// All NATS keys for a hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsKeys {
    pub operator: NatsOperatorKeys,
    pub accounts: Vec<NatsAccountKeys>,
    pub users: Vec<NatsUserKeys>,
    pub signing_keys: Vec<NatsSigningKey>,
}

/// NATS permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsPermissions {
    pub publish: NatsSubjectPermissions,
    pub subscribe: NatsSubjectPermissions,
    pub allow_responses: bool,
    pub max_payload: Option<i64>,
}

/// Subject-based permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSubjectPermissions {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

/// JWT claims for NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub subject: String,
    pub issuer: String,
    pub audience: Option<String>,
    pub name: String,
    pub nats: NatsJwtClaims,
}

/// NATS-specific JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsJwtClaims {
    pub version: i32,
    pub r#type: String,
    pub permissions: Option<NatsPermissions>,
    pub limits: Option<NatsLimits>,
}

/// NATS limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsLimits {
    pub subs: Option<i64>,
    pub payload: Option<i64>,
    pub data: Option<i64>,
}

/// Export format for NATS keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsKeyExport {
    /// NSC store format
    pub nsc_format: NscStoreExport,

    /// Resolver configuration
    pub resolver_config: ResolverConfig,

    /// NATS server configuration
    pub server_config: ServerConfig,
}

/// NSC store export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NscStoreExport {
    pub operators: HashMap<String, OperatorExport>,
    pub accounts: HashMap<String, AccountExport>,
    pub users: HashMap<String, UserExport>,
}

/// Operator export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorExport {
    pub name: String,
    pub public_key: String,
    pub jwt_file: String,
    pub seed_file: String,
}

/// Account export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountExport {
    pub name: String,
    pub public_key: String,
    pub jwt_file: String,
    pub seed_file: String,
}

/// User export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserExport {
    pub name: String,
    pub public_key: String,
    pub creds_file: String,
}

/// Resolver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverConfig {
    pub operator_jwt_path: String,
    pub system_account: String,
    pub resolver_url: Option<String>,
}

/// NATS server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub operator: String,
    pub system_account: String,
    pub jwt_path: String,
    pub resolver: ResolverType,
}

/// Resolver type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResolverType {
    #[serde(rename = "URL")]
    Url { url: String },

    #[serde(rename = "MEMORY")]
    Memory,

    #[serde(rename = "FULL")]
    Full { dir: String },
}

/// Errors for NATS key operations
#[derive(Debug, thiserror::Error)]
pub enum NatsKeyError {
    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("JWT creation failed: {0}")]
    JwtCreationFailed(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("IO error: {0}")]
    IoError(String),
}