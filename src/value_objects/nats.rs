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

    /// Generate a new NKey pair using nkeys crate
    pub fn generate(key_type: NKeyType, name: Option<String>) -> Result<Self, String> {
        use nkeys::KeyPair as NKeysKeyPair;

        // Generate key pair using nkeys crate
        let nkeys_pair = match key_type {
            NKeyType::Operator => NKeysKeyPair::new_operator(),
            NKeyType::Account => NKeysKeyPair::new_account(),
            NKeyType::User => NKeysKeyPair::new_user(),
            NKeyType::Server => NKeysKeyPair::new_server(),
            NKeyType::Cluster => NKeysKeyPair::new_cluster(),
        };

        // Extract seed and public key
        let seed_string = nkeys_pair
            .seed()
            .map_err(|e| format!("Failed to extract seed: {}", e))?;
        let public_key_string = nkeys_pair.public_key();

        let now = Utc::now();

        // Create domain value objects
        let seed = NKeySeed::new(key_type, seed_string, now);
        let public_key = NKeyPublic::new(key_type, public_key_string);

        // Verify prefixes
        if !seed.is_valid_prefix() {
            return Err(format!("Generated seed has invalid prefix for {:?}", key_type));
        }
        if !public_key.is_valid_prefix() {
            return Err(format!("Generated public key has invalid prefix for {:?}", key_type));
        }

        Ok(Self::new(key_type, seed, public_key, name))
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

    /// Sign data using this key pair
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        use nkeys::KeyPair as NKeysKeyPair;

        // Reconstruct nkeys keypair from seed
        let nkeys_pair = NKeysKeyPair::from_seed(self.seed_string())
            .map_err(|e| format!("Failed to create keypair from seed: {}", e))?;

        // Sign the data
        let signature = nkeys_pair
            .sign(data)
            .map_err(|e| format!("Failed to sign data: {}", e))?;

        Ok(signature)
    }

    /// Verify a signature
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, String> {
        use nkeys::KeyPair as NKeysKeyPair;

        // Reconstruct nkeys keypair from seed
        let nkeys_pair = NKeysKeyPair::from_seed(self.seed_string())
            .map_err(|e| format!("Failed to create keypair from seed: {}", e))?;

        // Verify the signature (returns Ok(()) if valid, Err if invalid)
        match nkeys_pair.verify(data, signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
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

    /// Generate Operator JWT (self-signed)
    pub fn generate_operator(
        operator_keypair: &NKeyPair,
        operator_name: String,
        signing_keys: Vec<String>,
    ) -> Result<Self, String> {
        if operator_keypair.key_type != NKeyType::Operator {
            return Err("Key pair must be of type Operator".to_string());
        }

        let now = Utc::now();
        let iat = now.timestamp();

        // Create operator claims
        let claims = OperatorClaims {
            jti: Uuid::now_v7().to_string(),
            iat,
            iss: operator_keypair.public_key_string().to_string(),
            sub: operator_keypair.public_key_string().to_string(), // Self-signed
            nats: OperatorData {
                name: operator_name,
                signing_keys,
                version: 2,
                account_server_url: None,
                operator_service_urls: None,
            },
        };

        // Encode and sign JWT
        let jwt_token = Self::encode_and_sign(&NatsJwtHeader::default(), &claims, operator_keypair)?;

        Ok(Self::new(
            NKeyType::Operator,
            jwt_token,
            operator_keypair.public_key.clone(),
            operator_keypair.public_key.clone(), // Self-signed
            now,
            None, // Operators typically don't expire
        ))
    }

    /// Generate Account JWT (signed by operator)
    pub fn generate_account(
        account_keypair: &NKeyPair,
        operator_keypair: &NKeyPair,
        account_name: String,
        signing_keys: Vec<String>,
        limits: Option<AccountLimits>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, String> {
        if account_keypair.key_type != NKeyType::Account {
            return Err("Account key pair must be of type Account".to_string());
        }
        if operator_keypair.key_type != NKeyType::Operator {
            return Err("Operator key pair must be of type Operator".to_string());
        }

        let now = Utc::now();
        let iat = now.timestamp();
        let exp = expires_at.map(|dt| dt.timestamp());

        // Create account claims
        let claims = AccountClaims {
            jti: Uuid::now_v7().to_string(),
            iat,
            iss: operator_keypair.public_key_string().to_string(),
            sub: account_keypair.public_key_string().to_string(),
            exp,
            nats: AccountData {
                name: account_name,
                signing_keys,
                version: 2,
                limits,
                default_permissions: None,
            },
        };

        // Encode and sign JWT with operator key
        let jwt_token = Self::encode_and_sign(&NatsJwtHeader::default(), &claims, operator_keypair)?;

        Ok(Self::new(
            NKeyType::Account,
            jwt_token,
            operator_keypair.public_key.clone(),
            account_keypair.public_key.clone(),
            now,
            expires_at,
        ))
    }

    /// Generate User JWT (signed by account)
    pub fn generate_user(
        user_keypair: &NKeyPair,
        account_keypair: &NKeyPair,
        user_name: String,
        permissions: Option<Permissions>,
        limits: Option<UserLimits>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, String> {
        if user_keypair.key_type != NKeyType::User {
            return Err("User key pair must be of type User".to_string());
        }
        if account_keypair.key_type != NKeyType::Account {
            return Err("Account key pair must be of type Account".to_string());
        }

        let now = Utc::now();
        let iat = now.timestamp();
        let exp = expires_at.map(|dt| dt.timestamp());

        // Create user claims
        let claims = UserClaims {
            jti: Uuid::now_v7().to_string(),
            iat,
            iss: account_keypair.public_key_string().to_string(),
            sub: user_keypair.public_key_string().to_string(),
            exp,
            nats: UserData {
                name: user_name,
                version: 2,
                permissions,
                limits,
            },
        };

        // Encode and sign JWT with account key
        let jwt_token = Self::encode_and_sign(&NatsJwtHeader::default(), &claims, account_keypair)?;

        Ok(Self::new(
            NKeyType::User,
            jwt_token,
            account_keypair.public_key.clone(),
            user_keypair.public_key.clone(),
            now,
            expires_at,
        ))
    }

    /// Encode header and claims, then sign to create JWT token
    fn encode_and_sign<C: serde::Serialize>(
        header: &NatsJwtHeader,
        claims: &C,
        signing_keypair: &NKeyPair,
    ) -> Result<String, String> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        // Encode header to base64
        let header_json = serde_json::to_string(header)
            .map_err(|e| format!("Failed to serialize header: {}", e))?;
        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());

        // Encode claims to base64
        let claims_json = serde_json::to_string(claims)
            .map_err(|e| format!("Failed to serialize claims: {}", e))?;
        let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

        // Create signing input (header.claims)
        let signing_input = format!("{}.{}", header_b64, claims_b64);

        // Sign using nkey
        let signature = signing_keypair.sign(signing_input.as_bytes())?;

        // Encode signature to base64
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);

        // Combine into JWT format
        Ok(format!("{}.{}", signing_input, signature_b64))
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

    #[test]
    fn test_nkey_pair_generation_operator() {
        let keypair = NKeyPair::generate(NKeyType::Operator, Some("test-operator".to_string()))
            .expect("Failed to generate operator key pair");

        assert_eq!(keypair.key_type, NKeyType::Operator);
        assert_eq!(keypair.name, Some("test-operator".to_string()));
        assert!(keypair.seed.is_valid_prefix());
        assert!(keypair.public_key.is_valid_prefix());
        assert!(keypair.seed_string().starts_with("SO"));
        assert!(keypair.public_key_string().starts_with('O'));
    }

    #[test]
    fn test_nkey_pair_generation_account() {
        let keypair = NKeyPair::generate(NKeyType::Account, Some("test-account".to_string()))
            .expect("Failed to generate account key pair");

        assert_eq!(keypair.key_type, NKeyType::Account);
        assert!(keypair.seed_string().starts_with("SA"));
        assert!(keypair.public_key_string().starts_with('A'));
    }

    #[test]
    fn test_nkey_pair_generation_user() {
        let keypair = NKeyPair::generate(NKeyType::User, Some("test-user".to_string()))
            .expect("Failed to generate user key pair");

        assert_eq!(keypair.key_type, NKeyType::User);
        assert!(keypair.seed_string().starts_with("SU"));
        assert!(keypair.public_key_string().starts_with('U'));
    }

    #[test]
    fn test_nkey_pair_signing_and_verification() {
        let keypair = NKeyPair::generate(NKeyType::User, None)
            .expect("Failed to generate user key pair");

        let data = b"test message to sign";
        let signature = keypair.sign(data).expect("Failed to sign data");

        let is_valid = keypair.verify(data, &signature).expect("Failed to verify signature");
        assert!(is_valid, "Signature should be valid");

        // Verify with wrong data fails
        let wrong_data = b"wrong message";
        let is_invalid = keypair.verify(wrong_data, &signature).expect("Failed to verify signature");
        assert!(!is_invalid, "Signature should be invalid for wrong data");
    }

    #[test]
    fn test_operator_jwt_generation() {
        let operator_keypair = NKeyPair::generate(NKeyType::Operator, Some("test-operator".to_string()))
            .expect("Failed to generate operator key pair");

        let jwt = NatsJwt::generate_operator(
            &operator_keypair,
            "Test Operator".to_string(),
            vec![],
        ).expect("Failed to generate operator JWT");

        assert_eq!(jwt.jwt_type, NKeyType::Operator);
        assert_eq!(jwt.issuer, jwt.subject); // Self-signed
        assert!(!jwt.is_expired());
        assert!(jwt.is_valid());
        assert!(!jwt.token().is_empty());
    }

    #[test]
    fn test_account_jwt_generation() {
        let operator_keypair = NKeyPair::generate(NKeyType::Operator, Some("test-operator".to_string()))
            .expect("Failed to generate operator key pair");

        let account_keypair = NKeyPair::generate(NKeyType::Account, Some("test-account".to_string()))
            .expect("Failed to generate account key pair");

        let jwt = NatsJwt::generate_account(
            &account_keypair,
            &operator_keypair,
            "Test Account".to_string(),
            vec![],
            Some(AccountLimits::default()),
            None,
        ).expect("Failed to generate account JWT");

        assert_eq!(jwt.jwt_type, NKeyType::Account);
        assert_eq!(jwt.issuer, operator_keypair.public_key);
        assert_eq!(jwt.subject, account_keypair.public_key);
        assert!(!jwt.is_expired());
        assert!(jwt.is_valid());
    }

    #[test]
    fn test_user_jwt_generation() {
        let account_keypair = NKeyPair::generate(NKeyType::Account, Some("test-account".to_string()))
            .expect("Failed to generate account key pair");

        let user_keypair = NKeyPair::generate(NKeyType::User, Some("test-user".to_string()))
            .expect("Failed to generate user key pair");

        let permissions = Permissions {
            pub_allow: Some(vec!["test.>".to_string()]),
            pub_deny: None,
            sub_allow: Some(vec!["test.>".to_string()]),
            sub_deny: None,
        };

        let jwt = NatsJwt::generate_user(
            &user_keypair,
            &account_keypair,
            "Test User".to_string(),
            Some(permissions),
            None,
            None,
        ).expect("Failed to generate user JWT");

        assert_eq!(jwt.jwt_type, NKeyType::User);
        assert_eq!(jwt.issuer, account_keypair.public_key);
        assert_eq!(jwt.subject, user_keypair.public_key);
        assert!(!jwt.is_expired());
        assert!(jwt.is_valid());
    }

    #[test]
    fn test_nats_credential_file_format() {
        let account_keypair = NKeyPair::generate(NKeyType::Account, None)
            .expect("Failed to generate account key pair");

        let user_keypair = NKeyPair::generate(NKeyType::User, None)
            .expect("Failed to generate user key pair");

        let jwt = NatsJwt::generate_user(
            &user_keypair,
            &account_keypair,
            "Test User".to_string(),
            None,
            None,
            None,
        ).expect("Failed to generate user JWT");

        let credential = NatsCredential::new(jwt, user_keypair.seed.clone(), Some("test-user".to_string()));
        let creds_file = credential.to_credential_file();

        assert!(creds_file.contains("-----BEGIN NATS USER JWT-----"));
        assert!(creds_file.contains("------END NATS USER JWT------"));
        assert!(creds_file.contains("-----BEGIN USER NKEY SEED-----"));
        assert!(creds_file.contains("------END USER NKEY SEED------"));
        assert!(creds_file.contains("IMPORTANT"));
    }

    #[test]
    fn test_complete_credential_workflow() {
        // Generate operator
        let operator_keypair = NKeyPair::generate(NKeyType::Operator, Some("thecowboyai".to_string()))
            .expect("Failed to generate operator key pair");

        let operator_jwt = NatsJwt::generate_operator(
            &operator_keypair,
            "thecowboyai".to_string(),
            vec![],
        ).expect("Failed to generate operator JWT");

        // Generate account
        let account_keypair = NKeyPair::generate(NKeyType::Account, Some("Core".to_string()))
            .expect("Failed to generate account key pair");

        let account_jwt = NatsJwt::generate_account(
            &account_keypair,
            &operator_keypair,
            "Core".to_string(),
            vec![],
            Some(AccountLimits::default()),
            None,
        ).expect("Failed to generate account JWT");

        // Generate user
        let user_keypair = NKeyPair::generate(NKeyType::User, Some("organization-service".to_string()))
            .expect("Failed to generate user key pair");

        let permissions = Permissions {
            pub_allow: Some(vec!["thecowboyai.org.organization.>".to_string()]),
            pub_deny: None,
            sub_allow: Some(vec!["thecowboyai.org.organization.>".to_string()]),
            sub_deny: None,
        };

        let user_jwt = NatsJwt::generate_user(
            &user_keypair,
            &account_keypair,
            "organization-service".to_string(),
            Some(permissions),
            None,
            None,
        ).expect("Failed to generate user JWT");

        let credential = NatsCredential::new(
            user_jwt,
            user_keypair.seed.clone(),
            Some("organization-service".to_string()),
        );

        // Verify hierarchy
        assert!(operator_jwt.is_valid());
        assert!(account_jwt.is_valid());
        assert_eq!(account_jwt.issuer, operator_keypair.public_key);
        assert_eq!(credential.jwt.issuer, account_keypair.public_key);

        // Verify credential file format
        let creds_file = credential.to_credential_file();
        assert!(creds_file.contains("-----BEGIN NATS USER JWT-----"));
        assert!(creds_file.contains(user_keypair.seed_string()));
    }
}
