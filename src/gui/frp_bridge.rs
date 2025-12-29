//! FRP Bridge - Practical Integration with Existing Iced GUI
//!
//! This module demonstrates how to integrate the FRP system with the existing
//! imperative Iced GUI. It provides adapters, converters, and integration
//! patterns that allow gradual migration from imperative to FRP.
//!
//! ## Integration Strategy
//!
//! ```text
//! Existing Iced GUI
//!       ↓
//! ┌─────────────────────────────────────────────┐
//! │  Message Adapter (converts Iced messages)   │
//! └─────────────────────────────────────────────┘
//!       ↓
//! ┌─────────────────────────────────────────────┐
//! │  FRP Application (pure functional core)     │
//! └─────────────────────────────────────────────┘
//!       ↓
//! ┌─────────────────────────────────────────────┐
//! │  State Synchronizer (updates Iced state)    │
//! └─────────────────────────────────────────────┘
//!       ↓
//! Iced View Rendering
//! ```

use crate::gui::frp_integration::{FrpApplication, PipelineResult, FrpAppState};
use crate::gui::routing::AppMessage;
use crate::gui::graph_signals::LayoutAlgorithm;
use crate::gui::graph::OrganizationGraph;
use iced::Task;
use uuid::Uuid;

/// Message adapter - converts existing Iced messages to FRP AppMessages
///
/// This allows the existing GUI to gradually adopt FRP patterns without
/// rewriting all message handling at once.
pub struct MessageAdapter;

impl MessageAdapter {
    /// Convert a legacy Iced message to an FRP AppMessage
    ///
    /// # Example
    ///
    /// ```rust
    /// use cim_keys::gui::frp_bridge::MessageAdapter;
    ///
    /// // In your Iced update() function:
    /// // match message {
    /// //     Message::SearchQueryChanged(q) => {
    /// //         let app_msg = MessageAdapter::to_app_message(&message);
    /// //         self.frp.handle_message(app_msg);
    /// //     }
    /// // }
    /// ```
    pub fn to_app_message(legacy_message: &LegacyMessage) -> Option<AppMessage> {
        match legacy_message {
            LegacyMessage::GraphNodeClicked(id) => {
                Some(AppMessage::GraphNodeClicked(*id))
            }
            LegacyMessage::SearchQueryChanged(query) => {
                Some(AppMessage::SearchQueryChanged(query.clone()))
            }
            LegacyMessage::FilterPeopleToggled(show) => {
                Some(AppMessage::FilterPeopleToggled(*show))
            }
            LegacyMessage::FilterOrgsToggled(show) => {
                Some(AppMessage::FilterOrgsToggled(*show))
            }
            LegacyMessage::FilterNatsToggled(show) => {
                Some(AppMessage::FilterNatsToggled(*show))
            }
            LegacyMessage::FilterPkiToggled(show) => {
                Some(AppMessage::FilterPkiToggled(*show))
            }
            LegacyMessage::FilterYubikeyToggled(show) => {
                Some(AppMessage::FilterYubikeyToggled(*show))
            }
            LegacyMessage::SearchCleared => {
                Some(AppMessage::SearchCleared)
            }
            // Messages not yet migrated to FRP
            _ => None,
        }
    }
}

/// Legacy message types from the existing Iced GUI
///
/// This represents the existing message enum. As features migrate to FRP,
/// these messages get converted to AppMessage types.
#[derive(Clone, Debug)]
pub enum LegacyMessage {
    // Graph interactions
    GraphNodeClicked(Uuid),
    GraphNodeDragged { node_id: Uuid, position: (f32, f32) },

    // Search
    SearchQueryChanged(String),
    SearchCleared,

    // Filters
    FilterPeopleToggled(bool),
    FilterOrgsToggled(bool),
    FilterNatsToggled(bool),
    FilterPkiToggled(bool),
    FilterYubikeyToggled(bool),

    // Layout
    LayoutChanged(String),

    // Not yet migrated
    YubiKeyDetected,
    ExportRequested,
    ImportRequested,
}

/// FRP Integration Layer
///
/// Wraps the FRP application and provides integration points for the
/// existing Iced GUI. This allows gradual migration.
pub struct FrpIntegrationLayer {
    /// The FRP application (pure functional core)
    frp_app: FrpApplication,

    /// Tracks which features are using FRP
    frp_enabled_features: FrpFeatures,

    /// Cache for expensive computations
    cache: IntegrationCache,
}

