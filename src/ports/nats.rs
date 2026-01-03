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

// ============================================================================
// JETSTREAM PORT
// ============================================================================

/// Port for JetStream event publishing and subscribing
///
/// This is the interface that our domain uses for event streaming.
/// The actual implementation connects to a NATS JetStream server.
#[async_trait]
pub trait JetStreamPort: Send + Sync {
    /// Publish an event to JetStream
    ///
    /// Returns the sequence number assigned by JetStream
    async fn publish(
        &self,
        subject: &str,
        payload: &[u8],
        headers: Option<&JetStreamHeaders>,
    ) -> Result<PublishAck, JetStreamError>;

    /// Publish with explicit message ID for deduplication
    async fn publish_with_id(
        &self,
        subject: &str,
        payload: &[u8],
        message_id: &str,
        headers: Option<&JetStreamHeaders>,
    ) -> Result<PublishAck, JetStreamError>;

    /// Subscribe to a stream with a durable consumer
    async fn subscribe(
        &self,
        stream: &str,
        consumer: &str,
        filter_subject: Option<&str>,
    ) -> Result<Box<dyn JetStreamSubscription>, JetStreamError>;

    /// Get stream info
    async fn stream_info(&self, stream: &str) -> Result<StreamInfo, JetStreamError>;

    /// Create or update a stream
    async fn create_stream(&self, config: &JetStreamStreamConfig) -> Result<StreamInfo, JetStreamError>;

    /// Create or update a consumer
    async fn create_consumer(
        &self,
        stream: &str,
        config: &JetStreamConsumerConfig,
    ) -> Result<ConsumerInfo, JetStreamError>;

    /// Check if connected to JetStream
    async fn is_connected(&self) -> bool;

    // =========================================================================
    // Key-Value Store Operations
    // =========================================================================

    /// Get a value from a KV bucket
    async fn kv_get(&self, bucket: &str, key: &str) -> Result<Option<Vec<u8>>, JetStreamError>;

    /// Put a value into a KV bucket
    async fn kv_put(&self, bucket: &str, key: &str, value: &[u8]) -> Result<u64, JetStreamError>;

    /// Delete a key from a KV bucket
    async fn kv_delete(&self, bucket: &str, key: &str) -> Result<(), JetStreamError>;

    /// List keys matching a prefix in a KV bucket
    async fn kv_keys(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, JetStreamError>;

    /// Create a KV bucket if it doesn't exist
    async fn kv_create_bucket(&self, bucket: &str, config: &KvBucketConfig) -> Result<(), JetStreamError>;
}

/// Configuration for KV bucket
#[derive(Debug, Clone, Default)]
pub struct KvBucketConfig {
    /// Bucket description
    pub description: Option<String>,
    /// Maximum bucket size in bytes
    pub max_bytes: Option<i64>,
    /// TTL for entries (in seconds)
    pub ttl_seconds: Option<u64>,
    /// Number of replicas
    pub replicas: Option<usize>,
}

impl KvBucketConfig {
    /// Create new config with description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set max size in bytes
    pub fn with_max_bytes(mut self, bytes: i64) -> Self {
        self.max_bytes = Some(bytes);
        self
    }

    /// Set TTL in seconds
    pub fn with_ttl(mut self, seconds: u64) -> Self {
        self.ttl_seconds = Some(seconds);
        self
    }

    /// Set replica count
    pub fn with_replicas(mut self, replicas: usize) -> Self {
        self.replicas = Some(replicas);
        self
    }
}

/// JetStream message subscription interface
#[async_trait]
pub trait JetStreamSubscription: Send + Sync {
    /// Get the next message from the subscription
    async fn next(&mut self) -> Option<JetStreamMessage>;

    /// Unsubscribe from the stream
    async fn unsubscribe(self: Box<Self>) -> Result<(), JetStreamError>;
}

/// A message received from JetStream
#[derive(Debug, Clone)]
pub struct JetStreamMessage {
    /// Subject the message was published on
    pub subject: String,

    /// Message payload
    pub payload: Vec<u8>,

    /// Message headers
    pub headers: JetStreamHeaders,

    /// Stream sequence number
    pub sequence: u64,

    /// Timestamp (nanoseconds since epoch)
    pub timestamp: i64,

    /// Delivery count (for redelivery tracking)
    pub num_delivered: u64,

    /// Reply subject for acknowledgement
    pub reply: Option<String>,
}

impl JetStreamMessage {
    /// Acknowledge the message
    pub fn ack_data(&self) -> Option<Vec<u8>> {
        // Standard NATS ack payload
        Some(b"+ACK".to_vec())
    }

