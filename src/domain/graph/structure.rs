// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Graph Structure - Adjacency List Based Graph
//!
//! Provides a proper graph-theoretic structure for domain relationships.
//! Uses adjacency lists for O(degree) neighbor lookup.
//!
//! ## Key Features
//! - O(degree) neighbor lookup via adjacency lists
//! - Uses DomainRelation as first-class edges
//! - BFS/DFS/TopSort graph algorithms
//! - Cycle detection for DAG validation
//!
//! ## Graph Theory Compliance
//! - Nodes as vertices (HashSet<Uuid>)
//! - DomainRelations as edges (HashMap<Uuid, DomainRelation>)
//! - Adjacency structure for O(1) neighbor lookup
//! - Multi-graph support (multiple edges between nodes)

use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use super::relations::{DomainRelation, RelationType};

// ============================================================================
// DOMAIN GRAPH
// ============================================================================

/// A proper graph structure for domain relationships
///
/// Uses adjacency lists for O(degree) neighbor lookup.
/// Stores DomainRelation as first-class edges.
#[derive(Debug, Clone, Default)]
pub struct DomainGraph {
    /// All relations indexed by their ID
    relations: HashMap<Uuid, DomainRelation>,

    /// Adjacency list: node -> outgoing relation IDs
    outgoing: HashMap<Uuid, Vec<Uuid>>,

    /// Adjacency list: node -> incoming relation IDs
    incoming: HashMap<Uuid, Vec<Uuid>>,

    /// All known node IDs (for iteration)
    nodes: HashSet<Uuid>,
}

impl DomainGraph {
    /// Create a new empty domain graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph (registers it for traversal)
    pub fn add_node(&mut self, node_id: Uuid) {
        self.nodes.insert(node_id);
        self.outgoing.entry(node_id).or_default();
        self.incoming.entry(node_id).or_default();
    }

    /// Add a relation (edge) to the graph
    pub fn add_relation(&mut self, relation: DomainRelation) -> Uuid {
        let id = relation.id;
        let from = relation.from;
        let to = relation.to;

        // Ensure nodes exist
        self.add_node(from);
        self.add_node(to);

        // Add to adjacency lists
        self.outgoing.entry(from).or_default().push(id);
        self.incoming.entry(to).or_default().push(id);

        // Store relation
        self.relations.insert(id, relation);

        id
    }

    /// Create and add a relation
    pub fn connect(&mut self, from: Uuid, to: Uuid, relation_type: RelationType) -> Uuid {
        let relation = DomainRelation::new(from, to, relation_type);
        self.add_relation(relation)
    }

    /// Get a relation by ID
    pub fn get_relation(&self, id: Uuid) -> Option<&DomainRelation> {
        self.relations.get(&id)
    }

    /// Get a mutable reference to a relation by ID
    pub fn get_relation_mut(&mut self, id: Uuid) -> Option<&mut DomainRelation> {
        self.relations.get_mut(&id)
    }

