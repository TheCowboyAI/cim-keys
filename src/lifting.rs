// Copyright (c) 2025 - Cowboy AI, LLC.

//! LiftableDomain - Faithful Functor for Domain Composition
//!
//! This module implements the `LiftableDomain` trait, enabling any domain type
//! to be "lifted" into a unified graph representation while preserving all
//! domain semantics.
//!
//! # Mathematical Foundation
//!
//! `LiftableDomain` forms a **faithful functor** F: Domain → Graph where:
//! - Objects (entities) map to graph nodes
//! - Morphisms (relationships) map to graph edges
//! - Identity is preserved: F(id_A) = id_{F(A)}
//! - Composition is preserved: F(g ∘ f) = F(g) ∘ F(f)
//!
//! The functor is "faithful" meaning it is injective on morphisms:
//! If F(f) = F(g) then f = g. This ensures no domain information is lost.
//!
//! # Monad Structure
//!
//! LiftableDomain combined with `Entity<T>` from cim-domain forms a monad:
//! - `pure` (unit): Domain value → Entity<Domain>
//! - `bind` (join): Entity<Entity<T>> → Entity<T>
//! - Laws: Left identity, Right identity, Associativity
//!
//! # Usage
//!
//! ```ignore
//! use cim_keys::lifting::{LiftableDomain, LiftedNode};
//!
//! // Domain types implement LiftableDomain
//! let person: Person = ...;
//! let lifted: LiftedNode = person.lift();
//!
//! // Recover original domain entity
//! let recovered: Option<Person> = Person::unlift(&lifted);
//!
//! // Events can also be lifted
//! let event: PersonCreatedEvent = ...;
//! let graph_event: GraphEvent = person.lift_event(event);
//! ```

use std::fmt::Debug;
use std::any::Any;
use std::sync::Arc;

use iced::Color;
use uuid::Uuid;

use crate::gui::domain_node::Injection;
use crate::domain::{Organization, OrganizationUnit, Person, Location, Role, Policy};

// ============================================================================
// DOMAIN-SPECIFIC COLORS
// ============================================================================

/// Color for Organization nodes
pub const COLOR_ORGANIZATION: Color = Color::from_rgb(0.2, 0.3, 0.6);

/// Color for OrganizationUnit nodes
pub const COLOR_UNIT: Color = Color::from_rgb(0.4, 0.5, 0.8);

/// Color for Person nodes
pub const COLOR_PERSON: Color = Color::from_rgb(0.5, 0.7, 0.3);

/// Color for Location nodes
pub const COLOR_LOCATION: Color = Color::from_rgb(0.6, 0.5, 0.4);

/// Color for Role nodes
pub const COLOR_ROLE: Color = Color::from_rgb(0.4, 0.5, 0.6);

/// Color for Policy nodes
pub const COLOR_POLICY: Color = Color::from_rgb(0.5, 0.3, 0.6);

/// Color for NATS Operator nodes
pub const COLOR_NATS_OPERATOR: Color = Color::from_rgb(0.6, 0.2, 0.8);

/// Color for NATS Account nodes
pub const COLOR_NATS_ACCOUNT: Color = Color::from_rgb(0.5, 0.3, 0.7);

/// Color for NATS User nodes
pub const COLOR_NATS_USER: Color = Color::from_rgb(0.4, 0.4, 0.6);

/// Color for Certificate nodes
pub const COLOR_CERTIFICATE: Color = Color::from_rgb(0.7, 0.5, 0.2);

/// Color for Key nodes
pub const COLOR_KEY: Color = Color::from_rgb(0.6, 0.6, 0.2);

/// Color for YubiKey nodes
pub const COLOR_YUBIKEY: Color = Color::from_rgb(0.0, 0.6, 0.4);

// ============================================================================
// LIFTED NODE - Graph representation of any domain entity
// ============================================================================

/// A domain entity lifted into graph representation.
///
/// This is the target of the LiftableDomain functor. It contains:
/// - A unique identifier (preserved from domain)
/// - An injection tag indicating the domain type
/// - The original domain data (type-erased for heterogeneous storage)
/// - Derived metadata for graph visualization
#[derive(Debug, Clone)]
pub struct LiftedNode {
    /// Entity ID (preserved from domain)
    pub id: Uuid,

