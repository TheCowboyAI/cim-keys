// Copyright (c) 2025 - Cowboy AI, LLC.

//! Graph Projection Layer - Materializes Graph State from Events
//!
//! This module provides the projection layer between the domain and UI layers.
//! It materializes the current graph state from the event history and provides
//! derived view data for the UI layer.
//!
//! ## DDD Architecture
//!
//! ```text
//! Events → GraphProjection → NodeView/EdgeView → UI Rendering
//!            ↑                     ↑
//!       (apply events)       (derive visuals)
//! ```
//!
//! ## Responsibilities
//!
//! - Materialize current graph state from domain events
//! - Derive visualization properties from domain data
//! - Provide indexes for efficient querying
//! - Cache computed properties for performance

use iced::Color;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::graph::{DomainRelation, RelationType, RelationCategory, GraphDomainEvent};
use super::view_model::{EdgeView, EdgeStyle};
use super::domain_node::{DomainNode, Injection};

/// Graph projection - materializes current state from events
///
/// This is the read model for the organizational graph. It maintains:
/// - Current entity set (nodes)
/// - Current relation set (edges)
/// - Derived indexes for efficient querying
#[derive(Debug, Clone, Default)]
pub struct GraphProjection {
    /// Entities in the graph (node ID → domain node)
    entities: HashMap<Uuid, DomainNode>,

    /// Relations in the graph (relation ID → domain relation)
    relations: HashMap<Uuid, DomainRelation>,

    /// Index: from entity → list of outgoing relations
    outgoing_relations: HashMap<Uuid, Vec<Uuid>>,

    /// Index: to entity → list of incoming relations
    incoming_relations: HashMap<Uuid, Vec<Uuid>>,

    /// Index: entity type → list of entity IDs
    entities_by_type: HashMap<Injection, Vec<Uuid>>,

    /// Index: relation category → list of relation IDs
    relations_by_category: HashMap<RelationCategory, Vec<Uuid>>,

    /// Version number (incremented on each event)
    version: u64,

    /// Last update timestamp
    last_updated: Option<DateTime<Utc>>,
}

impl GraphProjection {
    /// Create a new empty projection
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a graph domain event to update the projection
    pub fn apply(&mut self, event: &GraphDomainEvent) {
        match event {
            GraphDomainEvent::EntityAdded { entity_id, entity_type, .. } => {
                // Note: The actual DomainNode would need to be passed separately
                // or the event would need to include it. For now, we track the ID.
                tracing::debug!("Entity added: {} ({})", entity_id, entity_type);
            }

            GraphDomainEvent::EntityRemoved { entity_id, .. } => {
                if let Some(node) = self.entities.remove(entity_id) {
                    // Remove from type index
                    let injection = node.injection();
                    if let Some(ids) = self.entities_by_type.get_mut(&injection) {
                        ids.retain(|id| id != entity_id);
                    }

                    // Remove all relations involving this entity
                    let relations_to_remove: Vec<Uuid> = self.relations
                        .iter()
                        .filter(|(_, r)| r.from == *entity_id || r.to == *entity_id)
                        .map(|(id, _)| *id)
                        .collect();

                    for rel_id in relations_to_remove {
                        self.remove_relation(&rel_id);
                    }
                }
            }

            GraphDomainEvent::RelationEstablished {
                relation_id,
                from,
                to,
                relation_type,
                established_at,
                expires_at,
                metadata,
                ..
            } => {
                let relation = DomainRelation {
                    id: *relation_id,
                    from: *from,
                    to: *to,
                    relation_type: relation_type.clone(),
                    established_at: *established_at,
                    expires_at: *expires_at,
                    metadata: metadata.clone(),
                };

                self.add_relation(relation);
            }

            GraphDomainEvent::RelationDissolved { relation_id, .. } => {
                self.remove_relation(relation_id);
            }

            GraphDomainEvent::RelationModified { .. } => {
                // Handle relation modifications
            }

            GraphDomainEvent::GraphRestructured { .. } => {
                // Handle graph restructuring
            }

            GraphDomainEvent::SubgraphMerged { .. } => {
                // Handle subgraph merging
            }
        }

        self.version += 1;
        self.last_updated = Some(event.timestamp());
    }

    /// Add an entity to the projection
    pub fn add_entity(&mut self, id: Uuid, node: DomainNode) {
        let injection = node.injection();
        self.entities.insert(id, node);
        self.entities_by_type
            .entry(injection)
            .or_default()
            .push(id);
        self.version += 1;
        self.last_updated = Some(Utc::now());
    }

    /// Add a relation to the projection
    fn add_relation(&mut self, relation: DomainRelation) {
        let id = relation.id;
        let from = relation.from;
        let to = relation.to;
        let category = relation.category();

        self.relations.insert(id, relation);
        self.outgoing_relations.entry(from).or_default().push(id);
        self.incoming_relations.entry(to).or_default().push(id);
        self.relations_by_category.entry(category).or_default().push(id);
    }

    /// Remove a relation from the projection
    fn remove_relation(&mut self, relation_id: &Uuid) {
        if let Some(relation) = self.relations.remove(relation_id) {
            // Remove from outgoing index
            if let Some(outgoing) = self.outgoing_relations.get_mut(&relation.from) {
                outgoing.retain(|id| id != relation_id);
            }

            // Remove from incoming index
            if let Some(incoming) = self.incoming_relations.get_mut(&relation.to) {
                incoming.retain(|id| id != relation_id);
            }

            // Remove from category index
            let category = relation.category();
            if let Some(by_cat) = self.relations_by_category.get_mut(&category) {
                by_cat.retain(|id| id != relation_id);
            }
        }
    }

