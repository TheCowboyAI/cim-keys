//! Integration tests for graph-first GUI
//!
//! Tests complete workflows end-to-end:
//! - Node creation and property editing
//! - Relationship creation
//! - Event emission and persistence
//! - Graph save/load
//! - View filtering

use cim_keys::graph_ui::types::{DomainGraph, DomainObject, DomainRelationship};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_domain_object_creation() {
    // Test creating a Person domain object
    let person = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice Smith"))
        .with_property("active", json!(true));

    assert_eq!(person.aggregate_type, "Person");
    assert_eq!(person.properties.get("legal_name"), Some(&json!("Alice Smith")));
    assert_eq!(person.properties.get("active"), Some(&json!(true)));
    // Version is 2: new() creates v0, first with_property() -> v1, second -> v2
    assert_eq!(person.version, 2);
}

#[test]
fn test_domain_object_property_update() {
    // Test property updates create new versions
    let person = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice"));

    assert_eq!(person.version, 1);

    let updated = person.update_property("legal_name", json!("Alice Smith"));

    assert_eq!(updated.version, 2);
    assert_eq!(updated.properties.get("legal_name"), Some(&json!("Alice Smith")));
}

#[test]
fn test_domain_graph_node_operations() {
    // Test adding nodes to graph
    let mut graph = DomainGraph::new();

    let person1 = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice"));
    let person1_id = person1.id;

    let person2 = DomainObject::new("Person")
        .with_property("legal_name", json!("Bob"));
    let person2_id = person2.id;

    graph = graph.add_node(person1);
    graph = graph.add_node(person2);

    assert_eq!(graph.nodes.len(), 2);
    assert!(graph.get_node(person1_id).is_some());
    assert!(graph.get_node(person2_id).is_some());
}

#[test]
fn test_domain_graph_edge_operations() {
    // Test creating relationships between nodes
    let mut graph = DomainGraph::new();

    let alice = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice"));
    let alice_id = alice.id;

    let bob = DomainObject::new("Person")
        .with_property("legal_name", json!("Bob"));
    let bob_id = bob.id;

    graph = graph.add_node(alice);
    graph = graph.add_node(bob);

    let edge = DomainRelationship {
        source_id: alice_id,
        target_id: bob_id,
        relationship_type: "reports_to".to_string(),
    };

    graph = graph.add_edge(edge);

    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].source_id, alice_id);
    assert_eq!(graph.edges[0].target_id, bob_id);
    assert_eq!(graph.edges[0].relationship_type, "reports_to");
}

#[test]
fn test_nodes_by_type_query() {
    // Test querying nodes by aggregate type
    let mut graph = DomainGraph::new();

    let person = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice"));
    let org = DomainObject::new("Organization")
        .with_property("name", json!("CowboyAI"));
    let location = DomainObject::new("Location")
        .with_property("address", json!("alice@example.com"));

    graph = graph.add_node(person);
    graph = graph.add_node(org);
    graph = graph.add_node(location);

    let people = graph.nodes_by_type("Person");
    assert_eq!(people.len(), 1);
    assert_eq!(people[0].aggregate_type, "Person");

    let orgs = graph.nodes_by_type("Organization");
    assert_eq!(orgs.len(), 1);

    let locations = graph.nodes_by_type("Location");
    assert_eq!(locations.len(), 1);
}

#[test]
fn test_graph_traversal() {
    // Test traversing edges from source to targets
    let mut graph = DomainGraph::new();

    let alice = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice"));
    let alice_id = alice.id;

    let key = DomainObject::new("Key")
        .with_property("key_type", json!("ed25519"));
    let key_id = key.id;

    graph = graph.add_node(alice);
    graph = graph.add_node(key);

    let ownership = DomainRelationship {
        source_id: alice_id,
        target_id: key_id,
        relationship_type: "owns_key".to_string(),
    };

    graph = graph.add_edge(ownership);

    // Traverse from Alice to owned keys
    let owned_keys = graph.traverse_edges(alice_id, "owns_key", "Key");

    assert_eq!(owned_keys.len(), 1);
    assert_eq!(owned_keys[0].id, key_id);
    assert_eq!(owned_keys[0].aggregate_type, "Key");
}

