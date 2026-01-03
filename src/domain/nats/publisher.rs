// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Event Publisher for JetStream
//!
//! This module provides event publishing services that integrate domain events
//! with NATS JetStream, following CIM header specifications.
//!
//! ## Architecture
//!
//! ```text
//! Aggregate -> DomainEvent -> EventPublisher -> JetStream
//!                                    |
//!                            CIM Headers added
//!                            Subject routing
//!                            Deduplication ID
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use cim_keys::domain::nats::publisher::EventPublisher;
//!
//! let publisher = EventPublisher::new(jetstream_port);
//!
//! // Publish a single event
//! publisher.publish(&event, correlation_id, causation_id).await?;
//!
//! // Publish multiple events from an aggregate command
//! publisher.publish_batch(&events, correlation_id).await?;
//! ```

use crate::events::DomainEvent;
use crate::ports::{JetStreamError, JetStreamHeaders, JetStreamPort, PublishAck};
use uuid::Uuid;

/// Domain event publisher for JetStream
///
/// Handles the translation of domain events into JetStream messages
/// with proper CIM headers and subject routing.
pub struct EventPublisher<P: JetStreamPort> {
    port: P,
    source: String,
}

impl<P: JetStreamPort> EventPublisher<P> {
    /// Create a new event publisher with the given JetStream port
    pub fn new(port: P) -> Self {
        Self {
            port,
            source: "cim-keys".to_string(),
        }
    }

    /// Create a publisher with a custom source identifier
    pub fn with_source(port: P, source: impl Into<String>) -> Self {
        Self {
            port,
            source: source.into(),
        }
    }

    /// Publish a domain event to JetStream
    ///
    /// The event is serialized to JSON, CIM headers are added, and the
    /// appropriate subject is determined from the event type.
    pub async fn publish(
        &self,
        event: &DomainEvent,
        correlation_id: Uuid,
        causation_id: Uuid,
    ) -> Result<PublishAck, EventPublishError> {
        let event_id = Uuid::now_v7();
        let subject = self.event_to_subject(event);
        let event_type = event.event_type();

        // Serialize event payload
        let payload = serde_json::to_vec(event)
            .map_err(|e| EventPublishError::SerializationError(e.to_string()))?;

        // Create CIM headers
        let headers = JetStreamHeaders::from_cim_headers(
            &event_id,
            &correlation_id,
            &causation_id,
            event_type,
            &self.source,
        );

        // Publish with message ID for deduplication
        self.port
            .publish_with_id(&subject, &payload, &event_id.to_string(), Some(&headers))
            .await
            .map_err(|e| EventPublishError::PublishFailed(e.to_string()))
    }

    /// Publish a batch of events from an aggregate command
    ///
    /// All events share the same correlation ID. Each event gets its own
    /// event ID, and causation chains are maintained.
    pub async fn publish_batch(
        &self,
        events: &[DomainEvent],
        correlation_id: Uuid,
    ) -> Result<Vec<PublishAck>, EventPublishError> {
        if events.is_empty() {
            return Ok(vec![]);
        }

        let mut acks = Vec::with_capacity(events.len());
        let mut previous_event_id: Option<Uuid> = None;

        for event in events {
            let event_id = Uuid::now_v7();
            // First event uses correlation_id as causation, subsequent events use previous event
            let causation_id = previous_event_id.unwrap_or(correlation_id);

            let subject = self.event_to_subject(event);
            let event_type = event.event_type();

            let payload = serde_json::to_vec(event)
                .map_err(|e| EventPublishError::SerializationError(e.to_string()))?;

            let headers = JetStreamHeaders::from_cim_headers(
                &event_id,
                &correlation_id,
                &causation_id,
                event_type,
                &self.source,
            );

            let ack = self
                .port
                .publish_with_id(&subject, &payload, &event_id.to_string(), Some(&headers))
                .await
                .map_err(|e| EventPublishError::PublishFailed(e.to_string()))?;

            acks.push(ack);
            previous_event_id = Some(event_id);
        }

        Ok(acks)
    }