    /// Get all entities
    pub fn entities(&self) -> impl Iterator<Item = (&Uuid, &DomainNode)> {
        self.entities.iter()
    }

    /// Get all relations
    pub fn relations(&self) -> impl Iterator<Item = (&Uuid, &DomainRelation)> {
        self.relations.iter()
    }

    /// Get entity by ID
    pub fn get_entity(&self, id: &Uuid) -> Option<&DomainNode> {
        self.entities.get(id)
    }

    /// Get relation by ID
    pub fn get_relation(&self, id: &Uuid) -> Option<&DomainRelation> {
        self.relations.get(id)
    }

    /// Get outgoing relations from an entity
    pub fn outgoing_from(&self, entity_id: &Uuid) -> Vec<&DomainRelation> {
        self.outgoing_relations
            .get(entity_id)
            .map(|ids| ids.iter().filter_map(|id| self.relations.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get incoming relations to an entity
    pub fn incoming_to(&self, entity_id: &Uuid) -> Vec<&DomainRelation> {
        self.incoming_relations
            .get(entity_id)
            .map(|ids| ids.iter().filter_map(|id| self.relations.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get entities by type
    pub fn entities_of_type(&self, injection: Injection) -> Vec<&DomainNode> {
        self.entities_by_type
            .get(&injection)
            .map(|ids| ids.iter().filter_map(|id| self.entities.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get relations by category
    pub fn relations_in_category(&self, category: RelationCategory) -> Vec<&DomainRelation> {
        self.relations_by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.relations.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get the current version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get the last update timestamp
    pub fn last_updated(&self) -> Option<DateTime<Utc>> {
        self.last_updated
    }

    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get relation count
    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }
}

// ============================================================================
// VIEW DERIVATION - Convert domain data to UI views
// ============================================================================

/// Derive edge color from relation category
///
/// This function maps domain relation categories to UI colors.
/// The mapping is done here in the projection layer to maintain
/// separation of concerns.
pub fn derive_edge_color(category: RelationCategory) -> Color {
    match category {
        RelationCategory::Organizational => Color::from_rgb(0.5, 0.5, 0.6),  // Gray
        RelationCategory::KeyManagement => Color::from_rgb(0.2, 0.6, 0.8),   // Cyan
        RelationCategory::Policy => Color::from_rgb(0.6, 0.3, 0.8),          // Purple
        RelationCategory::Trust => Color::from_rgb(0.8, 0.6, 0.2),           // Gold
        RelationCategory::Nats => Color::from_rgb(0.3, 0.7, 0.4),            // Green
        RelationCategory::Pki => Color::from_rgb(0.8, 0.2, 0.2),             // Red
        RelationCategory::YubiKey => Color::from_rgb(0.2, 0.4, 0.8),         // Blue
    }
}

/// Derive edge style from relation type
pub fn derive_edge_style(relation_type: &RelationType) -> EdgeStyle {
    if relation_type.is_temporal() {
        EdgeStyle::Dashed
    } else if relation_type.is_bidirectional() {
        EdgeStyle::Dotted
    } else {
        EdgeStyle::Solid
    }
}

/// Create an EdgeView from a DomainRelation
pub fn create_edge_view(relation: &DomainRelation) -> EdgeView {
    let color = derive_edge_color(relation.category());
    let style = derive_edge_style(&relation.relation_type);
    let label = Some(relation.relation_type.label().to_string());

    EdgeView {
        from_id: relation.from,
        to_id: relation.to,
        color,
        style,
        label,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_creation() {
        let projection = GraphProjection::new();
        assert_eq!(projection.entity_count(), 0);
        assert_eq!(projection.relation_count(), 0);
        assert_eq!(projection.version(), 0);
    }

    #[test]
    fn test_relation_established_event() {
        let mut projection = GraphProjection::new();

        let from = Uuid::now_v7();
        let to = Uuid::now_v7();

        let event = GraphDomainEvent::relation_established(
            from,
            to,
            RelationType::MemberOf,
        );

        projection.apply(&event);

        assert_eq!(projection.relation_count(), 1);
        assert_eq!(projection.version(), 1);
    }

    #[test]
    fn test_edge_color_derivation() {
        assert_eq!(
            derive_edge_color(RelationCategory::Organizational),
            Color::from_rgb(0.5, 0.5, 0.6)
        );
        assert_eq!(
            derive_edge_color(RelationCategory::Pki),
            Color::from_rgb(0.8, 0.2, 0.2)
        );
    }

    #[test]
    fn test_edge_style_derivation() {
        let temporal = RelationType::HasRole {
            valid_from: Utc::now(),
            valid_until: None,
        };
        assert_eq!(derive_edge_style(&temporal), EdgeStyle::Dashed);

        let bidirectional = RelationType::IncompatibleWith;
        assert_eq!(derive_edge_style(&bidirectional), EdgeStyle::Dotted);

        let normal = RelationType::ParentChild;
        assert_eq!(derive_edge_style(&normal), EdgeStyle::Solid);
    }

    #[test]
    fn test_create_edge_view() {
        let relation = DomainRelation::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            RelationType::Signs,
        );

        let view = create_edge_view(&relation);

        assert_eq!(view.from_id, relation.from);
        assert_eq!(view.to_id, relation.to);
        assert_eq!(view.label, Some("signs".to_string()));
    }
}
