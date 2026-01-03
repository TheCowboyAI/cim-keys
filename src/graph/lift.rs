// Copyright (c) 2025 - Cowboy AI, LLC.

//! Lift Boundary - Explicit Kan Extension Utilities
//!
//! This module provides utilities for the "lift boundary" - the explicit point
//! where abstract graph operations transition to domain-aware operations.
//!
//! # Kan Extension Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    GRAPH LAYER (Abstract)                       │
//! │  - Nodes are UUIDs + opaque payloads                            │
//! │  - Edges are triples (source_id, relation, target_id)           │
//! │  - AbstractGraphOps works here (path, reachable, SCC, etc.)     │
//! │  - Returns UUIDs, never concrete types                          │
//! └─────────────────────────────────────────────────────────────────┘
//!                               │
//!                               │  ═══════════════════════════════
//!                               │  ║     LIFT BOUNDARY           ║
//!                               │  ║  (This Module's Utilities)  ║
//!                               │  ═══════════════════════════════
//!                               │
//!                               │  Kan Extension (Lan_K F)
//!                               │  lift() only when needed
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    DOMAIN LAYER (Concrete)                      │
//! │  - Person, Organization, Key, Certificate, etc.                 │
//! │  - Aggregates with state machines                               │
//! │  - Domain-specific operations (validate, sign, apply_policy)    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # When to Lift
//!
//! The boundary defines THREE legitimate lift points:
//!
//! 1. **Visualization Boundary**: Converting domain entities to visual elements
//!    ```rust,ignore
//!    let registry = VisualizationRegistry::new(&theme);
//!    let vis_data = registry.fold(&lifted_node)?; // Lift happens inside fold
//!    ```
//!
//! 2. **Query Boundary**: Extracting data for UI panels or search
//!    ```rust,ignore
//!    let registry = DetailPanelRegistry::new();
//!    let panel_data = registry.fold(&lifted_node)?; // Lift happens inside fold
//!    ```
//!
//! 3. **Command Boundary**: Domain operations triggered by user intent
//!    ```rust,ignore
//!    if let Some(person) = Person::unlift(&lifted_node) {
//!        // Domain operation on concrete type
//!        let updated = person.with_email(new_email);
//!    }
//!    ```
//!
//! # When NOT to Lift
//!
//! - During graph traversal (use AbstractGraphOps)
//! - During path finding, cycle detection, SCC
//! - During any structural graph operation
//!
//! # Usage
//!
//! ```rust,ignore
//! use cim_keys::graph::lift::{LiftBoundary, DeferredLift, BatchLift};
//!
//! // Deferred lifting - only lifts when accessed
//! let deferred = DeferredLift::new(uuid, &graph);
//! // ... later, at the boundary ...
//! let node = deferred.lift(); // Lift happens here
//!
//! // Batch lifting - efficient multi-node lifting
//! let uuids: Vec<Uuid> = ops.reachable_from(start).into_iter().collect();
//! let nodes = BatchLift::new(&uuids, &graph).lift_all();
//! ```

use std::collections::HashMap;
use uuid::Uuid;

use crate::lifting::{LiftedGraph, LiftedNode, LiftableDomain};

// ============================================================================
// LIFT BOUNDARY MARKER
// ============================================================================

/// Marker trait for types that represent a lift boundary.
///
/// This is a documentation/type-level marker indicating that lifting
/// occurs at this point in the architecture.
pub trait LiftBoundary {
    /// The domain type being lifted to
    type Target;

    /// Perform the lift operation.
    ///
    /// This is the explicit boundary crossing from abstract to concrete.
    fn cross_boundary(&self) -> Option<Self::Target>;
}

// ============================================================================
// DEFERRED LIFT
// ============================================================================

/// A deferred lift that delays lifting until explicitly requested.
///
/// This implements the "lazy Kan extension" pattern where we hold a
/// reference to the abstract identifier and only lift when domain
/// semantics are actually needed.
///
/// # Usage
///
/// ```rust,ignore
/// let deferred = DeferredLift::new(uuid, &graph);
///
/// // Graph operations work on the UUID
/// let reachable = ops.reachable_from(deferred.id());
///
/// // Lift only when we need domain data
/// if let Some(node) = deferred.lift() {
///     let vis = visualization_registry.fold(&node);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DeferredLift<'a> {
    /// The abstract identifier
    id: Uuid,

    /// Reference to the graph containing the node
    graph: &'a LiftedGraph,
}

impl<'a> DeferredLift<'a> {
    /// Create a new deferred lift.
    pub fn new(id: Uuid, graph: &'a LiftedGraph) -> Self {
        DeferredLift { id, graph }
    }

    /// Get the abstract identifier.
    #[inline]
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Lift to get the LiftedNode.
    ///
    /// This is the boundary crossing - domain semantics are now accessible.
    pub fn lift(&self) -> Option<&LiftedNode> {
        self.graph.find_node(self.id)
    }

    /// Lift and downcast to a specific domain type.
    ///
    /// Combines the lift boundary crossing with type recovery.
    pub fn lift_as<T: LiftableDomain>(&self) -> Option<T> {
        self.lift().and_then(T::unlift)
    }
}

