// Copyright (c) 2025 - Cowboy AI, LLC.

//! JetStream Configuration and Subject Patterns for cim-keys
//!
//! This module defines JetStream streams, consumers, and subject patterns
//! for durable event storage and replay.
//!
//! ## Stream Architecture
//!
//! ```text
//! KEYS_EVENTS Stream
//! ├── Subject: keys.events.>
//! │   ├── keys.events.key.generated
//! │   ├── keys.events.key.revoked
//! │   ├── keys.events.certificate.created
//! │   ├── keys.events.certificate.signed
//! │   ├── keys.events.yubikey.provisioned
//! │   └── keys.events.bootstrap.*
//! ├── Retention: Limits (WorkQueue for commands)
//! ├── Storage: File
//! └── Replicas: 3 (for production clusters)
//! ```
//!
//! ## Consumer Patterns
//!
//! - **Durable Consumers**: For services that need exactly-once processing
//! - **Ephemeral Consumers**: For real-time monitoring and dashboards
//!
//! ## Usage
//!
//! ```ignore
//! use cim_keys::domain::nats::jetstream::{KEYS_EVENTS_STREAM, events};
//!
//! // Publish a key generated event
//! let subject = events::key_generated();
//! jetstream.publish(subject.as_str(), payload).await?;
//!
//! // Subscribe to all key events
//! let consumer = jetstream.create_consumer(KEYS_EVENTS_STREAM, config).await?;
//! ```

use super::subjects::Subject;
use serde::{Deserialize, Serialize};

// ============================================================================
// STREAM CONSTANTS
// ============================================================================

/// Stream name for key management events
pub const KEYS_EVENTS_STREAM: &str = "KEYS_EVENTS";

/// Stream name for key commands (work queue pattern)
pub const KEYS_COMMANDS_STREAM: &str = "KEYS_COMMANDS";

/// Default stream subject prefix
pub const KEYS_SUBJECT_PREFIX: &str = "keys";

/// Default retention period in seconds (30 days)
pub const DEFAULT_RETENTION_SECONDS: u64 = 30 * 24 * 60 * 60;

/// Default max messages per subject (for deduplication window)
pub const DEFAULT_MAX_MSGS_PER_SUBJECT: i64 = 1000;

/// Deduplication window in nanoseconds (2 minutes)
pub const DEDUP_WINDOW_NS: i64 = 2 * 60 * 1_000_000_000;

// ============================================================================
// STREAM CONFIGURATION
// ============================================================================

/// JetStream stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Stream name
    pub name: String,

    /// Subject patterns this stream captures
    pub subjects: Vec<String>,

    /// Retention policy
    pub retention: RetentionPolicy,

    /// Storage type
    pub storage: StorageType,

    /// Number of replicas (for HA)
    pub replicas: u32,

    /// Maximum age of messages in seconds
    pub max_age_seconds: Option<u64>,

    /// Maximum messages per subject
    pub max_msgs_per_subject: Option<i64>,

    /// Deduplication window in nanoseconds
    pub duplicate_window: Option<i64>,

    /// Description
    pub description: Option<String>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            name: KEYS_EVENTS_STREAM.to_string(),
            subjects: vec![format!("{}.events.>", KEYS_SUBJECT_PREFIX)],
            retention: RetentionPolicy::Limits,
            storage: StorageType::File,
            replicas: 1,
            max_age_seconds: Some(DEFAULT_RETENTION_SECONDS),
            max_msgs_per_subject: Some(DEFAULT_MAX_MSGS_PER_SUBJECT),
            duplicate_window: Some(DEDUP_WINDOW_NS),
            description: Some("CIM Keys domain events".to_string()),
        }
    }
}

impl StreamConfig {
    /// Create configuration for the KEYS_EVENTS stream
    pub fn keys_events() -> Self {
        Self::default()
    }