    /// Get all outgoing relations from a node - O(degree)
    pub fn outgoing_relations(&self, node_id: Uuid) -> Vec<&DomainRelation> {
        self.outgoing
            .get(&node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.relations.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all incoming relations to a node - O(degree)
    pub fn incoming_relations(&self, node_id: Uuid) -> Vec<&DomainRelation> {
        self.incoming
            .get(&node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.relations.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all neighbors (both directions) - O(degree)
    pub fn neighbors(&self, node_id: Uuid) -> Vec<Uuid> {
        let mut neighbors = HashSet::new();

        for rel in self.outgoing_relations(node_id) {
            neighbors.insert(rel.to);
        }
        for rel in self.incoming_relations(node_id) {
            neighbors.insert(rel.from);
        }

        neighbors.into_iter().collect()
    }

    /// Get outgoing neighbors only - O(degree)
    pub fn successors(&self, node_id: Uuid) -> Vec<Uuid> {
        self.outgoing_relations(node_id)
            .iter()
            .map(|rel| rel.to)
            .collect()
    }

    /// Get incoming neighbors only - O(degree)
    pub fn predecessors(&self, node_id: Uuid) -> Vec<Uuid> {
        self.incoming_relations(node_id)
            .iter()
            .map(|rel| rel.from)
            .collect()
    }

    /// Get relations between two nodes
    pub fn relations_between(&self, from: Uuid, to: Uuid) -> Vec<&DomainRelation> {
        self.outgoing_relations(from)
            .into_iter()
            .filter(|rel| rel.to == to)
            .collect()
    }

    /// Check if edge exists between two nodes
    pub fn has_edge(&self, from: Uuid, to: Uuid) -> bool {
        !self.relations_between(from, to).is_empty()
    }

    /// Check if edge of specific type exists
    pub fn has_edge_of_type(&self, from: Uuid, to: Uuid, relation_type: RelationType) -> bool {
        self.relations_between(from, to)
            .iter()
            .any(|rel| rel.relation_type == relation_type)
    }

    /// Get all nodes
    pub fn node_ids(&self) -> impl Iterator<Item = &Uuid> {
        self.nodes.iter()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if a node exists
    pub fn contains_node(&self, node_id: Uuid) -> bool {
        self.nodes.contains(&node_id)
    }

    /// Get all relations
    pub fn all_relations(&self) -> impl Iterator<Item = &DomainRelation> {
        self.relations.values()
    }

    /// Get relation count
    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    /// Remove a relation
    pub fn remove_relation(&mut self, id: Uuid) -> Option<DomainRelation> {
        if let Some(rel) = self.relations.remove(&id) {
            // Remove from adjacency lists
            if let Some(out) = self.outgoing.get_mut(&rel.from) {
                out.retain(|r| *r != id);
            }
            if let Some(inc) = self.incoming.get_mut(&rel.to) {
                inc.retain(|r| *r != id);
            }
            Some(rel)
        } else {
            None
        }
    }

    /// Remove a node and all its relations
    pub fn remove_node(&mut self, node_id: Uuid) -> bool {
        if !self.nodes.remove(&node_id) {
            return false;
        }

        // Collect relation IDs to remove
        let outgoing_ids: Vec<Uuid> = self
            .outgoing
            .remove(&node_id)
            .unwrap_or_default();
        let incoming_ids: Vec<Uuid> = self
            .incoming
            .remove(&node_id)
            .unwrap_or_default();

        // Remove outgoing relations
        for id in outgoing_ids {
            if let Some(rel) = self.relations.remove(&id) {
                if let Some(inc) = self.incoming.get_mut(&rel.to) {
                    inc.retain(|r| *r != id);
                }
            }
        }

        // Remove incoming relations
        for id in incoming_ids {
            if let Some(rel) = self.relations.remove(&id) {
                if let Some(out) = self.outgoing.get_mut(&rel.from) {
                    out.retain(|r| *r != id);
                }
            }
        }

        true
    }

    /// Get degree (number of edges) for a node
    pub fn degree(&self, node_id: Uuid) -> usize {
        self.in_degree(node_id) + self.out_degree(node_id)
    }

    /// Get in-degree (incoming edges) for a node
    pub fn in_degree(&self, node_id: Uuid) -> usize {
        self.incoming.get(&node_id).map(|v| v.len()).unwrap_or(0)
    }

    /// Get out-degree (outgoing edges) for a node
    pub fn out_degree(&self, node_id: Uuid) -> usize {
        self.outgoing.get(&node_id).map(|v| v.len()).unwrap_or(0)
    }

    /// Filter relations by type
    pub fn relations_of_type(&self, relation_type: RelationType) -> Vec<&DomainRelation> {
        self.relations
            .values()
            .filter(|rel| rel.relation_type == relation_type)
            .collect()
    }

    /// Get only valid (non-expired) relations
    pub fn valid_relations(&self) -> Vec<&DomainRelation> {
        self.relations.values().filter(|rel| rel.is_valid()).collect()
    }
}

// ============================================================================
// GRAPH ALGORITHMS
// ============================================================================

impl DomainGraph {
    /// Breadth-First Search from a starting node
    ///
    /// Returns nodes in BFS order (level by level).
    pub fn bfs(&self, start: Uuid) -> Vec<Uuid> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        if !self.nodes.contains(&start) {
            return result;
        }

        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            result.push(node);

            for successor in self.successors(node) {
                if !visited.contains(&successor) {
                    visited.insert(successor);
                    queue.push_back(successor);
                }
            }
        }

        result
    }

    /// Depth-First Search from a starting node
    ///
    /// Returns nodes in DFS order (deepest first).
    pub fn dfs(&self, start: Uuid) -> Vec<Uuid> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        if self.nodes.contains(&start) {
            self.dfs_recursive(start, &mut visited, &mut result);
        }

        result
    }

    fn dfs_recursive(&self, node: Uuid, visited: &mut HashSet<Uuid>, result: &mut Vec<Uuid>) {
        if visited.contains(&node) {
            return;
        }

        visited.insert(node);
        result.push(node);

        for successor in self.successors(node) {
            self.dfs_recursive(successor, visited, result);
        }
    }

    /// Topological sort using Kahn's algorithm
    ///
    /// Returns None if the graph contains a cycle.
    pub fn topological_sort(&self) -> Option<Vec<Uuid>> {
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degrees
        for node in &self.nodes {
            in_degree.insert(*node, self.in_degree(*node));
        }

        // Start with nodes that have no incoming edges
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(*node);
            }
        }

        while let Some(node) = queue.pop_front() {
            result.push(node);

            for successor in self.successors(node) {
                if let Some(degree) = in_degree.get_mut(&successor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(successor);
                    }
                }
            }
        }