impl FrpIntegrationLayer {
    /// Create new integration layer
    pub fn new(initial_graph: OrganizationGraph) -> Self {
        FrpIntegrationLayer {
            frp_app: FrpApplication::new(initial_graph),
            frp_enabled_features: FrpFeatures::default(),
            cache: IntegrationCache::default(),
        }
    }

    /// Handle a message - tries FRP first, falls back to legacy if not enabled
    pub fn handle_message(&mut self, message: LegacyMessage) -> IntegrationResult {
        // Check if this message type should use FRP
        if self.should_use_frp(&message) {
            // Convert to AppMessage and process through FRP
            if let Some(app_message) = MessageAdapter::to_app_message(&message) {
                let result = self.frp_app.handle_message(app_message);

                match result {
                    PipelineResult::StateUpdated { new_state: _, visible_nodes, workflow } => {
                        // Invalidate cache
                        self.cache.invalidate();

                        return IntegrationResult::FrpHandled {
                            state_changed: true,
                            visible_node_count: visible_nodes.len(),
                            workflow_triggered: workflow.is_some(),
                        };
                    }
                    PipelineResult::Ignored => {
                        return IntegrationResult::FrpHandled {
                            state_changed: false,
                            visible_node_count: 0,
                            workflow_triggered: false,
                        };
                    }
                }
            }
        }

        // Fall back to legacy handling
        IntegrationResult::UseLegacy
    }

    /// Enable FRP for a specific feature
    pub fn enable_frp_feature(&mut self, feature: FrpFeature) {
        match feature {
            FrpFeature::Search => self.frp_enabled_features.search = true,
            FrpFeature::Filters => self.frp_enabled_features.filters = true,
            FrpFeature::GraphInteraction => self.frp_enabled_features.graph_interaction = true,
            FrpFeature::Animations => self.frp_enabled_features.animations = true,
            FrpFeature::Workflows => self.frp_enabled_features.workflows = true,
        }
    }

    /// Check if a message should use FRP
    fn should_use_frp(&self, message: &LegacyMessage) -> bool {
        match message {
            LegacyMessage::SearchQueryChanged(_) | LegacyMessage::SearchCleared => {
                self.frp_enabled_features.search
            }
            LegacyMessage::FilterPeopleToggled(_)
            | LegacyMessage::FilterOrgsToggled(_)
            | LegacyMessage::FilterNatsToggled(_)
            | LegacyMessage::FilterPkiToggled(_)
            | LegacyMessage::FilterYubikeyToggled(_) => {
                self.frp_enabled_features.filters
            }
            LegacyMessage::GraphNodeClicked(_) | LegacyMessage::GraphNodeDragged { .. } => {
                self.frp_enabled_features.graph_interaction
            }
            _ => false,
        }
    }

    /// Update animation time (call every frame)
    pub fn update(&mut self, delta_time: f32) {
        if self.frp_enabled_features.animations {
            self.frp_app.update(delta_time);
        }
    }

    /// Get current FRP state for rendering
    pub fn get_state(&self) -> &FrpAppState {
        self.frp_app.state()
    }

    /// Get visible nodes (cached)
    pub fn get_visible_nodes(&mut self) -> &[Uuid] {
        if self.cache.visible_nodes.is_none() {
            let view = self.frp_app.render(0.0);
            let node_ids: Vec<Uuid> = view.visible_nodes.iter().map(|n| n.id).collect();
            self.cache.visible_nodes = Some(node_ids);
        }
        self.cache.visible_nodes.as_ref().unwrap()
    }

    /// Change layout (triggers animation if enabled)
    pub fn change_layout(&mut self, layout: LayoutAlgorithm) {
        self.frp_app.change_layout(layout);
        self.cache.invalidate();
    }
}

/// Features that can be migrated to FRP
#[derive(Clone, Copy, Debug)]
pub enum FrpFeature {
    Search,
    Filters,
    GraphInteraction,
    Animations,
    Workflows,
}

/// Tracks which features are using FRP
#[derive(Default)]
struct FrpFeatures {
    search: bool,
    filters: bool,
    graph_interaction: bool,
    animations: bool,
    workflows: bool,
}

/// Result of handling a message through the integration layer
#[derive(Debug, PartialEq)]
pub enum IntegrationResult {
    /// Message was handled by FRP
    FrpHandled {
        state_changed: bool,
        visible_node_count: usize,
        workflow_triggered: bool,
    },

    /// Message should be handled by legacy code
    UseLegacy,
}

/// Cache for expensive computations
#[derive(Default)]
struct IntegrationCache {
    visible_nodes: Option<Vec<Uuid>>,
}