    /// Determine the JetStream subject for an event
    fn event_to_subject(&self, event: &DomainEvent) -> String {
        use crate::domain::nats::jetstream::events;

        match event {
            DomainEvent::Key(key_event) => {
                use crate::events::KeyEvents;
                match key_event {
                    KeyEvents::KeyGenerated(_) => events::key_generated().as_str(),
                    KeyEvents::KeyImported(_) => events::key_imported().as_str(),
                    KeyEvents::KeyExported(_) => events::key_exported().as_str(),
                    KeyEvents::KeyStoredOffline(_) => "keys.events.key.stored-offline".to_string(),
                    KeyEvents::KeyRevoked(_) => events::key_revoked().as_str(),
                    KeyEvents::KeyRotationInitiated(_) => events::key_rotated().as_str(),
                    KeyEvents::KeyRotationCompleted(_) => events::key_rotated().as_str(),
                    KeyEvents::SshKeyGenerated(_) => "keys.events.key.ssh-generated".to_string(),
                    KeyEvents::GpgKeyGenerated(_) => "keys.events.key.gpg-generated".to_string(),
                    KeyEvents::TotpSecretGenerated(_) => "keys.events.key.totp-generated".to_string(),
                }
            }
            DomainEvent::Certificate(cert_event) => {
                use crate::events::CertificateEvents;
                match cert_event {
                    CertificateEvents::CertificateGenerated(_) => {
                        events::certificate_created().as_str()
                    }
                    CertificateEvents::CertificateSigned(_) => events::certificate_signed().as_str(),
                    CertificateEvents::CertificateRevoked(_) => {
                        events::certificate_revoked().as_str()
                    }
                    CertificateEvents::CertificateRenewed(_) => {
                        events::certificate_renewed().as_str()
                    }
                    CertificateEvents::CertificateValidated(_) => {
                        "keys.events.certificate.validated".to_string()
                    }
                    CertificateEvents::CertificateExported(_) => {
                        "keys.events.certificate.exported".to_string()
                    }
                    CertificateEvents::PkiHierarchyCreated(_) => {
                        "keys.events.certificate.pki-hierarchy-created".to_string()
                    }
                    CertificateEvents::CertificateActivated(_) => {
                        "keys.events.certificate.activated".to_string()
                    }
                    CertificateEvents::CertificateSuspended(_) => {
                        "keys.events.certificate.suspended".to_string()
                    }
                    CertificateEvents::CertificateExpired(_) => {
                        "keys.events.certificate.expired".to_string()
                    }
                }
            }
            DomainEvent::YubiKey(yubikey_event) => {
                use crate::events::YubiKeyEvents;
                match yubikey_event {
                    YubiKeyEvents::YubiKeyDetected(_) => "keys.events.yubikey.detected".to_string(),
                    YubiKeyEvents::YubiKeyProvisioned(_) => events::yubikey_provisioned().as_str(),
                    YubiKeyEvents::PinConfigured(_) => "keys.events.yubikey.pin-configured".to_string(),
                    YubiKeyEvents::PukConfigured(_) => "keys.events.yubikey.puk-configured".to_string(),
                    YubiKeyEvents::ManagementKeyRotated(_) => "keys.events.yubikey.mgmt-key-rotated".to_string(),
                    YubiKeyEvents::SlotAllocationPlanned(_) => "keys.events.yubikey.slot-planned".to_string(),
                    YubiKeyEvents::KeyGeneratedInSlot(_) => events::yubikey_slot_populated().as_str(),
                    YubiKeyEvents::CertificateImportedToSlot(_) => "keys.events.yubikey.cert-imported".to_string(),
                }
            }
            DomainEvent::NatsOperator(_) => events::nats_operator_created().as_str(),
            DomainEvent::NatsAccount(_) => events::nats_account_created().as_str(),
            DomainEvent::NatsUser(_) => events::nats_user_created().as_str(),
            // Organization events go to a general subject
            DomainEvent::Organization(_) => "keys.events.organization.updated".to_string(),
            DomainEvent::Person(_) => "keys.events.person.updated".to_string(),
            DomainEvent::Location(_) => "keys.events.location.updated".to_string(),
            DomainEvent::Relationship(_) => "keys.events.relationship.updated".to_string(),
            DomainEvent::Manifest(_) => "keys.events.manifest.updated".to_string(),
        }
    }

    /// Check if the publisher is connected to JetStream
    pub async fn is_connected(&self) -> bool {
        self.port.is_connected().await
    }
}

/// Errors that can occur during event publishing
#[derive(Debug, thiserror::Error)]
pub enum EventPublishError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Publish failed: {0}")]
    PublishFailed(String),

    #[error("Not connected to JetStream")]
    NotConnected,

    #[error("Invalid event type: {0}")]
    InvalidEventType(String),
}

impl From<JetStreamError> for EventPublishError {
    fn from(err: JetStreamError) -> Self {
        EventPublishError::PublishFailed(err.to_string())
    }
}

/// Extension trait for publishing events from aggregate results
pub trait PublishableEvents {
    /// Get the events to publish
    fn events(&self) -> &[DomainEvent];

    /// Get the correlation ID for the batch
    fn correlation_id(&self) -> Uuid;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{KeyEvents, key::KeyGeneratedEvent};
    use crate::types::{KeyAlgorithm, KeyMetadata, KeyPurpose};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock JetStream port for testing
    struct MockJetStreamPort {
        published: Arc<Mutex<Vec<(String, Vec<u8>, String)>>>,
    }