    /// Get the message ID (for deduplication)
    pub fn message_id(&self) -> Option<&str> {
        self.headers.get("Nats-Msg-Id")
    }

    /// Get the correlation ID
    pub fn correlation_id(&self) -> Option<&str> {
        self.headers.get("CIM-Correlation-Id")
    }

    /// Get the causation ID
    pub fn causation_id(&self) -> Option<&str> {
        self.headers.get("CIM-Causation-Id")
    }

    /// Get the event type
    pub fn event_type(&self) -> Option<&str> {
        self.headers.get("CIM-Event-Type")
    }
}

/// Headers for JetStream messages
#[derive(Debug, Clone, Default)]
pub struct JetStreamHeaders {
    inner: HashMap<String, String>,
}

impl JetStreamHeaders {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|s| s.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.inner.iter()
    }

    /// Create headers from CIM header specification
    pub fn from_cim_headers(
        event_id: &Uuid,
        correlation_id: &Uuid,
        causation_id: &Uuid,
        event_type: &str,
        source: &str,
    ) -> Self {
        use chrono::Utc;

        let mut headers = Self::new();
        headers.insert("Nats-Msg-Id", event_id.to_string());
        headers.insert("CIM-Correlation-Id", correlation_id.to_string());
        headers.insert("CIM-Causation-Id", causation_id.to_string());
        headers.insert("CIM-Event-Type", event_type);
        headers.insert("CIM-Timestamp", Utc::now().to_rfc3339());
        headers.insert("CIM-Source", source);
        headers.insert("CIM-Content-Type", "application/json");
        headers.insert("CIM-Schema-Version", "1.0");
        headers
    }
}

/// Acknowledgement from JetStream publish
#[derive(Debug, Clone)]
pub struct PublishAck {
    /// Stream the message was published to
    pub stream: String,

    /// Sequence number in the stream
    pub sequence: u64,

    /// Whether this is a duplicate message
    pub duplicate: bool,

    /// Domain the message was published in (if set)
    pub domain: Option<String>,
}

/// Stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    /// Stream name
    pub name: String,

    /// Number of messages in stream
    pub messages: u64,

    /// Number of bytes in stream
    pub bytes: u64,

    /// First sequence number
    pub first_seq: u64,

    /// Last sequence number
    pub last_seq: u64,

    /// Number of consumers
    pub consumer_count: usize,

    /// Subjects the stream captures
    pub subjects: Vec<String>,
}

/// Consumer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerInfo {
    /// Consumer name
    pub name: String,

    /// Stream name
    pub stream: String,

    /// Number of pending messages
    pub num_pending: u64,

    /// Number of redelivered messages
    pub num_redelivered: u64,

    /// Last delivered sequence
    pub delivered_seq: u64,

    /// Ack floor sequence
    pub ack_floor_seq: u64,
}

/// JetStream stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamStreamConfig {
    /// Stream name
    pub name: String,

    /// Subjects to capture
    pub subjects: Vec<String>,

    /// Retention policy
    pub retention: JsRetentionPolicy,

    /// Storage type
    pub storage: JsStorageType,

    /// Number of replicas
    pub replicas: u32,

    /// Maximum message age (seconds)
    pub max_age_secs: Option<u64>,

    /// Maximum messages per subject
    pub max_msgs_per_subject: Option<i64>,

    /// Deduplication window (nanoseconds)
    pub duplicate_window_ns: Option<i64>,

    /// Stream description
    pub description: Option<String>,
}

impl Default for JetStreamStreamConfig {
    fn default() -> Self {
        Self {
            name: "KEYS_EVENTS".to_string(),
            subjects: vec!["keys.events.>".to_string()],
            retention: JsRetentionPolicy::Limits,
            storage: JsStorageType::File,
            replicas: 1,
            max_age_secs: Some(30 * 24 * 60 * 60), // 30 days
            max_msgs_per_subject: Some(1000),
            duplicate_window_ns: Some(2 * 60 * 1_000_000_000), // 2 minutes
            description: Some("CIM Keys domain events".to_string()),
        }
    }
}

/// JetStream consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamConsumerConfig {
    /// Consumer name
    pub name: String,

    /// Durable name
    pub durable_name: Option<String>,

    /// Filter subject
    pub filter_subject: Option<String>,

    /// Ack policy
    pub ack_policy: JsAckPolicy,

    /// Ack wait (nanoseconds)
    pub ack_wait_ns: Option<i64>,

    /// Max deliver attempts
    pub max_deliver: Option<i64>,

    /// Deliver policy
    pub deliver_policy: JsDeliverPolicy,

    /// Description
    pub description: Option<String>,
}

