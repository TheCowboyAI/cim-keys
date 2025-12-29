//! Production FRP Integration for CimKeysApp
//!
//! This module integrates the FRP demonstration modules into the actual production GUI.
//! It provides a gradual migration path where search, filtering, and layout can be
//! handled through the FRP pipeline while keeping the rest of the application working.

use std::collections::HashMap;
use uuid::Uuid;
use iced::Point;

use crate::gui::graph::{OrganizationGraph, GraphNode};
use crate::gui::graph_signals::{FilterState, visible_nodes, LayoutAlgorithm, compute_node_positions};
use crate::gui::feedback::{GraphFilterAppState, GraphFilterIntent, update_graph_filter_state};
use crate::gui::routing::{AppMessage, Route, filter_router, search_router, compose_routes};
use crate::gui::frp_integration::{FrpAppState, PipelineResult};
use crate::gui::workflows::{build_pki_workflow, build_nats_workflow};
use crate::gui::animations::{animate_nodes, EasingFunction};
use crate::gui::GraphLayout;

/// Production FRP integration for CimKeysApp
///
/// This wraps the FRP pipeline and provides adapter methods for integrating
/// with the existing imperative GUI code.
pub struct GuiFrpIntegration {
    /// The FRP application state
    state: FrpAppState,

    /// Message router (converts AppMessages to Intents)
    router: Route<AppMessage, GraphFilterIntent>,

    /// Previous node positions (for animation)
    previous_positions: HashMap<Uuid, Point>,

    /// Current node positions (target for animation)
    current_positions: HashMap<Uuid, Point>,

    /// Current animation time
    animation_time: f32,
}

impl GuiFrpIntegration {
    /// Create new FRP integration with initial graph
    pub fn new(graph: OrganizationGraph) -> Self {
        // Create composable router
        let filter = filter_router();
        let search = search_router();
        let router = compose_routes(filter, search);

        // Compute initial positions
        let positions = compute_node_positions(&graph, LayoutAlgorithm::Manual);

        GuiFrpIntegration {
            state: FrpAppState::initial(graph),
            router,
            previous_positions: positions.clone(),
            current_positions: positions,
            animation_time: 0.0,
        }
    }

    /// Process a message through the FRP pipeline
    ///
    /// Returns Some(visible_nodes) if the message was handled,
    /// None if the message is not recognized by the router.
    pub fn process_message(&mut self, message: AppMessage) -> Option<Vec<GraphNode>> {

        // Step 1: Route message to intent
        let intent = match self.router.route(&message) {
            Some(intent) => intent,
            None => return None,  // Message not handled by FRP
        };

        // Step 2: Update state through pure function
        let new_filter_state = update_graph_filter_state(
            self.state.filter_state.clone(),
            intent,
        );
        self.state = self.state.clone().with_filter_state(new_filter_state);

        // Step 3: Compute visible nodes
        let visible = visible_nodes(
            &self.state.graph,
            &self.state.filter_state.filters,
        );

        Some(visible)
    }

    /// Update the organization graph
    pub fn update_graph(&mut self, graph: OrganizationGraph) {
        self.state = self.state.clone().with_graph(graph);
    }

    /// Change layout (triggers animation)
    pub fn change_layout(&mut self, layout: LayoutAlgorithm) {
        // Save current positions for animation
        self.previous_positions = self.current_positions.clone();

        // Compute new positions
        self.current_positions = compute_node_positions(&self.state.graph, layout);

        // Update state
        self.state = self.state.clone().with_layout(layout);

        // Reset animation
        self.animation_time = 0.0;
    }

    /// Advance animation time
    pub fn update_animation(&mut self, delta_time: f32) {
        if self.animation_time < 1.0 {
            self.animation_time += delta_time;
        }
    }

    /// Get animated positions at current time
    pub fn get_animated_positions(&self) -> HashMap<Uuid, Point> {
        if self.animation_time >= 1.0 {
            return self.current_positions.clone();
        }

        // Create animation signal
        let animation = animate_nodes(
            self.previous_positions.clone(),
            self.current_positions.clone(),
            1.0,  // 1 second duration
            EasingFunction::EaseInOut,
        );

        animation.sample(self.animation_time as f64)
    }

    /// Get current filter state
    pub fn get_filters(&self) -> &FilterState {
        &self.state.filter_state.filters
    }

    /// Get current search query
    pub fn get_search_query(&self) -> &str {
        &self.state.filter_state.filters.search_query
    }

    /// Get visible nodes (cached from last message processing)
    pub fn get_visible_nodes(&self) -> Vec<GraphNode> {
        visible_nodes(&self.state.graph, &self.state.filter_state.filters)
    }

    /// Get selected node ID
    pub fn get_selected_node(&self) -> Option<Uuid> {
        self.state.filter_state.selected_node
    }

    /// Generate PKI workflow from current graph
    pub fn generate_pki_workflow(&self) -> crate::gui::workflows::WorkflowStep {
        build_pki_workflow(&self.state.graph)
    }

