//! Graph persistence tests
//!
//! Tests that graphs can be saved to/loaded from JSON files.
//! This ensures the save/load functionality works correctly.

use cim_keys::graph_ui::types::{DomainGraph, DomainObject, DomainRelationship};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Helper to get a temporary test directory
fn get_test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cim-keys-test-{}", Uuid::now_v7()));
    fs::create_dir_all(&dir).expect("Failed to create test directory");
    dir
}

/// Helper to clean up test directory
fn cleanup_test_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_save_and_load_empty_graph() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("graph.json");

    // Create empty graph
    let graph = DomainGraph::new();

    // Save to file
    let json = serde_json::to_string_pretty(&graph).expect("Failed to serialize");
    fs::write(&graph_path, json).expect("Failed to write file");

    // Load from file
    let loaded_json = fs::read_to_string(&graph_path).expect("Failed to read file");
    let loaded: DomainGraph =
        serde_json::from_str(&loaded_json).expect("Failed to deserialize");

    // Verify
    assert_eq!(loaded.nodes.len(), 0);
    assert_eq!(loaded.edges.len(), 0);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_graph_with_nodes() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("graph.json");

    // Create graph with nodes
    let mut graph = DomainGraph::new();

    let person = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice Smith"))
        .with_property("active", json!(true));
    let person_id = person.id;

    let org = DomainObject::new("Organization")
        .with_property("name", json!("CowboyAI"));
    let org_id = org.id;

    graph = graph.add_node(person);
    graph = graph.add_node(org);

    // Save to file
    let json = serde_json::to_string_pretty(&graph).expect("Failed to serialize");
    fs::write(&graph_path, json).expect("Failed to write file");

    // Load from file
    let loaded_json = fs::read_to_string(&graph_path).expect("Failed to read file");
    let loaded: DomainGraph =
        serde_json::from_str(&loaded_json).expect("Failed to deserialize");

    // Verify
    assert_eq!(loaded.nodes.len(), 2);
    assert!(loaded.get_node(person_id).is_some());
    assert!(loaded.get_node(org_id).is_some());

    let loaded_person = loaded.get_node(person_id).unwrap();
    assert_eq!(loaded_person.aggregate_type, "Person");
    assert_eq!(
        loaded_person.properties.get("legal_name"),
        Some(&json!("Alice Smith"))
    );

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_graph_with_edges() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("graph.json");

    // Create graph with nodes and edges
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

    // Save to file
    let json = serde_json::to_string_pretty(&graph).expect("Failed to serialize");
    fs::write(&graph_path, json).expect("Failed to write file");

    // Load from file
    let loaded_json = fs::read_to_string(&graph_path).expect("Failed to read file");
    let loaded: DomainGraph =
        serde_json::from_str(&loaded_json).expect("Failed to deserialize");

    // Verify
    assert_eq!(loaded.nodes.len(), 2);
    assert_eq!(loaded.edges.len(), 1);

    let loaded_edge = &loaded.edges[0];
    assert_eq!(loaded_edge.source_id, alice_id);
    assert_eq!(loaded_edge.target_id, bob_id);
    assert_eq!(loaded_edge.relationship_type, "reports_to");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_complex_graph() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("graph.json");

    // Create complex graph
    let mut graph = DomainGraph::new();

    // Create multiple domain types
    let person1 = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice"));
    let person1_id = person1.id;

    let person2 = DomainObject::new("Person")
        .with_property("legal_name", json!("Bob"));
    let person2_id = person2.id;

    let org = DomainObject::new("Organization")
        .with_property("name", json!("CowboyAI"));
    let org_id = org.id;

    let location = DomainObject::new("Location")
        .with_property("address", json!("alice@cowboyai.com"));
    let location_id = location.id;

    let key = DomainObject::new("Key")
        .with_property("key_type", json!("ed25519"));
    let key_id = key.id;

    graph = graph.add_node(person1);
    graph = graph.add_node(person2);
    graph = graph.add_node(org);
    graph = graph.add_node(location);
    graph = graph.add_node(key);

    // Create multiple edges
    graph = graph.add_edge(DomainRelationship {
        source_id: person1_id,
        target_id: person2_id,
        relationship_type: "reports_to".to_string(),
    });

    graph = graph.add_edge(DomainRelationship {
        source_id: person1_id,
        target_id: location_id,
        relationship_type: "located_at".to_string(),
    });

    graph = graph.add_edge(DomainRelationship {
        source_id: person1_id,
        target_id: key_id,
        relationship_type: "owns_key".to_string(),
    });

    // Save to file
    let json = serde_json::to_string_pretty(&graph).expect("Failed to serialize");
    fs::write(&graph_path, json).expect("Failed to write file");

    // Load from file
    let loaded_json = fs::read_to_string(&graph_path).expect("Failed to read file");
    let loaded: DomainGraph =
        serde_json::from_str(&loaded_json).expect("Failed to deserialize");

    // Verify
    assert_eq!(loaded.nodes.len(), 5);
    assert_eq!(loaded.edges.len(), 3);

    // Verify all node types exist
    assert!(loaded.get_node(person1_id).is_some());
    assert!(loaded.get_node(person2_id).is_some());
    assert!(loaded.get_node(org_id).is_some());
    assert!(loaded.get_node(location_id).is_some());
    assert!(loaded.get_node(key_id).is_some());

    // Verify edge traversal works after load
    let alice_keys = loaded.traverse_edges(person1_id, "owns_key", "Key");
    assert_eq!(alice_keys.len(), 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_load_example_graph_files() {
    // Test loading the example graph files we created
    let example_files = vec![
        "examples/graph-data/simple-graph.json",
        "examples/graph-data/organization-example.json",
        "examples/graph-data/nats-infrastructure.json",
        "examples/graph-data/pki-hierarchy.json",
    ];

    for file_path in example_files {
        let full_path = PathBuf::from(file_path);
        if !full_path.exists() {
            // Skip if example files don't exist (e.g., in CI)
            continue;
        }

        let json = fs::read_to_string(&full_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path));

        let graph: DomainGraph = serde_json::from_str(&json)
            .unwrap_or_else(|_| panic!("Failed to deserialize {}", file_path));

        // Basic validation
        assert!(
            graph.nodes.len() > 0,
            "Example file {} has no nodes",
            file_path
        );

        // Verify all edges reference valid nodes
        for edge in &graph.edges {
            assert!(
                graph.get_node(edge.source_id).is_some(),
                "Edge source not found in {}",
                file_path
            );
            assert!(
                graph.get_node(edge.target_id).is_some(),
                "Edge target not found in {}",
                file_path
            );
        }
    }
}