    impl MockJetStreamPort {
        fn new() -> Self {
            Self {
                published: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn published_count(&self) -> usize {
            self.published.lock().await.len()
        }
    }

    #[async_trait::async_trait]
    impl JetStreamPort for MockJetStreamPort {
        async fn publish(
            &self,
            subject: &str,
            payload: &[u8],
            _headers: Option<&JetStreamHeaders>,
        ) -> Result<PublishAck, JetStreamError> {
            let mut published = self.published.lock().await;
            published.push((subject.to_string(), payload.to_vec(), String::new()));
            Ok(PublishAck {
                stream: "KEYS_EVENTS".to_string(),
                sequence: published.len() as u64,
                duplicate: false,
                domain: None,
            })
        }

        async fn publish_with_id(
            &self,
            subject: &str,
            payload: &[u8],
            message_id: &str,
            _headers: Option<&JetStreamHeaders>,
        ) -> Result<PublishAck, JetStreamError> {
            let mut published = self.published.lock().await;
            published.push((subject.to_string(), payload.to_vec(), message_id.to_string()));
            Ok(PublishAck {
                stream: "KEYS_EVENTS".to_string(),
                sequence: published.len() as u64,
                duplicate: false,
                domain: None,
            })
        }

        async fn subscribe(
            &self,
            _stream: &str,
            _consumer: &str,
            _filter_subject: Option<&str>,
        ) -> Result<Box<dyn crate::ports::JetStreamSubscription>, JetStreamError> {
            Err(JetStreamError::SubscribeFailed("Not implemented".to_string()))
        }

        async fn stream_info(&self, _stream: &str) -> Result<crate::ports::StreamInfo, JetStreamError> {
            Err(JetStreamError::StreamNotFound("Not implemented".to_string()))
        }

        async fn create_stream(
            &self,
            _config: &crate::ports::JetStreamStreamConfig,
        ) -> Result<crate::ports::StreamInfo, JetStreamError> {
            Err(JetStreamError::StreamCreationFailed("Not implemented".to_string()))
        }

        async fn create_consumer(
            &self,
            _stream: &str,
            _config: &crate::ports::JetStreamConsumerConfig,
        ) -> Result<crate::ports::ConsumerInfo, JetStreamError> {
            Err(JetStreamError::ConsumerCreationFailed("Not implemented".to_string()))
        }

        async fn is_connected(&self) -> bool {
            true
        }
    }

    fn create_test_key_event() -> DomainEvent {
        DomainEvent::Key(KeyEvents::KeyGenerated(KeyGeneratedEvent {
            key_id: Uuid::now_v7(),
            algorithm: KeyAlgorithm::Ed25519,
            purpose: KeyPurpose::Signing,
            generated_at: chrono::Utc::now(),
            generated_by: "test".to_string(),
            hardware_backed: false,
            metadata: KeyMetadata {
                label: "test-key".to_string(),
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

    #[tokio::test]
    async fn test_publish_single_event() {
        let port = MockJetStreamPort::new();
        let publisher = EventPublisher::new(port);

        let event = create_test_key_event();
        let correlation_id = Uuid::now_v7();
        let causation_id = Uuid::now_v7();

        let result = publisher.publish(&event, correlation_id, causation_id).await;
        assert!(result.is_ok());

        let ack = result.unwrap();
        assert_eq!(ack.stream, "KEYS_EVENTS");
        assert_eq!(ack.sequence, 1);
    }

    #[tokio::test]
    async fn test_publish_batch() {
        let port = MockJetStreamPort::new();
        let publisher = EventPublisher::new(port);

        let events = vec![
            create_test_key_event(),
            create_test_key_event(),
            create_test_key_event(),
        ];
        let correlation_id = Uuid::now_v7();

        let result = publisher.publish_batch(&events, correlation_id).await;
        assert!(result.is_ok());

        let acks = result.unwrap();
        assert_eq!(acks.len(), 3);
        assert_eq!(acks[0].sequence, 1);
        assert_eq!(acks[1].sequence, 2);
        assert_eq!(acks[2].sequence, 3);
    }

    #[tokio::test]
    async fn test_publish_empty_batch() {
        let port = MockJetStreamPort::new();
        let publisher = EventPublisher::new(port);

        let events: Vec<DomainEvent> = vec![];
        let correlation_id = Uuid::now_v7();

        let result = publisher.publish_batch(&events, correlation_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_event_to_subject() {
        let port = MockJetStreamPort::new();
        let publisher = EventPublisher::new(port);

        let event = create_test_key_event();
        let subject = publisher.event_to_subject(&event);
        assert_eq!(subject, "keys.events.key.generated");
    }

    #[tokio::test]
    async fn test_custom_source() {
        let port = MockJetStreamPort::new();
        let publisher = EventPublisher::with_source(port, "custom-source");

        assert_eq!(publisher.source, "custom-source");
    }
}
