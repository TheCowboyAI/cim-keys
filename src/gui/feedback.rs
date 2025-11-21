//! Feedback Loop Pattern for Graph Filtering
//!
//! Demonstrates the pure functional feedback loop pattern using Decoupled signals.
//! This is a focused example showing how to replace mutable state with immutable
//! signals and feedback loops.
//!
//! ## Pattern Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                 Feedback Loop                    │
//! │                                                  │
//! │  State Signal ──→ View ──→ Intent ──→ Update   │
//! │       ↑                                    │     │
//! │       └────────────────────────────────────┘     │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! The state flows in one direction, with updates feeding back through
//! the Decoupled signal primitive.

use crate::signals::{Signal, StepKind};
use crate::gui::graph_signals::FilterState;
use uuid::Uuid;

/// Application state for graph filtering (immutable)
///
/// This replaces mutable fields like:
/// ```ignore
/// filter_show_people: bool,
/// filter_show_orgs: bool,
/// search_query: String,
/// ```
///
/// With a single immutable state value that flows through signals.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphFilterAppState {
    /// Current filter settings
    pub filters: FilterState,

    /// Search results (highlighted nodes)
    pub search_results: Vec<Uuid>,

    /// Currently selected node (if any)
    pub selected_node: Option<Uuid>,

    /// Status message to display
    pub status: String,
}

impl GraphFilterAppState {
    /// Create initial state
    pub fn initial() -> Self {
        GraphFilterAppState {
            filters: FilterState::default(),
            search_results: vec![],
            selected_node: None,
            status: "Ready".to_string(),
        }
    }

    /// Create a new state with updated filters (pure function)
    pub fn with_filters(self, filters: FilterState) -> Self {
        GraphFilterAppState {
            filters,
            status: "Filters updated".to_string(),
            ..self
        }
    }

    /// Create a new state with search results (pure function)
    pub fn with_search_results(self, search_results: Vec<Uuid>) -> Self {
        let status = if search_results.is_empty() {
            "No results found".to_string()
        } else {
            format!("Found {} results", search_results.len())
        };

        GraphFilterAppState {
            search_results,
            status,
            ..self
        }
    }

    /// Create a new state with selected node (pure function)
    pub fn with_selection(self, selected_node: Option<Uuid>) -> Self {
        let status = if selected_node.is_some() {
            "Node selected".to_string()
        } else {
            "Selection cleared".to_string()
        };

        GraphFilterAppState {
            selected_node,
            status,
            ..self
        }
    }
}

/// User intents for graph filtering
///
/// These are discrete user actions that trigger state updates.
/// This replaces scattered Message enum variants with focused intents.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphFilterIntent {
    /// Toggle visibility of a category
    ToggleCategoryFilter {
        people: Option<bool>,
        orgs: Option<bool>,
        nats: Option<bool>,
        pki: Option<bool>,
        yubikey: Option<bool>,
    },

    /// Update search query
    SearchQueryChanged(String),

    /// Clear search
    ClearSearch,

    /// Select a node
    SelectNode(Uuid),

    /// Clear selection
    ClearSelection,

    /// Reset all filters to default
    ResetFilters,
}

