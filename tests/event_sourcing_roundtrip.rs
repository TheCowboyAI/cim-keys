//! Event-sourcing roundtrip tests
//!
//! Tests that events can be replayed to reconstruct projection state.
//! This is the CORE principle of event-sourcing: events are the source of truth.

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use std::collections::HashMap;

// Minimal graph types for testing event sourcing concepts
#[derive(Debug, Clone)]
struct DomainGraph {
    nodes: HashMap<Uuid, DomainObject>,
    edges: Vec<DomainRelationship>,
}

#[derive(Debug, Clone)]
struct DomainObject {
    id: Uuid,
    aggregate_type: String,
    properties: serde_json::Map<String, serde_json::Value>,
    version: usize,
}

#[derive(Debug, Clone)]
struct DomainRelationship {
    source_id: Uuid,
    target_id: Uuid,
    relationship_type: String,
}

impl DomainGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    fn add_node(mut self, node: DomainObject) -> Self {
        self.nodes.insert(node.id, node);
        self
    }

    fn update_node(mut self, node: DomainObject) -> Self {
        self.nodes.insert(node.id, node);
        self
    }

    fn add_edge(mut self, edge: DomainRelationship) -> Self {
        self.edges.push(edge);
        self
    }

    fn get_node(&self, id: Uuid) -> Option<&DomainObject> {
        self.nodes.get(&id)
    }

    fn traverse_edges(&self, source_id: Uuid, rel_type: &str, target_type: &str) -> Vec<Uuid> {
        self.edges
            .iter()
            .filter(|e| e.source_id == source_id && e.relationship_type == rel_type)
            .filter(|e| {
                self.nodes
                    .get(&e.target_id)
                    .map(|n| n.aggregate_type == target_type)
                    .unwrap_or(false)
            })
            .map(|e| e.target_id)
            .collect()
    }
}

impl DomainObject {
    fn new(aggregate_type: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            aggregate_type,
            properties: serde_json::Map::new(),
            version: 0,
        }
    }

    fn update_property(mut self, key: &str, value: serde_json::Value) -> Self {
        self.properties.insert(key.to_string(), value);
        self.version += 1;
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "event_type")]
enum GraphEvent {
    DomainObjectCreated {
        object_id: Uuid,
        aggregate_type: String,
        properties: serde_json::Value,
        timestamp: chrono::DateTime<Utc>,
    },
    DomainObjectUpdated {
        object_id: Uuid,
        property_key: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
        timestamp: chrono::DateTime<Utc>,
    },
    DomainObjectDeleted {
        object_id: Uuid,
        aggregate_type: String,
        timestamp: chrono::DateTime<Utc>,
    },
    RelationshipEstablished {
        source_id: Uuid,
        target_id: Uuid,
        relationship_type: String,
        timestamp: chrono::DateTime<Utc>,
    },
    RelationshipRemoved {
        source_id: Uuid,
        target_id: Uuid,
        relationship_type: String,
        timestamp: chrono::DateTime<Utc>,
    },
}

/// Apply an event to a graph projection (pure function)
fn apply_event(mut graph: DomainGraph, event: &GraphEvent) -> DomainGraph {
    match event {
        GraphEvent::DomainObjectCreated {
            object_id,
            aggregate_type,
            properties,
            ..
        } => {
            let mut node = DomainObject::new(aggregate_type.clone());
            node.id = *object_id;

            // Deserialize properties
            if let Ok(props) = serde_json::from_value(properties.clone()) {
                node.properties = props;
            }

            graph.add_node(node)
        }
        GraphEvent::DomainObjectUpdated {
            object_id,
            property_key,
            new_value,
            ..
        } => {
            if let Some(node) = graph.nodes.get(object_id) {
                let updated = node.clone().update_property(property_key, new_value.clone());
                graph.update_node(updated)
            } else {
                graph
            }
        }
        GraphEvent::DomainObjectDeleted { object_id, .. } => {
            graph.nodes.remove(object_id);
            graph.edges.retain(|e| e.source_id != *object_id && e.target_id != *object_id);
            graph
        }
        GraphEvent::RelationshipEstablished {
            source_id,
            target_id,
            relationship_type,
            ..
        } => {
            let edge = DomainRelationship {
                source_id: *source_id,
                target_id: *target_id,
                relationship_type: relationship_type.clone(),
            };
            graph.add_edge(edge)
        }
        GraphEvent::RelationshipRemoved {
            source_id,
            target_id,
            relationship_type,
            ..
        } => {
            graph.edges.retain(|e| {
                !(e.source_id == *source_id
                    && e.target_id == *target_id
                    && e.relationship_type == *relationship_type)
            });
            graph
        }
    }
}

/// Replay all events to reconstruct projection (fold over events)
fn replay_events(events: &[GraphEvent]) -> DomainGraph {
    events
        .iter()
        .fold(DomainGraph::new(), |graph, event| apply_event(graph, event))
}

