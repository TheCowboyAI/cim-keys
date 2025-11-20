// NATS Security Value Objects
//
// NKeys and JWTs for NATS decentralized authentication and authorization.
//
// NKeys are Ed25519 key pairs with specific prefixes:
// - Operator: O prefix
// - Account: A prefix
// - User: U prefix
// - Server: N prefix
// - Cluster: C prefix
//
// JWTs are signed tokens containing identity claims and permissions.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// NKey Value Objects
// ============================================================================

/// NATS NKey type (determines prefix and purpose)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NKeyType {
    /// Operator key (O prefix) - Root authority
    Operator,
    /// Account key (A prefix) - Organization/tenant
    Account,
    /// User key (U prefix) - Individual identity
    User,
    /// Server key (N prefix) - NATS server identity
    Server,
    /// Cluster key (C prefix) - Cluster identity
    Cluster,
}

impl NKeyType {
    /// Get the NKey prefix character
    pub fn prefix(&self) -> char {
        match self {
            NKeyType::Operator => 'O',
            NKeyType::Account => 'A',
            NKeyType::User => 'U',
            NKeyType::Server => 'N',
            NKeyType::Cluster => 'C',
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            NKeyType::Operator => "Operator",
            NKeyType::Account => "Account",
            NKeyType::User => "User",
            NKeyType::Server => "Server",
            NKeyType::Cluster => "Cluster",
        }
    }
}

impl fmt::Display for NKeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// NATS NKey Seed (private key material)
///
/// CRITICAL: This is SECRET KEY MATERIAL and must be protected!
#[derive(Clone, Serialize, Deserialize)]
pub struct NKeySeed {
    /// Key type (determines prefix)
    pub key_type: NKeyType,
    /// Encoded seed (starts with 'S' followed by type prefix)
    /// Format: S[O|A|U|N|C][base32-encoded-seed]
    seed: String,
    /// When this seed was generated
    pub generated_at: DateTime<Utc>,
}

impl NKeySeed {
    /// Create a new NKey seed
    pub fn new(key_type: NKeyType, seed: String, generated_at: DateTime<Utc>) -> Self {
        Self {
            key_type,
            seed,
            generated_at,
        }
    }

    /// Get the seed string (SENSITIVE!)
    pub fn seed(&self) -> &str {
        &self.seed
    }

    /// Check if seed matches expected prefix
    pub fn is_valid_prefix(&self) -> bool {
        if self.seed.len() < 2 {
            return false;
        }
        self.seed.starts_with('S') && self.seed.chars().nth(1) == Some(self.key_type.prefix())
    }
}

impl fmt::Debug for NKeySeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NKeySeed")
            .field("key_type", &self.key_type)
            .field("seed", &"[REDACTED]")
            .field("generated_at", &self.generated_at)
            .finish()
    }
}

impl fmt::Display for NKeySeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NKeySeed({}, generated: {})",
            self.key_type,
            self.generated_at.format("%Y-%m-%d")
        )
    }
}

/// NATS NKey Public Key
///
/// Public key derived from NKey seed, safe to share
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct NKeyPublic {
    /// Key type (determines prefix)
    pub key_type: NKeyType,
    /// Encoded public key (starts with type prefix)
    /// Format: [O|A|U|N|C][base32-encoded-public-key]
    public_key: String,
}

impl NKeyPublic {
    /// Create a new NKey public key
    pub fn new(key_type: NKeyType, public_key: String) -> Self {
        Self {
            key_type,
            public_key,
        }
    }

    /// Get the public key string
    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    /// Check if public key has correct prefix
    pub fn is_valid_prefix(&self) -> bool {
        if self.public_key.is_empty() {
            return false;
        }
        self.public_key.chars().next() == Some(self.key_type.prefix())
    }

    /// Get the prefix character
    pub fn prefix(&self) -> char {
        self.key_type.prefix()
    }
}

impl fmt::Display for NKeyPublic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.public_key)
    }
}

/// Complete NKey Pair (seed + public key)
#[derive(Clone, Serialize, Deserialize)]
pub struct NKeyPair {
    /// Unique identifier for this key pair
    pub id: Uuid,
    /// Key type
    pub key_type: NKeyType,
    /// Private seed (SENSITIVE!)
    pub seed: NKeySeed,
    /// Public key (safe to share)
    pub public_key: NKeyPublic,
    /// Optional human-readable name
    pub name: Option<String>,
    /// When this key pair was created
    pub created_at: DateTime<Utc>,
    /// Optional expiration
    pub expires_at: Option<DateTime<Utc>>,
}