        // If we processed all nodes, no cycle exists
        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            None // Cycle detected
        }
    }

    /// Detect if the graph contains a cycle
    ///
    /// Uses DFS-based cycle detection.
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in &self.nodes {
            if self.has_cycle_recursive(*node, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    fn has_cycle_recursive(
        &self,
        node: Uuid,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
    ) -> bool {
        if rec_stack.contains(&node) {
            return true; // Back edge = cycle
        }

        if visited.contains(&node) {
            return false; // Already fully processed
        }

        visited.insert(node);
        rec_stack.insert(node);

        for successor in self.successors(node) {
            if self.has_cycle_recursive(successor, visited, rec_stack) {
                return true;
            }
        }

        rec_stack.remove(&node);
        false
    }

    /// Find all nodes reachable from a starting node
    pub fn reachable_from(&self, start: Uuid) -> HashSet<Uuid> {
        self.bfs(start).into_iter().collect()
    }

    /// Find all ancestors of a node (nodes that can reach this node)
    pub fn ancestors(&self, node: Uuid) -> HashSet<Uuid> {
        let mut visited = HashSet::new();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            for predecessor in self.predecessors(current) {
                if !visited.contains(&predecessor) {
                    stack.push(predecessor);
                }
            }
        }

        visited.remove(&node); // Don't include the node itself
        visited
    }

    /// Find all descendants of a node (nodes reachable from this node)
    pub fn descendants(&self, node: Uuid) -> HashSet<Uuid> {
        let mut visited = self.reachable_from(node);
        visited.remove(&node); // Don't include the node itself
        visited
    }

    /// Find shortest path between two nodes (unweighted)
    ///
    /// Returns None if no path exists.
    pub fn shortest_path(&self, from: Uuid, to: Uuid) -> Option<Vec<Uuid>> {
        if from == to {
            return Some(vec![from]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<Uuid, Uuid> = HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(node) = queue.pop_front() {
            if node == to {
                // Reconstruct path
                let mut path = vec![to];
                let mut current = to;
                while let Some(&p) = parent.get(&current) {
                    path.push(p);
                    current = p;
                }
                path.reverse();
                return Some(path);
            }

            for successor in self.successors(node) {
                if !visited.contains(&successor) {
                    visited.insert(successor);
                    parent.insert(successor, node);
                    queue.push_back(successor);
                }
            }
        }

        None // No path found
    }

    /// Find all connected components
    pub fn connected_components(&self) -> Vec<HashSet<Uuid>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for &node in &self.nodes {
            if !visited.contains(&node) {
                let component = self.bfs_undirected(node);
                for &n in &component {
                    visited.insert(n);
                }
                components.push(component);
            }
        }

        components
    }

    /// BFS treating graph as undirected
    fn bfs_undirected(&self, start: Uuid) -> HashSet<Uuid> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            for neighbor in self.neighbors(node) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        visited
    }
}

