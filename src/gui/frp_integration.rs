//! FRP System Integration
//!
//! Demonstrates how to wire together all FRP components into a complete,
//! working reactive system. This module shows the full architecture:
//!
//! ```text
//! User Interaction
//!       ↓
//! Message (UI Event)
//!       ↓
//! Router (routing.rs) ──→ Intent
//!       ↓
//! Update Function (feedback.rs) + State
//!       ↓
//! New State Signal (graph_signals.rs)
//!       ↓
//! ├─→ Workflow Pipeline (workflows.rs) ──→ CausalChain
//! └─→ Animation Pipeline (animations.rs) ──→ Smooth Transitions
//!       ↓
//! View Rendering
//! ```
//!
//! ## Complete Example
//!
//! This module provides a **runnable demonstration** of the full FRP architecture
//! that can be adapted to integrate with the actual GUI.

use crate::signals::{Signal, StepKind, ContinuousKind};
use crate::causality::CausalChain;
use crate::gui::graph::{OrganizationGraph, GraphNode};
use crate::gui::graph_signals::{FilterState, visible_nodes, LayoutAlgorithm, compute_node_positions};
use crate::gui::graph_causality::GraphOperation;
use crate::gui::feedback::{GraphFilterAppState, GraphFilterIntent, update_graph_filter_state};
use crate::gui::routing::{AppMessage, Route, graph_router, filter_router, search_router, compose_routes, GraphIntent};
use crate::gui::workflows::{WorkflowStep, build_pki_workflow, build_nats_workflow};
use crate::gui::animations::{animate_nodes, animate_value, EasingFunction};
use uuid::Uuid;
use iced::Point;
use std::collections::HashMap;

/// Complete FRP Application State
///
/// Combines all reactive components into a unified state model.
/// This replaces the traditional imperative GUI state with pure signals.
#[derive(Clone)]
pub struct FrpAppState {
    /// The organizational graph (source of truth)
    pub graph: OrganizationGraph,

    /// Filter/search state (from feedback.rs)
    pub filter_state: GraphFilterAppState,

    /// Current layout algorithm
    pub layout: LayoutAlgorithm,

    /// Animation progress (0.0 to 1.0)
    pub animation_progress: f32,
}

impl FrpAppState {
    /// Create initial state
    pub fn initial(graph: OrganizationGraph) -> Self {
        FrpAppState {
            graph,
            filter_state: GraphFilterAppState::initial(),
            layout: LayoutAlgorithm::Manual,
            animation_progress: 1.0, // No animation initially
        }
    }

    /// Update graph (triggers workflows and animations)
    pub fn with_graph(self, graph: OrganizationGraph) -> Self {
        FrpAppState {
            graph,
            animation_progress: 0.0, // Start new animation
            ..self
        }
    }

    /// Update filter state
    pub fn with_filter_state(self, filter_state: GraphFilterAppState) -> Self {
        FrpAppState {
            filter_state,
            ..self
        }
    }

    /// Change layout algorithm (triggers animation)
    pub fn with_layout(self, layout: LayoutAlgorithm) -> Self {
        FrpAppState {
            layout,
            animation_progress: 0.0, // Start layout transition animation
            ..self
        }
    }

    /// Update animation progress
    pub fn with_animation_progress(self, progress: f32) -> Self {
        FrpAppState {
            animation_progress: progress.clamp(0.0, 1.0),
            ..self
        }
    }
}

/// The Complete FRP Pipeline
///
/// Demonstrates how messages flow through the entire system:
/// Message → Intent → State Update → Workflow → Animation → View
pub struct FrpPipeline {
    /// Message router (from routing.rs)
    router: Route<AppMessage, GraphFilterIntent>,

    /// Current application state
    state: FrpAppState,

    /// Previous node positions (for animation interpolation)
    previous_positions: HashMap<Uuid, Point>,

    /// Current node positions (target for animation)
    current_positions: HashMap<Uuid, Point>,
}

impl FrpPipeline {
    /// Create a new FRP pipeline
    pub fn new(initial_graph: OrganizationGraph) -> Self {
        // Create composable router (from routing.rs)
        let filter = filter_router();
        let search = search_router();
        let router = compose_routes(filter, search);

        // Compute initial positions
        let positions = compute_node_positions(&initial_graph, LayoutAlgorithm::Manual);

        FrpPipeline {
            router,
            state: FrpAppState::initial(initial_graph),
            previous_positions: positions.clone(),
            current_positions: positions,
        }
    }