impl NKeyPair {
    /// Create a new NKey pair
    pub fn new(
        key_type: NKeyType,
        seed: NKeySeed,
        public_key: NKeyPublic,
        name: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            key_type,
            seed,
            public_key,
            name,
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Check if key pair is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    /// Get public key string
    pub fn public_key_string(&self) -> &str {
        self.public_key.public_key()
    }

    /// Get seed string (SENSITIVE!)
    pub fn seed_string(&self) -> &str {
        self.seed.seed()
    }
}

impl fmt::Debug for NKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NKeyPair")
            .field("id", &self.id)
            .field("key_type", &self.key_type)
            .field("seed", &"[REDACTED]")
            .field("public_key", &self.public_key)
            .field("name", &self.name)
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

// ============================================================================
// JWT Value Objects
// ============================================================================

/// NATS JWT Header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsJwtHeader {
    /// Algorithm (always "ed25519" for NKeys)
    pub alg: String,
    /// Type (always "JWT")
    pub typ: String,
}

impl Default for NatsJwtHeader {
    fn default() -> Self {
        Self {
            alg: "ed25519".to_string(),
            typ: "JWT".to_string(),
        }
    }
}

/// NATS Operator JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorClaims {
    /// JWT ID (unique identifier)
    pub jti: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Issuer (operator public key - self-signed)
    pub iss: String,
    /// Subject (operator public key)
    pub sub: String,
    /// NATS-specific operator data
    pub nats: OperatorData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorData {
    /// Operator name
    pub name: String,
    /// Signing keys (additional keys that can sign account JWTs)
    pub signing_keys: Vec<String>,
    /// Version (should be 2)
    pub version: u32,
    /// Optional service URLs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_server_url: Option<String>,
    /// Optional operator service URLs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_service_urls: Option<Vec<String>>,
}

/// NATS Account JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountClaims {
    /// JWT ID
    pub jti: String,
    /// Issued at
    pub iat: i64,
    /// Issuer (operator public key)
    pub iss: String,
    /// Subject (account public key)
    pub sub: String,
    /// Optional expiration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    /// NATS-specific account data
    pub nats: AccountData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    /// Account name
    pub name: String,
    /// Signing keys (keys that can sign user JWTs)
    pub signing_keys: Vec<String>,
    /// Version
    pub version: u32,
    /// Account limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<AccountLimits>,
    /// Default permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_permissions: Option<Permissions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLimits {
    /// Max connections (-1 = unlimited)
    pub conn: i64,
    /// Max data transfer in bytes (-1 = unlimited)
    pub data: i64,
    /// Max exports (-1 = unlimited)
    pub exports: i64,
    /// Max imports (-1 = unlimited)
    pub imports: i64,
    /// Max leaf nodes (-1 = unlimited)
    pub leaf: i64,
    /// Max payload size in bytes (-1 = unlimited)
    pub payload: i64,
    /// Max subscriptions (-1 = unlimited)
    pub subs: i64,
    /// Allow wildcards in subscriptions
    pub wildcards: bool,
}

impl Default for AccountLimits {
    fn default() -> Self {
        Self {
            conn: -1,
            data: -1,
            exports: -1,
            imports: -1,
            leaf: -1,
            payload: -1,
            subs: -1,
            wildcards: true,
        }
    }
}

/// NATS User JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    /// JWT ID
    pub jti: String,
    /// Issued at
    pub iat: i64,
    /// Issuer (account public key)
    pub iss: String,
    /// Subject (user public key)
    pub sub: String,
    /// Optional expiration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    /// NATS-specific user data
    pub nats: UserData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    /// User name
    pub name: String,
    /// Version
    pub version: u32,
    /// User permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    /// User-specific limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<UserLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLimits {
    /// Max subscriptions
    pub subs: i64,
    /// Max data transfer
    pub data: i64,
    /// Max payload size
    pub payload: i64,
}

/// NATS Permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    /// Publish permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pub_allow: Option<Vec<String>>,
    /// Publish denials
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pub_deny: Option<Vec<String>>,
    /// Subscribe permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_allow: Option<Vec<String>>,
    /// Subscribe denials
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_deny: Option<Vec<String>>,
}

