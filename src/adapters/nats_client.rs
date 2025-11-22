//! Real NATS Client Adapter with JetStream and IPLD Support
//!
//! This adapter provides real NATS connectivity with:
//! - Event publishing to NATS JetStream
//! - IPLD/CID content-addressed events
//! - Offline-first design with local queueing
//! - Automatic retry and reconnection
//! - Subject-based routing for different event types
//!
//! ## Air-Gapped Offline Architecture
//!
//! **CRITICAL**: This adapter ONLY connects to localhost:4222. This system is
//! designed for air-gapped offline operation with NATS running locally on the
//! same machine. There is no configuration for remote NATS servers.

use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use cim_domain::DomainEvent;
use crate::config::{NatsConfig, NATS_URL};
use crate::events::KeyEvent;

#[cfg(feature = "ipld")]
use crate::ipld_support::ContentAddressedEvent;

#[cfg(feature = "nats-client")]
use async_nats::{Client, ConnectOptions};

/// NATS Client Adapter State
pub struct NatsClientAdapter {
    config: NatsConfig,

    #[cfg(feature = "nats-client")]
    client: Arc<RwLock<Option<Client>>>,

    /// Local queue for offline events
    offline_queue: Arc<RwLock<Vec<QueuedEvent>>>,

    /// Path to offline queue storage
    queue_path: PathBuf,
}

/// Event queued for later publishing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct QueuedEvent {
    subject: String,
    payload: Vec<u8>,
    timestamp: chrono::DateTime<chrono::Utc>,
    retries: u32,
}

impl NatsClientAdapter {
    /// Create a new NATS client adapter
    pub fn new(config: NatsConfig, queue_path: PathBuf) -> Self {
        Self {
            config,
            #[cfg(feature = "nats-client")]
            client: Arc::new(RwLock::new(None)),
            offline_queue: Arc::new(RwLock::new(Vec::new())),
            queue_path,
        }
    }

    /// Connect to NATS server (localhost:4222 only)
    #[cfg(feature = "nats-client")]
    pub async fn connect(&self) -> Result<(), NatsClientError> {
        if !self.config.enabled {
            debug!("NATS is disabled, skipping connection");
            return Ok(());
        }

        info!("Connecting to NATS at {} (air-gapped localhost only)", NATS_URL);

        let mut connect_opts = ConnectOptions::new();

        // Set connection timeout
        connect_opts = connect_opts.connection_timeout(
            std::time::Duration::from_secs(self.config.connection_timeout_secs)
        );

        // Add credentials if provided (typically not needed for localhost)
        if let Some(creds_path) = &self.config.credentials_file {
            connect_opts = connect_opts
                .credentials_file(creds_path.to_str().ok_or_else(|| {
                    NatsClientError::ConfigError("Invalid credentials path".to_string())
                })?)
                .await
                .map_err(|e| NatsClientError::ConnectionError(e.to_string()))?;
        }

        // Connect to NATS (localhost:4222 only - hardcoded for air-gapped operation)
        let client = connect_opts
            .connect(NATS_URL)
            .await
            .map_err(|e| NatsClientError::ConnectionError(e.to_string()))?;

        info!("Successfully connected to NATS");

        // Store client
        let mut client_lock = self.client.write().await;
        *client_lock = Some(client);

        Ok(())
    }

    #[cfg(not(feature = "nats-client"))]
    pub async fn connect(&self) -> Result<(), NatsClientError> {
        warn!("NATS client feature not enabled, operating in offline mode");
        Ok(())
    }

    /// Publish an event to NATS
    pub async fn publish_event(&self, event: &KeyEvent) -> Result<(), NatsClientError> {
        let subject = self.build_subject(event);

        // Serialize event
        let payload = serde_json::to_vec(event)
            .map_err(|e| NatsClientError::SerializationError(e.to_string()))?;

        // Add CID if IPLD is enabled
        #[cfg(feature = "ipld")]
        let payload_with_cid = if self.config.enable_ipld {
            let ca_event = ContentAddressedEvent::new(event)
                .map_err(|e| NatsClientError::IpldError(e.to_string()))?;

            serde_json::to_vec(&ca_event)
                .map_err(|e| NatsClientError::SerializationError(e.to_string()))?
        } else {
            payload.clone()
        };

        #[cfg(not(feature = "ipld"))]
        let payload_with_cid = payload.clone();

        // Try to publish if connected
        #[cfg(feature = "nats-client")]
        {
            let client_lock = self.client.read().await;
            if let Some(client) = client_lock.as_ref() {
                match self.publish_to_nats(client, &subject, &payload_with_cid).await {
                    Ok(_) => {
                        debug!("Published event to NATS: {}", subject);
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("Failed to publish to NATS: {}, queuing for later", e);
                        // Fall through to queue
                    }
                }
            }
        }

        // Queue for later if not connected or publish failed
        self.queue_event(subject, payload_with_cid).await?;
        Ok(())
    }