#[test]
fn test_simple_creation_replay() {
    // Create an event log with single creation event
    let person_id = Uuid::now_v7();

    let events = vec![GraphEvent::DomainObjectCreated {
        object_id: person_id,
        aggregate_type: "Person".to_string(),
        properties: json!({
            "legal_name": "Alice Smith",
            "active": true
        }),
        timestamp: Utc::now(),
    }];

    // Replay events
    let graph = replay_events(&events);

    // Verify projection
    assert_eq!(graph.nodes.len(), 1);
    assert!(graph.get_node(person_id).is_some());

    let node = graph.get_node(person_id).unwrap();
    assert_eq!(node.aggregate_type, "Person");
    assert_eq!(node.properties.get("legal_name"), Some(&json!("Alice Smith")));
}

#[test]
fn test_creation_and_update_replay() {
    // Event log: Create person, then update name
    let person_id = Uuid::now_v7();

    let events = vec![
        GraphEvent::DomainObjectCreated {
            object_id: person_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectUpdated {
            object_id: person_id,
            property_key: "legal_name".to_string(),
            old_value: json!("Alice"),
            new_value: json!("Alice Smith"),
            timestamp: Utc::now(),
        },
    ];

    let graph = replay_events(&events);

    let node = graph.get_node(person_id).unwrap();
    assert_eq!(node.properties.get("legal_name"), Some(&json!("Alice Smith")));
    // Version should be incremented by update
    assert_eq!(node.version, 1);
}

#[test]
fn test_creation_and_deletion_replay() {
    // Event log: Create person, then delete
    let person_id = Uuid::now_v7();

    let events = vec![
        GraphEvent::DomainObjectCreated {
            object_id: person_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectDeleted {
            object_id: person_id,
            aggregate_type: "Person".to_string(),
            timestamp: Utc::now(),
        },
    ];

    let graph = replay_events(&events);

    // Node should not exist after deletion
    assert_eq!(graph.nodes.len(), 0);
    assert!(graph.get_node(person_id).is_none());
}

#[test]
fn test_relationship_replay() {
    // Event log: Create two people, establish relationship
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();

    let events = vec![
        GraphEvent::DomainObjectCreated {
            object_id: alice_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectCreated {
            object_id: bob_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Bob"}),
            timestamp: Utc::now(),
        },
        GraphEvent::RelationshipEstablished {
            source_id: alice_id,
            target_id: bob_id,
            relationship_type: "reports_to".to_string(),
            timestamp: Utc::now(),
        },
    ];

    let graph = replay_events(&events);

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);

    let edge = &graph.edges[0];
    assert_eq!(edge.source_id, alice_id);
    assert_eq!(edge.target_id, bob_id);
    assert_eq!(edge.relationship_type, "reports_to");
}

#[test]
fn test_relationship_removal_replay() {
    // Event log: Create relationship, then remove it
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();

    let events = vec![
        GraphEvent::DomainObjectCreated {
            object_id: alice_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectCreated {
            object_id: bob_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Bob"}),
            timestamp: Utc::now(),
        },
        GraphEvent::RelationshipEstablished {
            source_id: alice_id,
            target_id: bob_id,
            relationship_type: "reports_to".to_string(),
            timestamp: Utc::now(),
        },
        GraphEvent::RelationshipRemoved {
            source_id: alice_id,
            target_id: bob_id,
            relationship_type: "reports_to".to_string(),
            timestamp: Utc::now(),
        },
    ];

    let graph = replay_events(&events);

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 0); // Relationship removed
}

#[test]
fn test_cascade_delete_replay() {
    // Event log: Create nodes + edges, delete node (should cascade to edges)
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    let key_id = Uuid::now_v7();

    let events = vec![
        GraphEvent::DomainObjectCreated {
            object_id: alice_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectCreated {
            object_id: bob_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Bob"}),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectCreated {
            object_id: key_id,
            aggregate_type: "Key".to_string(),
            properties: json!({"key_type": "ed25519"}),
            timestamp: Utc::now(),
        },
        GraphEvent::RelationshipEstablished {
            source_id: alice_id,
            target_id: bob_id,
            relationship_type: "reports_to".to_string(),
            timestamp: Utc::now(),
        },
        GraphEvent::RelationshipEstablished {
            source_id: alice_id,
            target_id: key_id,
            relationship_type: "owns_key".to_string(),
            timestamp: Utc::now(),
        },
        // Delete Alice - should cascade to both edges
        GraphEvent::DomainObjectDeleted {
            object_id: alice_id,
            aggregate_type: "Person".to_string(),
            timestamp: Utc::now(),
        },
    ];

    let graph = replay_events(&events);

    // Alice should be gone
    assert_eq!(graph.nodes.len(), 2);
    assert!(graph.get_node(alice_id).is_none());

    // Both edges connected to Alice should be gone
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_complex_workflow_replay() {
    // Complex event log: Organization setup with multiple entities
    let org_id = Uuid::now_v7();
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    let key_id = Uuid::now_v7();
    let location_id = Uuid::now_v7();

    let events = vec![
        // Create organization
        GraphEvent::DomainObjectCreated {
            object_id: org_id,
            aggregate_type: "Organization".to_string(),
            properties: json!({"name": "CowboyAI"}),
            timestamp: Utc::now(),
        },
        // Create people
        GraphEvent::DomainObjectCreated {
            object_id: alice_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectCreated {
            object_id: bob_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Bob"}),
            timestamp: Utc::now(),
        },
        // Update Alice's name
        GraphEvent::DomainObjectUpdated {
            object_id: alice_id,
            property_key: "legal_name".to_string(),
            old_value: json!("Alice"),
            new_value: json!("Alice Smith"),
            timestamp: Utc::now(),
        },
        // Create location
        GraphEvent::DomainObjectCreated {
            object_id: location_id,
            aggregate_type: "Location".to_string(),
            properties: json!({"address": "alice@cowboyai.com"}),
            timestamp: Utc::now(),
        },
        // Create key
        GraphEvent::DomainObjectCreated {
            object_id: key_id,
            aggregate_type: "Key".to_string(),
            properties: json!({"key_type": "ed25519"}),
            timestamp: Utc::now(),
        },
        // Establish relationships
        GraphEvent::RelationshipEstablished {
            source_id: bob_id,
            target_id: alice_id,
            relationship_type: "reports_to".to_string(),
            timestamp: Utc::now(),
        },
        GraphEvent::RelationshipEstablished {
            source_id: alice_id,
            target_id: location_id,
            relationship_type: "located_at".to_string(),
            timestamp: Utc::now(),
        },
        GraphEvent::RelationshipEstablished {
            source_id: alice_id,
            target_id: key_id,
            relationship_type: "owns_key".to_string(),
            timestamp: Utc::now(),
        },
    ];

    let graph = replay_events(&events);

    // Verify final state
    assert_eq!(graph.nodes.len(), 5); // org, 2 people, location, key
    assert_eq!(graph.edges.len(), 3); // reports_to, located_at, owns_key

    // Verify Alice's updated name
    let alice = graph.get_node(alice_id).unwrap();
    assert_eq!(
        alice.properties.get("legal_name"),
        Some(&json!("Alice Smith"))
    );

    // Verify relationships
    let alice_keys = graph.traverse_edges(alice_id, "owns_key", "Key");
    assert_eq!(alice_keys.len(), 1);

    let alice_location = graph.traverse_edges(alice_id, "located_at", "Location");
    assert_eq!(alice_location.len(), 1);
}

#[test]
fn test_event_log_persistence() {
    // Test that events can be serialized/deserialized for persistence
    let events = vec![
        GraphEvent::DomainObjectCreated {
            object_id: Uuid::now_v7(),
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
        GraphEvent::RelationshipEstablished {
            source_id: Uuid::now_v7(),
            target_id: Uuid::now_v7(),
            relationship_type: "reports_to".to_string(),
            timestamp: Utc::now(),
        },
    ];

    // Serialize to JSON (as would be written to events.json)
    let json = serde_json::to_string_pretty(&events).expect("Failed to serialize");

    // Deserialize back
    let loaded: Vec<GraphEvent> =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(loaded.len(), 2);
}

#[test]
fn test_partial_replay() {
    // Test replaying events up to a specific point in time
    let person_id = Uuid::now_v7();

    let events = vec![
        GraphEvent::DomainObjectCreated {
            object_id: person_id,
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectUpdated {
            object_id: person_id,
            property_key: "legal_name".to_string(),
            old_value: json!("Alice"),
            new_value: json!("Alice Smith"),
            timestamp: Utc::now(),
        },
        GraphEvent::DomainObjectUpdated {
            object_id: person_id,
            property_key: "legal_name".to_string(),
            old_value: json!("Alice Smith"),
            new_value: json!("Alice J. Smith"),
            timestamp: Utc::now(),
        },
    ];

    // Replay only first 2 events
    let graph = replay_events(&events[0..2]);

    let node = graph.get_node(person_id).unwrap();
    assert_eq!(
        node.properties.get("legal_name"),
        Some(&json!("Alice Smith"))
    );

    // Replay all events
    let full_graph = replay_events(&events);

    let full_node = full_graph.get_node(person_id).unwrap();
    assert_eq!(
        full_node.properties.get("legal_name"),
        Some(&json!("Alice J. Smith"))
    );
}

#[test]
fn test_idempotent_replay() {
    // Test that replaying the same events multiple times gives same result
    let events = vec![
        GraphEvent::DomainObjectCreated {
            object_id: Uuid::now_v7(),
            aggregate_type: "Person".to_string(),
            properties: json!({"legal_name": "Alice"}),
            timestamp: Utc::now(),
        },
    ];

    let graph1 = replay_events(&events);
    let graph2 = replay_events(&events);

    assert_eq!(graph1.nodes.len(), graph2.nodes.len());
    assert_eq!(graph1.edges.len(), graph2.edges.len());
}
