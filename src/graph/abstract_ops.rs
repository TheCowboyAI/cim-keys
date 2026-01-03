// Copyright (c) 2025 - Cowboy AI, LLC.

//! Abstract Graph Operations - UUID-Only Graph Algorithms
//!
//! This module provides graph algorithms that operate ONLY on the abstract
//! structure (UUIDs and edges), never lifting to domain types during traversal.
//!
//! # Kan Extension Pattern
//!
//! These operations embody the Kan extension principle:
//! - Graph layer works on abstract UUIDs
//! - Domain layer has concrete types (Person, Organization, etc.)
//! - Lift ONLY at boundaries when domain semantics are needed
//!
//! # Key Principle
//!
//! All functions in this module return `Uuid` or collections of `Uuid`.
//! They NEVER call `lift()`, `unlift()`, or `downcast()` during operation.
//!
//! # Usage
//!
//! ```rust,ignore
//! use cim_keys::graph::abstract_ops::AbstractGraphOps;
//! use cim_keys::lifting::LiftedGraph;
//!
//! let graph: LiftedGraph = build_graph();
//! let ops = AbstractGraphOps::from(&graph);
//!
//! // Find all nodes reachable from a starting point
//! let reachable: HashSet<Uuid> = ops.reachable_from(start_id);
//!
//! // Find shortest path (if any)
//! let path: Option<Vec<Uuid>> = ops.shortest_path(from_id, to_id);
//!
//! // Find root nodes (no incoming edges)
//! let roots: Vec<Uuid> = ops.find_roots();
//! ```

use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::lifting::LiftedGraph;

/// Abstract graph operations that work on UUIDs only.
///
/// This struct provides a view of the graph that exposes only the
/// structural relationships, not the domain content of nodes.
///
/// # Design Rationale
///
/// By constructing adjacency lists at creation time, we achieve:
/// - O(1) neighbor lookup during traversal
/// - Clear separation from the domain-aware `LiftedGraph`
/// - Explicit "no lifting" guarantee through the type signature
pub struct AbstractGraphOps {
    /// All node IDs in the graph
    node_ids: HashSet<Uuid>,

    /// Outgoing edges: node_id -> Vec<(target_id, edge_label)>
    outgoing: HashMap<Uuid, Vec<(Uuid, String)>>,

    /// Incoming edges: node_id -> Vec<(source_id, edge_label)>
    incoming: HashMap<Uuid, Vec<(Uuid, String)>>,

    /// Edge weights for weighted graph algorithms
    edge_weights: HashMap<(Uuid, Uuid), f64>,
}

impl AbstractGraphOps {
    /// Create abstract operations view from a lifted graph.
    ///
    /// This extracts only the structural information (IDs and edges),
    /// deliberately discarding domain content to enforce the Kan extension
    /// pattern.
    pub fn from_graph(graph: &LiftedGraph) -> Self {
        let mut node_ids = HashSet::new();
        let mut outgoing: HashMap<Uuid, Vec<(Uuid, String)>> = HashMap::new();
        let mut incoming: HashMap<Uuid, Vec<(Uuid, String)>> = HashMap::new();
        let mut edge_weights = HashMap::new();

        // Collect all node IDs (never access domain data)
        for node in graph.nodes() {
            node_ids.insert(node.id);
            outgoing.entry(node.id).or_default();
            incoming.entry(node.id).or_default();
        }

        // Build adjacency lists from edges
        for edge in graph.edges() {
            outgoing
                .entry(edge.from_id)
                .or_default()
                .push((edge.to_id, edge.label.clone()));

            incoming
                .entry(edge.to_id)
                .or_default()
                .push((edge.from_id, edge.label.clone()));

            // Store edge weight (default to 1.0 if not set)
            let weight = edge.weight.unwrap_or(1.0);
            edge_weights.insert((edge.from_id, edge.to_id), weight);
        }

        AbstractGraphOps {
            node_ids,
            outgoing,
            incoming,
            edge_weights,
        }
    }

    /// Get all node IDs in the graph.
    #[inline]
    pub fn all_nodes(&self) -> &HashSet<Uuid> {
        &self.node_ids
    }

    /// Check if a node exists in the graph.
    #[inline]
    pub fn contains(&self, id: Uuid) -> bool {
        self.node_ids.contains(&id)
    }

