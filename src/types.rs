//! Shared Domain Types and Ontologies
//!
//! This module contains shared value objects, enums, and type definitions
//! that form the domain vocabulary (ontology) used across multiple aggregates.
//! These are NOT events, but rather the conceptual building blocks that
//! events and aggregates reference.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// NATS Domain Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

// ============================================================================
// Cryptographic Key Types
// ============================================================================

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
    pub attributes: HashMap<String, String>,
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

// ============================================================================
// Key Import/Export Types
// ============================================================================

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

// ============================================================================
// Hardware Security Module Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeySlot {
    pub slot_id: String,
    pub key_id: Uuid,
    pub purpose: KeyPurpose,
}

// ============================================================================
// Specific Key Types
// ============================================================================

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

// ============================================================================
// Certificate and Trust Types
// ============================================================================

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
