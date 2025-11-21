//! Graph Signal Foundation
//!
//! Represents the organization graph as reactive signals, enabling:
//! - Declarative graph transformations
//! - Automatic updates when filters/search change
//! - Type-safe node/edge operations

use crate::signals::{Signal, EventKind, StepKind, ContinuousKind, Time};
use crate::gui::graph::{GraphNode, GraphEdge, OrganizationGraph, NodeType, EdgeType};
use std::collections::HashMap;
use uuid::Uuid;
use iced::Point;

/// Node signal - discrete node events (add/remove/select)
pub type NodeSignal = Signal<EventKind, GraphNode>;

/// Edge signal - discrete edge events (create/delete)
pub type EdgeSignal = Signal<EventKind, GraphEdge>;

/// Graph state signal - current graph structure (piecewise constant)
pub type GraphStateSignal = Signal<StepKind, OrganizationGraph>;

/// Filter state for controlling node/edge visibility
#[derive(Clone, Debug, PartialEq)]
pub struct FilterState {
    pub show_people: bool,
    pub show_orgs: bool,
    pub show_nats: bool,
    pub show_pki: bool,
    pub show_yubikey: bool,
    pub search_query: String,
}

impl Default for FilterState {
    fn default() -> Self {
        FilterState {
            show_people: true,
            show_orgs: true,
            show_nats: true,
            show_pki: true,
            show_yubikey: true,
            search_query: String::new(),
        }
    }
}

/// Compute visible nodes based on current filters
///
/// This is a pure function that can be used as a signal transformation
pub fn visible_nodes(
    graph: &OrganizationGraph,
    filters: &FilterState,
) -> Vec<GraphNode> {
    graph.nodes.values()
        .filter(|node| {
            // Apply category filters
            let category_match = match &node.node_type {
                NodeType::Person { .. } => filters.show_people,
                NodeType::Organization { .. } | NodeType::OrganizationalUnit { .. } => filters.show_orgs,
                NodeType::NatsOperator { .. } | NodeType::NatsAccount { .. } | NodeType::NatsUser { .. } => filters.show_nats,
                NodeType::RootCertificate { .. } | NodeType::IntermediateCertificate { .. } | NodeType::LeafCertificate { .. } => filters.show_pki,
                NodeType::YubiKey { .. } | NodeType::PivSlot { .. } | NodeType::YubiKeyStatus { .. } => filters.show_yubikey,
                _ => true,
            };

            // Apply search filter
            let search_match = if filters.search_query.is_empty() {
                true
            } else {
                let query = filters.search_query.to_lowercase();
                node.label.to_lowercase().contains(&query)
            };

            category_match && search_match
        })
        .cloned()
        .collect()
}

/// Compute highlighted nodes based on search query
///
/// Returns node IDs that should be emphasized in the visualization
pub fn highlighted_nodes(
    graph: &OrganizationGraph,
    search_query: &str,
) -> Vec<Uuid> {
    if search_query.is_empty() {
        return vec![];
    }

    let query = search_query.to_lowercase();
    graph.nodes.values()
        .filter(|node| {
            node.label.to_lowercase().contains(&query)
        })
        .map(|node| node.id)
        .collect()
}

/// Compute node positions for a given layout algorithm
///
/// This can be used as a continuous signal for animated layout transitions
pub fn compute_node_positions(
    graph: &OrganizationGraph,
    layout: LayoutAlgorithm,
) -> HashMap<Uuid, Point> {
    match layout {
        LayoutAlgorithm::Manual => {
            // Keep current positions
            graph.nodes.iter()
                .map(|(id, node)| (*id, node.position))
                .collect()
        }
        LayoutAlgorithm::Hierarchical => {
            compute_hierarchical_layout(graph)
        }
        LayoutAlgorithm::ForceDirected => {
            compute_force_directed_layout(graph)
        }
        LayoutAlgorithm::Circular => {
            compute_circular_layout(graph)
        }
    }
}

/// Layout algorithms supported by the graph
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutAlgorithm {
    Manual,
    Hierarchical,
    ForceDirected,
    Circular,
}

/// Compute hierarchical layout (tree-like)
fn compute_hierarchical_layout(graph: &OrganizationGraph) -> HashMap<Uuid, Point> {
    let mut positions = HashMap::new();

    // Find root nodes (no incoming edges)
    let root_nodes: Vec<_> = graph.nodes.keys()
        .filter(|id| {
            !graph.edges.iter().any(|edge| &edge.to == *id)
        })
        .collect();

    // Layout from roots
    let mut level = 0;
    let mut current_level = root_nodes;
    let mut visited = std::collections::HashSet::new();

    while !current_level.is_empty() {
        let y = 100.0 + level as f32 * 150.0;
        let node_count = current_level.len();

        for (i, node_id) in current_level.iter().enumerate() {
            if visited.contains(*node_id) {
                continue;
            }
            visited.insert(**node_id);

            let x = 100.0 + (i as f32 / node_count.max(1) as f32) * 800.0;
            positions.insert(**node_id, Point::new(x, y));
        }

        // Find children
        let mut next_level = Vec::new();
        for parent_id in &current_level {
            for edge in &graph.edges {
                if &edge.from == *parent_id && !visited.contains(&edge.to) {
                    next_level.push(&edge.to);
                }
            }
        }

        current_level = next_level;
        level += 1;
    }

    positions
}

