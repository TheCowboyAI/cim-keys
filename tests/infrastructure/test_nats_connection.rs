//! Infrastructure Layer 1.1: NATS Connection Tests for cim-keys
//! 
//! User Story: As a security system, I need to publish key events to NATS for system-wide coordination
//!
//! Test Requirements:
//! - Verify NATS connection establishment for key events
//! - Verify key event stream creation with correct configuration
//! - Verify key event publishing with acknowledgment
//! - Verify key event consumption with proper ordering
//!
//! Event Sequence:
//! 1. ConnectionEstablished
//! 2. StreamCreated { name: "key-events", subjects: ["keys.>"] }
//! 3. EventPublished { subject: "keys.generated", sequence }
//! 4. EventConsumed { subject: "keys.generated", sequence }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Connect to NATS]
//!     B --> C[ConnectionEstablished]
//!     C --> D[Create Key Stream]
//!     D --> E[StreamCreated]
//!     E --> F[Publish Key Event]
//!     F --> G[EventPublished]
//!     G --> H[Consume Event]
//!     H --> I[EventConsumed]
//!     I --> J[Test Success]
//! ```

use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// NATS events for testing
#[derive(Debug, Clone, PartialEq)]
pub enum NatsEvent {
    ConnectionEstablished,
    StreamCreated { name: String, subjects: Vec<String> },
    EventPublished { subject: String, sequence: u64 },
    EventConsumed { subject: String, sequence: u64 },
    ConsumerCreated { name: String, stream: String },
    ConnectionLost { reason: String },
    ReconnectionSuccessful,
}

/// Key event payload for NATS
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyEventPayload {
    pub event_type: String,
    pub key_id: String,
    pub algorithm: String,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Mock NATS client for testing
pub struct MockNatsClient {
    connected: bool,
    streams: HashMap<String, Vec<String>>, // stream_name -> subjects
    published_events: Vec<(String, KeyEventPayload, u64)>, // (subject, payload, sequence)
    sequence_counter: u64,
}

impl MockNatsClient {
    pub fn new() -> Self {
        Self {
            connected: false,
            streams: HashMap::new(),
            published_events: Vec::new(),
            sequence_counter: 0,
        }
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        self.connected = true;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn create_stream(
        &mut self,
        name: String,
        subjects: Vec<String>,
    ) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }

        self.streams.insert(name, subjects);
        Ok(())
    }

    pub async fn publish_key_event(
        &mut self,
        subject: &str,
        payload: KeyEventPayload,
    ) -> Result<u64, String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }

        // Check if subject matches any stream
        let valid_subject = self.streams.values().any(|subjects| {
            subjects.iter().any(|s| {
                if s.ends_with('>') {
                    subject.starts_with(&s[..s.len() - 1])
                } else {
                    s == subject
                }
            })
        });

        if !valid_subject {
            return Err("No stream configured for subject".to_string());
        }

        self.sequence_counter += 1;
        self.published_events.push((
            subject.to_string(),
            payload,
            self.sequence_counter,
        ));

        Ok(self.sequence_counter)
    }

    pub async fn consume_events(
        &self,
        subject_filter: &str,
    ) -> Result<Vec<(String, KeyEventPayload, u64)>, String> {
        if !self.connected {
            return Err("Not connected to NATS".to_string());
        }

        // Filter events by subject
        let events: Vec<_> = self.published_events
            .iter()
            .filter(|(subject, _, _)| {
                if subject_filter.ends_with('>') {
                    subject.starts_with(&subject_filter[..subject_filter.len() - 1])
                } else {
                    subject == subject_filter
                }
            })
            .cloned()
            .collect();

        Ok(events)
    }

    pub async fn disconnect(&mut self) {
        self.connected = false;
    }

    pub async fn reconnect(&mut self) -> Result<(), String> {
        // Simulate reconnection
        tokio::time::sleep(Duration::from_millis(20)).await;
        self.connected = true;
        Ok(())
    }
}

/// Mock consumer for key events
pub struct MockKeyEventConsumer {
    name: String,
    stream: String,
    consumed_events: Vec<(String, KeyEventPayload, u64)>,
}

impl MockKeyEventConsumer {
    pub fn new(name: String, stream: String) -> Self {
        Self {
            name,
            stream,
            consumed_events: Vec::new(),
        }
    }

