// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Infrastructure Bounded Context
//!
//! This module implements the NATS domain with:
//! - Message enum for all NATS operations
//! - State struct for NATS-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **Hierarchy Generation**: Generate NATS hierarchy from organization
//! 2. **Bootstrap**: Create NATS bootstrap configuration
//! 3. **Visualization**: UI state for exploring NATS hierarchy
//! 4. **Management**: Add/remove accounts and users
//! 5. **Configuration**: Export configuration options

use std::collections::HashSet;
use std::path::PathBuf;

use iced::Task;
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

/// NATS Infrastructure State
///
/// Contains all state related to NATS infrastructure management.
#[derive(Debug, Clone, Default)]
pub struct NatsState {
    // === Bootstrap state ===
    /// NATS bootstrap configuration
    pub nats_bootstrap: Option<crate::domain_projections::OrganizationBootstrap>,

    // === Configuration state ===
    /// Whether to include NATS config in export
    pub include_nats_config: bool,
    /// Whether NATS hierarchy has been generated
    pub nats_hierarchy_generated: bool,
    /// NATS operator ID
    pub nats_operator_id: Option<Uuid>,
    /// Path for NATS export
    pub nats_export_path: PathBuf,

    // === Visualization state ===
    /// Whether NATS visualization section is collapsed
    pub nats_viz_section_collapsed: bool,
    /// Set of expanded account names in tree view
    pub nats_viz_expanded_accounts: HashSet<String>,
    /// Whether the operator is selected
    pub nats_viz_selected_operator: bool,
    /// Selected account name
    pub nats_viz_selected_account: Option<String>,
    /// Selected user (account_name, person_id)
    pub nats_viz_selected_user: Option<(String, Uuid)>,
    /// Cached hierarchy data for display
    pub nats_viz_hierarchy_data: Option<NatsHierarchyFull>,

    // === UI state ===
    /// Whether NATS section is collapsed
    pub nats_section_collapsed: bool,
    /// Filter toggle for showing NATS in graph
    pub filter_show_nats: bool,
}

impl NatsState {
    /// Create a new NatsState with sensible defaults
    pub fn new() -> Self {
        Self {
            nats_bootstrap: None,
            include_nats_config: true,
            nats_hierarchy_generated: false,
            nats_operator_id: None,
            nats_export_path: PathBuf::from("nsc"),
            nats_viz_section_collapsed: true,
            nats_viz_expanded_accounts: HashSet::new(),
            nats_viz_selected_operator: false,
            nats_viz_selected_account: None,
            nats_viz_selected_user: None,
            nats_viz_hierarchy_data: None,
            nats_section_collapsed: false,
            filter_show_nats: true,
        }
    }

    /// Check if NATS hierarchy is ready for operations
    pub fn is_hierarchy_ready(&self) -> bool {
        self.nats_hierarchy_generated && self.nats_operator_id.is_some()
    }

    /// Get the number of expanded accounts
    pub fn expanded_account_count(&self) -> usize {
        self.nats_viz_expanded_accounts.len()
    }