/// Compute force-directed layout (Fruchterman-Reingold)
fn compute_force_directed_layout(graph: &OrganizationGraph) -> HashMap<Uuid, Point> {
    // Simplified force-directed layout
    // In production, this would iterate to minimize energy
    let mut positions = HashMap::new();

    let center = Point::new(400.0, 300.0);
    let node_ids: Vec<_> = graph.nodes.keys().collect();

    for (i, node_id) in node_ids.iter().enumerate() {
        let angle = (i as f32 / node_ids.len() as f32) * 2.0 * std::f32::consts::PI;
        let radius = 200.0;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        positions.insert(**node_id, Point::new(x, y));
    }

    positions
}

/// Compute circular layout
fn compute_circular_layout(graph: &OrganizationGraph) -> HashMap<Uuid, Point> {
    let mut positions = HashMap::new();

    let center = Point::new(400.0, 300.0);
    let node_ids: Vec<_> = graph.nodes.keys().collect();
    let radius = 250.0;

    for (i, node_id) in node_ids.iter().enumerate() {
        let angle = (i as f32 / node_ids.len() as f32) * 2.0 * std::f32::consts::PI;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        positions.insert(**node_id, Point::new(x, y));
    }

    positions
}

/// Create a step signal for graph state
pub fn create_graph_state_signal(initial_graph: OrganizationGraph) -> GraphStateSignal {
    Signal::<StepKind, OrganizationGraph>::step(initial_graph)
}

/// Update graph state signal with new graph
pub fn update_graph_state(
    signal: GraphStateSignal,
    new_graph: OrganizationGraph,
) -> GraphStateSignal {
    signal.with_value(new_graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Person, Organization, KeyOwnerRole};

    fn test_person(name: &str) -> Person {
        Person {
            id: Uuid::now_v7(),
            name: name.to_string(),
            email: format!("{}@example.com", name.to_lowercase()),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            created_at: chrono::Utc::now(),
            active: true,
        }
    }

    fn test_org(name: &str) -> Organization {
        Organization {
            id: Uuid::now_v7(),
            name: name.to_string(),
            display_name: name.to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_visible_nodes_all_enabled() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Alice");
        let org = test_org("Acme Corp");

        graph.add_node(person, KeyOwnerRole::Developer);
        graph.add_organization_node(org);

        let filters = FilterState::default();
        let visible = visible_nodes(&graph, &filters);

        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn test_visible_nodes_people_only() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Alice");
        let person_id = person.id;
        let org = test_org("Acme Corp");

        graph.add_node(person, KeyOwnerRole::Developer);
        graph.add_organization_node(org);

        let mut filters = FilterState::default();
        filters.show_orgs = false;

        let visible = visible_nodes(&graph, &filters);

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, person_id);
    }

    #[test]
    fn test_search_filter() {
        let mut graph = OrganizationGraph::new();
        graph.add_node(test_person("Alice"), KeyOwnerRole::Developer);
        graph.add_node(test_person("Bob"), KeyOwnerRole::Developer);

        let mut filters = FilterState::default();
        filters.search_query = "alice".to_string();

        let visible = visible_nodes(&graph, &filters);

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].label, "Alice");
    }

    #[test]
    fn test_highlighted_nodes() {
        let mut graph = OrganizationGraph::new();
        let alice = test_person("Alice");
        let alice_id = alice.id;
        let bob = test_person("Bob");

        graph.add_node(alice, KeyOwnerRole::Developer);
        graph.add_node(bob, KeyOwnerRole::Developer);

        let highlighted = highlighted_nodes(&graph, "alice");

        assert_eq!(highlighted.len(), 1);
        assert_eq!(highlighted[0], alice_id);
    }

    #[test]
    fn test_graph_state_signal() {
        let graph = OrganizationGraph::new();
        let signal = create_graph_state_signal(graph.clone());

        let sampled = signal.sample(0.0);
        assert_eq!(sampled.nodes.len(), 0);
    }

    #[test]
    fn test_layout_algorithms() {
        let mut graph = OrganizationGraph::new();
        let alice = test_person("Alice");
        let bob = test_person("Bob");
        let id1 = alice.id;
        let id2 = bob.id;

        graph.add_node(alice, KeyOwnerRole::Developer);
        graph.add_node(bob, KeyOwnerRole::Developer);

        let circular = compute_node_positions(&graph, LayoutAlgorithm::Circular);
        assert_eq!(circular.len(), 2);
        assert!(circular.contains_key(&id1));
        assert!(circular.contains_key(&id2));

        let hierarchical = compute_node_positions(&graph, LayoutAlgorithm::Hierarchical);
        assert_eq!(hierarchical.len(), 2);
    }
}