    /// Process a message through the complete pipeline
    ///
    /// This is the **core integration point** showing how all FRP components work together.
    pub fn process_message(&mut self, message: AppMessage) -> PipelineResult {
        // Step 1: Route message to intent (routing.rs)
        let intent = match self.router.route(&message) {
            Some(intent) => intent,
            None => return PipelineResult::Ignored,
        };

        // Step 2: Update state (feedback.rs)
        let new_filter_state = update_graph_filter_state(
            self.state.filter_state.clone(),
            intent.clone(),
        );
        self.state = self.state.clone().with_filter_state(new_filter_state);

        // Step 3: Compute visible nodes (graph_signals.rs)
        let visible = visible_nodes(
            &self.state.graph,
            &self.state.filter_state.filters,
        );

        // Step 4: Generate workflows if needed (workflows.rs)
        let workflow = match intent {
            GraphFilterIntent::ResetFilters => {
                // Trigger PKI workflow on reset
                Some(build_pki_workflow(&self.state.graph))
            }
            _ => None,
        };

        PipelineResult::StateUpdated {
            new_state: self.state.clone(),
            visible_nodes: visible,
            workflow,
        }
    }

    /// Trigger layout change (demonstrates animation pipeline)
    pub fn change_layout(&mut self, layout: LayoutAlgorithm) -> LayoutTransition {
        // Save current positions for animation interpolation
        self.previous_positions = self.current_positions.clone();

        // Compute new positions
        self.current_positions = compute_node_positions(&self.state.graph, layout);

        // Update state with new layout
        self.state = self.state.clone().with_layout(layout);

        LayoutTransition {
            start_positions: self.previous_positions.clone(),
            end_positions: self.current_positions.clone(),
            duration: 1.0, // 1 second animation
        }
    }

    /// Get animated positions at current time
    ///
    /// Demonstrates integration with animations.rs
    pub fn animated_positions(&self, time: f32) -> HashMap<Uuid, Point> {
        if self.state.animation_progress >= 1.0 {
            return self.current_positions.clone();
        }

        // Create animation signal (animations.rs)
        let animation = animate_nodes(
            self.previous_positions.clone(),
            self.current_positions.clone(),
            1.0,
            EasingFunction::EaseInOut,
        );

        animation.sample(time as f64)
    }

    /// Get current state
    pub fn state(&self) -> &FrpAppState {
        &self.state
    }

    /// Create complete infrastructure workflows
    ///
    /// Demonstrates workflow.rs integration - generates both PKI and NATS
    pub fn generate_infrastructure(&self) -> InfrastructureWorkflows {
        InfrastructureWorkflows {
            pki_workflow: build_pki_workflow(&self.state.graph),
            nats_workflow: build_nats_workflow(&self.state.graph),
        }
    }
}

/// Result of processing a message through the pipeline
#[derive(Clone)]
pub enum PipelineResult {
    /// Message was routed but didn't change state
    Ignored,

    /// State was updated
    StateUpdated {
        new_state: FrpAppState,
        visible_nodes: Vec<GraphNode>,
        workflow: Option<WorkflowStep>,
    },
}

/// Layout transition data for animation
#[derive(Clone, Debug)]
pub struct LayoutTransition {
    pub start_positions: HashMap<Uuid, Point>,
    pub end_positions: HashMap<Uuid, Point>,
    pub duration: f32,
}

/// Complete infrastructure generation result
#[derive(Clone, Debug)]
pub struct InfrastructureWorkflows {
    pub pki_workflow: WorkflowStep,
    pub nats_workflow: WorkflowStep,
}

/// Signal-based view model
///
/// Demonstrates how to expose the FRP system as signals for rendering.
/// The GUI can sample these signals at render time instead of storing mutable state.
pub struct FrpViewModel {
    /// State signal (StepKind - changes discretely)
    pub state_signal: Signal<StepKind, FrpAppState>,

    /// Animation progress signal (ContinuousKind - changes smoothly)
    pub animation_signal: Signal<ContinuousKind, f32>,

    /// Visible nodes signal (derived from state)
    pub visible_nodes_signal: Signal<StepKind, Vec<GraphNode>>,
}

impl FrpViewModel {
    /// Create view model from current state
    pub fn new(state: FrpAppState) -> Self {
        // Create state signal
        let state_signal = Signal::<StepKind, FrpAppState>::step(state.clone());

        // Create animation signal (0.0 to 1.0 over 1 second)
        let animation_signal = animate_value(0.0, 1.0, 1.0, EasingFunction::EaseInOut);

        // Create derived signal for visible nodes
        let state_for_visible = state.clone();
        let visible_nodes_signal = Signal::<StepKind, Vec<GraphNode>>::step(
            visible_nodes(&state_for_visible.graph, &state_for_visible.filter_state.filters)
        );

        FrpViewModel {
            state_signal,
            animation_signal,
            visible_nodes_signal,
        }
    }

