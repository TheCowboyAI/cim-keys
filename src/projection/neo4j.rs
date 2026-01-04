// Copyright (c) 2025 - Cowboy AI, LLC.

//! # Neo4j Graph Projection
//!
//! Composable projections for domain elements → Neo4j graph database.
//!
//! ## Architecture (Ports & Adapters)
//!
//! ```text
//! Domain Layer (Pure Projections)
//!     ↓ generates
//! CypherBatch (data structure)
//!     ↓ via
//! Neo4jPort (interface in ports/)
//!     ↓ implemented by
//! Adapter (neo4rs, file, mock)
//! ```
//!
//! The projections in this module are **pure transformations**:
//! - `DomainGraphData → CypherBatch` (graph_to_cypher)
//! - `CypherBatch → String` (cypher_to_file)
//!
//! Actual I/O execution happens through the `Neo4jPort` trait.
//!
//! ## Usage
//!
//! ```text
//! // Build domain graph
//! let mut graph = DomainGraphData::new();
//! graph.add_node(person.to_graph_node());
//! graph.add_edge(relationship.to_graph_edge());
//!
//! // Project to Cypher (pure, no I/O)
//! let cypher = graph_to_cypher().project(graph)?;
//!
//! // Option 1: Save to .cypher file (via storage port)
//! let content = cypher_to_file().project(cypher)?;
//! storage_port.write("export.cypher", content.as_bytes()).await?;
//!
//! // Option 2: Execute against Neo4j (via neo4j port)
//! let result = neo4j_port.execute_batch(&cypher).await?;
//! ```

use crate::projection::{Projection, ProjectionError};
use crate::ports::neo4j::{
    CypherBatch, CypherValue, DomainGraphData, GraphNode, GraphEdge,
    ToGraphNode, ToGraphEdge,
};
use chrono::Utc;
use uuid::Uuid;

// ============================================================================
// PURE PROJECTIONS (no I/O)
// ============================================================================

/// Projection: DomainGraphData → CypherBatch
///
/// This is a pure transformation - no I/O occurs.
/// The resulting CypherBatch can be:
/// - Saved to a .cypher file (via storage port)
/// - Executed against Neo4j (via neo4j port)
pub struct GraphToCypherProjection {
    source: &'static str,
    description: &'static str,
}

impl Default for GraphToCypherProjection {
    fn default() -> Self {
        Self {
            source: "cim-keys",
            description: "Domain graph projection",
        }
    }
}

impl GraphToCypherProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metadata(mut self, source: &'static str, description: &'static str) -> Self {
        self.source = source;
        self.description = description;
        self
    }
}

impl Projection<DomainGraphData, CypherBatch, ProjectionError> for GraphToCypherProjection {
    fn project(&self, input: DomainGraphData) -> Result<CypherBatch, ProjectionError> {
        let mut batch = input.to_cypher_batch();
        batch = batch.with_metadata(self.source, self.description);
        Ok(batch)
    }

    fn name(&self) -> &'static str {
        "GraphToCypher"
    }
}

/// Projection: CypherBatch → String (file content)
///
/// Pure transformation to .cypher file format.
pub struct CypherToFileProjection;

impl Projection<CypherBatch, String, ProjectionError> for CypherToFileProjection {
    fn project(&self, input: CypherBatch) -> Result<String, ProjectionError> {
        Ok(input.to_cypher_file())
    }

    fn name(&self) -> &'static str {
        "CypherToFile"
    }
}

/// Projection: Vec<T: ToGraphNode> → DomainGraphData
///
/// Collect any collection of domain entities into a graph.
pub struct CollectToGraphProjection<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for CollectToGraphProjection<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: ToGraphNode + Send + Sync> Projection<Vec<T>, DomainGraphData, ProjectionError>
    for CollectToGraphProjection<T>
{
    fn project(&self, input: Vec<T>) -> Result<DomainGraphData, ProjectionError> {
        let mut graph = DomainGraphData::new();
        for item in input {
            graph.add_node(item.to_graph_node());
        }
        Ok(graph)
    }

    fn name(&self) -> &'static str {
        "CollectToGraph"
    }
}