impl Default for JetStreamConsumerConfig {
    fn default() -> Self {
        Self {
            name: "keys-processor".to_string(),
            durable_name: Some("keys-processor".to_string()),
            filter_subject: None,
            ack_policy: JsAckPolicy::Explicit,
            ack_wait_ns: Some(30 * 1_000_000_000), // 30 seconds
            max_deliver: Some(3),
            deliver_policy: JsDeliverPolicy::All,
            description: Some("CIM Keys event processor".to_string()),
        }
    }
}

/// JetStream retention policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsRetentionPolicy {
    Limits,
    WorkQueue,
    Interest,
}

/// JetStream storage type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsStorageType {
    File,
    Memory,
}

/// JetStream ack policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsAckPolicy {
    None,
    All,
    Explicit,
}

/// JetStream deliver policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsDeliverPolicy {
    All,
    New,
    ByStartSequence(u64),
    ByStartTime(i64),
    LastPerSubject,
}

/// Errors for JetStream operations
#[derive(Debug, thiserror::Error)]
pub enum JetStreamError {
    #[error("Publish failed: {0}")]
    PublishFailed(String),

    #[error("Subscribe failed: {0}")]
    SubscribeFailed(String),

    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Consumer not found: {0}")]
    ConsumerNotFound(String),

    #[error("Stream creation failed: {0}")]
    StreamCreationFailed(String),

    #[error("Consumer creation failed: {0}")]
    ConsumerCreationFailed(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Duplicate message: {0}")]
    DuplicateMessage(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Acknowledgement failed: {0}")]
    AckFailed(String),

    #[error("KV operation failed: {0}")]
    KvError(String),
}

// ============================================================================
// JETSTREAM PORT TESTS
// ============================================================================

#[cfg(test)]
mod jetstream_tests {
    use super::*;

    #[test]
    fn test_jetstream_headers_new() {
        let mut headers = JetStreamHeaders::new();
        headers.insert("key", "value");
        assert_eq!(headers.get("key"), Some("value"));
    }

    #[test]
    fn test_jetstream_headers_from_cim() {
        let event_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();
        let causation_id = Uuid::now_v7();

        let headers = JetStreamHeaders::from_cim_headers(
            &event_id,
            &correlation_id,
            &causation_id,
            "KeyGenerated",
            "cim-keys",
        );

        assert_eq!(headers.get("CIM-Event-Type"), Some("KeyGenerated"));
        assert_eq!(headers.get("CIM-Source"), Some("cim-keys"));
        assert_eq!(headers.get("Nats-Msg-Id"), Some(event_id.to_string().as_str()));
    }

    #[test]
    fn test_stream_config_default() {
        let config = JetStreamStreamConfig::default();

        assert_eq!(config.name, "KEYS_EVENTS");
        assert_eq!(config.subjects, vec!["keys.events.>"]);
        assert_eq!(config.retention, JsRetentionPolicy::Limits);
        assert_eq!(config.storage, JsStorageType::File);
    }

    #[test]
    fn test_consumer_config_default() {
        let config = JetStreamConsumerConfig::default();

        assert_eq!(config.name, "keys-processor");
        assert!(config.durable_name.is_some());
        assert_eq!(config.ack_policy, JsAckPolicy::Explicit);
    }

    #[test]
    fn test_publish_ack() {
        let ack = PublishAck {
            stream: "KEYS_EVENTS".to_string(),
            sequence: 42,
            duplicate: false,
            domain: None,
        };

        assert_eq!(ack.stream, "KEYS_EVENTS");
        assert_eq!(ack.sequence, 42);
        assert!(!ack.duplicate);
    }

    #[test]
    fn test_jetstream_message() {
        let mut headers = JetStreamHeaders::new();
        headers.insert("Nats-Msg-Id", "msg-123");
        headers.insert("CIM-Correlation-Id", "corr-456");
        headers.insert("CIM-Event-Type", "KeyGenerated");

        let msg = JetStreamMessage {
            subject: "keys.events.key.generated".to_string(),
            payload: b"{}".to_vec(),
            headers,
            sequence: 1,
            timestamp: 0,
            num_delivered: 1,
            reply: None,
        };

        assert_eq!(msg.message_id(), Some("msg-123"));
        assert_eq!(msg.correlation_id(), Some("corr-456"));
        assert_eq!(msg.event_type(), Some("KeyGenerated"));
    }
}