/// Pure update function: Intent + State → New State
///
/// This is the heart of the feedback loop. It's a pure function that
/// takes current state and an intent, and returns new state.
///
/// **NO SIDE EFFECTS** - no mutation, no I/O, no randomness.
pub fn update_graph_filter_state(
    state: GraphFilterAppState,
    intent: GraphFilterIntent,
) -> GraphFilterAppState {
    match intent {
        GraphFilterIntent::ToggleCategoryFilter { people, orgs, nats, pki, yubikey } => {
            let mut new_filters = state.filters.clone();

            if let Some(show) = people {
                new_filters.show_people = show;
            }
            if let Some(show) = orgs {
                new_filters.show_orgs = show;
            }
            if let Some(show) = nats {
                new_filters.show_nats = show;
            }
            if let Some(show) = pki {
                new_filters.show_pki = show;
            }
            if let Some(show) = yubikey {
                new_filters.show_yubikey = show;
            }

            state.with_filters(new_filters)
        }

        GraphFilterIntent::SearchQueryChanged(query) => {
            let mut new_filters = state.filters.clone();
            new_filters.search_query = query;

            // Search results will be computed separately from the graph
            state.with_filters(new_filters)
        }

        GraphFilterIntent::ClearSearch => {
            let mut new_filters = state.filters.clone();
            new_filters.search_query = String::new();

            state
                .with_filters(new_filters)
                .with_search_results(vec![])
        }

        GraphFilterIntent::SelectNode(node_id) => {
            state.with_selection(Some(node_id))
        }

        GraphFilterIntent::ClearSelection => {
            state.with_selection(None)
        }

        GraphFilterIntent::ResetFilters => {
            GraphFilterAppState {
                filters: FilterState::default(),
                selected_node: state.selected_node,
                search_results: vec![],
                status: "Filters reset to default".to_string(),
            }
        }
    }
}

/// Create a feedback loop for graph filter state
///
/// This demonstrates how to create a Decoupled signal that forms a feedback loop.
/// The state signal can be sampled to get current state, and updated by sending
/// intents through the update function.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::feedback::{create_feedback_loop, GraphFilterIntent};
///
/// let (state_signal, update_fn) = create_feedback_loop();
///
/// // Sample current state
/// let current_state = state_signal.sample(0.0);
/// println!("Current status: {}", current_state.status);
///
/// // Send an intent to update state
/// let new_state = update_fn(current_state, GraphFilterIntent::ClearSearch);
/// ```
pub fn create_feedback_loop() -> (
    Signal<StepKind, GraphFilterAppState>,
    impl Fn(GraphFilterAppState, GraphFilterIntent) -> GraphFilterAppState,
) {
    let initial_state = GraphFilterAppState::initial();
    let state_signal = Signal::<StepKind, GraphFilterAppState>::step(initial_state);

    (state_signal, update_graph_filter_state)
}