    /// Create configuration for the KEYS_COMMANDS stream (work queue)
    pub fn keys_commands() -> Self {
        Self {
            name: KEYS_COMMANDS_STREAM.to_string(),
            subjects: vec![format!("{}.commands.>", KEYS_SUBJECT_PREFIX)],
            retention: RetentionPolicy::WorkQueue,
            storage: StorageType::File,
            replicas: 1,
            max_age_seconds: Some(60 * 60), // 1 hour for commands
            max_msgs_per_subject: None,
            duplicate_window: Some(DEDUP_WINDOW_NS),
            description: Some("CIM Keys command queue".to_string()),
        }
    }

    /// Set number of replicas (for production clusters)
    pub fn with_replicas(mut self, replicas: u32) -> Self {
        self.replicas = replicas;
        self
    }

    /// Set retention period
    pub fn with_max_age(mut self, seconds: u64) -> Self {
        self.max_age_seconds = Some(seconds);
        self
    }
}

/// Stream retention policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionPolicy {
    /// Keep messages based on limits (max age, max messages, max bytes)
    Limits,
    /// Work queue - delete messages once acknowledged
    WorkQueue,
    /// Interest - delete messages when no consumers are interested
    Interest,
}

/// Stream storage type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    /// Store on disk
    File,
    /// Store in memory
    Memory,
}

// ============================================================================
// CONSUMER CONFIGURATION
// ============================================================================

/// Consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    /// Consumer name (durable name)
    pub name: String,

    /// Durable name for resumable consumers
    pub durable_name: Option<String>,

    /// Filter subject
    pub filter_subject: Option<String>,

    /// Acknowledgement policy
    pub ack_policy: AckPolicy,

    /// Acknowledgement wait time in nanoseconds
    pub ack_wait: Option<i64>,

    /// Maximum deliver attempts
    pub max_deliver: Option<i64>,

    /// Deliver policy
    pub deliver_policy: DeliverPolicy,

    /// Description
    pub description: Option<String>,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            name: "keys-processor".to_string(),
            durable_name: Some("keys-processor".to_string()),
            filter_subject: None,
            ack_policy: AckPolicy::Explicit,
            ack_wait: Some(30 * 1_000_000_000), // 30 seconds
            max_deliver: Some(3),
            deliver_policy: DeliverPolicy::All,
            description: Some("CIM Keys event processor".to_string()),
        }
    }
}

impl ConsumerConfig {
    /// Create a durable consumer for key events
    pub fn key_events_processor() -> Self {
        Self::default()
    }

    /// Create a consumer for bootstrap events only
    pub fn bootstrap_processor() -> Self {
        Self {
            name: "bootstrap-processor".to_string(),
            durable_name: Some("bootstrap-processor".to_string()),
            filter_subject: Some(format!("{}.events.bootstrap.>", KEYS_SUBJECT_PREFIX)),
            ack_policy: AckPolicy::Explicit,
            ack_wait: Some(60 * 1_000_000_000), // 60 seconds for bootstrap
            max_deliver: Some(1), // Bootstrap events should be processed once
            deliver_policy: DeliverPolicy::All,
            description: Some("Bootstrap event processor".to_string()),
        }
    }

    /// Create an ephemeral consumer for monitoring
    pub fn monitoring() -> Self {
        Self {
            name: "keys-monitor".to_string(),
            durable_name: None, // Ephemeral
            filter_subject: None,
            ack_policy: AckPolicy::None,
            ack_wait: None,
            max_deliver: None,
            deliver_policy: DeliverPolicy::New,
            description: Some("Real-time event monitor".to_string()),
        }
    }

    /// Set filter subject
    pub fn with_filter(mut self, subject: impl Into<String>) -> Self {
        self.filter_subject = Some(subject.into());
        self
    }
}

/// Acknowledgement policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckPolicy {
    /// No acknowledgement required
    None,
    /// Acknowledge all messages up to sequence
    All,
    /// Acknowledge each message explicitly
    Explicit,
}

/// Deliver policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliverPolicy {
    /// Deliver all messages
    All,
    /// Deliver only new messages
    New,
    /// Deliver from a specific sequence
    ByStartSequence(u64),
    /// Deliver from a specific time
    ByStartTime(i64),
    /// Deliver last message per subject
    LastPerSubject,
}