// ============================================================================
// DOMAIN TYPE IMPLEMENTATIONS
// ============================================================================

// Implement ToGraphNode for domain types
impl ToGraphNode for super::domain::PersonOutput {
    fn to_graph_node(&self) -> GraphNode {
        GraphNode::new(self.id, "Person")
            .with_property("name", self.name.clone())
            .with_property("email", self.email.clone())
            .with_property("created_at", CypherValue::DateTime(self.created_at.to_rfc3339()))
    }
}

impl ToGraphNode for super::domain::LocationOutput {
    fn to_graph_node(&self) -> GraphNode {
        let mut node = GraphNode::new(self.id, "Location")
            .with_property("name", self.name.clone())
            .with_property("type", format!("{:?}", self.location_type))
            .with_property("created_at", CypherValue::DateTime(self.created_at.to_rfc3339()));

        if let Some(url) = &self.virtual_url {
            node = node.with_property("virtual_url", url.clone());
        }

        node
    }
}

// Implement for KeyInfo
impl ToGraphNode for crate::domain::pki::KeyInfo {
    fn to_graph_node(&self) -> GraphNode {
        let mut node = GraphNode::new(self.id.as_uuid(), "CryptographicKey")
            .with_property("algorithm", format!("{:?}", self.algorithm))
            .with_property("purpose", format!("{:?}", self.purpose))
            .with_property("created_at", CypherValue::DateTime(self.created_at.to_rfc3339()));

        if let Some(serial) = &self.yubikey_serial {
            node = node.with_property("yubikey_serial", serial.clone());
        }
        if let Some(slot) = &self.piv_slot {
            node = node.with_property("piv_slot", slot.clone());
        }

        node
    }
}

// Implement for CertificateInfo
impl ToGraphNode for crate::domain::pki::CertificateInfo {
    fn to_graph_node(&self) -> GraphNode {
        GraphNode::new(self.id.as_uuid(), "Certificate")
            .with_property("type", format!("{:?}", self.cert_type))
            .with_property("subject", self.subject.clone())
            .with_property("status", format!("{:?}", self.status))
            .with_property("not_before", CypherValue::DateTime(self.not_before.to_rfc3339()))
            .with_property("not_after", CypherValue::DateTime(self.not_after.to_rfc3339()))
    }
}

// ============================================================================
// RELATIONSHIP PROJECTIONS
// ============================================================================

/// Helper to create ownership edges
pub fn person_owns_key(person_id: Uuid, key_id: Uuid) -> GraphEdge {
    GraphEdge::new(person_id, key_id, "OWNS_KEY")
        .with_property("created_at", CypherValue::DateTime(Utc::now().to_rfc3339()))
}

/// Helper to create certificate signing edges
pub fn certificate_signs(issuer_id: Uuid, subject_id: Uuid) -> GraphEdge {
    GraphEdge::new(issuer_id, subject_id, "SIGNS")
}

/// Helper to create organization membership edges
pub fn belongs_to_org(person_id: Uuid, org_id: Uuid) -> GraphEdge {
    GraphEdge::new(person_id, org_id, "BELONGS_TO")
}

