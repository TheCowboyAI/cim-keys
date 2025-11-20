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
use cim_graph::events::{GraphEvent, EventPayload};

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
    ///
    /// Functor properties:
    /// - Identity: F(id_A) = id_F(A)
    /// - Composition: F(g ∘ f) = F(g) ∘ F(f)
    #[cfg(feature = "policy")]
    pub fn lift_person_event(&self, event: &PersonEvent) -> Result<Vec<GraphEvent>, ProjectionError> {
        match event {
            PersonEvent::PersonCreated(created) => {
                // Person is a CONTEXT (bounded context containing Person aggregate)
                // First event: Create the bounded context
                let context_event = GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *created.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None, // Root event
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::BoundedContextCreated {
                            context_id: created.person_id.as_uuid().to_string(),
                            name: format!("Person: {}", created.name.display_name()),
                            description: format!("Person aggregate created at {}", created.created_at),
                        }
                    ),
                };

                // Second event: Add the Person aggregate to the context
                let aggregate_event = GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *created.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: Some(context_event.event_id), // Caused by context creation
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::AggregateAdded {
                            context_id: created.person_id.as_uuid().to_string(),
                            aggregate_id: *created.person_id.as_uuid(),
                            aggregate_type: "Person".to_string(),
                        }
                    ),
                };

                Ok(vec![context_event, aggregate_event])
            }

            PersonEvent::NameUpdated(updated) => {
                // Entity modification within aggregate
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *updated.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *updated.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "NameUpdate".to_string(),
                            properties: serde_json::json!({
                                "old_name": updated.old_name.display_name(),
                                "new_name": updated.new_name.display_name(),
                                "reason": updated.reason.as_ref().map(|r| r.to_string()),
                                "updated_at": updated.updated_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::PersonUpdated(updated) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *updated.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *updated.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "PersonUpdate".to_string(),
                            properties: serde_json::json!({
                                "name": updated.name.display_name(),
                                "updated_at": updated.updated_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::BirthDateSet(birth_date) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *birth_date.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *birth_date.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "BirthDate".to_string(),
                            properties: serde_json::json!({
                                "birth_date": birth_date.birth_date,
                                "set_at": birth_date.set_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::DeathRecorded(death) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *death.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *death.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "DeathRecord".to_string(),
                            properties: serde_json::json!({
                                "date_of_death": death.date_of_death,
                                "recorded_at": death.recorded_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::PersonDeactivated(deactivated) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *deactivated.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *deactivated.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "Deactivation".to_string(),
                            properties: serde_json::json!({
                                "reason": deactivated.reason,
                                "deactivated_at": deactivated.deactivated_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::PersonReactivated(reactivated) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *reactivated.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *reactivated.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "Reactivation".to_string(),
                            properties: serde_json::json!({
                                "reactivated_at": reactivated.reactivated_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::PersonMergedInto(merged) => {
                // Create relationship between source and target persons
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *merged.source_person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *merged.source_person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "MergeRecord".to_string(),
                            properties: serde_json::json!({
                                "merged_into_id": merged.merged_into_id.as_uuid().to_string(),
                                "merge_reason": format!("{:?}", merged.merge_reason),
                                "merged_at": merged.merged_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::AttributeRecorded(attr) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *attr.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *attr.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "Attribute".to_string(),
                            properties: serde_json::json!({
                                "attribute": format!("{:?}", attr.attribute),
                                "recorded_at": attr.recorded_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::AttributeUpdated(attr) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *attr.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *attr.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "AttributeUpdate".to_string(),
                            properties: serde_json::json!({
                                "attribute_type": format!("{:?}", attr.attribute_type),
                                "old_attribute": format!("{:?}", attr.old_attribute),
                                "new_attribute": format!("{:?}", attr.new_attribute),
                                "updated_at": attr.updated_at,
                            }),
                        }
                    ),
                }])
            }

            PersonEvent::AttributeInvalidated(attr) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *attr.person_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *attr.person_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "AttributeInvalidation".to_string(),
                            properties: serde_json::json!({
                                "attribute_type": attr.attribute_type,
                                "reason": attr.reason,
                                "invalidated_at": attr.invalidated_at,
                            }),
                        }
                    ),
                }])
            }
        }
    }

    /// Lift an Organization domain event into graph events (functor)
    #[cfg(feature = "policy")]
    pub fn lift_organization_event(&self, event: &OrganizationEvent) -> Result<Vec<GraphEvent>, ProjectionError> {
        match event {
            OrganizationEvent::OrganizationCreated(created) => {
                // Organization is a CONTEXT (bounded context)
                let context_event = GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *created.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::BoundedContextCreated {
                            context_id: created.organization_id.as_uuid().to_string(),
                            name: format!("Organization: {}", created.name),
                            description: format!("{} ({})", created.display_name, format!("{:?}", created.organization_type)),
                        }
                    ),
                };

                let aggregate_event = GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *created.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: Some(context_event.event_id),
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::AggregateAdded {
                            context_id: created.organization_id.as_uuid().to_string(),
                            aggregate_id: *created.organization_id.as_uuid(),
                            aggregate_type: "Organization".to_string(),
                        }
                    ),
                };

                Ok(vec![context_event, aggregate_event])
            }

            OrganizationEvent::OrganizationUpdated(updated) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *updated.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *updated.organization_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "OrganizationUpdate".to_string(),
                            properties: serde_json::json!({
                                "changes": format!("{:?}", updated.changes),
                                "occurred_at": updated.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::OrganizationDissolved(dissolved) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *dissolved.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *dissolved.organization_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "Dissolution".to_string(),
                            properties: serde_json::json!({
                                "reason": dissolved.reason,
                                "effective_date": dissolved.effective_date,
                                "occurred_at": dissolved.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::OrganizationMerged(merged) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *merged.surviving_organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *merged.surviving_organization_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "MergeRecord".to_string(),
                            properties: serde_json::json!({
                                "merged_organization_id": merged.merged_organization_id.as_uuid().to_string(),
                                "merger_type": format!("{:?}", merged.merger_type),
                                "effective_date": merged.effective_date,
                                "occurred_at": merged.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::OrganizationStatusChanged(status) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *status.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *status.organization_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "StatusChange".to_string(),
                            properties: serde_json::json!({
                                "previous_status": format!("{:?}", status.previous_status),
                                "new_status": format!("{:?}", status.new_status),
                                "reason": status.reason.as_ref().map(|r| r.to_string()),
                                "occurred_at": status.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::DepartmentCreated(dept) => {
                // Department is also a CONTEXT (organizational unit)
                let context_event = GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *dept.department_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::BoundedContextCreated {
                            context_id: dept.department_id.as_uuid().to_string(),
                            name: format!("Department: {}", dept.name),
                            description: format!("Department in org {}", dept.organization_id.as_uuid()),
                        }
                    ),
                };

                let aggregate_event = GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *dept.department_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: Some(context_event.event_id),
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::AggregateAdded {
                            context_id: dept.department_id.as_uuid().to_string(),
                            aggregate_id: *dept.department_id.as_uuid(),
                            aggregate_type: "Department".to_string(),
                        }
                    ),
                };

                Ok(vec![context_event, aggregate_event])
            }

            OrganizationEvent::DepartmentUpdated(dept) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *dept.department_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *dept.department_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "DepartmentUpdate".to_string(),
                            properties: serde_json::json!({
                                "changes": format!("{:?}", dept.changes),
                                "occurred_at": dept.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::DepartmentRestructured(dept) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *dept.department_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *dept.department_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "Restructure".to_string(),
                            properties: serde_json::json!({
                                "new_parent_id": dept.new_parent_id.as_ref().map(|id| id.as_uuid().to_string()),
                                "restructure_type": format!("{:?}", dept.restructure_type),
                                "occurred_at": dept.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::DepartmentDissolved(dept) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *dept.department_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *dept.department_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "Dissolution".to_string(),
                            properties: serde_json::json!({
                                "reason": dept.reason.clone(),
                                "transfer_to": dept.transfer_to.as_ref().map(|id| id.as_uuid().to_string()),
                                "occurred_at": dept.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::TeamFormed(team) => {
                // Team is also a CONTEXT
                let description = match &team.department_id {
                    Some(dept_id) => format!("Team in dept {}", dept_id.as_uuid()),
                    None => format!("Team {} ({})", team.name, format!("{:?}", team.team_type)),
                };

                let context_event = GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *team.team_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::BoundedContextCreated {
                            context_id: team.team_id.as_uuid().to_string(),
                            name: format!("Team: {}", team.name),
                            description,
                        }
                    ),
                };

                let aggregate_event = GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *team.team_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: Some(context_event.event_id),
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::AggregateAdded {
                            context_id: team.team_id.as_uuid().to_string(),
                            aggregate_id: *team.team_id.as_uuid(),
                            aggregate_type: "Team".to_string(),
                        }
                    ),
                };

                Ok(vec![context_event, aggregate_event])
            }

            OrganizationEvent::TeamUpdated(team) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *team.team_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *team.team_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "TeamUpdate".to_string(),
                            properties: serde_json::json!({
                                "changes": format!("{:?}", team.changes),
                                "occurred_at": team.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::TeamDisbanded(team) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *team.team_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *team.team_id.as_uuid(),
                            entity_id: Uuid::now_v7(),
                            entity_type: "Disbandment".to_string(),
                            properties: serde_json::json!({
                                "reason": team.reason.clone(),
                                "members_transferred_to": team.members_transferred_to.as_ref().map(|id| id.as_uuid().to_string()),
                                "occurred_at": team.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::RoleCreated(role) => {
                // Role is a CONCEPT in conceptual spaces
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *role.role_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Concept(
                        cim_graph::events::ConceptPayload::ConceptDefined {
                            concept_id: role.role_id.as_uuid().to_string(),
                            name: role.title.clone(),
                            definition: role.description.clone().unwrap_or_else(|| format!("{} role in organization", role.title)),
                        }
                    ),
                }])
            }

            OrganizationEvent::RoleUpdated(role) => {
                // Role updates are entity changes within the organization context
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *role.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *role.organization_id.as_uuid(),
                            entity_id: *role.role_id.as_uuid(),
                            entity_type: "RoleUpdate".to_string(),
                            properties: serde_json::json!({
                                "changes": format!("{:?}", role.changes),
                                "occurred_at": role.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::RoleDeprecated(role) => {
                // Role deprecation is an entity change within the organization context
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *role.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *role.organization_id.as_uuid(),
                            entity_id: *role.role_id.as_uuid(),
                            entity_type: "RoleDeprecation".to_string(),
                            properties: serde_json::json!({
                                "reason": role.reason.clone(),
                                "replacement_role_id": role.replacement_role_id.as_ref().map(|id| id.as_uuid().to_string()),
                                "effective_date": role.effective_date,
                                "occurred_at": role.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::FacilityCreated(facility) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *facility.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *facility.organization_id.as_uuid(),
                            entity_id: *facility.facility_id.as_uuid(),
                            entity_type: "Facility".to_string(),
                            properties: serde_json::json!({
                                "name": facility.name.clone(),
                                "code": facility.code.clone(),
                                "facility_type": format!("{:?}", facility.facility_type),
                                "description": facility.description.clone(),
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::FacilityUpdated(facility) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *facility.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *facility.organization_id.as_uuid(),
                            entity_id: *facility.facility_id.as_uuid(),
                            entity_type: "FacilityUpdate".to_string(),
                            properties: serde_json::json!({
                                "changes": format!("{:?}", facility.changes),
                                "occurred_at": facility.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::FacilityRemoved(facility) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *facility.organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *facility.organization_id.as_uuid(),
                            entity_id: *facility.facility_id.as_uuid(),
                            entity_type: "FacilityRemoval".to_string(),
                            properties: serde_json::json!({
                                "reason": facility.reason.clone(),
                                "occurred_at": facility.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::ChildOrganizationAdded(child) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *child.parent_organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *child.parent_organization_id.as_uuid(),
                            entity_id: child.child_organization_id,
                            entity_type: "ChildOrganization".to_string(),
                            properties: serde_json::json!({
                                "child_name": child.child_name.clone(),
                                "child_type": format!("{:?}", child.child_type),
                                "occurred_at": child.occurred_at,
                            }),
                        }
                    ),
                }])
            }

            OrganizationEvent::ChildOrganizationRemoved(child) => {
                Ok(vec![GraphEvent {
                    event_id: Uuid::now_v7(),
                    aggregate_id: *child.parent_organization_id.as_uuid(),
                    correlation_id: self.correlation_id,
                    causation_id: None,
                    payload: EventPayload::Context(
                        cim_graph::events::ContextPayload::EntityAdded {
                            aggregate_id: *child.parent_organization_id.as_uuid(),
                            entity_id: child.child_organization_id,
                            entity_type: "ChildOrganizationRemoval".to_string(),
                            properties: serde_json::json!({
                                "occurred_at": child.occurred_at,
                            }),
                        }
                    ),
                }])
            }
        }
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
            name: PersonName::new("Alice".to_string(), "Smith".to_string()),
            source: "test".to_string(),
            created_at: Utc::now(),
        });

        let graph_events = projector.lift_person_event(&event).unwrap();
        // Should generate 2 events: BoundedContextCreated and AggregateAdded
        assert_eq!(graph_events.len(), 2);

        // First event: BoundedContextCreated
        if let EventPayload::Context(payload) = &graph_events[0].payload {
            match payload {
                cim_graph::events::ContextPayload::BoundedContextCreated { name, description, .. } => {
                    assert!(name.contains("Alice Smith"));
                    assert!(description.contains("Person aggregate created"));
                }
                _ => panic!("Expected BoundedContextCreated"),
            }
        } else {
            panic!("Expected Context payload");
        }

        // Second event: AggregateAdded (caused by first event)
        assert_eq!(graph_events[1].causation_id, Some(graph_events[0].event_id));
        if let EventPayload::Context(payload) = &graph_events[1].payload {
            match payload {
                cim_graph::events::ContextPayload::AggregateAdded { aggregate_type, .. } => {
                    assert_eq!(aggregate_type, "Person");
                }
                _ => panic!("Expected AggregateAdded"),
            }
        } else {
            panic!("Expected Context payload");
        }

        // Verify functor properties: both events share same correlation_id
        assert_eq!(graph_events[0].correlation_id, graph_events[1].correlation_id);
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