    /// Get the number of nodes.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.node_ids.len()
    }

    // ========================================================================
    // NEIGHBOR OPERATIONS
    // ========================================================================

    /// Get all outgoing neighbors (targets of outgoing edges).
    ///
    /// Returns UUIDs only - never domain types.
    pub fn outgoing_neighbors(&self, id: Uuid) -> Vec<Uuid> {
        self.outgoing
            .get(&id)
            .map(|edges| edges.iter().map(|(target, _)| *target).collect())
            .unwrap_or_default()
    }

    /// Get all incoming neighbors (sources of incoming edges).
    ///
    /// Returns UUIDs only - never domain types.
    pub fn incoming_neighbors(&self, id: Uuid) -> Vec<Uuid> {
        self.incoming
            .get(&id)
            .map(|edges| edges.iter().map(|(source, _)| *source).collect())
            .unwrap_or_default()
    }

    /// Get all neighbors (both directions).
    ///
    /// Returns UUIDs only - never domain types.
    pub fn neighbors(&self, id: Uuid) -> Vec<Uuid> {
        let mut result: HashSet<Uuid> = HashSet::new();

        if let Some(out) = self.outgoing.get(&id) {
            result.extend(out.iter().map(|(t, _)| *t));
        }
        if let Some(inc) = self.incoming.get(&id) {
            result.extend(inc.iter().map(|(s, _)| *s));
        }

        result.into_iter().collect()
    }

    /// Get outgoing edges with their labels.
    pub fn outgoing_edges(&self, id: Uuid) -> Vec<(Uuid, String)> {
        self.outgoing
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get incoming edges with their labels.
    pub fn incoming_edges(&self, id: Uuid) -> Vec<(Uuid, String)> {
        self.incoming
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }

    // ========================================================================
    // REACHABILITY
    // ========================================================================

    /// Find all nodes reachable from a starting node (following outgoing edges).
    ///
    /// Uses breadth-first search. Returns UUIDs only.
    /// The starting node is NOT included in the result.
    pub fn reachable_from(&self, start: Uuid) -> HashSet<Uuid> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.outgoing_neighbors(current) {
                if !visited.contains(&neighbor) && neighbor != start {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        visited
    }

    /// Find all nodes that can reach a target node (following incoming edges backwards).
    ///
    /// Uses breadth-first search. Returns UUIDs only.
    /// The target node is NOT included in the result.
    pub fn nodes_reaching(&self, target: Uuid) -> HashSet<Uuid> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(target);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.incoming_neighbors(current) {
                if !visited.contains(&neighbor) && neighbor != target {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        visited
    }

    /// Check if target is reachable from source.
    pub fn is_reachable(&self, source: Uuid, target: Uuid) -> bool {
        if source == target {
            return true;
        }
        self.reachable_from(source).contains(&target)
    }

    // ========================================================================
    // SHORTEST PATH
    // ========================================================================

    /// Find shortest path between two nodes (unweighted BFS).
    ///
    /// Returns the path as a sequence of UUIDs including both start and end.
    /// Returns `None` if no path exists.
    pub fn shortest_path(&self, from: Uuid, to: Uuid) -> Option<Vec<Uuid>> {
        if from == to {
            return Some(vec![from]);
        }

        if !self.node_ids.contains(&from) || !self.node_ids.contains(&to) {
            return None;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<Uuid, Uuid> = HashMap::new();

        visited.insert(from);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.outgoing_neighbors(current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    parent.insert(neighbor, current);

                    if neighbor == to {
                        // Reconstruct path
                        return Some(self.reconstruct_path(&parent, from, to));
                    }

                    queue.push_back(neighbor);
                }
            }
        }

        None
    }

    /// Find all shortest paths between two nodes.
    ///
    /// Returns all paths of minimum length.
    pub fn all_shortest_paths(&self, from: Uuid, to: Uuid) -> Vec<Vec<Uuid>> {
        if from == to {
            return vec![vec![from]];
        }

        if !self.node_ids.contains(&from) || !self.node_ids.contains(&to) {
            return vec![];
        }

        let mut paths = Vec::new();
        let mut visited_at_distance: HashMap<Uuid, usize> = HashMap::new();
        let mut queue = VecDeque::new();

        visited_at_distance.insert(from, 0);
        queue.push_back((from, vec![from]));

        let mut min_distance: Option<usize> = None;

        while let Some((current, path)) = queue.pop_front() {
            // If we've found paths and current distance exceeds minimum, stop
            if let Some(min_d) = min_distance {
                if path.len() > min_d {
                    continue;
                }
            }

            for neighbor in self.outgoing_neighbors(current) {
                let new_dist = path.len();

                // Allow revisiting if at same distance (for multiple paths)
                let should_visit = match visited_at_distance.get(&neighbor) {
                    Some(&old_dist) => new_dist <= old_dist,
                    None => true,
                };

                if should_visit {
                    visited_at_distance.insert(neighbor, new_dist);
                    let mut new_path = path.clone();
                    new_path.push(neighbor);

                    if neighbor == to {
                        if min_distance.is_none() {
                            min_distance = Some(new_path.len());
                        }
                        if new_path.len() == min_distance.unwrap() {
                            paths.push(new_path);
                        }
                    } else if min_distance.is_none() || new_path.len() < min_distance.unwrap() {
                        queue.push_back((neighbor, new_path));
                    }
                }
            }
        }

        paths
    }

    /// Reconstruct path from parent map.
    fn reconstruct_path(&self, parent: &HashMap<Uuid, Uuid>, from: Uuid, to: Uuid) -> Vec<Uuid> {
        let mut path = vec![to];
        let mut current = to;

        while current != from {
            if let Some(&p) = parent.get(&current) {
                path.push(p);
                current = p;
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// Get the weight of an edge between two nodes.
    ///
    /// Returns `None` if no edge exists between the nodes.
    pub fn edge_weight(&self, from: Uuid, to: Uuid) -> Option<f64> {
        self.edge_weights.get(&(from, to)).copied()
    }

    /// Calculate the weighted path cost.
    ///
    /// Given a path (sequence of node IDs), sums the edge weights.
    /// Returns `None` if any edge in the path doesn't exist.
    pub fn path_weight(&self, path: &[Uuid]) -> Option<f64> {
        if path.len() < 2 {
            return Some(0.0);
        }

        let mut total = 0.0;
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            total += self.edge_weight(from, to)?;
        }

        Some(total)
    }

    // ========================================================================
    // ROOT AND LEAF DETECTION
    // ========================================================================

    /// Find all root nodes (nodes with no incoming edges).
    ///
    /// In a DAG, these are the entry points.
    pub fn find_roots(&self) -> Vec<Uuid> {
        self.node_ids
            .iter()
            .filter(|id| {
                self.incoming
                    .get(*id)
                    .map(|edges| edges.is_empty())
                    .unwrap_or(true)
            })
            .copied()
            .collect()
    }

    /// Find all leaf nodes (nodes with no outgoing edges).
    ///
    /// In a DAG, these are the terminal points.
    pub fn find_leaves(&self) -> Vec<Uuid> {
        self.node_ids
            .iter()
            .filter(|id| {
                self.outgoing
                    .get(*id)
                    .map(|edges| edges.is_empty())
                    .unwrap_or(true)
            })
            .copied()
            .collect()
    }

    /// Find isolated nodes (no edges in either direction).
    pub fn find_isolated(&self) -> Vec<Uuid> {
        self.node_ids
            .iter()
            .filter(|id| {
                let no_out = self.outgoing.get(*id).map(|e| e.is_empty()).unwrap_or(true);
                let no_in = self.incoming.get(*id).map(|e| e.is_empty()).unwrap_or(true);
                no_out && no_in
            })
            .copied()
            .collect()
    }

    // ========================================================================
    // TOPOLOGICAL OPERATIONS
    // ========================================================================

    /// Perform topological sort (Kahn's algorithm).
    ///
    /// Returns `None` if the graph contains cycles.
    pub fn topological_sort(&self) -> Option<Vec<Uuid>> {
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();

        // Calculate in-degrees for each node
        for id in &self.node_ids {
            let count = self.incoming.get(id).map(|e| e.len()).unwrap_or(0);
            in_degree.insert(*id, count);
        }

        let mut queue: VecDeque<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node);

            for neighbor in self.outgoing_neighbors(node) {
                if let Some(deg) = in_degree.get_mut(&neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if result.len() == self.node_ids.len() {
            Some(result)
        } else {
            None // Graph has cycles
        }
    }

    /// Check if the graph is a DAG (directed acyclic graph).
    pub fn is_dag(&self) -> bool {
        self.topological_sort().is_some()
    }

    // ========================================================================
    // CYCLE DETECTION
    // ========================================================================

    /// Find all cycles in the graph.
    ///
    /// Uses depth-first search with color marking.
    /// Returns cycles as vectors of UUIDs.
    pub fn find_cycles(&self) -> Vec<Vec<Uuid>> {
        let mut cycles = Vec::new();
        let mut color: HashMap<Uuid, Color> = HashMap::new();

        for &start in &self.node_ids {
            color.insert(start, Color::White);
        }

        for &start in &self.node_ids {
            if color.get(&start) == Some(&Color::White) {
                let mut path = Vec::new();
                self.dfs_cycle(&start, &mut color, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: &Uuid,
        color: &mut HashMap<Uuid, Color>,
        path: &mut Vec<Uuid>,
        cycles: &mut Vec<Vec<Uuid>>,
    ) {
        color.insert(*node, Color::Gray);
        path.push(*node);

        for neighbor in self.outgoing_neighbors(*node) {
            match color.get(&neighbor) {
                Some(Color::White) => {
                    self.dfs_cycle(&neighbor, color, path, cycles);
                }
                Some(Color::Gray) => {
                    // Found a cycle - extract it
                    if let Some(cycle_start) = path.iter().position(|&n| n == neighbor) {
                        let cycle: Vec<Uuid> = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
                _ => {}
            }
        }

        path.pop();
        color.insert(*node, Color::Black);
    }

    /// Check if the graph contains any cycles.
    pub fn has_cycles(&self) -> bool {
        !self.find_cycles().is_empty()
    }

    // ========================================================================
    // STRONGLY CONNECTED COMPONENTS
    // ========================================================================

    /// Find strongly connected components (Kosaraju's algorithm).
    ///
    /// Returns components as vectors of UUIDs.
    pub fn strongly_connected_components(&self) -> Vec<Vec<Uuid>> {
        // First pass: get finish order
        let mut visited = HashSet::new();
        let mut finish_order = Vec::new();

        for &node in &self.node_ids {
            if !visited.contains(&node) {
                self.dfs_finish_order(node, &mut visited, &mut finish_order);
            }
        }

        // Second pass: DFS on transposed graph in reverse finish order
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for node in finish_order.into_iter().rev() {
            if !visited.contains(&node) {
                let mut component = Vec::new();
                self.dfs_scc_collect(node, &mut visited, &mut component);
                if !component.is_empty() {
                    components.push(component);
                }
            }
        }

        components
    }

    fn dfs_finish_order(&self, node: Uuid, visited: &mut HashSet<Uuid>, finish_order: &mut Vec<Uuid>) {
        visited.insert(node);

        for neighbor in self.outgoing_neighbors(node) {
            if !visited.contains(&neighbor) {
                self.dfs_finish_order(neighbor, visited, finish_order);
            }
        }

        finish_order.push(node);
    }

    fn dfs_scc_collect(&self, node: Uuid, visited: &mut HashSet<Uuid>, component: &mut Vec<Uuid>) {
        visited.insert(node);
        component.push(node);

        // Use incoming edges (transpose graph)
        for neighbor in self.incoming_neighbors(node) {
            if !visited.contains(&neighbor) {
                self.dfs_scc_collect(neighbor, visited, component);
            }
        }
    }

    // ========================================================================
    // DISTANCE AND METRICS
    // ========================================================================

    /// Get the distance (shortest path length) between two nodes.
    ///
    /// Returns `None` if no path exists.
    pub fn distance(&self, from: Uuid, to: Uuid) -> Option<usize> {
        self.shortest_path(from, to).map(|path| path.len() - 1)
    }

    /// Calculate the eccentricity of a node (maximum distance to any other node).
    ///
    /// Returns `None` if the node cannot reach all other nodes.
    pub fn eccentricity(&self, node: Uuid) -> Option<usize> {
        let reachable = self.reachable_from(node);

        // Check if all other nodes are reachable
        if reachable.len() + 1 != self.node_ids.len() {
            return None;
        }

        let mut max_dist = 0;
        for target in &self.node_ids {
            if *target != node {
                if let Some(dist) = self.distance(node, *target) {
                    max_dist = max_dist.max(dist);
                }
            }
        }

        Some(max_dist)
    }

    /// Get in-degree of a node (number of incoming edges).
    pub fn in_degree(&self, node: Uuid) -> usize {
        self.incoming.get(&node).map(|e| e.len()).unwrap_or(0)
    }

    /// Get out-degree of a node (number of outgoing edges).
    pub fn out_degree(&self, node: Uuid) -> usize {
        self.outgoing.get(&node).map(|e| e.len()).unwrap_or(0)
    }

    /// Get total degree of a node (in-degree + out-degree).
    pub fn degree(&self, node: Uuid) -> usize {
        self.in_degree(node) + self.out_degree(node)
    }
}

/// DFS color for cycle detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    White, // Unvisited
    Gray,  // In progress
    Black, // Finished
}

// ============================================================================
// EDGE FILTERING
// ============================================================================

/// A filtered view of abstract graph operations based on edge labels.
pub struct FilteredGraphOps<'a> {
    base: &'a AbstractGraphOps,
    allowed_labels: HashSet<String>,
}

impl<'a> FilteredGraphOps<'a> {
    /// Create a filtered view that only considers edges with specified labels.
    pub fn new(base: &'a AbstractGraphOps, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        FilteredGraphOps {
            base,
            allowed_labels: labels.into_iter().map(|l| l.into()).collect(),
        }
    }

    /// Get outgoing neighbors through allowed edge labels only.
    pub fn outgoing_neighbors(&self, id: Uuid) -> Vec<Uuid> {
        self.base
            .outgoing
            .get(&id)
            .map(|edges| {
                edges
                    .iter()
                    .filter(|(_, label)| self.allowed_labels.contains(label))
                    .map(|(target, _)| *target)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find reachable nodes through allowed edge labels only.
    pub fn reachable_from(&self, start: Uuid) -> HashSet<Uuid> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.outgoing_neighbors(current) {
                if !visited.contains(&neighbor) && neighbor != start {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        visited
    }

    /// Find shortest path through allowed edge labels only.
    pub fn shortest_path(&self, from: Uuid, to: Uuid) -> Option<Vec<Uuid>> {
        if from == to {
            return Some(vec![from]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<Uuid, Uuid> = HashMap::new();

        visited.insert(from);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.outgoing_neighbors(current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    parent.insert(neighbor, current);

                    if neighbor == to {
                        return Some(self.base.reconstruct_path(&parent, from, to));
                    }

                    queue.push_back(neighbor);
                }
            }
        }

        None
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifting::{LiftedGraph, LiftedNode, Injection};
    use iced::Color;

    fn make_test_node(id: Uuid, label: &str) -> LiftedNode {
        LiftedNode::new::<String>(id, Injection::Person, label, Color::WHITE, String::new())
    }

    fn build_simple_graph() -> LiftedGraph {
        // A -> B -> C
        //      |
        //      v
        //      D
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let c = Uuid::now_v7();
        let d = Uuid::now_v7();

        let mut graph = LiftedGraph::new();
        graph.add_node(make_test_node(a, "A"));
        graph.add_node(make_test_node(b, "B"));
        graph.add_node(make_test_node(c, "C"));
        graph.add_node(make_test_node(d, "D"));

        graph.connect(a, b, "to", Color::WHITE);
        graph.connect(b, c, "to", Color::WHITE);
        graph.connect(b, d, "to", Color::WHITE);

        graph
    }

    fn build_cycle_graph() -> (LiftedGraph, Uuid, Uuid, Uuid) {
        // A -> B -> C -> A (cycle)
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let c = Uuid::now_v7();

        let mut graph = LiftedGraph::new();
        graph.add_node(make_test_node(a, "A"));
        graph.add_node(make_test_node(b, "B"));
        graph.add_node(make_test_node(c, "C"));

        graph.connect(a, b, "to", Color::WHITE);
        graph.connect(b, c, "to", Color::WHITE);
        graph.connect(c, a, "to", Color::WHITE);

        (graph, a, b, c)
    }

    fn build_dag() -> (LiftedGraph, Uuid, Uuid, Uuid, Uuid) {
        // A -> B -> D
        // |         ^
        // v         |
        // C --------+
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let c = Uuid::now_v7();
        let d = Uuid::now_v7();

        let mut graph = LiftedGraph::new();
        graph.add_node(make_test_node(a, "A"));
        graph.add_node(make_test_node(b, "B"));
        graph.add_node(make_test_node(c, "C"));
        graph.add_node(make_test_node(d, "D"));

        graph.connect(a, b, "to", Color::WHITE);
        graph.connect(a, c, "to", Color::WHITE);
        graph.connect(b, d, "to", Color::WHITE);
        graph.connect(c, d, "to", Color::WHITE);

        (graph, a, b, c, d)
    }

    // ========================================================================
    // BASIC OPERATIONS TESTS
    // ========================================================================

    #[test]
    fn test_from_graph() {
        let graph = build_simple_graph();
        let ops = AbstractGraphOps::from_graph(&graph);

        assert_eq!(ops.node_count(), 4);
    }

    #[test]
    fn test_neighbors() {
        let (graph, a, b, c, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        // A has outgoing to B and C
        let a_out = ops.outgoing_neighbors(a);
        assert_eq!(a_out.len(), 2);
        assert!(a_out.contains(&b));
        assert!(a_out.contains(&c));

        // A has no incoming
        assert!(ops.incoming_neighbors(a).is_empty());

        // D has two incoming
        assert_eq!(ops.incoming_neighbors(d).len(), 2);
    }

    // ========================================================================
    // REACHABILITY TESTS
    // ========================================================================

    #[test]
    fn test_reachable_from() {
        let (graph, a, b, c, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        let reachable = ops.reachable_from(a);
        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains(&b));
        assert!(reachable.contains(&c));
        assert!(reachable.contains(&d));
        assert!(!reachable.contains(&a)); // Start not included
    }

    #[test]
    fn test_nodes_reaching() {
        let (graph, a, b, c, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        let reaching = ops.nodes_reaching(d);
        assert_eq!(reaching.len(), 3);
        assert!(reaching.contains(&a));
        assert!(reaching.contains(&b));
        assert!(reaching.contains(&c));
    }

    #[test]
    fn test_is_reachable() {
        let (graph, a, b, _, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        assert!(ops.is_reachable(a, d));
        assert!(ops.is_reachable(b, d));
        assert!(!ops.is_reachable(d, a)); // No path back
    }

    // ========================================================================
    // SHORTEST PATH TESTS
    // ========================================================================

    #[test]
    fn test_shortest_path() {
        let (graph, a, b, _, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        let path = ops.shortest_path(a, d);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.len(), 3); // A -> B -> D or A -> C -> D
        assert_eq!(path.first(), Some(&a));
        assert_eq!(path.last(), Some(&d));
    }

    #[test]
    fn test_shortest_path_no_path() {
        let (graph, _, _, _, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        // No path from D back to anywhere except D
        let path = ops.shortest_path(d, ops.all_nodes().iter().find(|&&n| n != d).copied().unwrap());
        assert!(path.is_none());
    }

    #[test]
    fn test_shortest_path_self() {
        let (graph, a, _, _, _) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        let path = ops.shortest_path(a, a);
        assert_eq!(path, Some(vec![a]));
    }

    // ========================================================================
    // ROOTS AND LEAVES TESTS
    // ========================================================================

    #[test]
    fn test_find_roots() {
        let (graph, a, _, _, _) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        let roots = ops.find_roots();
        assert_eq!(roots.len(), 1);
        assert!(roots.contains(&a));
    }

    #[test]
    fn test_find_leaves() {
        let (graph, _, _, _, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        let leaves = ops.find_leaves();
        assert_eq!(leaves.len(), 1);
        assert!(leaves.contains(&d));
    }

    // ========================================================================
    // TOPOLOGICAL SORT TESTS
    // ========================================================================

    #[test]
    fn test_topological_sort() {
        let (graph, a, b, c, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        let sorted = ops.topological_sort();
        assert!(sorted.is_some());

        let sorted = sorted.unwrap();
        assert_eq!(sorted.len(), 4);

        // A must come before B, C, D
        let a_pos = sorted.iter().position(|&n| n == a).unwrap();
        let b_pos = sorted.iter().position(|&n| n == b).unwrap();
        let c_pos = sorted.iter().position(|&n| n == c).unwrap();
        let d_pos = sorted.iter().position(|&n| n == d).unwrap();

        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        let (graph, _, _, _) = build_cycle_graph();
        let ops = AbstractGraphOps::from_graph(&graph);

        assert!(ops.topological_sort().is_none());
    }

    #[test]
    fn test_is_dag() {
        let (dag, _, _, _, _) = build_dag();
        let (cycle, _, _, _) = build_cycle_graph();

        assert!(AbstractGraphOps::from_graph(&dag).is_dag());
        assert!(!AbstractGraphOps::from_graph(&cycle).is_dag());
    }

    // ========================================================================
    // CYCLE DETECTION TESTS
    // ========================================================================

    #[test]
    fn test_find_cycles() {
        let (graph, _, _, _) = build_cycle_graph();
        let ops = AbstractGraphOps::from_graph(&graph);

        let cycles = ops.find_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_has_cycles() {
        let (dag, _, _, _, _) = build_dag();
        let (cycle, _, _, _) = build_cycle_graph();

        assert!(!AbstractGraphOps::from_graph(&dag).has_cycles());
        assert!(AbstractGraphOps::from_graph(&cycle).has_cycles());
    }

    // ========================================================================
    // SCC TESTS
    // ========================================================================

    #[test]
    fn test_strongly_connected_components() {
        let (graph, _, _, _) = build_cycle_graph();
        let ops = AbstractGraphOps::from_graph(&graph);

        let sccs = ops.strongly_connected_components();
        // All 3 nodes are in one SCC due to cycle
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 3);
    }

    #[test]
    fn test_scc_dag() {
        let (graph, _, _, _, _) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        let sccs = ops.strongly_connected_components();
        // Each node is its own SCC in a DAG
        assert_eq!(sccs.len(), 4);
    }

    // ========================================================================
    // DEGREE TESTS
    // ========================================================================

    #[test]
    fn test_degree() {
        let (graph, a, b, _, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        // A: 0 in, 2 out
        assert_eq!(ops.in_degree(a), 0);
        assert_eq!(ops.out_degree(a), 2);
        assert_eq!(ops.degree(a), 2);

        // B: 1 in, 1 out
        assert_eq!(ops.in_degree(b), 1);
        assert_eq!(ops.out_degree(b), 1);

        // D: 2 in, 0 out
        assert_eq!(ops.in_degree(d), 2);
        assert_eq!(ops.out_degree(d), 0);
    }

    // ========================================================================
    // FILTERED GRAPH TESTS
    // ========================================================================

    #[test]
    fn test_filtered_graph() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let c = Uuid::now_v7();

        let mut graph = LiftedGraph::new();
        graph.add_node(make_test_node(a, "A"));
        graph.add_node(make_test_node(b, "B"));
        graph.add_node(make_test_node(c, "C"));

        graph.connect(a, b, "owns", Color::WHITE);
        graph.connect(b, c, "manages", Color::WHITE);

        let ops = AbstractGraphOps::from_graph(&graph);
        let filtered = FilteredGraphOps::new(&ops, ["owns"]);

        // Should only follow "owns" edges
        let reachable = filtered.reachable_from(a);
        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains(&b));
        assert!(!reachable.contains(&c)); // "manages" edge filtered out
    }

    // ========================================================================
    // NO LIFT VERIFICATION
    // ========================================================================

    #[test]
    fn test_no_lift_during_operations() {
        // This test verifies that AbstractGraphOps never accesses domain data.
        // The proof is in the API: all methods return Uuid or collections of Uuid,
        // and the struct doesn't store LiftedNode or any domain types.

        let (graph, a, _, _, d) = build_dag();
        let ops = AbstractGraphOps::from_graph(&graph);

        // All operations return UUIDs only
        let _reachable: HashSet<Uuid> = ops.reachable_from(a);
        let _path: Option<Vec<Uuid>> = ops.shortest_path(a, d);
        let _roots: Vec<Uuid> = ops.find_roots();
        let _leaves: Vec<Uuid> = ops.find_leaves();
        let _neighbors: Vec<Uuid> = ops.neighbors(a);
        let _cycles: Vec<Vec<Uuid>> = ops.find_cycles();
        let _sccs: Vec<Vec<Uuid>> = ops.strongly_connected_components();
        let _topo: Option<Vec<Uuid>> = ops.topological_sort();

        // The type system guarantees no domain access
        // (AbstractGraphOps doesn't have access to LiftedNode's domain data)
    }
}