    /// Sample all signals at a given time
    pub fn sample_at(&self, time: f32) -> ViewModelSnapshot {
        ViewModelSnapshot {
            state: self.state_signal.sample(time as f64),
            animation_progress: self.animation_signal.sample(time as f64),
            visible_nodes: self.visible_nodes_signal.sample(time as f64),
        }
    }
}

/// Snapshot of view model at a specific time
#[derive(Clone)]
pub struct ViewModelSnapshot {
    pub state: FrpAppState,
    pub animation_progress: f32,
    pub visible_nodes: Vec<GraphNode>,
}

/// Complete FRP Application
///
/// This is the **top-level integration** showing how to build a complete
/// application using all FRP components.
///
/// # Example Usage
///
/// ```rust
/// use cim_keys::gui::frp_integration::FrpApplication;
/// use cim_keys::gui::routing::AppMessage;
/// use cim_keys::gui::graph::OrganizationGraph;
///
/// // Create application
/// let graph = OrganizationGraph::new();
/// let mut app = FrpApplication::new(graph);
///
/// // Process user interaction
/// let message = AppMessage::SearchQueryChanged("alice".to_string());
/// app.handle_message(message);
///
/// // Render at current time
/// let view = app.render(1.5);
/// ```
pub struct FrpApplication {
    pipeline: FrpPipeline,
    view_model: FrpViewModel,
    current_time: f32,
}

impl FrpApplication {
    /// Create new FRP application
    pub fn new(initial_graph: OrganizationGraph) -> Self {
        let pipeline = FrpPipeline::new(initial_graph.clone());
        let view_model = FrpViewModel::new(FrpAppState::initial(initial_graph));

        FrpApplication {
            pipeline,
            view_model,
            current_time: 0.0,
        }
    }

    /// Handle user message (integrates routing + feedback + workflows)
    pub fn handle_message(&mut self, message: AppMessage) -> PipelineResult {
        let result = self.pipeline.process_message(message);

        // Update view model with new state
        if let PipelineResult::StateUpdated { ref new_state, .. } = result {
            self.view_model = FrpViewModel::new(new_state.clone());
        }

        result
    }

    /// Change layout (triggers animation)
    pub fn change_layout(&mut self, layout: LayoutAlgorithm) -> LayoutTransition {
        let transition = self.pipeline.change_layout(layout);

        // Update view model
        self.view_model = FrpViewModel::new(self.pipeline.state().clone());

        transition
    }

    /// Advance time (for animations)
    pub fn update(&mut self, delta_time: f32) {
        self.current_time += delta_time;

        // Update animation progress in state
        let new_progress = (self.current_time).min(1.0);
        let new_state = self.pipeline.state().clone()
            .with_animation_progress(new_progress);

        self.pipeline.state = new_state.clone();
        self.view_model = FrpViewModel::new(new_state);
    }

    /// Render view at current time (samples all signals)
    pub fn render(&self, time: f32) -> ViewModelSnapshot {
        self.view_model.sample_at(time)
    }

    /// Get current animated positions
    pub fn get_animated_positions(&self) -> HashMap<Uuid, Point> {
        self.pipeline.animated_positions(self.current_time)
    }

    /// Generate complete infrastructure
    pub fn generate_infrastructure(&self) -> InfrastructureWorkflows {
        self.pipeline.generate_infrastructure()
    }

