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
use tracing::{info, warn};
#[cfg(feature = "nats-client")]
use tracing::{debug, error};

use crate::config::NatsConfig;
#[cfg(feature = "nats-client")]
use crate::config::NATS_URL;
use crate::events::DomainEvent;

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
    pub async fn publish_event(&self, event: &DomainEvent) -> Result<(), NatsClientError> {
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
    fn build_subject(&self, event: &DomainEvent) -> String {
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
    fn aggregate_type_from_event(event: &DomainEvent) -> &'static str {
        match event {
            DomainEvent::Key(_) => "key",
            DomainEvent::Certificate(_) => "certificate",
            DomainEvent::YubiKey(_) => "yubikey",
            DomainEvent::NatsOperator(_) => "nats.operator",
            DomainEvent::NatsAccount(_) => "nats.account",
            DomainEvent::NatsUser(_) => "nats.user",
            DomainEvent::Person(_) => "person",
            DomainEvent::Organization(_) => "organization",
            DomainEvent::Location(_) => "location",
            DomainEvent::Relationship(_) => "relationship",
            DomainEvent::Manifest(_) => "manifest",
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

// ============================================================================
// JETSTREAM ADAPTER
// ============================================================================

/// JetStream Adapter implementing the JetStreamPort trait
///
/// This provides full JetStream functionality including:
/// - Event publishing with CIM headers
/// - Stream and consumer management
/// - Durable subscriptions
/// - Deduplication via message IDs
#[cfg(feature = "nats-client")]
pub struct JetStreamAdapter {
    /// NATS client connection
    client: Arc<RwLock<Option<Client>>>,

    /// JetStream context (cached after connection)
    jetstream: Arc<RwLock<Option<async_nats::jetstream::Context>>>,

    /// Connection URL (localhost:4222 for air-gapped operation)
    url: String,
}

#[cfg(feature = "nats-client")]
impl JetStreamAdapter {
    /// Create a new JetStream adapter
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            jetstream: Arc::new(RwLock::new(None)),
            url: NATS_URL.to_string(),
        }
    }

    /// Connect to NATS and initialize JetStream context
    pub async fn connect(&self) -> Result<(), crate::ports::JetStreamError> {
        use crate::ports::JetStreamError;

        info!("Connecting to NATS JetStream at {}", self.url);

        let connect_opts = ConnectOptions::new()
            .connection_timeout(std::time::Duration::from_secs(10));

        let client = connect_opts
            .connect(&self.url)
            .await
            .map_err(|e| JetStreamError::ConnectionError(e.to_string()))?;

        info!("Connected to NATS, initializing JetStream context");

        // Create JetStream context
        let js = async_nats::jetstream::new(client.clone());

        // Store both
        {
            let mut client_lock = self.client.write().await;
            *client_lock = Some(client);
        }
        {
            let mut js_lock = self.jetstream.write().await;
            *js_lock = Some(js);
        }

        info!("JetStream adapter ready");
        Ok(())
    }

    /// Get the JetStream context
    async fn get_jetstream(&self) -> Result<async_nats::jetstream::Context, crate::ports::JetStreamError> {
        use crate::ports::JetStreamError;

        let js_lock = self.jetstream.read().await;
        js_lock.clone().ok_or_else(|| JetStreamError::ConnectionError("Not connected to JetStream".to_string()))
    }

    /// Convert port config to async-nats stream config
    fn to_nats_stream_config(config: &crate::ports::JetStreamStreamConfig) -> async_nats::jetstream::stream::Config {
        use async_nats::jetstream::stream;

        let retention = match config.retention {
            crate::ports::JsRetentionPolicy::Limits => stream::RetentionPolicy::Limits,
            crate::ports::JsRetentionPolicy::WorkQueue => stream::RetentionPolicy::WorkQueue,
            crate::ports::JsRetentionPolicy::Interest => stream::RetentionPolicy::Interest,
        };

        let storage = match config.storage {
            crate::ports::JsStorageType::File => stream::StorageType::File,
            crate::ports::JsStorageType::Memory => stream::StorageType::Memory,
        };

        stream::Config {
            name: config.name.clone(),
            subjects: config.subjects.clone(),
            retention,
            storage,
            num_replicas: config.replicas as usize,
            max_age: config.max_age_secs.map(std::time::Duration::from_secs).unwrap_or_default(),
            max_messages_per_subject: config.max_msgs_per_subject.unwrap_or(0),
            duplicate_window: config.duplicate_window_ns
                .map(|ns| std::time::Duration::from_nanos(ns as u64))
                .unwrap_or(std::time::Duration::from_secs(120)),
            description: config.description.clone(),
            ..Default::default()
        }
    }

    /// Convert port config to async-nats consumer config
    fn to_nats_consumer_config(config: &crate::ports::JetStreamConsumerConfig) -> async_nats::jetstream::consumer::pull::Config {
        use async_nats::jetstream::consumer;

        let ack_policy = match config.ack_policy {
            crate::ports::JsAckPolicy::None => consumer::AckPolicy::None,
            crate::ports::JsAckPolicy::All => consumer::AckPolicy::All,
            crate::ports::JsAckPolicy::Explicit => consumer::AckPolicy::Explicit,
        };

        let deliver_policy = match config.deliver_policy {
            crate::ports::JsDeliverPolicy::All => consumer::DeliverPolicy::All,
            crate::ports::JsDeliverPolicy::New => consumer::DeliverPolicy::New,
            crate::ports::JsDeliverPolicy::ByStartSequence(seq) => consumer::DeliverPolicy::ByStartSequence { start_sequence: seq },
            crate::ports::JsDeliverPolicy::ByStartTime(time) => {
                // Convert nanoseconds since epoch to time::OffsetDateTime
                let offset_dt = time::OffsetDateTime::from_unix_timestamp_nanos(time as i128)
                    .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
                consumer::DeliverPolicy::ByStartTime {
                    start_time: offset_dt
                }
            }
            crate::ports::JsDeliverPolicy::LastPerSubject => consumer::DeliverPolicy::LastPerSubject,
        };

        consumer::pull::Config {
            name: Some(config.name.clone()),
            durable_name: config.durable_name.clone(),
            filter_subject: config.filter_subject.clone().unwrap_or_default(),
            ack_policy,
            ack_wait: config.ack_wait_ns
                .map(|ns| std::time::Duration::from_nanos(ns as u64))
                .unwrap_or(std::time::Duration::from_secs(30)),
            max_deliver: config.max_deliver.unwrap_or(0),
            deliver_policy,
            description: config.description.clone(),
            ..Default::default()
        }
    }

    /// Convert port headers to async-nats headers
    fn to_nats_headers(headers: &crate::ports::JetStreamHeaders) -> async_nats::HeaderMap {
        let mut map = async_nats::HeaderMap::new();
        for (key, value) in headers.iter() {
            map.insert(key.as_str(), value.as_str());
        }
        map
    }
}

#[cfg(feature = "nats-client")]
impl Default for JetStreamAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "nats-client")]
#[async_trait::async_trait]
impl crate::ports::JetStreamPort for JetStreamAdapter {
    async fn publish(
        &self,
        subject: &str,
        payload: &[u8],
        headers: Option<&crate::ports::JetStreamHeaders>,
    ) -> Result<crate::ports::PublishAck, crate::ports::JetStreamError> {
        use crate::ports::JetStreamError;

        let js = self.get_jetstream().await?;

        let ack = if let Some(h) = headers {
            let nats_headers = Self::to_nats_headers(h);
            js.publish_with_headers(subject.to_string(), nats_headers, payload.to_vec().into())
                .await
                .map_err(|e| JetStreamError::PublishFailed(e.to_string()))?
                .await
                .map_err(|e| JetStreamError::PublishFailed(e.to_string()))?
        } else {
            js.publish(subject.to_string(), payload.to_vec().into())
                .await
                .map_err(|e| JetStreamError::PublishFailed(e.to_string()))?
                .await
                .map_err(|e| JetStreamError::PublishFailed(e.to_string()))?
        };

        debug!("Published to JetStream: {} (seq: {})", subject, ack.sequence);

        Ok(crate::ports::PublishAck {
            stream: ack.stream.to_string(),
            sequence: ack.sequence,
            duplicate: ack.duplicate,
            domain: if ack.domain.is_empty() { None } else { Some(ack.domain.to_string()) },
        })
    }