// ============================================================================
// CYCLE ERROR
// ============================================================================

/// Error when a cycle is detected
#[derive(Debug, Clone)]
pub enum CycleError {
    /// Adding a relation would create a cycle
    WouldCreateCycle {
        from: Uuid,
        to: Uuid,
        relation_type: RelationType,
    },
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CycleError::WouldCreateCycle { from, to, relation_type } => {
                write!(
                    f,
                    "Adding {:?} relation from {} to {} would create a cycle",
                    relation_type, from, to
                )
            }
        }
    }
}

impl std::error::Error for CycleError {}

// ============================================================================
// DAG OPERATIONS
// ============================================================================

impl DomainGraph {
    /// Add a relation only if it doesn't create a cycle
    ///
    /// Useful for causation chains and hierarchies that must remain acyclic.
    pub fn add_relation_acyclic(
        &mut self,
        from: Uuid,
        to: Uuid,
        relation_type: RelationType,
    ) -> Result<Uuid, CycleError> {
        // Add the edge temporarily
        let rel_id = self.connect(from, to, relation_type.clone());

        // Check if this creates a cycle
        if self.has_cycle() {
            // Remove the edge
            self.remove_relation(rel_id);
            return Err(CycleError::WouldCreateCycle {
                from,
                to,
                relation_type,
            });
        }

        Ok(rel_id)
    }

    /// Check if adding an edge would create a cycle (without adding it)
    pub fn would_create_cycle(&self, from: Uuid, to: Uuid) -> bool {
        // If to can already reach from, adding from->to creates a cycle
        if !self.nodes.contains(&from) || !self.nodes.contains(&to) {
            return false;
        }
        self.reachable_from(to).contains(&from)
    }
}

// ============================================================================
// BOOTSTRAP CONVERSION HELPERS
// ============================================================================

/// Extension trait for converting bootstrap types to graph relationships.
///
/// Bootstrap types (Organization, Person, etc.) contain embedded Vec<>
/// collections for JSON serialization convenience. These helpers convert
/// them to proper graph relationships.
impl DomainGraph {
    /// Add an organization and all its units as graph nodes with proper edges.
    ///
    /// Converts `Organization.units: Vec<OrganizationUnit>` to graph edges.
    pub fn add_organization_with_units(
        &mut self,
        org_id: Uuid,
        unit_ids: &[Uuid],
    ) {
        self.add_node(org_id);

        for &unit_id in unit_ids {
            self.add_node(unit_id);
            // Organization contains unit (parent-child)
            self.connect(org_id, unit_id, RelationType::ParentChild);
        }
    }

    /// Add a person with their organizational relationships.
    ///
    /// Converts `Person.unit_ids: Vec<UnitId>` to MemberOf edges
    /// and establishes the employment relationship.
    pub fn add_person_to_organization(
        &mut self,
        person_id: Uuid,
        org_id: Uuid,
        unit_ids: &[Uuid],
    ) {
        self.add_node(person_id);
        self.add_node(org_id);

        // Organization employs person
        self.connect(org_id, person_id, RelationType::Manages);

        // Person member of units
        for &unit_id in unit_ids {
            self.add_node(unit_id);
            self.connect(person_id, unit_id, RelationType::MemberOf);
        }
    }