// ============================================================================
// EVENT SUBJECT FACTORIES
// ============================================================================

/// JetStream event subjects for key domain events
pub mod events {
    use super::*;

    /// Base subject for all key events
    fn base() -> Subject {
        Subject::new(KEYS_SUBJECT_PREFIX).unit("events")
    }

    // ---- Key Events ----

    /// Subject for key generated events
    pub fn key_generated() -> Subject {
        base().entity("key").operation("generated")
    }

    /// Subject for key revoked events
    pub fn key_revoked() -> Subject {
        base().entity("key").operation("revoked")
    }

    /// Subject for key rotated events
    pub fn key_rotated() -> Subject {
        base().entity("key").operation("rotated")
    }

    /// Subject for key exported events
    pub fn key_exported() -> Subject {
        base().entity("key").operation("exported")
    }

    /// Subject for key imported events
    pub fn key_imported() -> Subject {
        base().entity("key").operation("imported")
    }

    // ---- Certificate Events ----

    /// Subject for certificate created events
    pub fn certificate_created() -> Subject {
        base().entity("certificate").operation("created")
    }

    /// Subject for certificate signed events
    pub fn certificate_signed() -> Subject {
        base().entity("certificate").operation("signed")
    }

    /// Subject for certificate revoked events
    pub fn certificate_revoked() -> Subject {
        base().entity("certificate").operation("revoked")
    }

    /// Subject for certificate renewed events
    pub fn certificate_renewed() -> Subject {
        base().entity("certificate").operation("renewed")
    }

    // ---- YubiKey Events ----

    /// Subject for YubiKey provisioned events
    pub fn yubikey_provisioned() -> Subject {
        base().entity("yubikey").operation("provisioned")
    }

    /// Subject for YubiKey slot populated events
    pub fn yubikey_slot_populated() -> Subject {
        base().entity("yubikey").operation("slot-populated")
    }

    /// Subject for YubiKey reset events
    pub fn yubikey_reset() -> Subject {
        base().entity("yubikey").operation("reset")
    }

    // ---- Bootstrap Events ----

    /// Subject for bootstrap started events
    pub fn bootstrap_started() -> Subject {
        base().entity("bootstrap").operation("started")
    }

    /// Subject for bootstrap completed events
    pub fn bootstrap_completed() -> Subject {
        base().entity("bootstrap").operation("completed")
    }

    /// Subject for bootstrap failed events
    pub fn bootstrap_failed() -> Subject {
        base().entity("bootstrap").operation("failed")
    }

    // ---- NATS Credential Events ----

    /// Subject for NATS operator created events
    pub fn nats_operator_created() -> Subject {
        base().entity("nats").operation("operator.created")
    }

    /// Subject for NATS account created events
    pub fn nats_account_created() -> Subject {
        base().entity("nats").operation("account.created")
    }

    /// Subject for NATS user created events
    pub fn nats_user_created() -> Subject {
        base().entity("nats").operation("user.created")
    }

    // ---- Wildcard Patterns ----

    /// Subscribe to all key events
    pub fn all_keys() -> Subject {
        base().entity("key").wildcard_suffix()
    }

    /// Subscribe to all certificate events
    pub fn all_certificates() -> Subject {
        base().entity("certificate").wildcard_suffix()
    }

    /// Subscribe to all YubiKey events
    pub fn all_yubikey() -> Subject {
        base().entity("yubikey").wildcard_suffix()
    }

    /// Subscribe to all bootstrap events
    pub fn all_bootstrap() -> Subject {
        base().entity("bootstrap").wildcard_suffix()
    }

    /// Subscribe to all NATS credential events
    pub fn all_nats() -> Subject {
        base().entity("nats").wildcard_suffix()
    }

    /// Subscribe to ALL key domain events
    pub fn all() -> Subject {
        base().wildcard_suffix()
    }
}

/// JetStream command subjects for key domain commands
pub mod commands {
    use super::*;