impl IntegrationCache {
    fn invalidate(&mut self) {
        self.visible_nodes = None;
    }
}

/// Example: Integrating with existing Iced Application
///
/// This shows how to modify the existing CimKeysApp to use FRP for some features.
pub struct HybridCimKeysApp {
    // Keep existing fields for non-migrated features
    pub legacy_state: LegacyAppState,

    // Add FRP integration layer
    pub frp: FrpIntegrationLayer,

    // Track time for animations
    pub elapsed_time: f32,
}

/// Represents the existing app state that hasn't been migrated yet
pub struct LegacyAppState {
    pub some_legacy_field: String,
    pub another_legacy_field: bool,
}

impl HybridCimKeysApp {
    /// Create hybrid app (uses both legacy and FRP)
    pub fn new(initial_graph: OrganizationGraph) -> Self {
        let mut frp = FrpIntegrationLayer::new(initial_graph);

        // Enable FRP for search and filters (gradual migration)
        frp.enable_frp_feature(FrpFeature::Search);
        frp.enable_frp_feature(FrpFeature::Filters);

        HybridCimKeysApp {
            legacy_state: LegacyAppState {
                some_legacy_field: String::new(),
                another_legacy_field: false,
            },
            frp,
            elapsed_time: 0.0,
        }
    }

    /// Update function - integrates FRP with legacy code
    pub fn update(&mut self, message: LegacyMessage) -> Task<LegacyMessage> {
        // Try FRP first
        let result = self.frp.handle_message(message.clone());

        match result {
            IntegrationResult::FrpHandled { .. } => {
                // FRP handled it - no legacy code needed
                Task::none()
            }
            IntegrationResult::UseLegacy => {
                // Fall back to legacy handling
                self.handle_legacy_message(message)
            }
        }
    }