    /// Publish to NATS JetStream
    #[cfg(feature = "nats-client")]
    async fn publish_to_nats(
        &self,
        client: &Client,
        subject: &str,
        payload: &[u8],
    ) -> Result<(), NatsClientError> {
        if self.config.enable_jetstream {
            // Get JetStream context
            let jetstream = async_nats::jetstream::new(client.clone());

            // Publish to JetStream
            jetstream
                .publish(subject.to_string(), payload.to_vec().into())
                .await
                .map_err(|e| NatsClientError::PublishError(e.to_string()))?;

            debug!("Published to JetStream: {}", subject);
        } else {
            // Core NATS publish
            client
                .publish(subject.to_string(), payload.to_vec().into())
                .await
                .map_err(|e| NatsClientError::PublishError(e.to_string()))?;

            debug!("Published to core NATS: {}", subject);
        }

        Ok(())
    }

    /// Queue event for later publishing
    async fn queue_event(&self, subject: String, payload: Vec<u8>) -> Result<(), NatsClientError> {
        let queued = QueuedEvent {
            subject,
            payload,
            timestamp: chrono::Utc::now(),
            retries: 0,
        };

        let mut queue = self.offline_queue.write().await;
        queue.push(queued);

        // Persist queue to disk
        self.save_queue(&queue).await?;

        info!("Event queued for later publishing (queue size: {})", queue.len());
        Ok(())
    }

    /// Save queue to disk
    async fn save_queue(&self, queue: &[QueuedEvent]) -> Result<(), NatsClientError> {
        let json = serde_json::to_string_pretty(queue)
            .map_err(|e| NatsClientError::SerializationError(e.to_string()))?;

        tokio::fs::write(&self.queue_path, json)
            .await
            .map_err(|e| NatsClientError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Load queue from disk
    pub async fn load_queue(&self) -> Result<(), NatsClientError> {
        if !self.queue_path.exists() {
            return Ok(());
        }

        let json = tokio::fs::read_to_string(&self.queue_path)
            .await
            .map_err(|e| NatsClientError::IoError(e.to_string()))?;

        let loaded: Vec<QueuedEvent> = serde_json::from_str(&json)
            .map_err(|e| NatsClientError::SerializationError(e.to_string()))?;

        let mut queue = self.offline_queue.write().await;
        *queue = loaded;

        info!("Loaded {} queued events from disk", queue.len());
        Ok(())
    }

    /// Flush queued events to NATS
    #[cfg(feature = "nats-client")]
    pub async fn flush_queue(&self) -> Result<usize, NatsClientError> {
        let client_lock = self.client.read().await;
        let client = client_lock.as_ref()
            .ok_or_else(|| NatsClientError::NotConnected)?;

        let mut queue = self.offline_queue.write().await;
        let mut published = 0;
        let mut failed = Vec::new();

        for mut event in queue.drain(..) {
            match self.publish_to_nats(client, &event.subject, &event.payload).await {
                Ok(_) => {
                    published += 1;
                    debug!("Flushed queued event: {}", event.subject);
                }
                Err(e) => {
                    event.retries += 1;
                    if event.retries < self.config.max_retries {
                        warn!("Failed to flush event (retry {}/{}): {}",
                              event.retries, self.config.max_retries, e);
                        failed.push(event);
                    } else {
                        error!("Dropping event after {} retries: {}",
                               self.config.max_retries, event.subject);
                    }
                }
            }
        }

        // Keep failed events in queue
        *queue = failed;
        self.save_queue(&queue).await?;

        info!("Flushed {} events, {} remain in queue", published, queue.len());
        Ok(published)
    }

    #[cfg(not(feature = "nats-client"))]
    pub async fn flush_queue(&self) -> Result<usize, NatsClientError> {
        Ok(0)
    }

    /// Build NATS subject from event type
    fn build_subject(&self, event: &KeyEvent) -> String {
        let event_type = event.event_type();
        let aggregate_type = Self::aggregate_type_from_event(event);

        format!(
            "{}.{}.{}",
            self.config.subject_prefix,
            aggregate_type,
            event_type.to_lowercase()
        )
    }

    /// Extract aggregate type from event
    fn aggregate_type_from_event(event: &KeyEvent) -> &'static str {
        use crate::events::KeyEvent;

        match event {
            KeyEvent::KeyGenerated(_) | KeyEvent::KeyImported(_) |
            KeyEvent::KeyExported(_) | KeyEvent::KeyRevoked(_) => "key",

            KeyEvent::CertificateGenerated(_) | KeyEvent::CertificateSigned(_) |
            KeyEvent::CertificateExported(_) => "certificate",

            KeyEvent::YubiKeyProvisioned(_) | KeyEvent::PinConfigured(_) |
            KeyEvent::PukConfigured(_) | KeyEvent::ManagementKeyRotated(_) |
            KeyEvent::YubiKeyDetected(_) | KeyEvent::KeyGeneratedInSlot(_) |
            KeyEvent::CertificateImportedToSlot(_) => "yubikey",

            KeyEvent::NatsOperatorCreated(_) => "nats.operator",
            KeyEvent::NatsAccountCreated(_) => "nats.account",
            KeyEvent::NatsUserCreated(_) => "nats.user",
            KeyEvent::NatsSigningKeyGenerated(_) | KeyEvent::NKeyGenerated(_) => "nats.key",
            KeyEvent::JwtClaimsCreated(_) | KeyEvent::JwtSigned(_) => "nats.jwt",

            KeyEvent::PersonCreated(_) => "person",
            KeyEvent::OrganizationCreated(_) => "organization",
            KeyEvent::OrganizationalUnitCreated(_) => "organizational_unit",
            KeyEvent::LocationCreated(_) => "location",
            KeyEvent::RoleCreated(_) => "role",
            KeyEvent::PolicyCreated(_) => "policy",
            KeyEvent::RelationshipEstablished(_) => "relationship",

            _ => "event",
        }
    }

    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        let queue = self.offline_queue.read().await;
        queue.len()
    }