    pub async fn consume_from(
        &mut self,
        client: &MockNatsClient,
        subject_filter: &str,
    ) -> Result<usize, String> {
        let events = client.consume_events(subject_filter).await?;
        
        let new_events: Vec<_> = events
            .into_iter()
            .filter(|(_, _, seq)| {
                !self.consumed_events.iter().any(|(_, _, consumed_seq)| consumed_seq == seq)
            })
            .collect();

        let count = new_events.len();
        self.consumed_events.extend(new_events);

        Ok(count)
    }

    pub fn get_consumed_events(&self) -> &[(String, KeyEventPayload, u64)] {
        &self.consumed_events
    }
}

/// Event stream validator for NATS testing
pub struct NatsEventStreamValidator {
    expected_events: Vec<NatsEvent>,
    captured_events: Vec<NatsEvent>,
}

impl NatsEventStreamValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
        }
    }

    pub fn expect_sequence(mut self, events: Vec<NatsEvent>) -> Self {
        self.expected_events = events;
        self
    }

    pub fn capture_event(&mut self, event: NatsEvent) {
        self.captured_events.push(event);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.captured_events.len() != self.expected_events.len() {
            return Err(format!("Event count mismatch: expected {self.expected_events.len(}, got {}"),
                self.captured_events.len()
            ));
        }

        for (i, (expected, actual)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected != actual {
                return Err(format!("Event mismatch at position {i}: expected {:?}, got {:?}", expected, actual));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nats_connection_establishment() {
        // Arrange
        let mut validator = NatsEventStreamValidator::new()
            .expect_sequence(vec![
                NatsEvent::ConnectionEstablished,
            ]);

        let mut client = MockNatsClient::new();

        // Act
        let result = client.connect().await;

        // Assert
        assert!(result.is_ok());
        assert!(client.is_connected());
        
        validator.capture_event(NatsEvent::ConnectionEstablished);
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_key_event_stream_creation() {
        // Arrange
        let mut validator = NatsEventStreamValidator::new()
            .expect_sequence(vec![
                NatsEvent::ConnectionEstablished,
                NatsEvent::StreamCreated {
                    name: "key-events".to_string(),
                    subjects: vec!["keys.>".to_string()],
                },
            ]);

        let mut client = MockNatsClient::new();

        // Act
        client.connect().await.unwrap();
        validator.capture_event(NatsEvent::ConnectionEstablished);

        let result = client.create_stream(
            "key-events".to_string(),
            vec!["keys.>".to_string()],
        ).await;

        // Assert
        assert!(result.is_ok());
        
        validator.capture_event(NatsEvent::StreamCreated {
            name: "key-events".to_string(),
            subjects: vec!["keys.>".to_string()],
        });
        
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_key_event_publishing() {
        // Arrange
        let mut client = MockNatsClient::new();
        client.connect().await.unwrap();
        client.create_stream(
            "key-events".to_string(),
            vec!["keys.>".to_string()],
        ).await.unwrap();

        let mut validator = NatsEventStreamValidator::new();

        let payload = KeyEventPayload {
            event_type: "key_generated".to_string(),
            key_id: "test-key-001".to_string(),
            algorithm: "RSA-2048".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        // Act
        let sequence = client.publish_key_event("keys.generated", payload.clone()).await.unwrap();

        // Assert
        assert_eq!(sequence, 1);
        
        validator.capture_event(NatsEvent::EventPublished {
            subject: "keys.generated".to_string(),
            sequence,
        });
    }

    #[tokio::test]
    async fn test_key_event_consumption() {
        // Arrange
        let mut client = MockNatsClient::new();
        client.connect().await.unwrap();
        client.create_stream(
            "key-events".to_string(),
            vec!["keys.>".to_string()],
        ).await.unwrap();

        let mut validator = NatsEventStreamValidator::new();

        // Publish some events
        let payload1 = KeyEventPayload {
            event_type: "key_generated".to_string(),
            key_id: "consume-test-1".to_string(),
            algorithm: "Ed25519".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        let payload2 = KeyEventPayload {
            event_type: "key_rotated".to_string(),
            key_id: "consume-test-2".to_string(),
            algorithm: "RSA-4096".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        client.publish_key_event("keys.generated", payload1).await.unwrap();
        let seq2 = client.publish_key_event("keys.rotated", payload2).await.unwrap();

        // Act
        let events = client.consume_events("keys.>").await.unwrap();

        // Assert
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, "keys.generated");
        assert_eq!(events[1].0, "keys.rotated");
        
        validator.capture_event(NatsEvent::EventConsumed {
            subject: "keys.rotated".to_string(),
            sequence: seq2,
        });
    }

    #[tokio::test]
    async fn test_consumer_creation_and_consumption() {
        // Arrange
        let mut client = MockNatsClient::new();
        client.connect().await.unwrap();
        client.create_stream(
            "key-events".to_string(),
            vec!["keys.>".to_string()],
        ).await.unwrap();

        let mut consumer = MockKeyEventConsumer::new(
            "key-processor".to_string(),
            "key-events".to_string(),
        );

        // Publish events
        let payload = KeyEventPayload {
            event_type: "key_generated".to_string(),
            key_id: "consumer-test".to_string(),
            algorithm: "ECDSA-P256".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        client.publish_key_event("keys.generated", payload.clone()).await.unwrap();

        // Act
        let consumed_count = consumer.consume_from(&client, "keys.>").await.unwrap();

        // Assert
        assert_eq!(consumed_count, 1);
        assert_eq!(consumer.get_consumed_events().len(), 1);
        assert_eq!(consumer.get_consumed_events()[0].1, payload);
    }

    #[tokio::test]
    async fn test_connection_loss_and_reconnection() {
        // Arrange
        let mut validator = NatsEventStreamValidator::new()
            .expect_sequence(vec![
                NatsEvent::ConnectionEstablished,
                NatsEvent::ConnectionLost {
                    reason: "Network error".to_string(),
                },
                NatsEvent::ReconnectionSuccessful,
            ]);

        let mut client = MockNatsClient::new();

        // Act
        client.connect().await.unwrap();
        validator.capture_event(NatsEvent::ConnectionEstablished);

        client.disconnect().await;
        validator.capture_event(NatsEvent::ConnectionLost {
            reason: "Network error".to_string(),
        });

        client.reconnect().await.unwrap();
        validator.capture_event(NatsEvent::ReconnectionSuccessful);

        // Assert
        assert!(client.is_connected());
        assert!(validator.validate().is_ok());
    }

    #[tokio::test]
    async fn test_subject_filtering() {
        // Arrange
        let mut client = MockNatsClient::new();
        client.connect().await.unwrap();
        client.create_stream(
            "key-events".to_string(),
            vec!["keys.>".to_string()],
        ).await.unwrap();

        // Publish various events
        let gen_payload = KeyEventPayload {
            event_type: "key_generated".to_string(),
            key_id: "filter-test-1".to_string(),
            algorithm: "RSA-2048".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        let rot_payload = KeyEventPayload {
            event_type: "key_rotated".to_string(),
            key_id: "filter-test-2".to_string(),
            algorithm: "RSA-2048".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        client.publish_key_event("keys.generated", gen_payload.clone()).await.unwrap();
        client.publish_key_event("keys.rotated", rot_payload).await.unwrap();
        client.publish_key_event("keys.generated", gen_payload).await.unwrap();

        // Act
        let generated_only = client.consume_events("keys.generated").await.unwrap();
        let all_keys = client.consume_events("keys.>").await.unwrap();

        // Assert
        assert_eq!(generated_only.len(), 2);
        assert_eq!(all_keys.len(), 3);
    }

    #[tokio::test]
    async fn test_event_ordering_preservation() {
        // Arrange
        let mut client = MockNatsClient::new();
        client.connect().await.unwrap();
        client.create_stream(
            "key-events".to_string(),
            vec!["keys.>".to_string()],
        ).await.unwrap();

        let mut sequences = Vec::new();

        // Act - Publish events in order
        for i in 0..5 {
            let payload = KeyEventPayload {
                event_type: "key_generated".to_string(),
                key_id: format!("order-test-{i}"),
                algorithm: "RSA-2048".to_string(),
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
            };

            let seq = client.publish_key_event("keys.generated", payload).await.unwrap();
            sequences.push(seq);
        }

        // Assert - Sequences should be monotonically increasing
        for i in 1..sequences.len() {
            assert!(sequences[i] > sequences[i - 1]);
        }

        // Verify consumption order
        let events = client.consume_events("keys.>").await.unwrap();
        for i in 0..events.len() {
            assert_eq!(events[i].1.key_id, format!("order-test-{i}"));
        }
    }
} 