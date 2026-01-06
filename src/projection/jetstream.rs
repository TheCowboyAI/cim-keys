// Copyright (c) 2025 - Cowboy AI, LLC.

//! # JetStream Projection
//!
//! Composable projections for domain events → NATS JetStream.
//!
//! ## Architecture
//!
//! ```text
//! Domain Events
//!     ↓ via
//! EventToMessageProjection (pure)
//!     ↓ produces
//! JetStreamBatch (messages ready to publish)
//!     ↓ via
//! JetStreamPort (async I/O)
//!     ↓ produces
//! PublishResult
//! ```
//!
//! ## Subject Naming Convention
//!
//! ```text
//! {organization}.{domain}.{aggregate}.{event_type}
//!
//! Examples:
//! - cowboyai.keys.key.generated
//! - cowboyai.keys.certificate.issued
//! - cowboyai.organization.person.joined
//! - cowboyai.nats.operator.created
//! ```

use crate::projection::{Projection, ProjectionError};
use crate::ports::nats::{JetStreamHeaders, PublishAck};
use crate::events::DomainEvent;
use cim_domain::DomainEvent as DomainEventTrait;  // Import trait for method access
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// MESSAGE TYPES
// ============================================================================

/// A message ready to be published to JetStream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamMessageOut {
    /// Subject to publish to
    pub subject: String,
    /// Message payload (JSON serialized event)
    pub payload: Vec<u8>,
    /// Message headers
    pub headers: JetStreamMessageHeaders,
    /// Message ID for deduplication
    pub message_id: String,
}

/// Headers for JetStream messages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JetStreamMessageHeaders {
    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
    /// Causation ID (what caused this event)
    pub causation_id: Option<String>,
    /// Event type
    pub event_type: String,
    /// Aggregate ID
    pub aggregate_id: Option<String>,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Schema version
    pub schema_version: String,
    /// Custom headers
    pub custom: HashMap<String, String>,
}

impl JetStreamMessageHeaders {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            correlation_id: None,
            causation_id: None,
            event_type: event_type.into(),
            aggregate_id: None,
            timestamp: Utc::now(),
            schema_version: "1.0.0".to_string(),
            custom: HashMap::new(),
        }
    }

    pub fn with_correlation(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    pub fn with_causation(mut self, id: impl Into<String>) -> Self {
        self.causation_id = Some(id.into());
        self
    }

    pub fn with_aggregate(mut self, id: impl Into<String>) -> Self {
        self.aggregate_id = Some(id.into());
        self
    }

    /// Convert to NATS JetStream headers
    pub fn to_jetstream_headers(&self) -> JetStreamHeaders {
        let mut headers = JetStreamHeaders::new();
        headers.insert("event-type".to_string(), self.event_type.clone());
        headers.insert("timestamp".to_string(), self.timestamp.to_rfc3339());
        headers.insert("schema-version".to_string(), self.schema_version.clone());

        if let Some(ref cid) = self.correlation_id {
            headers.insert("correlation-id".to_string(), cid.clone());
        }
        if let Some(ref cid) = self.causation_id {
            headers.insert("causation-id".to_string(), cid.clone());
        }
        if let Some(ref aid) = self.aggregate_id {
            headers.insert("aggregate-id".to_string(), aid.clone());
        }

        for (key, value) in &self.custom {
            headers.insert(key.clone(), value.clone());
        }

        headers
    }
}

/// A batch of messages ready for JetStream
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JetStreamBatch {
    /// Messages to publish
    pub messages: Vec<JetStreamMessageOut>,
    /// Batch metadata
    pub metadata: Option<BatchMetadata>,
}

/// Metadata about the batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMetadata {
    pub batch_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub source: String,
    pub stream: String,
}