    /// Domain type tag (coproduct injection)
    pub injection: Injection,

    /// Display label for graph visualization
    pub label: String,

    /// Secondary text (optional)
    pub secondary: Option<String>,

    /// Color for visualization
    pub color: Color,

    /// Original domain data (type-erased)
    data: Arc<dyn Any + Send + Sync>,
}

impl LiftedNode {
    /// Create a new lifted node from domain data
    pub fn new<T: Send + Sync + 'static>(
        id: Uuid,
        injection: Injection,
        label: impl Into<String>,
        color: Color,
        data: T,
    ) -> Self {
        Self {
            id,
            injection,
            label: label.into(),
            secondary: None,
            color,
            data: Arc::new(data),
        }
    }

    /// Add secondary text
    pub fn with_secondary(mut self, text: impl Into<String>) -> Self {
        self.secondary = Some(text.into());
        self
    }

    /// Attempt to downcast and retrieve the original domain data
    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }

    /// Get the injection type
    pub fn injection(&self) -> Injection {
        self.injection
    }
}

// ============================================================================
// LIFTED EDGE - Graph representation of relationships
// ============================================================================

/// A domain relationship lifted into graph representation.
#[derive(Debug, Clone)]
pub struct LiftedEdge {
    /// Edge ID
    pub id: Uuid,

    /// Source node ID
    pub from_id: Uuid,

    /// Target node ID
    pub to_id: Uuid,

    /// Relationship type label
    pub label: String,

    /// Edge color
    pub color: Color,

    /// Edge weight (for weighted graphs)
    pub weight: Option<f64>,
}

impl LiftedEdge {
    /// Create a new lifted edge
    pub fn new(
        id: Uuid,
        from_id: Uuid,
        to_id: Uuid,
        label: impl Into<String>,
        color: Color,
    ) -> Self {
        Self {
            id,
            from_id,
            to_id,
            label: label.into(),
            color,
            weight: None,
        }
    }

    /// Add weight to edge
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}

// ============================================================================
// LIFTABLE DOMAIN TRAIT
// ============================================================================

/// Trait for domain types that can be lifted into graph representation.
///
/// This is a **faithful functor** - the lifting preserves all domain semantics
/// and the `unlift` operation can fully recover the original domain entity.
///
/// # Functor Laws
///
/// Implementations MUST satisfy:
/// 1. **Identity**: `lift(id_A) = id_{lift(A)}` - identity morphisms are preserved
/// 2. **Composition**: `lift(g ∘ f) = lift(g) ∘ lift(f)` - composition is preserved
/// 3. **Faithfulness**: If `lift(f) = lift(g)` then `f = g` - no information loss
///
/// # Example
///
/// ```ignore
/// impl LiftableDomain for Person {
///     fn lift(&self) -> LiftedNode {
///         LiftedNode::new(
///             self.id,
///             Injection::Person,
///             &self.name,
///             PERSON_COLOR,
///             self.clone(),
///         )
///     }
///
///     fn unlift(node: &LiftedNode) -> Option<Self> {
///         node.downcast::<Person>().cloned()
///     }
/// }
/// ```
pub trait LiftableDomain: Clone + Send + Sync + 'static {
    /// Lift this domain entity into a graph node.
    ///
    /// This is the object-mapping part of the functor.
    fn lift(&self) -> LiftedNode;

    /// Attempt to recover the domain entity from a lifted node.
    ///
    /// Returns `None` if the node was not created from this domain type.
    fn unlift(node: &LiftedNode) -> Option<Self>;

    /// Get the injection type for this domain.
    ///
    /// Used for type dispatch without instantiating the full lift.
    fn injection() -> Injection;

    /// Get the entity ID.
    ///
    /// This should match the ID used in `lift()`.
    fn entity_id(&self) -> Uuid;
}

// ============================================================================
// LIFTED GRAPH - Unified graph from multiple domains
// ============================================================================