/// Complete NATS JWT (header + claims + signature)
#[derive(Clone, Serialize, Deserialize)]
pub struct NatsJwt {
    /// JWT ID for tracking
    pub id: Uuid,
    /// JWT type (Operator/Account/User)
    pub jwt_type: NKeyType,
    /// Encoded JWT string (header.claims.signature)
    jwt_token: String,
    /// Issuer public key
    pub issuer: NKeyPublic,
    /// Subject public key
    pub subject: NKeyPublic,
    /// When this JWT was issued
    pub issued_at: DateTime<Utc>,
    /// When this JWT expires (if at all)
    pub expires_at: Option<DateTime<Utc>>,
}

impl NatsJwt {
    /// Create a new NATS JWT
    pub fn new(
        jwt_type: NKeyType,
        jwt_token: String,
        issuer: NKeyPublic,
        subject: NKeyPublic,
        issued_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            jwt_type,
            jwt_token,
            issuer,
            subject,
            issued_at,
            expires_at,
        }
    }

    /// Get the JWT token string
    pub fn token(&self) -> &str {
        &self.jwt_token
    }

    /// Check if JWT is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    /// Check if JWT is valid (not expired, valid issuer/subject)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && self.issuer.is_valid_prefix() && self.subject.is_valid_prefix()
    }
}

impl fmt::Debug for NatsJwt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NatsJwt")
            .field("id", &self.id)
            .field("jwt_type", &self.jwt_type)
            .field("jwt_token", &format!("{}...", &self.jwt_token.chars().take(20).collect::<String>()))
            .field("issuer", &self.issuer)
            .field("subject", &self.subject)
            .field("issued_at", &self.issued_at)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

impl fmt::Display for NatsJwt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} JWT (issuer: {}, subject: {}, issued: {}{})",
            self.jwt_type,
            self.issuer.public_key().chars().take(8).collect::<String>(),
            self.subject.public_key().chars().take(8).collect::<String>(),
            self.issued_at.format("%Y-%m-%d"),
            if self.is_expired() { ", EXPIRED" } else { "" }
        )
    }
}

/// NATS credential file (seed + JWT combined)
///
/// Format:
/// -----BEGIN NATS USER JWT-----
/// <JWT token>
/// ------END NATS USER JWT------
///
/// ************************* IMPORTANT *************************
/// NKEY Seed printed below can be used to sign and prove identity.
/// NKEYs are sensitive and should be treated as secrets.
///
/// -----BEGIN USER NKEY SEED-----
/// <seed>
/// ------END USER NKEY SEED------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsCredential {
    /// Credential ID
    pub id: Uuid,
    /// The JWT token
    pub jwt: NatsJwt,
    /// The user's NKey seed (SENSITIVE!)
    pub seed: NKeySeed,
    /// Optional name for this credential
    pub name: Option<String>,
}

impl NatsCredential {
    /// Create a new NATS credential
    pub fn new(jwt: NatsJwt, seed: NKeySeed, name: Option<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            jwt,
            seed,
            name,
        }
    }

    /// Format as NATS credential file content
    pub fn to_credential_file(&self) -> String {
        format!(
            "-----BEGIN NATS USER JWT-----\n{}\n------END NATS USER JWT------\n\n************************* IMPORTANT *************************\nNKEY Seed printed below can be used to sign and prove identity.\nNKEYs are sensitive and should be treated as secrets.\n\n-----BEGIN USER NKEY SEED-----\n{}\n------END USER NKEY SEED------\n",
            self.jwt.token(),
            self.seed.seed()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nkey_type_prefix() {
        assert_eq!(NKeyType::Operator.prefix(), 'O');
        assert_eq!(NKeyType::Account.prefix(), 'A');
        assert_eq!(NKeyType::User.prefix(), 'U');
        assert_eq!(NKeyType::Server.prefix(), 'N');
        assert_eq!(NKeyType::Cluster.prefix(), 'C');
    }

    #[test]
    fn test_nkey_public_validation() {
        let valid_operator = NKeyPublic::new(
            NKeyType::Operator,
            "OABC123456789".to_string(),
        );
        assert!(valid_operator.is_valid_prefix());

        let invalid_operator = NKeyPublic::new(
            NKeyType::Operator,
            "AABC123456789".to_string(),
        );
        assert!(!invalid_operator.is_valid_prefix());
    }

    #[test]
    fn test_account_limits_default() {
        let limits = AccountLimits::default();
        assert_eq!(limits.conn, -1);
        assert_eq!(limits.data, -1);
        assert!(limits.wildcards);
    }
}