impl JetStreamBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metadata(mut self, source: &str, stream: &str) -> Self {
        self.metadata = Some(BatchMetadata {
            batch_id: Uuid::now_v7(),
            created_at: Utc::now(),
            source: source.to_string(),
            stream: stream.to_string(),
        });
        self
    }

    pub fn add_message(&mut self, message: JetStreamMessageOut) {
        self.messages.push(message);
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

// ============================================================================
// PROJECTIONS
// ============================================================================

/// Configuration for event-to-subject mapping
#[derive(Debug, Clone)]
pub struct SubjectConfig {
    /// Organization prefix (e.g., "cowboyai")
    pub organization: String,
    /// Domain prefix (e.g., "keys")
    pub domain: String,
}

impl Default for SubjectConfig {
    fn default() -> Self {
        Self {
            organization: "cim".to_string(),
            domain: "keys".to_string(),
        }
    }
}

/// Projection: Vec<DomainEvent> → JetStreamBatch
///
/// Transforms domain events into JetStream messages ready for publishing.
pub struct EventsToMessagesProjection {
    config: SubjectConfig,
}

impl Default for EventsToMessagesProjection {
    fn default() -> Self {
        Self {
            config: SubjectConfig::default(),
        }
    }
}

impl EventsToMessagesProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: SubjectConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.config.organization = org.into();
        self
    }

    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.config.domain = domain.into();
        self
    }

    /// Derive subject from event type
    fn event_to_subject(&self, event: &DomainEvent) -> String {
        use crate::events::DomainEvent as DE;

        let (aggregate, event_type) = match event {
            DE::Key(e) => ("key", e.event_type()),
            DE::Certificate(e) => ("certificate", e.event_type()),
            DE::Person(e) => ("person", e.event_type()),
            DE::Location(e) => ("location", e.event_type()),
            DE::Delegation(e) => ("delegation", e.event_type()),
            DE::NatsOperator(e) => ("nats.operator", e.event_type()),
            DE::NatsAccount(e) => ("nats.account", e.event_type()),
            DE::NatsUser(e) => ("nats.user", e.event_type()),
            DE::YubiKey(e) => ("yubikey", e.event_type()),
            DE::Relationship(e) => ("relationship", e.event_type()),
            DE::Organization(e) => ("organization", e.event_type()),
            DE::Manifest(e) => ("manifest", e.event_type()),
            DE::Saga(e) => ("saga", e.event_type()),
        };

        // Convert CamelCase event type to snake_case for subject
        let snake_event = event_type
            .chars()
            .fold(String::new(), |mut acc, c| {
                if c.is_uppercase() && !acc.is_empty() {
                    acc.push('_');
                }
                acc.push(c.to_lowercase().next().unwrap_or(c));
                acc
            });

        format!(
            "{}.{}.{}.{}",
            self.config.organization,
            self.config.domain,
            aggregate,
            snake_event
        )
    }

    /// Extract aggregate ID from event
    fn event_aggregate_id(&self, event: &DomainEvent) -> Option<Uuid> {
        use crate::events::DomainEvent as DE;

        match event {
            DE::Key(e) => Some(e.aggregate_id()),
            DE::Certificate(e) => Some(e.aggregate_id()),
            DE::Person(e) => Some(e.aggregate_id()),
            DE::Location(e) => Some(e.aggregate_id()),
            DE::Delegation(e) => Some(e.aggregate_id()),
            DE::NatsOperator(e) => Some(e.aggregate_id()),
            DE::NatsAccount(e) => Some(e.aggregate_id()),
            DE::NatsUser(e) => Some(e.aggregate_id()),
            DE::YubiKey(e) => Some(e.aggregate_id()),
            DE::Relationship(e) => Some(e.aggregate_id()),
            DE::Organization(e) => Some(e.aggregate_id()),
            DE::Manifest(e) => Some(e.aggregate_id()),
            DE::Saga(e) => Some(e.saga_id()),  // SagaEvents uses saga_id, not aggregate_id
        }
    }

    /// Get event type string from event
    fn get_event_type(&self, event: &DomainEvent) -> &'static str {
        use crate::events::DomainEvent as DE;

        match event {
            DE::Key(e) => e.event_type(),
            DE::Certificate(e) => e.event_type(),
            DE::Person(e) => e.event_type(),
            DE::Location(e) => e.event_type(),
            DE::Delegation(e) => e.event_type(),
            DE::NatsOperator(e) => e.event_type(),
            DE::NatsAccount(e) => e.event_type(),
            DE::NatsUser(e) => e.event_type(),
            DE::YubiKey(e) => e.event_type(),
            DE::Relationship(e) => e.event_type(),
            DE::Organization(e) => e.event_type(),
            DE::Manifest(e) => e.event_type(),
            DE::Saga(e) => e.event_type(),
        }
    }

    /// Create message from event
    fn event_to_message(&self, event: &DomainEvent) -> Result<JetStreamMessageOut, ProjectionError> {
        let subject = self.event_to_subject(event);
        let event_type = self.get_event_type(event);
        let aggregate_id = self.event_aggregate_id(event);
        let message_id = Uuid::now_v7().to_string();

        let payload = serde_json::to_vec(event)
            .map_err(|e| ProjectionError::SerializationError(e.to_string()))?;

        let mut headers = JetStreamMessageHeaders::new(event_type);
        if let Some(agg_id) = aggregate_id {
            headers = headers.with_aggregate(agg_id.to_string());
        }

        Ok(JetStreamMessageOut {
            subject,
            payload,
            headers,
            message_id,
        })
    }
}

impl Projection<Vec<DomainEvent>, JetStreamBatch, ProjectionError> for EventsToMessagesProjection {
    fn project(&self, events: Vec<DomainEvent>) -> Result<JetStreamBatch, ProjectionError> {
        let mut batch = JetStreamBatch::new()
            .with_metadata("cim-keys", "domain-events");

        for event in events {
            let message = self.event_to_message(&event)?;
            batch.add_message(message);
        }

        Ok(batch)
    }