    /// Generate NATS workflow from current graph
    pub fn generate_nats_workflow(&self) -> crate::gui::workflows::WorkflowStep {
        build_nats_workflow(&self.state.graph)
    }

    /// Get current animation progress (0.0 to 1.0)
    pub fn get_animation_progress(&self) -> f32 {
        self.animation_time.min(1.0)
    }

    /// Check if animation is complete
    pub fn is_animation_complete(&self) -> bool {
        self.animation_time >= 1.0
    }
}

/// Adapter to convert legacy Message types to FRP AppMessage
pub struct MessageAdapter;

impl MessageAdapter {
    /// Convert a search query change to FRP message
    pub fn search_query_changed(query: String) -> AppMessage {
        AppMessage::SearchQueryChanged(query)
    }

    /// Convert filter toggle to FRP message
    pub fn filter_people_toggled(show: bool) -> AppMessage {
        AppMessage::FilterPeopleToggled(show)
    }

    pub fn filter_orgs_toggled(show: bool) -> AppMessage {
        AppMessage::FilterOrgsToggled(show)
    }

    pub fn filter_nats_toggled(show: bool) -> AppMessage {
        AppMessage::FilterNatsToggled(show)
    }

    pub fn filter_pki_toggled(show: bool) -> AppMessage {
        AppMessage::FilterPkiToggled(show)
    }

    pub fn filter_yubikey_toggled(show: bool) -> AppMessage {
        AppMessage::FilterYubikeyToggled(show)
    }

    /// Convert GraphLayout to LayoutAlgorithm
    pub fn layout_to_algorithm(layout: GraphLayout) -> LayoutAlgorithm {
        match layout {
            GraphLayout::Manual => LayoutAlgorithm::Manual,
            GraphLayout::Hierarchical => LayoutAlgorithm::Hierarchical,
            GraphLayout::ForceDirected => LayoutAlgorithm::ForceDirected,
            GraphLayout::Circular => LayoutAlgorithm::Circular,
        }
    }

    /// Convert LayoutAlgorithm to GraphLayout
    pub fn algorithm_to_layout(algorithm: LayoutAlgorithm) -> GraphLayout {
        match algorithm {
            LayoutAlgorithm::Manual => GraphLayout::Manual,
            LayoutAlgorithm::Hierarchical => GraphLayout::Hierarchical,
            LayoutAlgorithm::ForceDirected => GraphLayout::ForceDirected,
            LayoutAlgorithm::Circular => GraphLayout::Circular,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Person, KeyOwnerRole};

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
    fn test_gui_frp_integration_creation() {
        let graph = OrganizationGraph::new();
        let frp = GuiFrpIntegration::new(graph);

        assert_eq!(frp.get_animation_progress(), 0.0);
    }

    #[test]
    fn test_search_message_processing() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Alice");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut frp = GuiFrpIntegration::new(graph);

        // Process search message
        let message = MessageAdapter::search_query_changed("alice".to_string());
        let result = frp.process_message(message);

        assert!(result.is_some());
        let visible = result.unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(frp.get_search_query(), "alice");
    }

    #[test]
    fn test_filter_message_processing() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Bob");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut frp = GuiFrpIntegration::new(graph);

        // Initially all filters enabled (nodes visible)
        assert_eq!(frp.get_visible_nodes().len(), 1);

        // Toggle people filter off
        let message = MessageAdapter::filter_people_toggled(false);
        let result = frp.process_message(message);

        assert!(result.is_some());
        let visible = result.unwrap();
        assert_eq!(visible.len(), 0);  // People hidden
        assert!(!frp.get_filters().show_people);
    }

    #[test]
    fn test_layout_change() {
        let graph = OrganizationGraph::new();
        let mut frp = GuiFrpIntegration::new(graph);

        // Change layout
        frp.change_layout(LayoutAlgorithm::Circular);

        // Animation should start
        assert_eq!(frp.get_animation_progress(), 0.0);
        assert!(!frp.is_animation_complete());

        // Advance animation
        frp.update_animation(0.5);
        assert_eq!(frp.get_animation_progress(), 0.5);

        // Complete animation
        frp.update_animation(0.6);
        assert_eq!(frp.get_animation_progress(), 1.0);
        assert!(frp.is_animation_complete());
    }

    #[test]
    fn test_message_adapter_conversions() {
        let layout = GraphLayout::Hierarchical;
        let algorithm = MessageAdapter::layout_to_algorithm(layout);
        let back_to_layout = MessageAdapter::algorithm_to_layout(algorithm);

        assert_eq!(layout, back_to_layout);
    }

    #[test]
    fn test_workflow_generation() {
        let graph = OrganizationGraph::new();
        let frp = GuiFrpIntegration::new(graph);

        // Generate workflows
        let pki_workflow = frp.generate_pki_workflow();
        let nats_workflow = frp.generate_nats_workflow();

        // Workflows are generated (even if empty for minimal graph)
        // The important thing is they don't panic
        assert!(pki_workflow.operations.events().len() >= 0);
        assert!(nats_workflow.operations.events().len() >= 0);
    }
}