    /// Check if an account is expanded
    pub fn is_account_expanded(&self, account_name: &str) -> bool {
        self.nats_viz_expanded_accounts.contains(account_name)
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        self.nats_viz_selected_operator = false;
        self.nats_viz_selected_account = None;
        self.nats_viz_selected_user = None;
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update NATS state based on message
///
/// This function handles NATS domain messages. Note that some messages
/// require access to the full application state or external services
/// and will be delegated back to the main update function.
pub fn update(state: &mut NatsState, message: NatsMessage) -> Task<Message> {
    use NatsMessage::*;

    match message {
        // === Configuration ===
        ToggleNatsConfig(enabled) => {
            state.include_nats_config = enabled;
            Task::none()
        }

        ToggleNatsSection => {
            state.nats_section_collapsed = !state.nats_section_collapsed;
            Task::none()
        }

        // === Visualization ===
        ToggleNatsVizSection => {
            state.nats_viz_section_collapsed = !state.nats_viz_section_collapsed;
            Task::none()
        }

        ToggleNatsAccountExpand(account_name) => {
            if state.nats_viz_expanded_accounts.contains(&account_name) {
                state.nats_viz_expanded_accounts.remove(&account_name);
            } else {
                state.nats_viz_expanded_accounts.insert(account_name);
            }
            Task::none()
        }

        SelectNatsOperator => {
            state.clear_selections();
            state.nats_viz_selected_operator = true;
            Task::none()
        }

        SelectNatsAccount(account_name) => {
            state.clear_selections();
            state.nats_viz_selected_account = Some(account_name);
            Task::none()
        }

        SelectNatsUser(account_name, person_id) => {
            state.clear_selections();
            state.nats_viz_selected_user = Some((account_name, person_id));
            Task::none()
        }

        NatsHierarchyRefreshed(result) => {
            match result {
                Ok(hierarchy) => {
                    state.nats_viz_hierarchy_data = Some(hierarchy);
                }
                Err(_e) => {
                    // Error handling - status message would be set by main app
                }
            }
            Task::none()
        }

        // === Filters ===
        ToggleFilterNats => {
            state.filter_show_nats = !state.filter_show_nats;
            Task::none()
        }

        // === Operations that require main app context ===
        // These operations need access to org_graph, projections, etc.
        // The main update function will handle the actual logic
        GenerateNatsHierarchy => {
            // Requires access to org_graph and projections - delegated to main
            Task::none()
        }

        NatsHierarchyGenerated(result) => {
            match &result {
                Ok(_path) => {
                    state.nats_hierarchy_generated = true;
                }
                Err(_) => {
                    state.nats_hierarchy_generated = false;
                }
            }
            Task::none()
        }

        NatsBootstrapCreated(bootstrap) => {
            state.nats_bootstrap = Some(*bootstrap);
            Task::none()
        }

        GenerateNatsFromGraph => {
            // Requires access to org_graph - delegated to main
            Task::none()
        }

        NatsFromGraphGenerated(_result) => {
            // Result handling done in main app (adds nodes to graph)
            Task::none()
        }

        RefreshNatsHierarchy => {
            // Requires access to org_graph and projections - delegated to main
            Task::none()
        }

        AddNatsAccount { .. } => {
            // Requires access to projections - delegated to main
            Task::none()
        }

        AddNatsUser { .. } => {
            // Requires access to projections - delegated to main
            Task::none()
        }

        RemoveNatsAccount(_account_name) => {
            // Clear selection if the removed account was selected
            if let Some(ref selected) = state.nats_viz_selected_account {
                if selected == &_account_name {
                    state.nats_viz_selected_account = None;
                }
            }
            state.nats_viz_expanded_accounts.remove(&_account_name);
            Task::none()
        }

        RemoveNatsUser(account_name, person_id) => {
            // Clear selection if the removed user was selected
            if let Some((ref acc, ref pid)) = state.nats_viz_selected_user {
                if acc == &account_name && *pid == person_id {
                    state.nats_viz_selected_user = None;
                }
            }
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_state_default() {
        let state = NatsState::default();
        assert!(state.nats_bootstrap.is_none());
        assert!(!state.include_nats_config);
        assert!(!state.nats_hierarchy_generated);
        assert!(state.nats_operator_id.is_none());
    }

    #[test]
    fn test_nats_state_new() {
        let state = NatsState::new();
        assert!(state.include_nats_config);
        assert!(!state.nats_hierarchy_generated);
        assert!(state.nats_viz_section_collapsed);
        assert!(!state.nats_section_collapsed);
        assert!(state.filter_show_nats);
    }

    #[test]
    fn test_is_hierarchy_ready() {
        let mut state = NatsState::new();
        assert!(!state.is_hierarchy_ready());

        state.nats_hierarchy_generated = true;
        assert!(!state.is_hierarchy_ready());

        state.nats_operator_id = Some(Uuid::now_v7());
        assert!(state.is_hierarchy_ready());
    }

    #[test]
    fn test_toggle_sections() {
        let mut state = NatsState::new();

        // Verify defaults
        assert!(state.nats_viz_section_collapsed);
        assert!(!state.nats_section_collapsed);
        assert!(state.filter_show_nats);

        let _ = update(&mut state, NatsMessage::ToggleNatsVizSection);
        assert!(!state.nats_viz_section_collapsed);

        let _ = update(&mut state, NatsMessage::ToggleNatsSection);
        assert!(state.nats_section_collapsed);

        let _ = update(&mut state, NatsMessage::ToggleFilterNats);
        assert!(!state.filter_show_nats);
    }

    #[test]
    fn test_account_expand() {
        let mut state = NatsState::new();
        let account = "test-account".to_string();

        assert!(!state.is_account_expanded(&account));

        let _ = update(&mut state, NatsMessage::ToggleNatsAccountExpand(account.clone()));
        assert!(state.is_account_expanded(&account));
        assert_eq!(state.expanded_account_count(), 1);

        let _ = update(&mut state, NatsMessage::ToggleNatsAccountExpand(account.clone()));
        assert!(!state.is_account_expanded(&account));
        assert_eq!(state.expanded_account_count(), 0);
    }

    #[test]
    fn test_selection() {
        let mut state = NatsState::new();
        let account = "test-account".to_string();
        let person_id = Uuid::now_v7();

        // Select operator
        let _ = update(&mut state, NatsMessage::SelectNatsOperator);
        assert!(state.nats_viz_selected_operator);
        assert!(state.nats_viz_selected_account.is_none());
        assert!(state.nats_viz_selected_user.is_none());

        // Select account - should clear operator selection
        let _ = update(&mut state, NatsMessage::SelectNatsAccount(account.clone()));
        assert!(!state.nats_viz_selected_operator);
        assert_eq!(state.nats_viz_selected_account, Some(account.clone()));
        assert!(state.nats_viz_selected_user.is_none());

        // Select user - should clear account selection
        let _ = update(
            &mut state,
            NatsMessage::SelectNatsUser(account.clone(), person_id),
        );
        assert!(!state.nats_viz_selected_operator);
        assert!(state.nats_viz_selected_account.is_none());
        assert_eq!(
            state.nats_viz_selected_user,
            Some((account.clone(), person_id))
        );
    }

    #[test]
    fn test_remove_clears_selection() {
        let mut state = NatsState::new();
        let account = "test-account".to_string();
        let person_id = Uuid::now_v7();

        // Select and expand account
        let _ = update(&mut state, NatsMessage::SelectNatsAccount(account.clone()));
        let _ = update(&mut state, NatsMessage::ToggleNatsAccountExpand(account.clone()));
        assert!(state.is_account_expanded(&account));
        assert_eq!(state.nats_viz_selected_account, Some(account.clone()));

        // Remove account - should clear selection and expansion
        let _ = update(&mut state, NatsMessage::RemoveNatsAccount(account.clone()));
        assert!(!state.is_account_expanded(&account));
        assert!(state.nats_viz_selected_account.is_none());

        // Test user removal
        let _ = update(
            &mut state,
            NatsMessage::SelectNatsUser(account.clone(), person_id),
        );
        assert_eq!(
            state.nats_viz_selected_user,
            Some((account.clone(), person_id))
        );

        let _ = update(&mut state, NatsMessage::RemoveNatsUser(account, person_id));
        assert!(state.nats_viz_selected_user.is_none());
    }

    #[test]
    fn test_config_toggle() {
        let mut state = NatsState::new();
        assert!(state.include_nats_config);

        let _ = update(&mut state, NatsMessage::ToggleNatsConfig(false));
        assert!(!state.include_nats_config);

        let _ = update(&mut state, NatsMessage::ToggleNatsConfig(true));
        assert!(state.include_nats_config);
    }

    #[test]
    fn test_hierarchy_generated() {
        let mut state = NatsState::new();
        assert!(!state.nats_hierarchy_generated);

        let _ = update(
            &mut state,
            NatsMessage::NatsHierarchyGenerated(Ok("/path/to/nsc".to_string())),
        );
        assert!(state.nats_hierarchy_generated);

        let _ = update(
            &mut state,
            NatsMessage::NatsHierarchyGenerated(Err("failed".to_string())),
        );
        assert!(!state.nats_hierarchy_generated);
    }
}
