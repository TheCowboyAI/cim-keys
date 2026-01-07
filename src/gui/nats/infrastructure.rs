// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Infrastructure Message Definitions
//!
//! This module defines the message types for the NATS bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **Hierarchy Generation**: Generate NATS hierarchy from organization
//! 2. **Bootstrap**: Create NATS bootstrap configuration
//! 3. **Visualization**: UI state for exploring NATS hierarchy
//! 4. **Management**: Add/remove accounts and users
//! 5. **Configuration**: Export configuration options

use uuid::Uuid;

use crate::gui::NatsHierarchyFull;

/// NATS Infrastructure Message
///
/// Organized by sub-domain:
/// - Hierarchy Generation (2 messages)
/// - Bootstrap (2 messages)
/// - Graph Integration (2 messages)
/// - Visualization (7 messages)
/// - Management (4 messages)
/// - Configuration (2 messages)
/// - Filters (1 message)
#[derive(Debug, Clone)]
pub enum NatsMessage {
    // === Hierarchy Generation ===
    /// Generate NATS hierarchy from organization structure
    GenerateNatsHierarchy,
    /// Result of hierarchy generation
    NatsHierarchyGenerated(Result<String, String>),

    // === Bootstrap ===
    /// NATS bootstrap configuration created
    NatsBootstrapCreated(Box<crate::domain_projections::OrganizationBootstrap>),

    // === Graph Integration ===
    /// Generate NATS nodes from organization graph
    GenerateNatsFromGraph,
    /// Result of NATS graph generation
    NatsFromGraphGenerated(
        Result<Vec<(crate::gui::graph::ConceptEntity, iced::Point, Option<Uuid>)>, String>,
    ),

    // === Visualization ===
    /// Toggle NATS visualization section
    ToggleNatsVizSection,
    /// Toggle account tree node expansion
    ToggleNatsAccountExpand(String),
    /// Select the operator node
    SelectNatsOperator,
    /// Select an account node
    SelectNatsAccount(String),
    /// Select a user node (account_name, person_id)
    SelectNatsUser(String, Uuid),
    /// Refresh NATS hierarchy data
    RefreshNatsHierarchy,
    /// Result of hierarchy refresh
    NatsHierarchyRefreshed(Result<NatsHierarchyFull, String>),

    // === Management ===
    /// Add a NATS account to an organizational unit
    AddNatsAccount { unit_id: Uuid, account_name: String },
    /// Add a NATS user to an account
    AddNatsUser { account_name: String, person_id: Uuid },
    /// Remove a NATS account
    RemoveNatsAccount(String),
    /// Remove a NATS user from an account
    RemoveNatsUser(String, Uuid),

    // === Configuration ===
    /// Toggle NATS configuration export
    ToggleNatsConfig(bool),
    /// Toggle NATS section in UI
    ToggleNatsSection,

    // === Filters ===
    /// Toggle NATS filter in graph view
    ToggleFilterNats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_message_variants() {
        let _ = NatsMessage::GenerateNatsHierarchy;
        let _ = NatsMessage::NatsHierarchyGenerated(Ok("/path".to_string()));
        let _ = NatsMessage::GenerateNatsFromGraph;
        let _ = NatsMessage::ToggleNatsVizSection;
        let _ = NatsMessage::ToggleNatsAccountExpand("account".to_string());
        let _ = NatsMessage::SelectNatsOperator;
        let _ = NatsMessage::SelectNatsAccount("account".to_string());
        let _ = NatsMessage::SelectNatsUser("account".to_string(), Uuid::nil());
        let _ = NatsMessage::RefreshNatsHierarchy;
        let _ = NatsMessage::AddNatsAccount { unit_id: Uuid::nil(), account_name: "acc".to_string() };
        let _ = NatsMessage::AddNatsUser { account_name: "acc".to_string(), person_id: Uuid::nil() };
        let _ = NatsMessage::RemoveNatsAccount("acc".to_string());
        let _ = NatsMessage::RemoveNatsUser("acc".to_string(), Uuid::nil());
        let _ = NatsMessage::ToggleNatsConfig(true);
        let _ = NatsMessage::ToggleNatsSection;
        let _ = NatsMessage::ToggleFilterNats;
    }
}