    /// Check if connected to NATS
    #[cfg(feature = "nats-client")]
    pub async fn is_connected(&self) -> bool {
        let client_lock = self.client.read().await;
        client_lock.is_some()
    }

    #[cfg(not(feature = "nats-client"))]
    pub async fn is_connected(&self) -> bool {
        false
    }
}

/// NATS client errors
#[derive(Debug, Error)]
pub enum NatsClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Publish error: {0}")]
    PublishError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("IPLD error: {0}")]
    IpldError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Not connected to NATS")]
    NotConnected,

    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_build_subject() {
        let config = NatsConfig::default();
        let adapter = NatsClientAdapter::new(config, PathBuf::from("/tmp/queue.json"));

        let event = KeyEvent::KeyGenerated(KeyGeneratedEvent {
            key_id: Uuid::now_v7(),
            algorithm: KeyAlgorithm::Ed25519,
            purpose: KeyPurpose::Signing,
            generated_at: chrono::Utc::now(),
            generated_by: "test".to_string(),
            hardware_backed: false,
            metadata: KeyMetadata {
                label: "test".to_string(),
                description: None,
                tags: vec![],
                attributes: std::collections::HashMap::new(),
                jwt_kid: None,
                jwt_alg: None,
                jwt_use: None,
            },
            owner: None,
        });

        let subject = adapter.build_subject(&event);
        assert_eq!(subject, "cim.keys.key.keygenerated");
    }

    #[tokio::test]
    async fn test_offline_queue() {
        let temp_dir = tempfile::tempdir().unwrap();
        let queue_path = temp_dir.path().join("queue.json");

        let config = NatsConfig::default();
        let adapter = NatsClientAdapter::new(config, queue_path.clone());

        // Queue an event
        adapter.queue_event(
            "test.subject".to_string(),
            b"test payload".to_vec(),
        ).await.unwrap();

        assert_eq!(adapter.queue_size().await, 1);

        // Create new adapter and load queue
        let adapter2 = NatsClientAdapter::new(NatsConfig::default(), queue_path);
        adapter2.load_queue().await.unwrap();

        assert_eq!(adapter2.queue_size().await, 1);
    }
}