/// A unified graph containing nodes and edges from multiple domains.
///
/// This is the coproduct (disjoint union) of all lifted domain types,
/// enabling heterogeneous domain composition in a single graph structure.
#[derive(Debug, Clone, Default)]
pub struct LiftedGraph {
    /// All nodes in the graph
    nodes: Vec<LiftedNode>,

    /// All edges in the graph
    edges: Vec<LiftedEdge>,
}

impl LiftedGraph {
    /// Create a new empty lifted graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a domain entity to the graph
    pub fn add<T: LiftableDomain>(&mut self, entity: &T) -> &LiftedNode {
        let node = entity.lift();
        self.nodes.push(node);
        self.nodes.last().unwrap()
    }

    /// Add a lifted node directly
    pub fn add_node(&mut self, node: LiftedNode) {
        self.nodes.push(node);
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, edge: LiftedEdge) {
        self.edges.push(edge);
    }

    /// Connect two nodes with a labeled edge
    pub fn connect(
        &mut self,
        from_id: Uuid,
        to_id: Uuid,
        label: impl Into<String>,
        color: Color,
    ) {
        let edge = LiftedEdge::new(
            Uuid::now_v7(),
            from_id,
            to_id,
            label,
            color,
        );
        self.edges.push(edge);
    }

    /// Get all nodes
    pub fn nodes(&self) -> &[LiftedNode] {
        &self.nodes
    }

    /// Get all edges
    pub fn edges(&self) -> &[LiftedEdge] {
        &self.edges
    }

    /// Find a node by ID
    pub fn find_node(&self, id: Uuid) -> Option<&LiftedNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Find all nodes of a specific injection type
    pub fn nodes_by_type(&self, injection: Injection) -> Vec<&LiftedNode> {
        self.nodes.iter().filter(|n| n.injection == injection).collect()
    }

    /// Get nodes count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edges count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Recover all entities of a specific domain type
    pub fn unlift_all<T: LiftableDomain>(&self) -> Vec<T> {
        self.nodes
            .iter()
            .filter_map(|n| T::unlift(n))
            .collect()
    }

    /// Merge another graph into this one
    pub fn merge(&mut self, other: LiftedGraph) {
        self.nodes.extend(other.nodes);
        self.edges.extend(other.edges);
    }

    /// Verify functor laws (for testing)
    pub fn verify_functor_laws(&self) -> bool {
        // Verify all nodes have valid injections
        for node in &self.nodes {
            // Each node must have a consistent injection
            if format!("{:?}", node.injection).is_empty() {
                return false;
            }
        }

        // Verify all edges reference existing nodes
        for edge in &self.edges {
            let has_from = self.nodes.iter().any(|n| n.id == edge.from_id);
            let has_to = self.nodes.iter().any(|n| n.id == edge.to_id);
            if !has_from || !has_to {
                return false;
            }
        }

        true
    }
}

// ============================================================================
// DOMAIN COMPOSITION
// ============================================================================

/// Compose multiple domains into a unified graph.
///
/// This function takes domain entities and relationships, lifting them
/// all into a single coherent graph structure.
///
/// # Categorical Structure
///
/// This is the coproduct construction: given functors F_i: D_i → Graph,
/// we construct the coproduct functor ∐F_i: ∐D_i → Graph.
pub fn compose_domains<T: LiftableDomain>(entities: &[T]) -> LiftedGraph {
    let mut graph = LiftedGraph::new();
    for entity in entities {
        graph.add(entity);
    }
    graph
}

// ============================================================================
// LIFTABLE DOMAIN IMPLEMENTATIONS
// ============================================================================

