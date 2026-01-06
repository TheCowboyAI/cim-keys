// Copyright (c) 2025 - Cowboy AI, LLC.

//! ValueObject Graph Contribution Traits
//!
//! This module defines traits and types for ValueObjects to contribute
//! Labels, Properties, and Relationships to graph Nodes (Entities/Aggregates).
//!
//! ## Key Distinction
//!
//! - **Entities** → Graph Nodes (via `LiftableDomain`)
//! - **ValueObjects** → Labels, Properties, Relationships ON Nodes (via `NodeContributor`)
//!
//! ## Design
//!
//! ValueObjects have NO graph identity but MAY have storage identity for persistence.
//! They contribute semantic information to their parent entity's graph node.
//!
//! ## Example
//!
//! ```ignore
//! impl NodeContributor for CertificateValidity {
//!     fn as_labels(&self) -> Vec<Label> {
//!         let mut labels = Vec::new();
//!         if self.is_expired() {
//!             labels.push(Label::new("Expired"));
//!         } else if self.expires_within_days(30) {
//!             labels.push(Label::new("ExpiringSoon"));
//!         }
//!         labels
//!     }
//!
//!     fn as_properties(&self) -> Vec<(PropertyKey, PropertyValue)> {
//!         vec![
//!             (PropertyKey::new("not_before"), PropertyValue::DateTime(self.not_before)),
//!             (PropertyKey::new("not_after"), PropertyValue::DateTime(self.not_after)),
//!         ]
//!     }
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

// ============================================================================
// Core Types
// ============================================================================

/// A label that can be applied to a graph node.
///
/// Labels are categorical tags that enable fast filtering and querying.
/// Examples: "CACertificate", "Expired", "WildcardCertificate", "Person"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Label(pub String);

impl Label {
    /// Create a new label
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the label as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Label {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Label {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A property key for graph node properties.
///
/// Property keys should use snake_case naming convention.
/// Examples: "subject_cn", "not_before", "key_usage_flags"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropertyKey(pub String);

impl PropertyKey {
    /// Create a new property key
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the key as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PropertyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for PropertyKey {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for PropertyKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A typed property value for graph node properties.
///
/// Supports common data types used in domain models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    /// String value
    String(String),
    /// Integer value (i64)
    Int(i64),
    /// Floating point value (f64)
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// DateTime value (UTC)
    DateTime(DateTime<Utc>),
    /// UUID value
    Uuid(Uuid),
    /// Raw bytes
    Bytes(Vec<u8>),
    /// List of property values
    List(Vec<PropertyValue>),
    /// Map of string to property values
    Map(HashMap<String, PropertyValue>),
    /// Null/absent value
    Null,
}

impl PropertyValue {
    /// Create a String property value
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    /// Create an Int property value
    pub fn int(i: i64) -> Self {
        Self::Int(i)
    }

    /// Create a Float property value
    pub fn float(f: f64) -> Self {
        Self::Float(f)
    }

    /// Create a Bool property value
    pub fn bool(b: bool) -> Self {
        Self::Bool(b)
    }

    /// Create a DateTime property value
    pub fn datetime(dt: DateTime<Utc>) -> Self {
        Self::DateTime(dt)
    }

    /// Create a Uuid property value
    pub fn uuid(id: Uuid) -> Self {
        Self::Uuid(id)
    }

    /// Create a Bytes property value
    pub fn bytes(data: Vec<u8>) -> Self {
        Self::Bytes(data)
    }

    /// Create a List property value
    pub fn list(items: Vec<PropertyValue>) -> Self {
        Self::List(items)
    }

    /// Create a List of strings
    pub fn string_list(items: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::List(items.into_iter().map(|s| Self::string(s)).collect())
    }

    /// Create a Map property value
    pub fn map(items: HashMap<String, PropertyValue>) -> Self {
        Self::Map(items)
    }

    /// Create a Null property value
    pub fn null() -> Self {
        Self::Null
    }

    /// Try to get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as int
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as datetime
    pub fn as_datetime(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::DateTime(dt) => Some(*dt),
            _ => None,
        }
    }

    /// Try to get as uuid
    pub fn as_uuid(&self) -> Option<Uuid> {
        match self {
            Self::Uuid(id) => Some(*id),
            _ => None,
        }
    }

    /// Check if this is a null value
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Int(i) => write!(f, "{}", i),
            Self::Float(fl) => write!(f, "{}", fl),
            Self::Bool(b) => write!(f, "{}", b),
            Self::DateTime(dt) => write!(f, "{}", dt.format("%Y-%m-%dT%H:%M:%SZ")),
            Self::Uuid(id) => write!(f, "{}", id),
            Self::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            Self::List(l) => write!(f, "[{} items]", l.len()),
            Self::Map(m) => write!(f, "{{{} entries}}", m.len()),
            Self::Null => write!(f, "null"),
        }
    }
}

/// A relationship from a ValueObject to another entity.
///
/// Used when a ValueObject references another entity by ID.
/// The relationship becomes an edge in the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueRelationship {
    /// Target entity ID
    pub target_id: Uuid,
    /// Relationship type (edge label)
    pub relationship_type: String,
    /// Additional properties on the relationship
    pub properties: HashMap<String, PropertyValue>,
}