    /// Get current application state
    pub fn state(&self) -> &FrpAppState {
        self.pipeline.state()
    }
}

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
            created_at: chrono::Utc::now(),
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
            created_at: chrono::Utc::now(),
            active: true,
        }
    }

    #[test]
    fn test_frp_app_state_initial() {
        let graph = OrganizationGraph::new();
        let state = FrpAppState::initial(graph);

        assert_eq!(state.animation_progress, 1.0);
        assert!(matches!(state.layout, LayoutAlgorithm::Manual));
    }

    #[test]
    fn test_frp_pipeline_creation() {
        let graph = OrganizationGraph::new();
        let pipeline = FrpPipeline::new(graph);

        assert_eq!(pipeline.state.animation_progress, 1.0);
    }

    #[test]
    fn test_message_processing() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Alice");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut pipeline = FrpPipeline::new(graph);

        // Process search message
        let message = AppMessage::SearchQueryChanged("alice".to_string());
        let result = pipeline.process_message(message);

        match result {
            PipelineResult::StateUpdated { new_state, visible_nodes, .. } => {
                assert_eq!(new_state.filter_state.filters.search_query, "alice");
                assert_eq!(visible_nodes.len(), 1);
            }
            PipelineResult::Ignored => panic!("Message should not be ignored"),
        }
    }

    #[test]
    fn test_layout_change_triggers_animation() {
        let graph = OrganizationGraph::new();
        let mut pipeline = FrpPipeline::new(graph);

        let transition = pipeline.change_layout(LayoutAlgorithm::Circular);

        assert_eq!(transition.duration, 1.0);
        assert_eq!(pipeline.state.animation_progress, 0.0);
        assert!(matches!(pipeline.state.layout, LayoutAlgorithm::Circular));
    }

    #[test]
    fn test_animated_positions() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Alice");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut pipeline = FrpPipeline::new(graph);

        // Trigger layout change
        pipeline.change_layout(LayoutAlgorithm::Circular);

        // Get positions at mid-animation
        let positions_start = pipeline.animated_positions(0.0);
        let positions_mid = pipeline.animated_positions(0.5);
        let positions_end = pipeline.animated_positions(1.0);

        // Should have positions at all time points
        assert!(!positions_start.is_empty());
        assert!(!positions_mid.is_empty());
        assert!(!positions_end.is_empty());
    }

    #[test]
    fn test_infrastructure_generation() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        graph.add_organization_node(org);

        let pipeline = FrpPipeline::new(graph);
        let infrastructure = pipeline.generate_infrastructure();

        // Both workflows should have operations
        assert!(infrastructure.pki_workflow.operations.len() > 0);
        assert!(infrastructure.nats_workflow.operations.len() > 0);
    }

    #[test]
    fn test_view_model_creation() {
        let graph = OrganizationGraph::new();
        let state = FrpAppState::initial(graph);
        let view_model = FrpViewModel::new(state.clone());

        // Sample at different times
        let snapshot_0 = view_model.sample_at(0.0);
        let snapshot_1 = view_model.sample_at(1.0);

        assert_eq!(snapshot_0.state.animation_progress, state.animation_progress);
        assert_eq!(snapshot_1.state.animation_progress, state.animation_progress);
    }

    #[test]
    fn test_complete_application() {
        let mut graph = OrganizationGraph::new();
        let person = test_person("Alice");
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut app = FrpApplication::new(graph);

        // Handle search message
        let message = AppMessage::SearchQueryChanged("alice".to_string());
        let result = app.handle_message(message);

        match result {
            PipelineResult::StateUpdated { .. } => {
                // Success
            }
            PipelineResult::Ignored => panic!("Should process search message"),
        }

        // Render at current time
        let view = app.render(0.0);
        assert!(view.visible_nodes.len() > 0);
    }

    #[test]
    fn test_application_update_advances_time() {
        let graph = OrganizationGraph::new();
        let mut app = FrpApplication::new(graph);

        assert_eq!(app.current_time, 0.0);

        app.update(0.016); // ~60fps frame
        assert!((app.current_time - 0.016).abs() < 0.001);

        app.update(0.016);
        assert!((app.current_time - 0.032).abs() < 0.001);
    }

    #[test]
    fn test_complete_pipeline_flow() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        let person = test_person("Alice");

        graph.add_organization_node(org);
        graph.add_node(person, KeyOwnerRole::Developer);

        let mut app = FrpApplication::new(graph);

        // 1. Search for person
        app.handle_message(AppMessage::SearchQueryChanged("alice".to_string()));

        // 2. Change layout (triggers animation)
        app.change_layout(LayoutAlgorithm::Circular);

        // 3. Update animation
        app.update(0.5); // Halfway through animation

        // 4. Render current state
        let view = app.render(0.5);

        // Verify complete flow worked
        assert_eq!(view.state.filter_state.filters.search_query, "alice");
        assert!(matches!(view.state.layout, LayoutAlgorithm::Circular));
        assert!(view.animation_progress > 0.0);
        assert!(view.visible_nodes.len() > 0);
    }

    #[test]
    fn test_infrastructure_generation_integration() {
        let mut graph = OrganizationGraph::new();
        let org = test_org("Acme Corp");
        graph.add_organization_node(org);

        let app = FrpApplication::new(graph);

        // Generate complete infrastructure
        let infrastructure = app.generate_infrastructure();

        // Verify both workflows executed
        assert!(infrastructure.pki_workflow.metadata.total_operations > 0);
        assert!(infrastructure.nats_workflow.metadata.total_operations > 0);

        // Verify PKI description
        assert!(infrastructure.pki_workflow.description.contains("PKI"));

        // Verify NATS description
        assert!(infrastructure.nats_workflow.description.contains("NATS"));
    }
}