    /// Add a unit hierarchy (parent-child relationships between units).
    ///
    /// Converts `OrganizationUnit.parent_unit_id` to graph edges.
    pub fn add_unit_hierarchy(&mut self, unit_id: Uuid, parent_unit_id: Option<Uuid>) {
        self.add_node(unit_id);

        if let Some(parent_id) = parent_unit_id {
            self.add_node(parent_id);
            self.connect(parent_id, unit_id, RelationType::ParentChild);
        }
    }

    /// Add a person's roles as edges.
    ///
    /// Converts `Person.roles: Vec<PersonRole>` to HasRole edges.
    pub fn add_person_roles(&mut self, person_id: Uuid, role_ids: &[Uuid]) {
        self.add_node(person_id);

        for &role_id in role_ids {
            self.add_node(role_id);
            self.connect(person_id, role_id, RelationType::HasRole {
                valid_from: chrono::Utc::now(),
                valid_until: None,
            });
        }
    }

    /// Add a manager-reports-to relationship.
    pub fn add_reports_to(&mut self, person_id: Uuid, manager_id: Uuid) {
        self.add_node(person_id);
        self.add_node(manager_id);
        self.connect(person_id, manager_id, RelationType::ManagedBy);
    }

    /// Get all units that belong to an organization (outgoing ParentChild edges)
    pub fn get_org_units(&self, org_id: Uuid) -> Vec<Uuid> {
        self.outgoing_relations(org_id)
            .into_iter()
            .filter(|rel| rel.relation_type == RelationType::ParentChild)
            .map(|rel| rel.to)
            .collect()
    }

    /// Get all units a person belongs to (outgoing MemberOf edges)
    pub fn get_person_units(&self, person_id: Uuid) -> Vec<Uuid> {
        self.outgoing_relations(person_id)
            .into_iter()
            .filter(|rel| rel.relation_type == RelationType::MemberOf)
            .map(|rel| rel.to)
            .collect()
    }

    /// Get all people in an organization (outgoing Manages edges to person nodes)
    pub fn get_org_people(&self, org_id: Uuid) -> Vec<Uuid> {
        self.outgoing_relations(org_id)
            .into_iter()
            .filter(|rel| rel.relation_type == RelationType::Manages)
            .map(|rel| rel.to)
            .collect()
    }

    /// Get all people in a unit (incoming MemberOf edges)
    pub fn get_unit_members(&self, unit_id: Uuid) -> Vec<Uuid> {
        self.incoming_relations(unit_id)
            .into_iter()
            .filter(|rel| rel.relation_type == RelationType::MemberOf)
            .map(|rel| rel.from)
            .collect()
    }

    /// Get the parent unit of a unit (if any)
    pub fn get_parent_unit(&self, unit_id: Uuid) -> Option<Uuid> {
        self.incoming_relations(unit_id)
            .into_iter()
            .find(|rel| rel.relation_type == RelationType::ParentChild)
            .map(|rel| rel.from)
    }

    /// Get child units of a unit
    pub fn get_child_units(&self, unit_id: Uuid) -> Vec<Uuid> {
        self.outgoing_relations(unit_id)
            .into_iter()
            .filter(|rel| rel.relation_type == RelationType::ParentChild)
            .map(|rel| rel.to)
            .collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_ids() -> (Uuid, Uuid, Uuid, Uuid) {
        (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        )
    }

    #[test]
    fn test_add_node_and_relation() {
        let mut graph = DomainGraph::new();
        let (a, b, _, _) = make_test_ids();

        graph.add_node(a);
        graph.add_node(b);
        let rel_id = graph.connect(a, b, RelationType::ParentChild);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.relation_count(), 1);
        assert!(graph.has_edge(a, b));
        assert!(!graph.has_edge(b, a));
        assert!(graph.get_relation(rel_id).is_some());
    }