/// Helper to create location assignment edges
pub fn located_at(entity_id: Uuid, location_id: Uuid) -> GraphEdge {
    GraphEdge::new(entity_id, location_id, "LOCATED_AT")
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/// Create a graph-to-cypher projection
pub fn graph_to_cypher() -> GraphToCypherProjection {
    GraphToCypherProjection::new()
}

/// Create a cypher-to-file projection
pub fn cypher_to_file() -> CypherToFileProjection {
    CypherToFileProjection
}

/// Create a collection-to-graph projection
pub fn collect_to_graph<T: ToGraphNode + Send + Sync>() -> CollectToGraphProjection<T> {
    CollectToGraphProjection::default()
}

/// Compose: DomainGraphData → String (complete pipeline to .cypher file)
pub fn domain_to_cypher_file() -> impl Projection<DomainGraphData, String, ProjectionError> {
    graph_to_cypher().then(cypher_to_file())
}

// ============================================================================
// BUILDER FOR COMPLEX GRAPHS
// ============================================================================

/// Builder for constructing domain graphs from heterogeneous collections
pub struct DomainGraphBuilder {
    graph: DomainGraphData,
}

impl Default for DomainGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainGraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DomainGraphData::new(),
        }
    }

    /// Add any entity that implements ToGraphNode
    pub fn add<T: ToGraphNode>(mut self, entity: &T) -> Self {
        self.graph.add_node(entity.to_graph_node());
        self
    }

    /// Add multiple entities
    pub fn add_all<T: ToGraphNode>(mut self, entities: &[T]) -> Self {
        for entity in entities {
            self.graph.add_node(entity.to_graph_node());
        }
        self
    }

    /// Add any edge that implements ToGraphEdge
    pub fn add_edge<T: ToGraphEdge>(mut self, edge: &T) -> Self {
        self.graph.add_edge(edge.to_graph_edge());
        self
    }

    /// Add a relationship between two nodes
    pub fn relate(mut self, from: Uuid, to: Uuid, relationship: &str) -> Self {
        self.graph.add_edge(GraphEdge::new(from, to, relationship));
        self
    }

    /// Build the final graph
    pub fn build(self) -> DomainGraphData {
        self.graph
    }

    /// Project directly to CypherBatch
    pub fn to_cypher(self) -> Result<CypherBatch, ProjectionError> {
        graph_to_cypher().project(self.graph)
    }

    /// Project directly to .cypher file content
    pub fn to_cypher_file(self) -> Result<String, ProjectionError> {
        domain_to_cypher_file().project(self.graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::neo4j::{CypherBatch, GraphNode};

    #[test]
    fn test_graph_to_cypher_projection() {
        let mut graph = DomainGraphData::new();
        graph.add_node(GraphNode::new(Uuid::now_v7(), "Person")
            .with_property("name", "Alice"));

        let proj = graph_to_cypher();
        let result = proj.project(graph);
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.queries.len(), 1);
        assert!(batch.metadata.is_some());
    }

    #[test]
    fn test_cypher_to_file_projection() {
        let mut batch = CypherBatch::new().with_metadata("test", "Test batch");
        batch.add("CREATE (n:Test {name: 'test'})");

        let proj = cypher_to_file();
        let result = proj.project(batch);
        assert!(result.is_ok());

        let file_content = result.unwrap();
        assert!(file_content.contains("// Generated:"));
        assert!(file_content.contains("CREATE (n:Test"));
    }

    #[test]
    fn test_composed_projection() {
        let mut graph = DomainGraphData::new();
        graph.add_node(GraphNode::new(Uuid::now_v7(), "Person")
            .with_property("name", "Alice"));

        let proj = domain_to_cypher_file();
        let result = proj.project(graph);
        assert!(result.is_ok());

        let file_content = result.unwrap();
        assert!(file_content.contains("MERGE"));
        assert!(file_content.contains(":Person"));
    }

    #[test]
    fn test_domain_graph_builder() {
        let person_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        let cypher = DomainGraphBuilder::new()
            .add(&GraphNode::new(person_id, "Person").with_property("name", "Alice"))
            .add(&GraphNode::new(org_id, "Organization").with_property("name", "CowboyAI"))
            .relate(person_id, org_id, "BELONGS_TO")
            .to_cypher();

        assert!(cypher.is_ok());
        let batch = cypher.unwrap();
        assert_eq!(batch.queries.len(), 3); // 2 nodes + 1 edge
    }

    #[test]
    fn test_relationship_helpers() {
        let person_id = Uuid::now_v7();
        let key_id = Uuid::now_v7();

        let edge = person_owns_key(person_id, key_id);
        assert_eq!(edge.relationship_type, "OWNS_KEY");
        assert_eq!(edge.from_id, person_id);
        assert_eq!(edge.to_id, key_id);
    }
}