    /// Base subject for all key commands
    fn base() -> Subject {
        Subject::new(KEYS_SUBJECT_PREFIX).unit("commands")
    }

    /// Subject for generate key commands
    pub fn generate_key() -> Subject {
        base().entity("key").operation("generate")
    }

    /// Subject for revoke key commands
    pub fn revoke_key() -> Subject {
        base().entity("key").operation("revoke")
    }

    /// Subject for rotate key commands
    pub fn rotate_key() -> Subject {
        base().entity("key").operation("rotate")
    }

    /// Subject for create certificate commands
    pub fn create_certificate() -> Subject {
        base().entity("certificate").operation("create")
    }

    /// Subject for sign certificate commands
    pub fn sign_certificate() -> Subject {
        base().entity("certificate").operation("sign")
    }

    /// Subject for provision YubiKey commands
    pub fn provision_yubikey() -> Subject {
        base().entity("yubikey").operation("provision")
    }

    /// Subject for bootstrap domain commands
    pub fn bootstrap_domain() -> Subject {
        base().entity("bootstrap").operation("domain")
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_config_defaults() {
        let config = StreamConfig::keys_events();

        assert_eq!(config.name, KEYS_EVENTS_STREAM);
        assert_eq!(config.subjects, vec!["keys.events.>"]);
        assert_eq!(config.replicas, 1);
    }

    #[test]
    fn test_stream_config_with_replicas() {
        let config = StreamConfig::keys_events().with_replicas(3);

        assert_eq!(config.replicas, 3);
    }

    #[test]
    fn test_commands_stream_config() {
        let config = StreamConfig::keys_commands();

        assert_eq!(config.name, KEYS_COMMANDS_STREAM);
        assert_eq!(config.retention, RetentionPolicy::WorkQueue);
    }

    #[test]
    fn test_consumer_config_defaults() {
        let config = ConsumerConfig::key_events_processor();

        assert_eq!(config.name, "keys-processor");
        assert!(config.durable_name.is_some());
        assert_eq!(config.ack_policy, AckPolicy::Explicit);
    }

    #[test]
    fn test_bootstrap_consumer() {
        let config = ConsumerConfig::bootstrap_processor();

        assert_eq!(config.filter_subject, Some("keys.events.bootstrap.>".to_string()));
        assert_eq!(config.max_deliver, Some(1));
    }

    #[test]
    fn test_monitoring_consumer() {
        let config = ConsumerConfig::monitoring();

        assert!(config.durable_name.is_none()); // Ephemeral
        assert_eq!(config.ack_policy, AckPolicy::None);
        assert_eq!(config.deliver_policy, DeliverPolicy::New);
    }

    #[test]
    fn test_event_subjects() {
        assert_eq!(events::key_generated().as_str(), "keys.events.key.generated");
        assert_eq!(events::certificate_created().as_str(), "keys.events.certificate.created");
        assert_eq!(events::yubikey_provisioned().as_str(), "keys.events.yubikey.provisioned");
        assert_eq!(events::bootstrap_started().as_str(), "keys.events.bootstrap.started");
    }

    #[test]
    fn test_event_wildcard_patterns() {
        assert_eq!(events::all_keys().as_str(), "keys.events.key.>");
        assert_eq!(events::all_certificates().as_str(), "keys.events.certificate.>");
        assert_eq!(events::all().as_str(), "keys.events.>");
    }

    #[test]
    fn test_command_subjects() {
        assert_eq!(commands::generate_key().as_str(), "keys.commands.key.generate");
        assert_eq!(commands::provision_yubikey().as_str(), "keys.commands.yubikey.provision");
        assert_eq!(commands::bootstrap_domain().as_str(), "keys.commands.bootstrap.domain");
    }

    #[test]
    fn test_nats_credential_events() {
        assert_eq!(events::nats_operator_created().as_str(), "keys.events.nats.operator.created");
        assert_eq!(events::nats_account_created().as_str(), "keys.events.nats.account.created");
        assert_eq!(events::nats_user_created().as_str(), "keys.events.nats.user.created");
    }
}