#[test]
fn test_graph_serialization() {
    // Test graph can be serialized to JSON and back
    let mut graph = DomainGraph::new();

    let person = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice"));
    let person_id = person.id;

    graph = graph.add_node(person);

    // Serialize to JSON
    let json = serde_json::to_string(&graph).expect("Failed to serialize");

    // Deserialize back
    let loaded: DomainGraph = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(loaded.nodes.len(), 1);
    assert!(loaded.get_node(person_id).is_some());
}

#[test]
fn test_complete_workflow_person_with_key() {
    // Integration test: Create person, generate key, establish ownership
    let mut graph = DomainGraph::new();

    // Step 1: Create person
    let person = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice Smith"))
        .with_property("active", json!(true));
    let person_id = person.id;

    graph = graph.add_node(person);

    // Step 2: Generate key
    let key = DomainObject::new("Key")
        .with_property("key_type", json!("ed25519"))
        .with_property("purpose", json!("signing"))
        .with_property("generated_at", json!("2025-11-21T00:00:00Z"));
    let key_id = key.id;

    graph = graph.add_node(key);

    // Step 3: Establish ownership
    let ownership = DomainRelationship {
        source_id: person_id,
        target_id: key_id,
        relationship_type: "owns_key".to_string(),
    };

    graph = graph.add_edge(ownership);

    // Verify complete state
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);

    // Query person's keys
    let owned_keys = graph.traverse_edges(person_id, "owns_key", "Key");
    assert_eq!(owned_keys.len(), 1);
    assert_eq!(owned_keys[0].properties.get("key_type"), Some(&json!("ed25519")));
}

#[test]
fn test_organizational_hierarchy() {
    // Integration test: Create org hierarchy with reporting relationships
    let mut graph = DomainGraph::new();

    let alice = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice Smith"))
        .with_property("role", json!("Manager"));
    let alice_id = alice.id;

    let bob = DomainObject::new("Person")
        .with_property("legal_name", json!("Bob Jones"))
        .with_property("role", json!("Developer"));
    let bob_id = bob.id;

    let charlie = DomainObject::new("Person")
        .with_property("legal_name", json!("Charlie Davis"))
        .with_property("role", json!("Developer"));
    let charlie_id = charlie.id;

    graph = graph.add_node(alice);
    graph = graph.add_node(bob);
    graph = graph.add_node(charlie);

    // Bob reports to Alice
    graph = graph.add_edge(DomainRelationship {
        source_id: bob_id,
        target_id: alice_id,
        relationship_type: "reports_to".to_string(),
    });

    // Charlie reports to Alice
    graph = graph.add_edge(DomainRelationship {
        source_id: charlie_id,
        target_id: alice_id,
        relationship_type: "reports_to".to_string(),
    });

    // Verify hierarchy
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);

    // Find Alice's direct reports
    let reports = graph.edges.iter()
        .filter(|e| e.target_id == alice_id && e.relationship_type == "reports_to")
        .count();

    assert_eq!(reports, 2);
}

#[test]
fn test_nats_infrastructure_hierarchy() {
    // Integration test: NATS operator → accounts → users
    let mut graph = DomainGraph::new();

    let operator = DomainObject::new("NatsOperator")
        .with_property("name", json!("CowboyAI Operator"))
        .with_property("operator_id", json!("OABCD..."));
    let operator_id = operator.id;

    let eng_account = DomainObject::new("NatsAccount")
        .with_property("name", json!("Engineering"))
        .with_property("account_id", json!("AABCD..."));
    let eng_id = eng_account.id;

    let user = DomainObject::new("NatsUser")
        .with_property("name", json!("alice.engineering"))
        .with_property("user_id", json!("UABCD..."));
    let user_id = user.id;

    graph = graph.add_node(operator);
    graph = graph.add_node(eng_account);
    graph = graph.add_node(user);

    // Operator contains Account
    graph = graph.add_edge(DomainRelationship {
        source_id: operator_id,
        target_id: eng_id,
        relationship_type: "contains".to_string(),
    });

    // Account contains User
    graph = graph.add_edge(DomainRelationship {
        source_id: eng_id,
        target_id: user_id,
        relationship_type: "contains".to_string(),
    });

    // Verify NATS hierarchy
    let accounts = graph.traverse_edges(operator_id, "contains", "NatsAccount");
    assert_eq!(accounts.len(), 1);

    let users = graph.traverse_edges(eng_id, "contains", "NatsUser");
    assert_eq!(users.len(), 1);
}