#[test]
fn test_roundtrip_preserves_data() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("graph.json");

    // Create graph
    let mut original = DomainGraph::new();

    let node = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice Smith"))
        .with_property("active", json!(true))
        .with_property("role", json!("Manager"))
        .with_property("level", json!(5));
    let node_id = node.id;

    original = original.add_node(node);

    // Save
    let json = serde_json::to_string_pretty(&original).expect("Failed to serialize");
    fs::write(&graph_path, &json).expect("Failed to write file");

    // Load
    let loaded_json = fs::read_to_string(&graph_path).expect("Failed to read file");
    let loaded: DomainGraph =
        serde_json::from_str(&loaded_json).expect("Failed to deserialize");

    // Verify all properties preserved
    let loaded_node = loaded.get_node(node_id).unwrap();
    assert_eq!(
        loaded_node.properties.get("legal_name"),
        Some(&json!("Alice Smith"))
    );
    assert_eq!(loaded_node.properties.get("active"), Some(&json!(true)));
    assert_eq!(loaded_node.properties.get("role"), Some(&json!("Manager")));
    assert_eq!(loaded_node.properties.get("level"), Some(&json!(5)));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_with_uuid_v7_ids() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("graph.json");

    // Create nodes with UUID v7
    let mut graph = DomainGraph::new();

    let node = DomainObject::new("Person");
    let node_id = node.id;

    graph = graph.add_node(node);

    // Save
    let json = serde_json::to_string_pretty(&graph).expect("Failed to serialize");
    fs::write(&graph_path, &json).expect("Failed to write file");

    // Load
    let loaded_json = fs::read_to_string(&graph_path).expect("Failed to read file");
    let loaded: DomainGraph =
        serde_json::from_str(&loaded_json).expect("Failed to deserialize");

    // Verify UUID preserved
    let loaded_node = loaded.get_node(node_id).unwrap();
    assert_eq!(loaded_node.id, node_id);

    // Verify it's UUID v7 (version field should be 7)
    assert_eq!(node_id.get_version_num(), 7);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_multiple_save_load_cycles() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("graph.json");

    // Create initial graph
    let mut graph = DomainGraph::new();

    let node1 = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice"));
    let node1_id = node1.id;

    graph = graph.add_node(node1);

    // Save cycle 1
    let json = serde_json::to_string_pretty(&graph).unwrap();
    fs::write(&graph_path, json).unwrap();

    // Load
    let loaded_json = fs::read_to_string(&graph_path).unwrap();
    let mut graph = serde_json::from_str::<DomainGraph>(&loaded_json).unwrap();

    // Add another node
    let node2 = DomainObject::new("Person")
        .with_property("legal_name", json!("Bob"));
    let node2_id = node2.id;

    graph = graph.add_node(node2);

    // Save cycle 2
    let json = serde_json::to_string_pretty(&graph).unwrap();
    fs::write(&graph_path, json).unwrap();

    // Load again
    let loaded_json = fs::read_to_string(&graph_path).unwrap();
    let final_graph = serde_json::from_str::<DomainGraph>(&loaded_json).unwrap();

    // Verify both nodes exist
    assert_eq!(final_graph.nodes.len(), 2);
    assert!(final_graph.get_node(node1_id).is_some());
    assert!(final_graph.get_node(node2_id).is_some());

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_json_format_is_readable() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("graph.json");

    // Create simple graph
    let mut graph = DomainGraph::new();

    let person = DomainObject::new("Person")
        .with_property("legal_name", json!("Alice Smith"));

    graph = graph.add_node(person);

    // Save with pretty printing
    let json = serde_json::to_string_pretty(&graph).unwrap();
    fs::write(&graph_path, &json).unwrap();

    // Read raw JSON
    let json_content = fs::read_to_string(&graph_path).unwrap();

    // Verify it's formatted (contains newlines and indentation)
    assert!(json_content.contains('\n'));
    assert!(json_content.contains("  ")); // Indentation

    // Verify it contains expected keys
    assert!(json_content.contains("\"nodes\""));
    assert!(json_content.contains("\"edges\""));
    assert!(json_content.contains("\"Person\""));
    assert!(json_content.contains("\"Alice Smith\""));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_error_handling_invalid_json() {
    // Test that invalid JSON is handled gracefully
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("invalid.json");

    // Write invalid JSON
    fs::write(&graph_path, "{ invalid json }").unwrap();

    // Try to load
    let loaded_json = fs::read_to_string(&graph_path).unwrap();
    let result = serde_json::from_str::<DomainGraph>(&loaded_json);

    // Should return an error
    assert!(result.is_err());

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_error_handling_missing_file() {
    let test_dir = get_test_dir();
    let graph_path = test_dir.join("nonexistent.json");

    // Try to load non-existent file
    let result = fs::read_to_string(&graph_path);

    // Should return an error
    assert!(result.is_err());

    cleanup_test_dir(&test_dir);
}
