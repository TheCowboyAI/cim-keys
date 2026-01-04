// Copyright (c) 2025 - Cowboy AI, LLC.

//! Neo4j Port - Interface for graph database operations
//!
//! This defines the interface for Neo4j operations that our domain uses.
//! The actual implementation (neo4rs, file export, mock) is separate.
//!
//! ## Architecture
//!
//! ```text
//! Domain Layer (Projections)
//!     ↓
//! Port Interface (Neo4jPort trait)
//!     ↓
//! Adapter (neo4rs, file, mock)
//! ```
//!
//! The projection system generates `CypherBatch` which is then executed
//! through this port.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ============================================================================
// PORT TRAIT
// ============================================================================

/// Port for Neo4j graph database operations
///
/// This is the interface that our domain uses for graph persistence.
/// Implementations could be:
/// - Neo4rs adapter (real Neo4j connection)
/// - File adapter (writes .cypher files)
/// - Mock adapter (for testing)
#[async_trait]
pub trait Neo4jPort: Send + Sync {
    /// Execute a batch of Cypher queries atomically
    async fn execute_batch(&self, batch: &CypherBatch) -> Result<ExecutionResult, Neo4jError>;

    /// Execute a single Cypher query
    async fn execute(&self, query: &CypherQuery) -> Result<QueryResult, Neo4jError>;

    /// Check connection health
    async fn health_check(&self) -> Result<(), Neo4jError>;

    /// Get database info
    async fn database_info(&self) -> Result<DatabaseInfo, Neo4jError>;

    /// Begin a transaction
    async fn begin_transaction(&self) -> Result<Box<dyn Neo4jTransaction>, Neo4jError>;
}

/// Transaction interface for multi-query operations
#[async_trait]
pub trait Neo4jTransaction: Send + Sync {
    /// Execute a query within this transaction
    async fn execute(&mut self, query: &CypherQuery) -> Result<QueryResult, Neo4jError>;

    /// Commit the transaction
    async fn commit(self: Box<Self>) -> Result<(), Neo4jError>;

    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> Result<(), Neo4jError>;
}

// ============================================================================
// QUERY TYPES (shared with projection module)
// ============================================================================

/// A Cypher query with parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CypherQuery {
    /// The Cypher query string
    pub query: String,
    /// Named parameters for the query
    pub parameters: HashMap<String, CypherValue>,
}

impl CypherQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            parameters: HashMap::new(),
        }
    }

    pub fn with_param(mut self, name: impl Into<String>, value: impl Into<CypherValue>) -> Self {
        self.parameters.insert(name.into(), value.into());
        self
    }

    /// Format the query with parameters inlined (for .cypher files)
    pub fn to_inline_cypher(&self) -> String {
        let mut result = self.query.clone();
        for (name, value) in &self.parameters {
            let placeholder = format!("${}", name);
            let replacement = value.to_cypher_literal();
            result = result.replace(&placeholder, &replacement);
        }
        result
    }
}

/// Values that can be used in Cypher queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CypherValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<CypherValue>),
    Map(HashMap<String, CypherValue>),
    DateTime(String), // ISO 8601 format
}

impl CypherValue {
    pub fn to_cypher_literal(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Self::Int(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::String(s) => format!("'{}'", s.replace('\'', "\\'")),
            Self::List(items) => {
                let items_str: Vec<String> = items.iter().map(|v| v.to_cypher_literal()).collect();
                format!("[{}]", items_str.join(", "))
            }
            Self::Map(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_cypher_literal()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Self::DateTime(dt) => format!("datetime('{}')", dt),
        }
    }
}

// Convenient conversions
impl From<&str> for CypherValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for CypherValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<i64> for CypherValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<i32> for CypherValue {
    fn from(i: i32) -> Self {
        Self::Int(i as i64)
    }
}

impl From<bool> for CypherValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f64> for CypherValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

/// A batch of Cypher queries for atomic execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CypherBatch {
    /// Individual queries in execution order
    pub queries: Vec<CypherQuery>,
    /// Optional batch metadata
    pub metadata: Option<BatchMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub description: String,
}

impl CypherBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metadata(mut self, source: &str, description: &str) -> Self {
        self.metadata = Some(BatchMetadata {
            created_at: chrono::Utc::now(),
            source: source.to_string(),
            description: description.to_string(),
        });
        self
    }

    pub fn add_query(&mut self, query: CypherQuery) {
        self.queries.push(query);
    }

    pub fn add(&mut self, query: impl Into<String>) {
        self.queries.push(CypherQuery::new(query));
    }

    /// Convert to a single .cypher file content
    pub fn to_cypher_file(&self) -> String {
        let mut lines = Vec::new();

        // Header comment
        if let Some(meta) = &self.metadata {
            lines.push(format!("// Generated: {}", meta.created_at.to_rfc3339()));
            lines.push(format!("// Source: {}", meta.source));
            lines.push(format!("// Description: {}", meta.description));
            lines.push(String::new());
        }

        // Queries
        for query in &self.queries {
            lines.push(query.to_inline_cypher());
            lines.push(String::new());
        }

        lines.join("\n")
    }

    /// Merge another batch into this one
    pub fn merge(&mut self, other: CypherBatch) {
        self.queries.extend(other.queries);
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }

    /// Get the number of queries
    pub fn len(&self) -> usize {
        self.queries.len()
    }
}