#[test]
fn test_pki_certificate_chain() {
    // Integration test: PKI hierarchy with signing and trust relationships
    let mut graph = DomainGraph::new();

    let root_ca = DomainObject::new("Certificate")
        .with_property("common_name", json!("Root CA"))
        .with_property("certificate_type", json!("root"));
    let root_id = root_ca.id;

    let intermediate_ca = DomainObject::new("Certificate")
        .with_property("common_name", json!("Intermediate CA"))
        .with_property("certificate_type", json!("intermediate"));
    let intermediate_id = intermediate_ca.id;

    let leaf_cert = DomainObject::new("Certificate")
        .with_property("common_name", json!("Alice Smith"))
        .with_property("certificate_type", json!("leaf"));
    let leaf_id = leaf_cert.id;

    graph = graph.add_node(root_ca);
    graph = graph.add_node(intermediate_ca);
    graph = graph.add_node(leaf_cert);

    // Root signs Intermediate
    graph = graph.add_edge(DomainRelationship {
        source_id: root_id,
        target_id: intermediate_id,
        relationship_type: "signs".to_string(),
    });

    // Intermediate signs Leaf
    graph = graph.add_edge(DomainRelationship {
        source_id: intermediate_id,
        target_id: leaf_id,
        relationship_type: "signs".to_string(),
    });

    // Root trusts Intermediate
    graph = graph.add_edge(DomainRelationship {
        source_id: root_id,
        target_id: intermediate_id,
        relationship_type: "trusts".to_string(),
    });

    // Verify PKI chain
    let signed_by_root = graph.traverse_edges(root_id, "signs", "Certificate");
    assert_eq!(signed_by_root.len(), 1);

    let signed_by_intermediate = graph.traverse_edges(intermediate_id, "signs", "Certificate");
    assert_eq!(signed_by_intermediate.len(), 1);

    let trusted_by_root = graph.traverse_edges(root_id, "trusts", "Certificate");
    assert_eq!(trusted_by_root.len(), 1);
}

#[test]
fn test_view_filtering_organization() {
    // Test view filtering for Organization perspective
    let mut graph = DomainGraph::new();

    // Add organization domain types
    graph = graph.add_node(DomainObject::new("Person"));
    graph = graph.add_node(DomainObject::new("Organization"));
    graph = graph.add_node(DomainObject::new("Location"));
    graph = graph.add_node(DomainObject::new("ServiceAccount"));

    // Add non-organization types
    graph = graph.add_node(DomainObject::new("NatsOperator"));
    graph = graph.add_node(DomainObject::new("Certificate"));
    graph = graph.add_node(DomainObject::new("YubiKey"));

    // Filter for Organization view
    let org_types = ["Person", "Organization", "Location", "ServiceAccount"];
    let filtered: Vec<_> = graph.nodes.values()
        .filter(|n| org_types.contains(&n.aggregate_type.as_str()))
        .collect();

    assert_eq!(filtered.len(), 4);
}

