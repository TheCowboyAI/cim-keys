// Copyright (c) 2025 - Cowboy AI, LLC.

//! Graph Domain Events - Events for Graph State Changes
//!
//! These events represent changes to the organizational graph structure.
//! They are used by the projection layer to materialize current state.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::relations::{RelationType, RelationMetadata};

/// Events that modify the graph structure.
///
/// These are domain events, not GUI events. They represent meaningful
/// business changes to the organizational graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphDomainEvent {
    // Node events
    /// An entity was added to the graph
    EntityAdded {
        entity_id: Uuid,
        entity_type: String,
        added_at: DateTime<Utc>,
        added_by: Option<Uuid>,
    },

    /// An entity was removed from the graph
    EntityRemoved {
        entity_id: Uuid,
        removed_at: DateTime<Utc>,
        removed_by: Option<Uuid>,
        reason: Option<String>,
    },

    // Relation events
    /// A relation was established between entities
    RelationEstablished {
        relation_id: Uuid,
        from: Uuid,
        to: Uuid,
        relation_type: RelationType,
        established_at: DateTime<Utc>,
        established_by: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
        metadata: RelationMetadata,
    },

    /// A relation was dissolved
    RelationDissolved {
        relation_id: Uuid,
        dissolved_at: DateTime<Utc>,
        dissolved_by: Option<Uuid>,
        reason: Option<String>,
    },

    /// A relation was modified (e.g., expiration extended)
    RelationModified {
        relation_id: Uuid,
        field_name: String,
        old_value: String,
        new_value: String,
        modified_at: DateTime<Utc>,
        modified_by: Option<Uuid>,
    },

    // Structural events
    /// The graph was restructured (e.g., organizational restructuring)
    GraphRestructured {
        restructured_at: DateTime<Utc>,
        initiated_by: Option<Uuid>,
        description: String,
        affected_entities: Vec<Uuid>,
        affected_relations: Vec<Uuid>,
    },

    /// A subgraph was merged into this graph
    SubgraphMerged {
        source_graph_id: Uuid,
        merged_at: DateTime<Utc>,
        merged_by: Option<Uuid>,
        entity_mapping: Vec<(Uuid, Uuid)>, // (source_id, target_id)
    },
}

impl GraphDomainEvent {
    /// Get the timestamp of this event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::EntityAdded { added_at, .. } => *added_at,
            Self::EntityRemoved { removed_at, .. } => *removed_at,
            Self::RelationEstablished { established_at, .. } => *established_at,
            Self::RelationDissolved { dissolved_at, .. } => *dissolved_at,
            Self::RelationModified { modified_at, .. } => *modified_at,
            Self::GraphRestructured { restructured_at, .. } => *restructured_at,
            Self::SubgraphMerged { merged_at, .. } => *merged_at,
        }
    }

    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::EntityAdded { .. } => "EntityAdded",
            Self::EntityRemoved { .. } => "EntityRemoved",
            Self::RelationEstablished { .. } => "RelationEstablished",
            Self::RelationDissolved { .. } => "RelationDissolved",
            Self::RelationModified { .. } => "RelationModified",
            Self::GraphRestructured { .. } => "GraphRestructured",
            Self::SubgraphMerged { .. } => "SubgraphMerged",
        }
    }

    /// Get affected entity IDs
    pub fn affected_entities(&self) -> Vec<Uuid> {
        match self {
            Self::EntityAdded { entity_id, .. } => vec![*entity_id],
            Self::EntityRemoved { entity_id, .. } => vec![*entity_id],
            Self::RelationEstablished { from, to, .. } => vec![*from, *to],
            Self::RelationDissolved { relation_id, .. } => vec![*relation_id],
            Self::RelationModified { relation_id, .. } => vec![*relation_id],
            Self::GraphRestructured { affected_entities, .. } => affected_entities.clone(),
            Self::SubgraphMerged { entity_mapping, .. } => {
                entity_mapping.iter().flat_map(|(s, t)| vec![*s, *t]).collect()
            }
        }
    }
}

/// Factory methods for creating graph events
impl GraphDomainEvent {
    /// Create an EntityAdded event
    pub fn entity_added(entity_id: Uuid, entity_type: impl Into<String>) -> Self {
        Self::EntityAdded {
            entity_id,
            entity_type: entity_type.into(),
            added_at: Utc::now(),
            added_by: None,
        }
    }

    /// Create an EntityRemoved event
    pub fn entity_removed(entity_id: Uuid) -> Self {
        Self::EntityRemoved {
            entity_id,
            removed_at: Utc::now(),
            removed_by: None,
            reason: None,
        }
    }

    /// Create a RelationEstablished event
    pub fn relation_established(
        from: Uuid,
        to: Uuid,
        relation_type: RelationType,
    ) -> Self {
        Self::RelationEstablished {
            relation_id: Uuid::now_v7(),
            from,
            to,
            relation_type,
            established_at: Utc::now(),
            established_by: None,
            expires_at: None,
            metadata: RelationMetadata::default(),
        }
    }

    /// Create a RelationDissolved event
    pub fn relation_dissolved(relation_id: Uuid) -> Self {
        Self::RelationDissolved {
            relation_id,
            dissolved_at: Utc::now(),
            dissolved_by: None,
            reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let entity_id = Uuid::now_v7();
        let event = GraphDomainEvent::entity_added(entity_id, "Person");

        assert_eq!(event.event_type(), "EntityAdded");
        assert_eq!(event.affected_entities(), vec![entity_id]);
    }

    #[test]
    fn test_relation_event() {
        let from = Uuid::now_v7();
        let to = Uuid::now_v7();
        let event = GraphDomainEvent::relation_established(
            from,
            to,
            RelationType::MemberOf,
        );

        assert_eq!(event.event_type(), "RelationEstablished");
        let affected = event.affected_entities();
        assert!(affected.contains(&from));
        assert!(affected.contains(&to));
    }
}