impl<'a> LiftBoundary for DeferredLift<'a> {
    type Target = LiftedNode;

    fn cross_boundary(&self) -> Option<Self::Target> {
        self.lift().cloned()
    }
}

// ============================================================================
// BATCH LIFT
// ============================================================================

/// Batch lifting for efficient multi-node operations.
///
/// When you have a collection of UUIDs from graph operations and need
/// to lift them all, this provides an efficient batch operation.
///
/// # Usage
///
/// ```rust,ignore
/// let uuids: Vec<Uuid> = ops.reachable_from(start).into_iter().collect();
/// let batch = BatchLift::new(&uuids, &graph);
///
/// // Lift all at once
/// let nodes: Vec<&LiftedNode> = batch.lift_all();
///
/// // Or lift with filtering by type
/// let people: Vec<Person> = batch.lift_as::<Person>();
/// ```
pub struct BatchLift<'a> {
    /// The abstract identifiers
    ids: &'a [Uuid],

    /// Reference to the graph
    graph: &'a LiftedGraph,
}

impl<'a> BatchLift<'a> {
    /// Create a new batch lift operation.
    pub fn new(ids: &'a [Uuid], graph: &'a LiftedGraph) -> Self {
        BatchLift { ids, graph }
    }

    /// Get the count of IDs to lift.
    #[inline]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Lift all nodes.
    ///
    /// Returns only nodes that exist in the graph.
    pub fn lift_all(&self) -> Vec<&LiftedNode> {
        self.ids
            .iter()
            .filter_map(|id| self.graph.find_node(*id))
            .collect()
    }

    /// Lift all and clone.
    pub fn lift_all_owned(&self) -> Vec<LiftedNode> {
        self.ids
            .iter()
            .filter_map(|id| self.graph.find_node(*id).cloned())
            .collect()
    }

    /// Lift all nodes of a specific domain type.
    ///
    /// Performs both the lift and the unlift in one operation.
    pub fn lift_as<T: LiftableDomain>(&self) -> Vec<T> {
        self.ids
            .iter()
            .filter_map(|id| {
                self.graph.find_node(*id).and_then(T::unlift)
            })
            .collect()
    }

    /// Lift all nodes into a map keyed by UUID.
    pub fn lift_to_map(&self) -> HashMap<Uuid, &LiftedNode> {
        self.ids
            .iter()
            .filter_map(|id| {
                self.graph.find_node(*id).map(|node| (*id, node))
            })
            .collect()
    }
}

// ============================================================================
// LIFT RESULT
// ============================================================================

/// Result of a lift operation with boundary metadata.
///
/// This wrapper provides additional context about the lift operation
/// for debugging and tracing purposes.
#[derive(Debug, Clone)]
pub struct LiftResult<T> {
    /// The lifted value
    pub value: T,

    /// The original UUID
    pub source_id: Uuid,

    /// Whether this was a deferred lift
    pub was_deferred: bool,
}

impl<T> LiftResult<T> {
    /// Create a new lift result.
    pub fn new(value: T, source_id: Uuid, was_deferred: bool) -> Self {
        LiftResult {
            value,
            source_id,
            was_deferred,
        }
    }

    /// Map the inner value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> LiftResult<U> {
        LiftResult {
            value: f(self.value),
            source_id: self.source_id,
            was_deferred: self.was_deferred,
        }
    }

    /// Unwrap the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }
}

// ============================================================================
// LIFT GUARD
// ============================================================================

/// A guard that tracks whether lifting has occurred.
///
/// Use this in development/testing to verify that lifting only happens
/// at expected boundary points.
#[cfg(debug_assertions)]
#[derive(Debug, Default)]
pub struct LiftGuard {
    lift_count: std::sync::atomic::AtomicUsize,
    allowed_boundaries: Vec<&'static str>,
}

#[cfg(debug_assertions)]
impl LiftGuard {
    /// Create a new lift guard.
    pub fn new() -> Self {
        LiftGuard {
            lift_count: std::sync::atomic::AtomicUsize::new(0),
            allowed_boundaries: Vec::new(),
        }
    }

    /// Allow lifting at a named boundary.
    pub fn allow_boundary(mut self, name: &'static str) -> Self {
        self.allowed_boundaries.push(name);
        self
    }

    /// Record a lift operation.
    pub fn record_lift(&self, boundary: &str) {
        self.lift_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if !self.allowed_boundaries.iter().any(|&b| b == boundary) {
            eprintln!(
                "WARNING: Lift at unexpected boundary '{}'. Allowed: {:?}",
                boundary, self.allowed_boundaries
            );
        }
    }