    async fn publish_with_id(
        &self,
        subject: &str,
        payload: &[u8],
        message_id: &str,
        headers: Option<&crate::ports::JetStreamHeaders>,
    ) -> Result<crate::ports::PublishAck, crate::ports::JetStreamError> {
        // Create headers with message ID if not present
        let mut combined_headers = headers.cloned().unwrap_or_default();
        combined_headers.insert("Nats-Msg-Id", message_id);

        self.publish(subject, payload, Some(&combined_headers)).await
    }

    async fn subscribe(
        &self,
        stream: &str,
        consumer: &str,
        _filter_subject: Option<&str>,
    ) -> Result<Box<dyn crate::ports::JetStreamSubscription>, crate::ports::JetStreamError> {
        use crate::ports::JetStreamError;
        use async_nats::jetstream::consumer::PullConsumer;

        let js = self.get_jetstream().await?;

        let stream_obj = js.get_stream(stream)
            .await
            .map_err(|e| JetStreamError::StreamNotFound(e.to_string()))?;

        let consumer_obj: PullConsumer = stream_obj.get_consumer(consumer)
            .await
            .map_err(|e| JetStreamError::ConsumerNotFound(e.to_string()))?;

        let messages = consumer_obj.messages()
            .await
            .map_err(|e| JetStreamError::SubscribeFailed(e.to_string()))?;

        Ok(Box::new(JetStreamSubscriptionImpl { messages }))
    }