    fn name(&self) -> &'static str {
        "EventsToMessages"
    }
}

/// Projection: DomainEvent → JetStreamMessageOut (single event)
pub struct SingleEventProjection {
    config: SubjectConfig,
}

impl Default for SingleEventProjection {
    fn default() -> Self {
        Self {
            config: SubjectConfig::default(),
        }
    }
}

impl SingleEventProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: SubjectConfig) -> Self {
        self.config = config;
        self
    }
}

impl Projection<DomainEvent, JetStreamMessageOut, ProjectionError> for SingleEventProjection {
    fn project(&self, event: DomainEvent) -> Result<JetStreamMessageOut, ProjectionError> {
        let batch_proj = EventsToMessagesProjection::new().with_config(self.config.clone());
        batch_proj.event_to_message(&event)
    }

    fn name(&self) -> &'static str {
        "SingleEvent"
    }
}

// ============================================================================
// PUBLISH RESULT
// ============================================================================

/// Result of publishing a batch to JetStream
#[derive(Debug, Clone)]
pub struct PublishResult {
    /// Number of messages published
    pub published: usize,
    /// Acknowledgements received
    pub acks: Vec<PublishAck>,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Stream name
    pub stream: String,
}

impl Default for PublishResult {
    fn default() -> Self {
        Self {
            published: 0,
            acks: Vec::new(),
            errors: Vec::new(),
            stream: String::new(),
        }
    }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/// Create an events-to-messages projection
pub fn events_to_messages() -> EventsToMessagesProjection {
    EventsToMessagesProjection::new()
}

/// Create a single event projection
pub fn single_event() -> SingleEventProjection {
    SingleEventProjection::new()
}

/// Create a configured events projection for a specific organization
pub fn events_for_org(org: impl Into<String>, domain: impl Into<String>) -> EventsToMessagesProjection {
    EventsToMessagesProjection::new()
        .with_organization(org)
        .with_domain(domain)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::KeyEvents;
    use crate::events::key::KeyGeneratedEvent;
    use crate::types::{KeyAlgorithm, KeyPurpose, KeyMetadata};

    fn sample_key_event() -> DomainEvent {
        DomainEvent::Key(KeyEvents::KeyGenerated(KeyGeneratedEvent {
            key_id: Uuid::now_v7(),
            algorithm: KeyAlgorithm::Ed25519,
            purpose: KeyPurpose::Authentication,
            generated_at: Utc::now(),
            generated_by: "test".to_string(),
            hardware_backed: false,
            metadata: KeyMetadata {
                label: "Test Key".to_string(),
                description: None,
                tags: vec![],
                attributes: std::collections::HashMap::new(),
                jwt_kid: None,
                jwt_alg: None,
                jwt_use: None,
            },
            ownership: None,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        }))
    }

    #[test]
    fn test_events_to_messages_projection() {
        let events = vec![sample_key_event()];
        let projection = events_to_messages();

        let result = projection.project(events);
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.messages.len(), 1);
        assert!(batch.metadata.is_some());
    }

    #[test]
    fn test_subject_naming() {
        let projection = EventsToMessagesProjection::new()
            .with_organization("cowboyai")
            .with_domain("keys");

        let event = sample_key_event();
        let message = projection.event_to_message(&event).unwrap();

        assert!(message.subject.starts_with("cowboyai.keys.key."));
    }

    #[test]
    fn test_headers_creation() {
        let headers = JetStreamMessageHeaders::new("test.event")
            .with_correlation("corr-123")
            .with_causation("cause-456")
            .with_aggregate("agg-789");

        assert_eq!(headers.correlation_id, Some("corr-123".to_string()));
        assert_eq!(headers.causation_id, Some("cause-456".to_string()));
        assert_eq!(headers.aggregate_id, Some("agg-789".to_string()));
    }

    #[test]
    fn test_single_event_projection() {
        let event = sample_key_event();
        let projection = single_event();

        let result = projection.project(event);
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(!message.subject.is_empty());
        assert!(!message.payload.is_empty());
    }

    #[test]
    fn test_batch_metadata() {
        let batch = JetStreamBatch::new()
            .with_metadata("test-source", "test-stream");

        assert!(batch.metadata.is_some());
        let meta = batch.metadata.unwrap();
        assert_eq!(meta.source, "test-source");
        assert_eq!(meta.stream, "test-stream");
    }

    #[test]
    fn test_events_for_org_factory() {
        let projection = events_for_org("myorg", "mydomain");
        let event = sample_key_event();
        let message = projection.event_to_message(&event).unwrap();

        assert!(message.subject.starts_with("myorg.mydomain."));
    }
}