// ============================================================================
// RESULT TYPES
// ============================================================================

/// Result of executing a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub queries_executed: usize,
    pub nodes_created: usize,
    pub nodes_deleted: usize,
    pub relationships_created: usize,
    pub relationships_deleted: usize,
    pub properties_set: usize,
    pub execution_time_ms: u64,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            queries_executed: 0,
            nodes_created: 0,
            nodes_deleted: 0,
            relationships_created: 0,
            relationships_deleted: 0,
            properties_set: 0,
            execution_time_ms: 0,
        }
    }
}

impl ExecutionResult {
    /// Merge statistics from another result
    pub fn merge(&mut self, other: &ExecutionResult) {
        self.queries_executed += other.queries_executed;
        self.nodes_created += other.nodes_created;
        self.nodes_deleted += other.nodes_deleted;
        self.relationships_created += other.relationships_created;
        self.relationships_deleted += other.relationships_deleted;
        self.properties_set += other.properties_set;
        self.execution_time_ms += other.execution_time_ms;
    }
}

/// Result of a single query
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows_affected: usize,
    pub columns: Vec<String>,
    pub records: Vec<Record>,
}

/// A single record from a query result
#[derive(Debug, Clone)]
pub struct Record {
    pub values: HashMap<String, CypherValue>,
}

impl Record {
    pub fn get(&self, key: &str) -> Option<&CypherValue> {
        self.values.get(key)
    }
}

/// Database info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub node_count: Option<u64>,
    pub relationship_count: Option<u64>,
}

// ============================================================================
// ERRORS
// ============================================================================

/// Neo4j-specific errors
#[derive(Debug, Error)]
pub enum Neo4jError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Query syntax error: {0}")]
    SyntaxError(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Database not found: {0}")]
    DatabaseNotFound(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Backend error: {0}")]
    BackendError(String),
}

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Neo4j connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    pub uri: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub database: Option<String>,
    pub max_connections: Option<u32>,
    pub connection_timeout_ms: Option<u64>,
}

impl Neo4jConfig {
    pub fn new(uri: impl Into<String>, username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            username: username.into(),
            password: password.into(),
            database: None,
            max_connections: None,
            connection_timeout_ms: None,
        }
    }

    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.connection_timeout_ms = Some(timeout_ms);
        self
    }
}

// ============================================================================
// GRAPH DATA TYPES (for domain → graph projection)
// ============================================================================

/// A domain entity ready for graph projection
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: uuid::Uuid,
    pub label: String,
    pub properties: HashMap<String, CypherValue>,
}

impl GraphNode {
    pub fn new(id: uuid::Uuid, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<CypherValue>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Generate MERGE query for this node
    pub fn to_merge_query(&self) -> CypherQuery {
        let props: Vec<String> = self
            .properties
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.to_cypher_literal()))
            .collect();

        let props_str = if props.is_empty() {
            String::new()
        } else {
            format!(", {}", props.join(", "))
        };

        CypherQuery::new(format!(
            "MERGE (n:{} {{id: '{}'{}}})",
            self.label, self.id, props_str
        ))
    }
}

/// A relationship between two nodes
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from_id: uuid::Uuid,
    pub to_id: uuid::Uuid,
    pub relationship_type: String,
    pub properties: HashMap<String, CypherValue>,
}

impl GraphEdge {
    pub fn new(from_id: uuid::Uuid, to_id: uuid::Uuid, relationship_type: impl Into<String>) -> Self {
        Self {
            from_id,
            to_id,
            relationship_type: relationship_type.into(),
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<CypherValue>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Generate MERGE query for this relationship
    pub fn to_merge_query(&self) -> CypherQuery {
        let props: Vec<String> = self
            .properties
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.to_cypher_literal()))
            .collect();

        let props_str = if props.is_empty() {
            String::new()
        } else {
            format!(" {{{}}}", props.join(", "))
        };

        CypherQuery::new(format!(
            "MATCH (a {{id: '{}'}}), (b {{id: '{}'}}) MERGE (a)-[:{}{}]->(b)",
            self.from_id, self.to_id, self.relationship_type, props_str
        ))
    }
}

/// A complete domain graph ready for projection
#[derive(Debug, Clone, Default)]
pub struct DomainGraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl DomainGraphData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    /// Generate complete Cypher batch for this graph
    pub fn to_cypher_batch(&self) -> CypherBatch {
        let mut batch = CypherBatch::new();

        // Create nodes first (MERGE)
        for node in &self.nodes {
            batch.add_query(node.to_merge_query());
        }

        // Then relationships (MERGE)
        for edge in &self.edges {
            batch.add_query(edge.to_merge_query());
        }

        batch
    }
}

/// Trait for types that can be projected to graph nodes
pub trait ToGraphNode {
    fn to_graph_node(&self) -> GraphNode;
}

/// Trait for relationships that can be projected to graph edges
pub trait ToGraphEdge {
    fn to_graph_edge(&self) -> GraphEdge;
}

// GraphNode implements ToGraphNode as identity
impl ToGraphNode for GraphNode {
    fn to_graph_node(&self) -> GraphNode {
        self.clone()
    }
}

// GraphEdge implements ToGraphEdge as identity
impl ToGraphEdge for GraphEdge {
    fn to_graph_edge(&self) -> GraphEdge {
        self.clone()
    }
}