    async fn stream_info(&self, stream: &str) -> Result<crate::ports::StreamInfo, crate::ports::JetStreamError> {
        use crate::ports::JetStreamError;

        let js = self.get_jetstream().await?;

        let mut stream_obj = js.get_stream(stream)
            .await
            .map_err(|e| JetStreamError::StreamNotFound(e.to_string()))?;

        let info = stream_obj.info()
            .await
            .map_err(|e| JetStreamError::StreamNotFound(e.to_string()))?;

        Ok(crate::ports::StreamInfo {
            name: info.config.name.clone(),
            messages: info.state.messages,
            bytes: info.state.bytes,
            first_seq: info.state.first_sequence,
            last_seq: info.state.last_sequence,
            consumer_count: info.state.consumer_count,
            subjects: info.config.subjects.clone(),
        })
    }

    async fn create_stream(
        &self,
        config: &crate::ports::JetStreamStreamConfig,
    ) -> Result<crate::ports::StreamInfo, crate::ports::JetStreamError> {
        use crate::ports::JetStreamError;

        let js = self.get_jetstream().await?;

        let nats_config = Self::to_nats_stream_config(config);

        let mut stream = js.create_stream(nats_config)
            .await
            .map_err(|e| JetStreamError::StreamCreationFailed(e.to_string()))?;

        let info = stream.info()
            .await
            .map_err(|e| JetStreamError::StreamCreationFailed(e.to_string()))?;

        info!("Created stream: {}", config.name);

        Ok(crate::ports::StreamInfo {
            name: info.config.name.clone(),
            messages: info.state.messages,
            bytes: info.state.bytes,
            first_seq: info.state.first_sequence,
            last_seq: info.state.last_sequence,
            consumer_count: info.state.consumer_count,
            subjects: info.config.subjects.clone(),
        })
    }

