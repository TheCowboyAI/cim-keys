//! Graph Projection Layer
//!
//! Functorial lifting of domain events into graph visualizations.
//!
//! ## Architecture
//!
//! This module provides the projection layer that lifts domain events from
//! bounded contexts (Person, Organization) and conceptual spaces (Role, PKI)
//! into cim-graph's event-driven graph model for visualization.
//!
//! ### Category Theory Foundation
//!
//! ```text
//! Domain Events (Category D)
//!        ↓ Functor F
//! Graph Events (Category G)
//!        ↓ Projection P
//! Graph Visualization
//! ```
//!
//! The functor F preserves:
//! - Identity: F(id_A) = id_F(A)
//! - Composition: F(g ∘ f) = F(g) ∘ F(f)
//!
//! ## Event Mapping
//!
//! ### Context Events (Aggregates)
//! - PersonEvent → ContextPayload (Person context)
//! - OrganizationEvent → ContextPayload (Organization/Unit context)
//!
//! ### Concept Events (Semantic Spaces)
//! - Role → ConceptPayload (Role concept)
//! - PKI → ConceptPayload (Certificate/Trust concept)
//!
//! ## Usage
//!
//! ```rust
//! use cim_keys::graph_projection::GraphProjector;
//! use cim_domain_person::events::PersonEvent;
//!
//! let projector = GraphProjector::new();
//!
//! // Lift domain event into graph event
//! let person_event = PersonEvent::PersonCreated(person_created);
//! let graph_events = projector.lift_person_event(&person_event)?;
//!
//! // Project into visualization
//! let graph = projector.project(graph_events)?;
//! ```

use uuid::Uuid;
use chrono::{DateTime, Utc};
use cim_graph::{
    events::{GraphEvent, EventPayload},
    core::GraphProjection,
};

#[cfg(feature = "policy")]
use cim_domain_person::events::PersonEvent;
#[cfg(feature = "policy")]
use cim_domain_organization::events::OrganizationEvent;

/// Graph projector for lifting domain events into graph visualizations
pub struct GraphProjector {
    /// Correlation ID for tracking related events
    correlation_id: Uuid,
}

impl GraphProjector {
    /// Create a new graph projector
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::now_v7(),
        }
    }

    /// Lift a Person domain event into graph events (functor)
    ///
    /// This is a functorial mapping from the Person context to the graph context.
    /// It preserves event structure and maintains causation/correlation tracking.
    #[cfg(feature = "policy")]
    pub fn lift_person_event(&self, event: &PersonEvent) -> Result<Vec<GraphEvent>, ProjectionError> {
        match event {
            PersonEvent::PersonCreated(created) => {
                // Map PersonCreated to ContextPayload
                // Person is a Context (bounded context with aggregates)
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: created.person_id.into_inner(), // Extract UUID from EntityId
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(cim_graph::events::ContextPayload::ContextDefined {
                        context_id: created.person_id.into_inner(),
                        name: format!("Person: {}", created.name.display_name()),
                        bounded_context_type: "Person".to_string(),
                    }),
                }])
            }
            PersonEvent::NameUpdated(updated) => {
                // TODO: Map to EntityAdded or PropertySet in Context
                Ok(vec![])
            }
            _ => {
                // TODO: Map other PersonEvent variants
                Ok(vec![])
            }
        }
    }

    /// Lift an Organization domain event into graph events (functor)
    #[cfg(feature = "policy")]
    pub fn lift_organization_event(&self, _event: &OrganizationEvent) -> Result<Vec<GraphEvent>, ProjectionError> {
        // TODO: Implement organization event lifting
        // Organization and OrganizationalUnit are Contexts
        Ok(vec![])
    }

    /// Create a Role concept event
    ///
    /// Roles are Concepts (semantic/conceptual spaces), not Contexts
    pub fn create_role_concept(&self, role_name: String, role_description: String) -> Result<GraphEvent, ProjectionError> {
        let role_id = Uuid::now_v7();
        Ok(GraphEvent {
            event_id: Uuid::now_v7(),
            aggregate_id: role_id,
            correlation_id: self.correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(cim_graph::events::ConceptPayload::ConceptDefined {
                concept_id: role_id.to_string(),
                name: role_name,
                definition: role_description,
            }),
        })
    }

    /// Create a PKI concept event
    ///
    /// PKI (certificates, trust chains) are Concepts, not Contexts
    pub fn create_pki_concept(&self, cert_name: String, cert_definition: String) -> Result<GraphEvent, ProjectionError> {
        let cert_id = Uuid::now_v7();
        Ok(GraphEvent {
            event_id: Uuid::now_v7(),
            aggregate_id: cert_id,
            correlation_id: self.correlation_id,
            causation_id: None,
            payload: EventPayload::Concept(cim_graph::events::ConceptPayload::ConceptDefined {
                concept_id: cert_id.to_string(),
                name: cert_name,
                definition: cert_definition,
            }),
        })
    }
}

impl Default for GraphProjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during projection
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Invalid event structure: {0}")]
    InvalidEvent(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Graph error: {0}")]
    GraphError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projector_creation() {
        let projector = GraphProjector::new();
        assert!(projector.correlation_id.get_version().is_some());
    }

    #[cfg(feature = "policy")]
    #[test]
    fn test_lift_person_created() {
        use cim_domain_person::events::PersonCreated;
        use cim_domain_person::value_objects::PersonName;
        use cim_domain::{EntityId, entity};

        let projector = GraphProjector::new();

        let person_id = entity::Entity::new_id();
        let event = PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: PersonName::new("Alice", Some("A."), "Smith").unwrap(),
            created_at: Utc::now(),
            created_by: None,
        });

        let graph_events = projector.lift_person_event(&event).unwrap();
        assert_eq!(graph_events.len(), 1);

        if let EventPayload::Context(payload) = &graph_events[0].payload {
            match payload {
                cim_graph::events::ContextPayload::ContextDefined { name, bounded_context_type, .. } => {
                    assert!(name.contains("Alice Smith"));
                    assert_eq!(bounded_context_type, "Person");
                }
                _ => panic!("Expected ContextDefined"),
            }
        } else {
            panic!("Expected Context payload");
        }
    }

    #[test]
    fn test_create_role_concept() {
        let projector = GraphProjector::new();
        let event = projector.create_role_concept(
            "Administrator".to_string(),
            "System administrator role with full access".to_string()
        ).unwrap();

        if let EventPayload::Concept(payload) = &event.payload {
            match payload {
                cim_graph::events::ConceptPayload::ConceptDefined { name, definition, .. } => {
                    assert_eq!(name, "Administrator");
                    assert!(definition.contains("administrator"));
                }
                _ => panic!("Expected ConceptDefined"),
            }
        } else {
            panic!("Expected Concept payload");
        }
    }
}