    /// Handle messages not yet migrated to FRP
    fn handle_legacy_message(&mut self, message: LegacyMessage) -> Task<LegacyMessage> {
        match message {
            LegacyMessage::YubiKeyDetected => {
                // Legacy imperative code
                self.legacy_state.another_legacy_field = true;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// Update time for animations
    pub fn tick(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;
        self.frp.update(delta_time);
    }
}

/// Migration checklist for existing GUI
///
/// Step-by-step guide to migrate a feature from imperative to FRP:
///
/// 1. **Identify the Feature**
///    - Choose a self-contained feature (e.g., search, filters)
///    - Ensure it has clear inputs (messages) and outputs (state changes)
///
/// 2. **Create Intent Types**
///    - Define GraphFilterIntent variants for the feature
///    - Map legacy messages to intents
///
/// 3. **Extract Pure Update Logic**
///    - Move update code into pure function
///    - Remove mutations, make it return new state
///
/// 4. **Create Router**
///    - Implement router to convert messages to intents
///    - Add to composed router
///
/// 5. **Enable in Integration Layer**
///    - Call enable_frp_feature() for the feature
///    - Test that messages route correctly
///
/// 6. **Remove Legacy Code**
///    - Once FRP version is tested, remove old imperative code
///    - Update message types if needed
///
/// 7. **Iterate**
///    - Move to next feature
///    - Gradually expand FRP coverage

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, Person, KeyOwnerRole};

    fn test_org(name: &str) -> Organization {
        Organization {
            id: Uuid::now_v7(),
            name: name.to_string(),
            display_name: name.to_string(),
            description: None,
            parent_id: None,
            units: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    fn test_person(name: &str) -> Person {
        Person {
            id: Uuid::now_v7(),
            name: name.to_string(),
            email: format!("{}@example.com", name.to_lowercase()),
            roles: vec![],
            organization_id: Uuid::now_v7(),
            unit_ids: vec![],
            active: true,
        }
    }

    #[test]
    fn test_message_adapter_search() {
        let message = LegacyMessage::SearchQueryChanged("test".to_string());
        let app_message = MessageAdapter::to_app_message(&message);

        assert!(app_message.is_some());
        match app_message.unwrap() {
            AppMessage::SearchQueryChanged(q) => assert_eq!(q, "test"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_message_adapter_filter() {
        let message = LegacyMessage::FilterPeopleToggled(false);
        let app_message = MessageAdapter::to_app_message(&message);

        assert!(app_message.is_some());
        match app_message.unwrap() {
            AppMessage::FilterPeopleToggled(show) => assert_eq!(show, false),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_message_adapter_unmigrated() {
        let message = LegacyMessage::YubiKeyDetected;
        let app_message = MessageAdapter::to_app_message(&message);

        assert!(app_message.is_none());
    }

    #[test]
    fn test_integration_layer_creation() {
        let graph = OrganizationGraph::new();
        let layer = FrpIntegrationLayer::new(graph);

        // Initially no features enabled
        assert!(!layer.frp_enabled_features.search);
        assert!(!layer.frp_enabled_features.filters);
    }

    #[test]
    fn test_enable_frp_feature() {
        let graph = OrganizationGraph::new();
        let mut layer = FrpIntegrationLayer::new(graph);

        layer.enable_frp_feature(FrpFeature::Search);
        assert!(layer.frp_enabled_features.search);
        assert!(!layer.frp_enabled_features.filters);
    }

    #[test]
    fn test_handle_message_with_frp_disabled() {
        let graph = OrganizationGraph::new();
        let mut layer = FrpIntegrationLayer::new(graph);

        // Search not enabled yet
        let message = LegacyMessage::SearchQueryChanged("test".to_string());
        let result = layer.handle_message(message);

        assert_eq!(result, IntegrationResult::UseLegacy);
    }

    #[test]
    fn test_handle_message_with_frp_enabled() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Alice");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut layer = FrpIntegrationLayer::new(graph);
        layer.enable_frp_feature(FrpFeature::Search);

        let message = LegacyMessage::SearchQueryChanged("alice".to_string());
        let result = layer.handle_message(message);

        match result {
            IntegrationResult::FrpHandled { state_changed, visible_node_count, .. } => {
                assert!(state_changed);
                assert_eq!(visible_node_count, 1);
            }
            IntegrationResult::UseLegacy => panic!("Should use FRP"),
        }
    }

    #[test]
    fn test_hybrid_app_creation() {
        let graph = OrganizationGraph::new();
        let app = HybridCimKeysApp::new(graph);

        // Search and filters should be FRP-enabled
        assert!(app.frp.frp_enabled_features.search);
        assert!(app.frp.frp_enabled_features.filters);
    }

    #[test]
    fn test_hybrid_app_frp_message() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Bob");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut app = HybridCimKeysApp::new(graph);

        // Search should be handled by FRP
        let message = LegacyMessage::SearchQueryChanged("bob".to_string());
        app.update(message);

        // Verify state changed in FRP
        let state = app.frp.get_state();
        assert_eq!(state.filter_state.filters.search_query, "bob");
    }

    #[test]
    fn test_hybrid_app_legacy_message() {
        let graph = OrganizationGraph::new();
        let mut app = HybridCimKeysApp::new(graph);

        // YubiKey detection not migrated to FRP yet
        let message = LegacyMessage::YubiKeyDetected;
        app.update(message);

        // Should have been handled by legacy code
        assert!(app.legacy_state.another_legacy_field);
    }

    #[test]
    fn test_integration_cache() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Charlie");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut layer = FrpIntegrationLayer::new(graph);
        layer.enable_frp_feature(FrpFeature::Search);

        // First call computes visible nodes
        let nodes1 = layer.get_visible_nodes().to_vec();
        assert_eq!(nodes1.len(), 1);

        // Second call should use cache (same result, no recomputation)
        let nodes2 = layer.get_visible_nodes().to_vec();
        assert_eq!(nodes1, nodes2);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Dave");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut layer = FrpIntegrationLayer::new(graph);
        layer.enable_frp_feature(FrpFeature::Search);

        // Get initial visible nodes (cached)
        let nodes_before = layer.get_visible_nodes().len();
        assert_eq!(nodes_before, 1);

        // Change search query (invalidates cache)
        let message = LegacyMessage::SearchQueryChanged("nonexistent".to_string());
        layer.handle_message(message);

        // Cache should be invalidated, recomputed on next access
        let nodes_after = layer.get_visible_nodes().len();
        assert_eq!(nodes_after, 0);
    }

    #[test]
    fn test_animation_update() {
        let graph = OrganizationGraph::new();
        let mut layer = FrpIntegrationLayer::new(graph);
        layer.enable_frp_feature(FrpFeature::Animations);

        // Trigger layout change
        layer.change_layout(LayoutAlgorithm::Circular);

        // Animation progress should start at 0
        let state_before = layer.get_state();
        assert_eq!(state_before.animation_progress, 0.0);

        // Advance time
        layer.update(0.5);

        // Animation should have progressed
        let state_after = layer.get_state();
        assert!(state_after.animation_progress > 0.0);
    }
}