    #[test]
    fn test_adjacency_lookup() {
        let mut graph = DomainGraph::new();
        let (org, unit1, unit2, person) = make_test_ids();

        graph.connect(org, unit1, RelationType::ParentChild);
        graph.connect(org, unit2, RelationType::ParentChild);
        graph.connect(org, person, RelationType::Manages);
        graph.connect(person, unit1, RelationType::MemberOf);

        // O(degree) lookups
        assert_eq!(graph.out_degree(org), 3);
        assert_eq!(graph.in_degree(org), 0);
        assert_eq!(graph.successors(org).len(), 3);
        // unit1 has predecessors: org (ParentChild) and person (MemberOf)
        assert_eq!(graph.predecessors(unit1).len(), 2);
    }

    #[test]
    fn test_bfs() {
        let mut graph = DomainGraph::new();
        let (a, b, c, d) = make_test_ids();

        // a -> b -> c
        //   \-> d
        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(b, c, RelationType::ParentChild);
        graph.connect(a, d, RelationType::ParentChild);

        let bfs = graph.bfs(a);
        assert_eq!(bfs.len(), 4);
        assert_eq!(bfs[0], a); // Start node first
        // b and d should be at level 1, c at level 2
        assert!(bfs[1] == b || bfs[1] == d);
    }

    #[test]
    fn test_dfs() {
        let mut graph = DomainGraph::new();
        let (a, b, c, d) = make_test_ids();

        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(b, c, RelationType::ParentChild);
        graph.connect(a, d, RelationType::ParentChild);

        let dfs = graph.dfs(a);
        assert_eq!(dfs.len(), 4);
        assert_eq!(dfs[0], a); // Start node first
    }

    #[test]
    fn test_topological_sort_dag() {
        let mut graph = DomainGraph::new();
        let (a, b, c, d) = make_test_ids();

        // DAG: a -> b -> c, a -> d -> c
        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(b, c, RelationType::ParentChild);
        graph.connect(a, d, RelationType::ParentChild);
        graph.connect(d, c, RelationType::ParentChild);

        let topo = graph.topological_sort();
        assert!(topo.is_some());
        let order = topo.unwrap();
        assert_eq!(order.len(), 4);

        // a must come before b and d
        let pos_a = order.iter().position(|&x| x == a).unwrap();
        let pos_b = order.iter().position(|&x| x == b).unwrap();
        let pos_d = order.iter().position(|&x| x == d).unwrap();
        let pos_c = order.iter().position(|&x| x == c).unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_d);
        assert!(pos_b < pos_c);
        assert!(pos_d < pos_c);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DomainGraph::new();
        let (a, b, c, _) = make_test_ids();

        // No cycle
        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(b, c, RelationType::ParentChild);
        assert!(!graph.has_cycle());
        assert!(graph.topological_sort().is_some());