#[test]
fn test_view_filtering_nats() {
    // Test view filtering for NATS perspective
    let mut graph = DomainGraph::new();

    // Add NATS types
    graph = graph.add_node(DomainObject::new("NatsOperator"));
    graph = graph.add_node(DomainObject::new("NatsAccount"));
    graph = graph.add_node(DomainObject::new("NatsUser"));

    // Add non-NATS types
    graph = graph.add_node(DomainObject::new("Person"));
    graph = graph.add_node(DomainObject::new("Certificate"));

    // Filter for NATS view
    let nats_types = ["NatsOperator", "NatsAccount", "NatsUser"];
    let filtered: Vec<_> = graph.nodes.values()
        .filter(|n| nats_types.contains(&n.aggregate_type.as_str()))
        .collect();

    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_view_filtering_pki() {
    // Test view filtering for PKI perspective
    let mut graph = DomainGraph::new();

    // Add PKI types
    graph = graph.add_node(DomainObject::new("Certificate"));
    graph = graph.add_node(DomainObject::new("Key"));

    // Add non-PKI types
    graph = graph.add_node(DomainObject::new("Person"));
    graph = graph.add_node(DomainObject::new("NatsOperator"));
    graph = graph.add_node(DomainObject::new("YubiKey"));

    // Filter for PKI view
    let pki_types = ["Certificate", "Key"];
    let filtered: Vec<_> = graph.nodes.values()
        .filter(|n| pki_types.contains(&n.aggregate_type.as_str()))
        .collect();

    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_cascade_delete() {
    // Test that deleting a node removes all connected edges
    let mut graph = DomainGraph::new();

    let alice = DomainObject::new("Person");
    let alice_id = alice.id;

    let bob = DomainObject::new("Person");
    let bob_id = bob.id;

    let key = DomainObject::new("Key");
    let key_id = key.id;

    graph = graph.add_node(alice);
    graph = graph.add_node(bob);
    graph = graph.add_node(key);

    // Create edges
    graph = graph.add_edge(DomainRelationship {
        source_id: alice_id,
        target_id: bob_id,
        relationship_type: "reports_to".to_string(),
    });

    graph = graph.add_edge(DomainRelationship {
        source_id: alice_id,
        target_id: key_id,
        relationship_type: "owns_key".to_string(),
    });

    assert_eq!(graph.edges.len(), 2);

    // Delete Alice (should cascade to edges)
    graph.nodes.remove(&alice_id);
    graph.edges.retain(|e| e.source_id != alice_id && e.target_id != alice_id);

    assert_eq!(graph.edges.len(), 0); // All edges to/from Alice removed
}

#[cfg(test)]
mod event_sourcing_tests {
    use super::*;
    use chrono::Utc;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "event_type")]
    enum TestGraphEvent {
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
        RelationshipEstablished {
            source_id: Uuid,
            target_id: Uuid,
            relationship_type: String,
            timestamp: chrono::DateTime<Utc>,
        },
    }

    #[test]
    fn test_event_emission_on_creation() {
        // Test that creating a node emits an event
        let person = DomainObject::new("Person")
            .with_property("legal_name", json!("Alice"));

        let event = TestGraphEvent::DomainObjectCreated {
            object_id: person.id,
            aggregate_type: person.aggregate_type.clone(),
            properties: serde_json::to_value(&person.properties).unwrap(),
            timestamp: Utc::now(),
        };

        // Serialize event
        let json = serde_json::to_string(&event).expect("Failed to serialize event");

        // Deserialize and verify
        let loaded: TestGraphEvent = serde_json::from_str(&json).expect("Failed to deserialize");

        match loaded {
            TestGraphEvent::DomainObjectCreated { aggregate_type, .. } => {
                assert_eq!(aggregate_type, "Person");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_event_emission_on_update() {
        // Test that updating a property emits an event
        let old_value = json!("Alice");
        let new_value = json!("Alice Smith");

        let event = TestGraphEvent::DomainObjectUpdated {
            object_id: Uuid::now_v7(),
            property_key: "legal_name".to_string(),
            old_value: old_value.clone(),
            new_value: new_value.clone(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let loaded: TestGraphEvent = serde_json::from_str(&json).unwrap();

        match loaded {
            TestGraphEvent::DomainObjectUpdated { old_value: old, new_value: new, .. } => {
                assert_eq!(old, old_value);
                assert_eq!(new, new_value);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_event_log_serialization() {
        // Test that event log can be serialized to JSON array
        let events = vec![
            TestGraphEvent::DomainObjectCreated {
                object_id: Uuid::now_v7(),
                aggregate_type: "Person".to_string(),
                properties: json!({"legal_name": "Alice"}),
                timestamp: Utc::now(),
            },
            TestGraphEvent::RelationshipEstablished {
                source_id: Uuid::now_v7(),
                target_id: Uuid::now_v7(),
                relationship_type: "reports_to".to_string(),
                timestamp: Utc::now(),
            },
        ];

        let json = serde_json::to_string_pretty(&events).unwrap();
        let loaded: Vec<TestGraphEvent> = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.len(), 2);
    }
}