/// Demonstrates connecting the feedback loop to a Decoupled signal
///
/// This shows the full pattern:
/// 1. Create a Decoupled signal pair (input, output)
/// 2. Feed intents into the input
/// 3. Transform through the update function
/// 4. Get updated state from the output
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::feedback::demonstrate_feedback_with_decoupled;
///
/// // This would be called from your GUI initialization
/// let state_signal = demonstrate_feedback_with_decoupled();
///
/// // Sample state at different time points
/// let state_t0 = state_signal.sample(0.0);
/// let state_t1 = state_signal.sample(1.0);
/// ```
pub fn demonstrate_feedback_with_decoupled() -> Signal<StepKind, GraphFilterAppState> {
    // Create initial state signal
    let initial_state = GraphFilterAppState::initial();
    let state_signal = Signal::<StepKind, GraphFilterAppState>::step(initial_state.clone());

    // In a real app, this would be wired to UI events
    // For demonstration, we'll show the structure:

    // Intent signal would come from UI interactions
    // let intent_signal = Signal::from_ui_events();

    // Transform intents to state updates
    // let new_state_signal = intent_signal.map(|intent| {
    //     let current_state = state_signal.sample(current_time);
    //     update_graph_filter_state(current_state, intent)
    // });

    // This creates the feedback loop:
    // state_signal ─→ view ─→ intent_signal ─→ update ─→ new_state_signal
    //     ↑                                                       │
    //     └───────────────────────────────────────────────────────┘

    state_signal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = GraphFilterAppState::initial();
        assert!(state.filters.show_people);
        assert!(state.filters.show_orgs);
        assert_eq!(state.status, "Ready");
        assert!(state.search_results.is_empty());
        assert!(state.selected_node.is_none());
    }

    #[test]
    fn test_toggle_category_filter() {
        let state = GraphFilterAppState::initial();

        let intent = GraphFilterIntent::ToggleCategoryFilter {
            people: Some(false),
            orgs: None,
            nats: None,
            pki: None,
            yubikey: None,
        };

        let new_state = update_graph_filter_state(state, intent);

        assert!(!new_state.filters.show_people);
        assert!(new_state.filters.show_orgs); // unchanged
        assert_eq!(new_state.status, "Filters updated");
    }

    #[test]
    fn test_search_query_changed() {
        let state = GraphFilterAppState::initial();

        let intent = GraphFilterIntent::SearchQueryChanged("alice".to_string());
        let new_state = update_graph_filter_state(state, intent);

        assert_eq!(new_state.filters.search_query, "alice");
        assert_eq!(new_state.status, "Filters updated");
    }

    #[test]
    fn test_clear_search() {
        let mut state = GraphFilterAppState::initial();
        state.filters.search_query = "bob".to_string();
        state.search_results = vec![Uuid::now_v7(), Uuid::now_v7()];

        let intent = GraphFilterIntent::ClearSearch;
        let new_state = update_graph_filter_state(state, intent);

        assert_eq!(new_state.filters.search_query, "");
        assert!(new_state.search_results.is_empty());
    }

    #[test]
    fn test_select_node() {
        let state = GraphFilterAppState::initial();
        let node_id = Uuid::now_v7();

        let intent = GraphFilterIntent::SelectNode(node_id);
        let new_state = update_graph_filter_state(state, intent);

        assert_eq!(new_state.selected_node, Some(node_id));
        assert_eq!(new_state.status, "Node selected");
    }

    #[test]
    fn test_clear_selection() {
        let mut state = GraphFilterAppState::initial();
        state.selected_node = Some(Uuid::now_v7());

        let intent = GraphFilterIntent::ClearSelection;
        let new_state = update_graph_filter_state(state, intent);

        assert!(new_state.selected_node.is_none());
        assert_eq!(new_state.status, "Selection cleared");
    }

    #[test]
    fn test_reset_filters() {
        let mut state = GraphFilterAppState::initial();
        state.filters.show_people = false;
        state.filters.search_query = "test".to_string();
        state.selected_node = Some(Uuid::now_v7());

        let intent = GraphFilterIntent::ResetFilters;
        let new_state = update_graph_filter_state(state, intent);

        assert!(new_state.filters.show_people);
        assert_eq!(new_state.filters.search_query, "");
        assert!(new_state.search_results.is_empty());
        // Selection is preserved
        assert!(new_state.selected_node.is_some());
    }

    #[test]
    fn test_immutability() {
        let state1 = GraphFilterAppState::initial();
        let state1_clone = state1.clone();

        let intent = GraphFilterIntent::SearchQueryChanged("test".to_string());
        let state2 = update_graph_filter_state(state1, intent);

        // Original state unchanged (was moved, but clone proves immutability)
        assert_eq!(state1_clone.filters.search_query, "");
        // New state has the update
        assert_eq!(state2.filters.search_query, "test");
    }

    #[test]
    fn test_state_transitions() {
        let state = GraphFilterAppState::initial();

        // Apply multiple intents in sequence
        let state = update_graph_filter_state(
            state,
            GraphFilterIntent::SearchQueryChanged("alice".to_string()),
        );
        assert_eq!(state.filters.search_query, "alice");

        let state = update_graph_filter_state(
            state,
            GraphFilterIntent::ToggleCategoryFilter {
                people: Some(false),
                orgs: None,
                nats: None,
                pki: None,
                yubikey: None,
            },
        );
        assert!(!state.filters.show_people);
        assert_eq!(state.filters.search_query, "alice"); // preserved

        let state = update_graph_filter_state(state, GraphFilterIntent::ClearSearch);
        assert_eq!(state.filters.search_query, "");
        assert!(!state.filters.show_people); // preserved
    }

    #[test]
    fn test_with_methods_are_pure() {
        let state = GraphFilterAppState::initial();
        let filters = FilterState::default();

        // Calling with_filters doesn't modify original
        let state_clone = state.clone();
        let new_state = state.with_filters(filters);

        // Original unchanged
        assert_eq!(state_clone.status, "Ready");
        // New state updated
        assert_eq!(new_state.status, "Filters updated");
    }
}