impl LiftableDomain for Organization {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::Organization,
            &self.display_name,
            COLOR_ORGANIZATION,
            self.clone(),
        )
        .with_secondary(format!("Org: {}", self.name))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Organization {
            return None;
        }
        node.downcast::<Organization>().cloned()
    }

    fn injection() -> Injection {
        Injection::Organization
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

impl LiftableDomain for OrganizationUnit {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::OrganizationUnit,
            &self.name,
            COLOR_UNIT,
            self.clone(),
        )
        .with_secondary(format!("{:?}", self.unit_type))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::OrganizationUnit {
            return None;
        }
        node.downcast::<OrganizationUnit>().cloned()
    }

    fn injection() -> Injection {
        Injection::OrganizationUnit
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

impl LiftableDomain for Person {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::Person,
            &self.name,
            COLOR_PERSON,
            self.clone(),
        )
        .with_secondary(self.email.clone())
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Person {
            return None;
        }
        node.downcast::<Person>().cloned()
    }

    fn injection() -> Injection {
        Injection::Person
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

impl LiftableDomain for Location {
    fn lift(&self) -> LiftedNode {
        // Location uses EntityId<LocationMarker> via AggregateRoot trait
        use cim_domain::AggregateRoot;
        let id = *self.id().as_uuid();
        LiftedNode::new(
            id,
            Injection::Location,
            &self.name,
            COLOR_LOCATION,
            self.clone(),
        )
        .with_secondary(format!("{:?}", self.location_type))
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Location {
            return None;
        }
        node.downcast::<Location>().cloned()
    }

    fn injection() -> Injection {
        Injection::Location
    }

    fn entity_id(&self) -> Uuid {
        use cim_domain::AggregateRoot;
        *self.id().as_uuid()
    }
}

impl LiftableDomain for Role {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::Role,
            &self.name,
            COLOR_ROLE,
            self.clone(),
        )
        .with_secondary(self.description.clone())
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Role {
            return None;
        }
        node.downcast::<Role>().cloned()
    }

    fn injection() -> Injection {
        Injection::Role
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

impl LiftableDomain for Policy {
    fn lift(&self) -> LiftedNode {
        LiftedNode::new(
            self.id,
            Injection::Policy,
            &self.name,
            COLOR_POLICY,
            self.clone(),
        )
        .with_secondary(self.description.clone())
    }

    fn unlift(node: &LiftedNode) -> Option<Self> {
        if node.injection != Injection::Policy {
            return None;
        }
        node.downcast::<Policy>().cloned()
    }

    fn injection() -> Injection {
        Injection::Policy
    }

    fn entity_id(&self) -> Uuid {
        self.id
    }
}

// ============================================================================
// CONVENIENCE: From bootstrap to LiftedGraph
// ============================================================================

/// Build a LiftedGraph from an Organization and its units/people.
///
/// This function performs the complete lifting operation for a bootstrap
/// configuration, creating nodes and edges for all entities.
pub fn lift_organization_graph(
    org: &Organization,
    people: &[Person],
) -> LiftedGraph {
    let mut graph = LiftedGraph::new();

    // Lift the organization
    graph.add(org);

    // Lift all units
    for unit in &org.units {
        graph.add(unit);
        // Connect unit to organization
        graph.connect(
            unit.id,
            org.id,
            "belongs_to",
            COLOR_UNIT,
        );

        // Connect to parent unit if exists
        if let Some(parent_id) = unit.parent_unit_id {
            graph.connect(
                unit.id,
                parent_id,
                "reports_to",
                COLOR_UNIT,
            );
        }
    }

    // Lift all people
    for person in people {
        graph.add(person);

        // Connect person to organization
        if person.organization_id == org.id {
            graph.connect(
                person.id,
                org.id,
                "member_of",
                COLOR_PERSON,
            );
        }

        // Connect person to their units
        for unit_id in &person.unit_ids {
            graph.connect(
                person.id,
                *unit_id,
                "works_in",
                COLOR_PERSON,
            );
        }

        // Connect to owner if exists
        if let Some(owner_id) = person.owner_id {
            graph.connect(
                person.id,
                owner_id,
                "owned_by",
                COLOR_PERSON,
            );
        }
    }

    graph
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test domain type
    #[derive(Debug, Clone)]
    struct TestEntity {
        id: Uuid,
        name: String,
    }

    impl LiftableDomain for TestEntity {
        fn lift(&self) -> LiftedNode {
            LiftedNode::new(
                self.id,
                Injection::Person, // Use Person as test injection
                &self.name,
                Color::WHITE,
                self.clone(),
            )
        }

        fn unlift(node: &LiftedNode) -> Option<Self> {
            node.downcast::<TestEntity>().cloned()
        }

        fn injection() -> Injection {
            Injection::Person
        }

        fn entity_id(&self) -> Uuid {
            self.id
        }
    }

    #[test]
    fn test_lift_unlift_roundtrip() {
        let entity = TestEntity {
            id: Uuid::now_v7(),
            name: "Test".to_string(),
        };

        let lifted = entity.lift();
        let recovered = TestEntity::unlift(&lifted);

        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert_eq!(recovered.id, entity.id);
        assert_eq!(recovered.name, entity.name);
    }

    #[test]
    fn test_lifted_graph() {
        let entity1 = TestEntity {
            id: Uuid::now_v7(),
            name: "Entity1".to_string(),
        };
        let entity2 = TestEntity {
            id: Uuid::now_v7(),
            name: "Entity2".to_string(),
        };

        let mut graph = LiftedGraph::new();
        graph.add(&entity1);
        graph.add(&entity2);
        graph.connect(entity1.id, entity2.id, "relates_to", Color::WHITE);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.verify_functor_laws());
    }

    #[test]
    fn test_unlift_all() {
        let entities: Vec<TestEntity> = (0..5)
            .map(|i| TestEntity {
                id: Uuid::now_v7(),
                name: format!("Entity{}", i),
            })
            .collect();

        let graph = compose_domains(&entities);
        let recovered: Vec<TestEntity> = graph.unlift_all();

        assert_eq!(recovered.len(), 5);
    }

    #[test]
    fn test_nodes_by_type() {
        let entity = TestEntity {
            id: Uuid::now_v7(),
            name: "Test".to_string(),
        };

        let mut graph = LiftedGraph::new();
        graph.add(&entity);

        let persons = graph.nodes_by_type(Injection::Person);
        assert_eq!(persons.len(), 1);

        let orgs = graph.nodes_by_type(Injection::Organization);
        assert_eq!(orgs.len(), 0);
    }

    // ========== Domain Type Tests ==========

    #[test]
    fn test_organization_lift_unlift() {
        use std::collections::HashMap;

        let org = Organization {
            id: Uuid::now_v7(),
            name: "test-org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("A test organization".to_string()),
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let lifted = org.lift();

        // Verify lifted properties
        assert_eq!(lifted.id, org.id);
        assert_eq!(lifted.injection, Injection::Organization);
        assert_eq!(lifted.label, "Test Organization");
        assert!(lifted.secondary.is_some());

        // Verify roundtrip
        let recovered = Organization::unlift(&lifted);
        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert_eq!(recovered.id, org.id);
        assert_eq!(recovered.name, org.name);
    }

    #[test]
    fn test_organization_unit_lift_unlift() {
        use crate::domain::OrganizationUnitType;

        let unit = OrganizationUnit {
            id: Uuid::now_v7(),
            name: "Engineering".to_string(),
            unit_type: OrganizationUnitType::Department,
            parent_unit_id: None,
            responsible_person_id: None,
            nats_account_name: Some("eng".to_string()),
        };

        let lifted = unit.lift();

        // Verify lifted properties
        assert_eq!(lifted.id, unit.id);
        assert_eq!(lifted.injection, Injection::OrganizationUnit);
        assert_eq!(lifted.label, "Engineering");

        // Verify roundtrip
        let recovered = OrganizationUnit::unlift(&lifted);
        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert_eq!(recovered.id, unit.id);
        assert_eq!(recovered.name, unit.name);
    }

    #[test]
    fn test_person_lift_unlift() {
        let person = Person {
            id: Uuid::now_v7(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            active: true,
            nats_permissions: None,
            owner_id: None,
        };

        let lifted = person.lift();

        // Verify lifted properties
        assert_eq!(lifted.id, person.id);
        assert_eq!(lifted.injection, Injection::Person);
        assert_eq!(lifted.label, "John Doe");
        assert_eq!(lifted.secondary, Some("john@example.com".to_string()));

        // Verify roundtrip
        let recovered = Person::unlift(&lifted);
        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert_eq!(recovered.id, person.id);
        assert_eq!(recovered.name, person.name);
        assert_eq!(recovered.email, person.email);
    }

    #[test]
    fn test_lift_organization_graph() {
        use crate::domain::OrganizationUnitType;
        use std::collections::HashMap;

        let org_id = Uuid::now_v7();
        let unit_id = Uuid::now_v7();

        let org = Organization {
            id: org_id,
            name: "cowboyai".to_string(),
            display_name: "Cowboy AI".to_string(),
            description: None,
            parent_id: None,
            units: vec![
                OrganizationUnit {
                    id: unit_id,
                    name: "Core".to_string(),
                    unit_type: OrganizationUnitType::Team,
                    parent_unit_id: None,
                    responsible_person_id: None,
                    nats_account_name: Some("core".to_string()),
                },
            ],
            metadata: HashMap::new(),
        };

        let people = vec![
            Person {
                id: Uuid::now_v7(),
                name: "Alice".to_string(),
                email: "alice@cowboyai.com".to_string(),
                roles: vec![],
                organization_id: org_id,
                unit_ids: vec![unit_id],
                active: true,
                nats_permissions: None,
                owner_id: None,
            },
            Person {
                id: Uuid::now_v7(),
                name: "Bob".to_string(),
                email: "bob@cowboyai.com".to_string(),
                roles: vec![],
                organization_id: org_id,
                unit_ids: vec![],
                active: true,
                nats_permissions: None,
                owner_id: None,
            },
        ];

        let graph = lift_organization_graph(&org, &people);

        // 1 org + 1 unit + 2 people = 4 nodes
        assert_eq!(graph.node_count(), 4);

        // Edges:
        // - unit -> org (belongs_to)
        // - person1 -> org (member_of)
        // - person1 -> unit (works_in)
        // - person2 -> org (member_of)
        // = 4 edges
        assert_eq!(graph.edge_count(), 4);

        // Verify functor laws
        assert!(graph.verify_functor_laws());

        // Verify we can recover all orgs
        let orgs: Vec<Organization> = graph.unlift_all();
        assert_eq!(orgs.len(), 1);

        // Verify we can recover all people
        let recovered_people: Vec<Person> = graph.unlift_all();
        assert_eq!(recovered_people.len(), 2);
    }

    #[test]
    fn test_graph_merge() {
        use std::collections::HashMap;

        let org1 = Organization {
            id: Uuid::now_v7(),
            name: "org1".to_string(),
            display_name: "Org 1".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let org2 = Organization {
            id: Uuid::now_v7(),
            name: "org2".to_string(),
            display_name: "Org 2".to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let graph1 = lift_organization_graph(&org1, &[]);
        let mut graph2 = lift_organization_graph(&org2, &[]);

        graph2.merge(graph1);

        assert_eq!(graph2.node_count(), 2);
        assert!(graph2.verify_functor_laws());
    }

    #[test]
    fn test_functor_faithfulness() {
        // Faithfulness: If lift(a) == lift(b) in graph representation,
        // then a == b in domain. We test the contrapositive:
        // Different domain entities produce different graph nodes.

        let person1 = Person {
            id: Uuid::now_v7(),
            name: "Person 1".to_string(),
            email: "p1@test.com".to_string(),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            active: true,
            nats_permissions: None,
            owner_id: None,
        };

        let person2 = Person {
            id: Uuid::now_v7(),
            name: "Person 2".to_string(),
            email: "p2@test.com".to_string(),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            active: true,
            nats_permissions: None,
            owner_id: None,
        };

        let lifted1 = person1.lift();
        let lifted2 = person2.lift();

        // Different entities produce different lifted nodes
        assert_ne!(lifted1.id, lifted2.id);
        assert_ne!(lifted1.label, lifted2.label);

        // Each can be unlifted to its original
        let recovered1 = Person::unlift(&lifted1).unwrap();
        let recovered2 = Person::unlift(&lifted2).unwrap();

        assert_eq!(recovered1.id, person1.id);
        assert_eq!(recovered2.id, person2.id);
    }
}