    async fn create_consumer(
        &self,
        stream: &str,
        config: &crate::ports::JetStreamConsumerConfig,
    ) -> Result<crate::ports::ConsumerInfo, crate::ports::JetStreamError> {
        use crate::ports::JetStreamError;

        let js = self.get_jetstream().await?;

        let stream_obj = js.get_stream(stream)
            .await
            .map_err(|e| JetStreamError::StreamNotFound(e.to_string()))?;

        let nats_config = Self::to_nats_consumer_config(config);

        let mut consumer = stream_obj.create_consumer(nats_config)
            .await
            .map_err(|e| JetStreamError::ConsumerCreationFailed(e.to_string()))?;

        let info = consumer.info()
            .await
            .map_err(|e| JetStreamError::ConsumerCreationFailed(e.to_string()))?;

        info!("Created consumer: {} on stream {}", config.name, stream);

        Ok(crate::ports::ConsumerInfo {
            name: info.name.clone(),
            stream: stream.to_string(),
            num_pending: info.num_pending,
            num_redelivered: info.num_waiting as u64,
            delivered_seq: info.delivered.stream_sequence,
            ack_floor_seq: info.ack_floor.stream_sequence,
        })
    }

    async fn is_connected(&self) -> bool {
        let client_lock = self.client.read().await;
        client_lock.is_some()
    }
}

/// JetStream subscription implementation
#[cfg(feature = "nats-client")]
pub struct JetStreamSubscriptionImpl {
    messages: async_nats::jetstream::consumer::pull::Stream,
}

#[cfg(feature = "nats-client")]
#[async_trait::async_trait]
impl crate::ports::JetStreamSubscription for JetStreamSubscriptionImpl {
    async fn next(&mut self) -> Option<crate::ports::JetStreamMessage> {
        use futures::StreamExt;

        match self.messages.next().await {
            Some(Ok(msg)) => {
                let mut headers = crate::ports::JetStreamHeaders::new();
                if let Some(ref h) = msg.headers {
                    // Use explicit type for header iteration
                    let header_map: &async_nats::HeaderMap = h;
                    for (key, values) in header_map.iter() {
                        // Get first value from the vector of HeaderValue
                        if let Some(first_value) = values.iter().next() {
                            headers.insert(key.to_string(), first_value.as_str().to_string());
                        }
                    }
                }

                // Extract message info
                let (sequence, timestamp, num_delivered) = match msg.info() {
                    Ok(info) => {
                        let ts_nanos = info.published.unix_timestamp_nanos() as i64;
                        (info.stream_sequence, ts_nanos, info.delivered as u64)
                    }
                    Err(_) => (0, 0, 0),
                };

                Some(crate::ports::JetStreamMessage {
                    subject: msg.subject.to_string(),
                    payload: msg.payload.to_vec(),
                    headers,
                    sequence,
                    timestamp,
                    num_delivered,
                    reply: msg.reply.as_ref().map(|r| r.to_string()),
                })
            }
            Some(Err(e)) => {
                error!("Error receiving message: {}", e);
                None
            }
            None => None,
        }
    }

    async fn unsubscribe(self: Box<Self>) -> Result<(), crate::ports::JetStreamError> {
        // The subscription is dropped when this struct is dropped
        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_build_subject() {
        let config = NatsConfig::default();
        let adapter = NatsClientAdapter::new(config, PathBuf::from("/tmp/queue.json"));

        // A4: Generate test event_id for causation tracking
        let test_event_id = Uuid::now_v7();
        let event = DomainEvent::Key(crate::events::KeyEvents::KeyGenerated(crate::events::key::KeyGeneratedEvent {
            key_id: Uuid::now_v7(),
            algorithm: crate::types::KeyAlgorithm::Ed25519,
            purpose: crate::types::KeyPurpose::Signing,
            generated_at: chrono::Utc::now(),
            generated_by: "test".to_string(),
            hardware_backed: false,
            metadata: crate::types::KeyMetadata {
                label: "test".to_string(),
                description: None,
                tags: vec![],
                attributes: std::collections::HashMap::new(),
                jwt_kid: None,
                jwt_alg: None,
                jwt_use: None,
            },
            ownership: None,
            correlation_id: Uuid::now_v7(),
            causation_id: Some(test_event_id), // A4: Self-reference for root event
        }));

        let subject = adapter.build_subject(&event);
        // Default config uses "cim.graph" prefix, not "cim.keys"
        assert_eq!(subject, "cim.graph.key.keygenerated");
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