        // Add cycle: c -> a
        graph.connect(c, a, RelationType::ParentChild);
        assert!(graph.has_cycle());
        assert!(graph.topological_sort().is_none());
    }

    #[test]
    fn test_acyclic_relation() {
        let mut graph = DomainGraph::new();
        let (a, b, c, _) = make_test_ids();

        // Valid chain: a -> b -> c
        assert!(graph.add_relation_acyclic(a, b, RelationType::ParentChild).is_ok());
        assert!(graph.add_relation_acyclic(b, c, RelationType::ParentChild).is_ok());

        // Invalid: c -> a would create cycle
        let result = graph.add_relation_acyclic(c, a, RelationType::ParentChild);
        assert!(result.is_err());
        assert!(!graph.has_cycle()); // Graph should still be acyclic
    }

    #[test]
    fn test_shortest_path() {
        let mut graph = DomainGraph::new();
        let (a, b, c, d) = make_test_ids();

        // a -> b -> c (length 2)
        // a -> d -> c (length 2)
        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(b, c, RelationType::ParentChild);
        graph.connect(a, d, RelationType::ParentChild);
        graph.connect(d, c, RelationType::ParentChild);

        let path = graph.shortest_path(a, c);
        assert!(path.is_some());
        let p = path.unwrap();
        assert_eq!(p.len(), 3); // a -> ? -> c
        assert_eq!(p[0], a);
        assert_eq!(p[2], c);
    }

    #[test]
    fn test_ancestors_descendants() {
        let mut graph = DomainGraph::new();
        let (root, mid, leaf, _) = make_test_ids();

        graph.connect(root, mid, RelationType::ParentChild);
        graph.connect(mid, leaf, RelationType::ParentChild);

        let ancestors = graph.ancestors(leaf);
        assert!(ancestors.contains(&root));
        assert!(ancestors.contains(&mid));
        assert!(!ancestors.contains(&leaf));

        let descendants = graph.descendants(root);
        assert!(descendants.contains(&mid));
        assert!(descendants.contains(&leaf));
        assert!(!descendants.contains(&root));
    }

    #[test]
    fn test_multi_graph() {
        let mut graph = DomainGraph::new();
        let (a, b, _, _) = make_test_ids();

        // Multiple edges between same nodes
        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(a, b, RelationType::Manages);

        let rels = graph.relations_between(a, b);
        assert_eq!(rels.len(), 2);

        assert!(graph.has_edge_of_type(a, b, RelationType::ParentChild));
        assert!(graph.has_edge_of_type(a, b, RelationType::Manages));
    }

    #[test]
    fn test_remove_relation() {
        let mut graph = DomainGraph::new();
        let (a, b, _, _) = make_test_ids();

        let rel_id = graph.connect(a, b, RelationType::ParentChild);
        assert!(graph.has_edge(a, b));

        graph.remove_relation(rel_id);
        assert!(!graph.has_edge(a, b));
        assert_eq!(graph.relation_count(), 0);
    }

    #[test]
    fn test_remove_node() {
        let mut graph = DomainGraph::new();
        let (a, b, c, _) = make_test_ids();

        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(b, c, RelationType::ParentChild);

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.relation_count(), 2);

        // Remove middle node
        graph.remove_node(b);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.relation_count(), 0);
        assert!(!graph.contains_node(b));
    }

    #[test]
    fn test_connected_components() {
        let mut graph = DomainGraph::new();
        let (a, b, c, d) = make_test_ids();

        // Two disconnected components: {a, b} and {c, d}
        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(c, d, RelationType::ParentChild);

        let components = graph.connected_components();
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn test_would_create_cycle() {
        let mut graph = DomainGraph::new();
        let (a, b, c, _) = make_test_ids();

        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(b, c, RelationType::ParentChild);

        assert!(graph.would_create_cycle(c, a));
        assert!(!graph.would_create_cycle(a, c));
    }

    #[test]
    fn test_relations_of_type() {
        let mut graph = DomainGraph::new();
        let (a, b, c, d) = make_test_ids();

        graph.connect(a, b, RelationType::ParentChild);
        graph.connect(b, c, RelationType::ParentChild);
        graph.connect(c, d, RelationType::Manages);

        let parent_child_rels = graph.relations_of_type(RelationType::ParentChild);
        assert_eq!(parent_child_rels.len(), 2);

        let manages_rels = graph.relations_of_type(RelationType::Manages);
        assert_eq!(manages_rels.len(), 1);
    }

    // ========================================================================
    // BOOTSTRAP CONVERSION TESTS
    // ========================================================================

    #[test]
    fn test_add_organization_with_units() {
        let mut graph = DomainGraph::new();
        let org_id = Uuid::now_v7();
        let unit1 = Uuid::now_v7();
        let unit2 = Uuid::now_v7();
        let unit3 = Uuid::now_v7();

        graph.add_organization_with_units(org_id, &[unit1, unit2, unit3]);

        // Should have 4 nodes
        assert_eq!(graph.node_count(), 4);
        // Should have 3 edges (org -> unit for each unit)
        assert_eq!(graph.relation_count(), 3);

        // Query units of organization
        let units = graph.get_org_units(org_id);
        assert_eq!(units.len(), 3);
        assert!(units.contains(&unit1));
        assert!(units.contains(&unit2));
        assert!(units.contains(&unit3));
    }

    #[test]
    fn test_add_person_to_organization() {
        let mut graph = DomainGraph::new();
        let org_id = Uuid::now_v7();
        let person_id = Uuid::now_v7();
        let unit1 = Uuid::now_v7();
        let unit2 = Uuid::now_v7();

        graph.add_person_to_organization(person_id, org_id, &[unit1, unit2]);

        // Should have 4 nodes (org, person, unit1, unit2)
        assert_eq!(graph.node_count(), 4);
        // Should have 3 edges (org->person, person->unit1, person->unit2)
        assert_eq!(graph.relation_count(), 3);

        // Query person's units
        let units = graph.get_person_units(person_id);
        assert_eq!(units.len(), 2);
        assert!(units.contains(&unit1));
        assert!(units.contains(&unit2));

        // Query org's people
        let people = graph.get_org_people(org_id);
        assert_eq!(people.len(), 1);
        assert!(people.contains(&person_id));
    }

    #[test]
    fn test_get_unit_members() {
        let mut graph = DomainGraph::new();
        let org_id = Uuid::now_v7();
        let unit_id = Uuid::now_v7();
        let person1 = Uuid::now_v7();
        let person2 = Uuid::now_v7();

        // Add org with unit
        graph.add_organization_with_units(org_id, &[unit_id]);

        // Add two people to the same unit
        graph.add_person_to_organization(person1, org_id, &[unit_id]);
        graph.add_person_to_organization(person2, org_id, &[unit_id]);

        // Query unit members
        let members = graph.get_unit_members(unit_id);
        assert_eq!(members.len(), 2);
        assert!(members.contains(&person1));
        assert!(members.contains(&person2));
    }

    #[test]
    fn test_unit_hierarchy() {
        let mut graph = DomainGraph::new();
        let parent_unit = Uuid::now_v7();
        let child_unit1 = Uuid::now_v7();
        let child_unit2 = Uuid::now_v7();

        graph.add_unit_hierarchy(parent_unit, None); // Root unit
        graph.add_unit_hierarchy(child_unit1, Some(parent_unit));
        graph.add_unit_hierarchy(child_unit2, Some(parent_unit));

        // Query parent of child
        assert_eq!(graph.get_parent_unit(child_unit1), Some(parent_unit));
        assert_eq!(graph.get_parent_unit(child_unit2), Some(parent_unit));
        assert_eq!(graph.get_parent_unit(parent_unit), None);

        // Query children of parent
        let children = graph.get_child_units(parent_unit);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child_unit1));
        assert!(children.contains(&child_unit2));
    }

    #[test]
    fn test_full_organization_graph() {
        let mut graph = DomainGraph::new();

        // Create organization structure
        let org = Uuid::now_v7();
        let engineering = Uuid::now_v7();
        let product = Uuid::now_v7();
        let frontend = Uuid::now_v7();
        let backend = Uuid::now_v7();

        let alice = Uuid::now_v7();
        let bob = Uuid::now_v7();
        let charlie = Uuid::now_v7();

        // Build org structure
        graph.add_organization_with_units(org, &[engineering, product]);
        graph.add_unit_hierarchy(frontend, Some(engineering));
        graph.add_unit_hierarchy(backend, Some(engineering));

        // Add people
        graph.add_person_to_organization(alice, org, &[engineering, frontend]);
        graph.add_person_to_organization(bob, org, &[engineering, backend]);
        graph.add_person_to_organization(charlie, org, &[product]);

        // Alice reports to Bob
        graph.add_reports_to(alice, bob);

        // Verify structure
        assert_eq!(graph.get_org_units(org).len(), 2); // engineering, product
        assert_eq!(graph.get_child_units(engineering).len(), 2); // frontend, backend
        assert_eq!(graph.get_org_people(org).len(), 3); // alice, bob, charlie

        // Alice is in both engineering and frontend
        let alice_units = graph.get_person_units(alice);
        assert!(alice_units.contains(&engineering));
        assert!(alice_units.contains(&frontend));

        // BFS from org should reach all nodes
        let reachable = graph.bfs(org);
        assert!(reachable.contains(&engineering));
        assert!(reachable.contains(&alice));
        assert!(reachable.contains(&frontend));
    }
}
