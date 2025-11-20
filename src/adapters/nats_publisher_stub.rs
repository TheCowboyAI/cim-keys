//! NATS Event Publisher Adapter (Stub)
//!
//! This is a stub implementation documenting the architecture for NATS event publishing.
//! Full implementation requires integration with async-nats crate.
//!
//! ## TODO: Full Implementation
//!
//! To complete this implementation:
//! 1. Enable `nats` feature in Cargo.toml
//! 2. Add async-nats dependency
//! 3. Implement actual NATS JetStream publishing
//! 4. Integrate with IPLD object store
//! 5. Add proper error handling and retries
//! 6. Add integration tests with running NATS server
//!
//! ## Architecture
//!
//! ```text
//! GraphEvent → EventEnvelope → NATS Subject
//!           ↓
//!     EventPayload → IPLD CID → Object Store
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cim_graph::events::EventPayload;

/// Event envelope for NATS publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub payload_cid: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub event_type: String,
}

/// Publishing configuration
#[derive(Debug, Clone)]
pub struct PublisherConfig {
    pub nats_url: String,
    pub stream_name: String,
    pub source_id: String,
    pub subject_prefix: String,
}

impl Default for PublisherConfig {
    fn default() -> Self {
        Self {
            nats_url: "nats://localhost:4222".to_string(),
            stream_name: "CIM_GRAPH_EVENTS".to_string(),
            source_id: "cim-keys-v0.8.0".to_string(),
            subject_prefix: "cim.graph".to_string(),
        }
    }
}

/// Extract event type from payload for subject construction
pub fn extract_event_type(payload: &EventPayload) -> (String, String) {
    use cim_graph::events::{ContextPayload, ConceptPayload};

    match payload {
        EventPayload::Context(ctx) => {
            let event_type = match ctx {
                ContextPayload::BoundedContextCreated { .. } => "bounded_context_created",
                ContextPayload::AggregateAdded { .. } => "aggregate_added",
                ContextPayload::EntityAdded { .. } => "entity_added",
                ContextPayload::RelationshipEstablished { .. } => "relationship_established",
                ContextPayload::ValueObjectAttached { .. } => "value_object_attached",
            };
            ("context".to_string(), event_type.to_string())
        }
        EventPayload::Concept(concept) => {
            let event_type = match concept {
                ConceptPayload::ConceptDefined { .. } => "concept_defined",
                ConceptPayload::PropertiesAdded { .. } => "properties_added",
                ConceptPayload::RelationAdded { .. } => "relation_added",
                ConceptPayload::PropertyInferred { .. } => "property_inferred",
            };
            ("concept".to_string(), event_type.to_string())
        }
        _ => ("unknown".to_string(), "unknown".to_string()),
    }
}

/// Build NATS subject from event information
pub fn build_subject(prefix: &str, context: &str, payload: &EventPayload) -> String {
    let (payload_type, event_type) = extract_event_type(payload);
    format!("{}.{}.events.{}.{}", prefix, context, payload_type, event_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_graph::events::ContextPayload;

    #[test]
    fn test_extract_event_type() {
        let payload = EventPayload::Context(ContextPayload::BoundedContextCreated {
            context_id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test context".to_string(),
        });

        let (payload_type, event_type) = extract_event_type(&payload);
        assert_eq!(payload_type, "context");
        assert_eq!(event_type, "bounded_context_created");
    }

    #[test]
    fn test_build_subject() {
        let payload = EventPayload::Context(ContextPayload::AggregateAdded {
            context_id: "person-ctx".to_string(),
            aggregate_id: uuid::Uuid::now_v7(),
            aggregate_type: "Person".to_string(),
        });

        let subject = build_subject("cim.graph", "person", &payload);
        assert_eq!(subject, "cim.graph.person.events.context.aggregate_added");
    }

    #[test]
    fn test_event_envelope_serialization() {
        let envelope = EventEnvelope {
            event_id: uuid::Uuid::now_v7(),
            aggregate_id: uuid::Uuid::now_v7(),
            correlation_id: uuid::Uuid::now_v7(),
            causation_id: None,
            payload_cid: "bafyrei1234567890abcdef".to_string(),
            timestamp: chrono::Utc::now(),
            source: "test".to_string(),
            event_type: "context.bounded_context_created".to_string(),
        };

        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("event_id"));
        assert!(json.contains("payload_cid"));
    }
}