    /// Get the total lift count.
    pub fn lift_count(&self) -> usize {
        self.lift_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Reset the counter.
    pub fn reset(&self) {
        self.lift_count.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}

// ============================================================================
// EXTENSION TRAITS
// ============================================================================

/// Extension trait for lifting from graph operation results.
pub trait LiftFromGraph {
    /// Lift a single UUID to its LiftedNode.
    fn lift_one(&self, id: Uuid) -> Option<&LiftedNode>;

    /// Lift multiple UUIDs to their LiftedNodes.
    fn lift_many<'a>(&'a self, ids: &[Uuid]) -> Vec<&'a LiftedNode>;

    /// Create a deferred lift for lazy evaluation.
    fn defer_lift(&self, id: Uuid) -> DeferredLift<'_>;
}

impl LiftFromGraph for LiftedGraph {
    fn lift_one(&self, id: Uuid) -> Option<&LiftedNode> {
        self.find_node(id)
    }

    fn lift_many<'a>(&'a self, ids: &[Uuid]) -> Vec<&'a LiftedNode> {
        ids.iter()
            .filter_map(|id| self.find_node(*id))
            .collect()
    }

    fn defer_lift(&self, id: Uuid) -> DeferredLift<'_> {
        DeferredLift::new(id, self)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, Person};
    use crate::domain::ids::BootstrapOrgId;
    use crate::lifting::LiftableDomain;
    use iced::Color;

    fn build_test_graph() -> (LiftedGraph, Uuid, Uuid) {
        let mut graph = LiftedGraph::new();

        let org = Organization::new("TestOrg", "Test Organization");
        let org_id = org.id.as_uuid();
        graph.add(&org);

        let org_id_for_person = BootstrapOrgId::new();
        let person = Person::new("Alice", "alice@example.com", org_id_for_person);
        let person_id = person.id.as_uuid();
        graph.add(&person);

        graph.connect(org_id, person_id, "employs", Color::WHITE);

        (graph, org_id, person_id)
    }

    #[test]
    fn test_deferred_lift() {
        let (graph, org_id, _) = build_test_graph();

        // Create deferred lift
        let deferred = DeferredLift::new(org_id, &graph);

        // ID is accessible without lifting
        assert_eq!(deferred.id(), org_id);

        // Lift at the boundary
        let node = deferred.lift();
        assert!(node.is_some());
    }

    #[test]
    fn test_deferred_lift_as() {
        let (graph, org_id, _) = build_test_graph();

        let deferred = DeferredLift::new(org_id, &graph);

        // Lift as specific type
        let org: Option<Organization> = deferred.lift_as();
        assert!(org.is_some());
        assert_eq!(org.unwrap().name, "TestOrg");
    }

    #[test]
    fn test_batch_lift() {
        let (graph, org_id, person_id) = build_test_graph();

        let ids = vec![org_id, person_id];
        let batch = BatchLift::new(&ids, &graph);

        assert_eq!(batch.len(), 2);

        let nodes = batch.lift_all();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_batch_lift_as() {
        let (graph, _, person_id) = build_test_graph();

        let ids = vec![person_id];
        let batch = BatchLift::new(&ids, &graph);

        let people: Vec<Person> = batch.lift_as();
        assert_eq!(people.len(), 1);
        assert_eq!(people[0].name, "Alice");
    }

    #[test]
    fn test_batch_lift_to_map() {
        let (graph, org_id, person_id) = build_test_graph();

        let ids = vec![org_id, person_id];
        let batch = BatchLift::new(&ids, &graph);

        let map = batch.lift_to_map();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&org_id));
        assert!(map.contains_key(&person_id));
    }

    #[test]
    fn test_lift_result() {
        let result = LiftResult::new("test", Uuid::now_v7(), false);

        assert_eq!(result.value, "test");
        assert!(!result.was_deferred);

        let mapped = result.map(|s| s.len());
        assert_eq!(mapped.value, 4);
    }

    #[test]
    fn test_lift_from_graph_extension() {
        let (graph, org_id, person_id) = build_test_graph();

        // lift_one
        let node = graph.lift_one(org_id);
        assert!(node.is_some());

        // lift_many
        let nodes = graph.lift_many(&[org_id, person_id]);
        assert_eq!(nodes.len(), 2);

        // defer_lift
        let deferred = graph.defer_lift(org_id);
        assert_eq!(deferred.id(), org_id);
    }

    #[test]
    fn test_nonexistent_lift() {
        let (graph, _, _) = build_test_graph();

        let fake_id = Uuid::now_v7();
        let deferred = DeferredLift::new(fake_id, &graph);

        assert!(deferred.lift().is_none());
    }

    #[test]
    fn test_wrong_type_lift_as() {
        let (graph, org_id, _) = build_test_graph();

        let deferred = DeferredLift::new(org_id, &graph);

        // Try to lift org as Person - should fail
        let person: Option<Person> = deferred.lift_as();
        assert!(person.is_none());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_lift_guard() {
        let guard = LiftGuard::new()
            .allow_boundary("visualization")
            .allow_boundary("detail_panel");

        assert_eq!(guard.lift_count(), 0);

        guard.record_lift("visualization");
        assert_eq!(guard.lift_count(), 1);

        guard.record_lift("detail_panel");
        assert_eq!(guard.lift_count(), 2);

        // This would print a warning but still count
        guard.record_lift("unauthorized");
        assert_eq!(guard.lift_count(), 3);

        guard.reset();
        assert_eq!(guard.lift_count(), 0);
    }
}