impl ValueRelationship {
    /// Create a new relationship with no properties
    pub fn new(target_id: Uuid, relationship_type: impl Into<String>) -> Self {
        Self {
            target_id,
            relationship_type: relationship_type.into(),
            properties: HashMap::new(),
        }
    }

    /// Create a relationship with properties
    pub fn with_properties(
        target_id: Uuid,
        relationship_type: impl Into<String>,
        properties: HashMap<String, PropertyValue>,
    ) -> Self {
        Self {
            target_id,
            relationship_type: relationship_type.into(),
            properties,
        }
    }

    /// Add a property to the relationship
    pub fn with_property(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

// ============================================================================
// NodeContributor Trait
// ============================================================================

/// Trait for ValueObjects that contribute to graph node representation.
///
/// This trait enables ValueObjects to contribute:
/// - **Labels**: Categorical tags for filtering (e.g., "Expired", "CACertificate")
/// - **Properties**: Key-value pairs for node attributes
/// - **Relationships**: Edges to other entities
///
/// ## Design Philosophy
///
/// ValueObjects have NO graph identity of their own. They contribute to
/// their parent entity's graph node. This follows DDD principles where
/// ValueObjects are immutable and defined by their attributes, not identity.
///
/// ## Example Implementation
///
/// ```ignore
/// impl NodeContributor for KeyUsage {
///     fn as_labels(&self) -> Vec<Label> {
///         let mut labels = Vec::new();
///         if self.is_ca() {
///             labels.push(Label::new("CACertificate"));
///         }
///         if self.has_key_cert_sign() {
///             labels.push(Label::new("SigningCapable"));
///         }
///         labels
///     }
///
///     fn as_properties(&self) -> Vec<(PropertyKey, PropertyValue)> {
///         vec![
///             (PropertyKey::new("key_usage_bits"), PropertyValue::string(self.to_string())),
///             (PropertyKey::new("digital_signature"), PropertyValue::bool(self.digital_signature)),
///         ]
///     }
/// }
/// ```
pub trait NodeContributor {
    /// Contribute labels to the parent entity's graph node.
    ///
    /// Labels are categorical tags that enable fast filtering.
    /// Default implementation returns no labels.
    fn as_labels(&self) -> Vec<Label> {
        Vec::new()
    }

    /// Contribute properties to the parent entity's graph node.
    ///
    /// Properties are key-value pairs stored on the node.
    fn as_properties(&self) -> Vec<(PropertyKey, PropertyValue)>;

    /// Contribute relationships from the parent entity to other entities.
    ///
    /// The `parent_id` is the ID of the entity that owns this ValueObject.
    /// Default implementation returns no relationships.
    fn as_relationships(&self, _parent_id: Uuid) -> Vec<ValueRelationship> {
        Vec::new()
    }

    /// Optional storage identity for persistence (NOT graph identity).
    ///
    /// Some ValueObjects may need a stable ID for storage/caching purposes,
    /// but this does not make them entities - they still have no domain identity.
    fn storage_id(&self) -> Option<Uuid> {
        None
    }
}

// ============================================================================
// Aggregate Trait
// ============================================================================

/// Trait for entities that aggregate ValueObject contributions.
///
/// Entities implement this to collect labels, properties, and relationships
/// from their composed ValueObjects into a unified graph representation.
pub trait AggregateContributions {
    /// Aggregate labels from all composed ValueObjects.
    fn aggregate_labels(&self) -> Vec<Label>;

    /// Aggregate properties from all composed ValueObjects.
    fn aggregate_properties(&self) -> Vec<(PropertyKey, PropertyValue)>;

    /// Aggregate relationships from all composed ValueObjects.
    fn aggregate_relationships(&self) -> Vec<ValueRelationship>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_creation() {
        let label = Label::new("TestLabel");
        assert_eq!(label.as_str(), "TestLabel");
        assert_eq!(label.to_string(), "TestLabel");
    }

    #[test]
    fn test_label_from_str() {
        let label: Label = "FromStr".into();
        assert_eq!(label.as_str(), "FromStr");
    }

    #[test]
    fn test_property_key_creation() {
        let key = PropertyKey::new("test_key");
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn test_property_value_string() {
        let val = PropertyValue::string("hello");
        assert_eq!(val.as_string(), Some("hello"));
        assert_eq!(val.as_int(), None);
    }

    #[test]
    fn test_property_value_int() {
        let val = PropertyValue::int(42);
        assert_eq!(val.as_int(), Some(42));
        assert_eq!(val.as_string(), None);
    }

    #[test]
    fn test_property_value_bool() {
        let val = PropertyValue::bool(true);
        assert_eq!(val.as_bool(), Some(true));
    }

    #[test]
    fn test_property_value_datetime() {
        let now = Utc::now();
        let val = PropertyValue::datetime(now);
        assert_eq!(val.as_datetime(), Some(now));
    }

    #[test]
    fn test_property_value_uuid() {
        let id = Uuid::now_v7();
        let val = PropertyValue::uuid(id);
        assert_eq!(val.as_uuid(), Some(id));
    }

    #[test]
    fn test_property_value_null() {
        let val = PropertyValue::null();
        assert!(val.is_null());
    }

    #[test]
    fn test_property_value_string_list() {
        let val = PropertyValue::string_list(vec!["a", "b", "c"]);
        match val {
            PropertyValue::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_string(), Some("a"));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_value_relationship_creation() {
        let target = Uuid::now_v7();
        let rel = ValueRelationship::new(target, "uses_key");
        assert_eq!(rel.target_id, target);
        assert_eq!(rel.relationship_type, "uses_key");
        assert!(rel.properties.is_empty());
    }

    #[test]
    fn test_value_relationship_with_property() {
        let target = Uuid::now_v7();
        let rel = ValueRelationship::new(target, "signed_by")
            .with_property("trust_level", PropertyValue::string("complete"));
        assert_eq!(rel.properties.len(), 1);
        assert_eq!(
            rel.properties.get("trust_level").and_then(|v| v.as_string()),
            Some("complete")
        );
    }

    // Test a simple NodeContributor implementation
    struct TestValueObject {
        name: String,
        is_active: bool,
    }

    impl NodeContributor for TestValueObject {
        fn as_properties(&self) -> Vec<(PropertyKey, PropertyValue)> {
            vec![
                (PropertyKey::new("name"), PropertyValue::string(&self.name)),
                (PropertyKey::new("is_active"), PropertyValue::bool(self.is_active)),
            ]
        }

        fn as_labels(&self) -> Vec<Label> {
            if self.is_active {
                vec![Label::new("Active")]
            } else {
                vec![Label::new("Inactive")]
            }
        }
    }

    #[test]
    fn test_node_contributor_impl() {
        let obj = TestValueObject {
            name: "Test".to_string(),
            is_active: true,
        };

        let labels = obj.as_labels();
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].as_str(), "Active");

        let props = obj.as_properties();
        assert_eq!(props.len(), 2);

        let rels = obj.as_relationships(Uuid::now_v7());
        assert!(rels.is_empty());
    }
}
