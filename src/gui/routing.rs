//! Message Routing Patterns
//!
//! Demonstrates composable message routing to replace giant match statements
//! with type-safe, composable routers.
//!
//! ## The Problem
//!
//! Traditional GUI update functions have giant match statements:
//!
//! ```ignore
//! fn update(&mut self, message: Message) -> Task<Message> {
//!     match message {
//!         Message::ConceptEntityClicked(id) => { ... }      // 10 lines
//!         Message::ConceptEntityDragged(id, pos) => { ... } // 15 lines
//!         Message::FilterToggled(filter) => { ... }      // 8 lines
//!         Message::SearchQueryChanged(q) => { ... }      // 12 lines
//!         Message::ExportClicked => { ... }              // 50 lines
//!         Message::ImportClicked => { ... }              // 60 lines
//!         // ... 100 more variants, 1000+ lines total
//!     }
//! }
//! ```
//!
//! ## The Solution
//!
//! Break the monolithic match into composable **routers** that can be combined:
//!
//! ```text
//! Message ──┬──→ GraphRouter ────→ GraphIntent
//!           ├──→ FilterRouter ───→ FilterIntent
//!           ├──→ SearchRouter ───→ SearchIntent
//!           └──→ ExportRouter ───→ ExportIntent
//!
//! Then compose:  compose(graph_router, filter_router)
//!      parallel: parallel(export_router, import_router)
//!       fanout:  fanout(log_router, metrics_router, main_router)
//! ```

use crate::gui::feedback::{GraphFilterIntent, GraphFilterAppState};
use uuid::Uuid;

/// Message types that can be routed
///
/// This is a simplified version of the real app's Message enum,
/// focused on demonstrating routing patterns.
#[derive(Clone, Debug, PartialEq)]
pub enum AppMessage {
    // Graph interaction messages
    ConceptEntityClicked(Uuid),
    ConceptEntityDragged { node_id: Uuid, position: (f32, f32) },
    ConceptRelationCreated { from: Uuid, to: Uuid },
    GraphLayoutChanged(LayoutType),

    // Filter messages
    FilterPeopleToggled(bool),
    FilterOrgsToggled(bool),
    FilterNatsToggled(bool),
    FilterPkiToggled(bool),
    FilterYubikeyToggled(bool),

    // Search messages
    SearchQueryChanged(String),
    SearchCleared,
    SearchResultSelected(Uuid),

    // Export/Import messages
    ExportRequested { path: String, include_keys: bool },
    ImportRequested { path: String },

    // System messages
    ErrorOccurred(String),
    StatusUpdated(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LayoutType {
    Manual,
    Hierarchical,
    ForceDirected,
    Circular,
}

/// Graph-specific intents
#[derive(Clone, Debug, PartialEq)]
pub enum GraphIntent {
    NodeClicked(Uuid),
    NodeDragged { node_id: Uuid, position: (f32, f32) },
    EdgeCreated { from: Uuid, to: Uuid },
    LayoutChanged(LayoutType),
}

/// Export/Import intents
#[derive(Clone, Debug, PartialEq)]
pub enum IoIntent {
    Export { path: String, include_keys: bool },
    Import { path: String },
}

/// System-level intents
#[derive(Clone, Debug, PartialEq)]
pub enum SystemIntent {
    Error(String),
    Status(String),
}

/// A route is a partial function from Message → Intent
///
/// It represents: "I can handle some messages and convert them to intents"
pub struct Route<M, I> {
    handler: Box<dyn Fn(&M) -> Option<I>>,
}

impl<M, I> Route<M, I> {
    /// Create a new route from a handler function
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&M) -> Option<I> + 'static,
    {
        Route {
            handler: Box::new(handler),
        }
    }

    /// Try to route a message
    pub fn route(&self, message: &M) -> Option<I> {
        (self.handler)(message)
    }

    /// Compose this route with another (try this, then that)
    pub fn or_else<F>(self, other: F) -> Route<M, I>
    where
        F: Fn(&M) -> Option<I> + 'static,
        M: 'static,
        I: 'static,
    {
        Route::new(move |msg| {
            self.route(msg).or_else(|| other(msg))
        })
    }
}

/// Router for graph-related messages
pub fn graph_router() -> Route<AppMessage, GraphIntent> {
    Route::new(|msg| match msg {
        AppMessage::ConceptEntityClicked(id) => Some(GraphIntent::NodeClicked(*id)),
        AppMessage::ConceptEntityDragged { node_id, position } => {
            Some(GraphIntent::NodeDragged {
                node_id: *node_id,
                position: *position,
            })
        }
        AppMessage::ConceptRelationCreated { from, to } => {
            Some(GraphIntent::EdgeCreated { from: *from, to: *to })
        }
        AppMessage::GraphLayoutChanged(layout) => {
            Some(GraphIntent::LayoutChanged(layout.clone()))
        }
        _ => None,
    })
}

/// Router for filter-related messages
pub fn filter_router() -> Route<AppMessage, GraphFilterIntent> {
    Route::new(|msg| match msg {
        AppMessage::FilterPeopleToggled(show) => {
            Some(GraphFilterIntent::ToggleCategoryFilter {
                people: Some(*show),
                orgs: None,
                nats: None,
                pki: None,
                yubikey: None,
            })
        }
        AppMessage::FilterOrgsToggled(show) => {
            Some(GraphFilterIntent::ToggleCategoryFilter {
                people: None,
                orgs: Some(*show),
                nats: None,
                pki: None,
                yubikey: None,
            })
        }
        AppMessage::FilterNatsToggled(show) => {
            Some(GraphFilterIntent::ToggleCategoryFilter {
                people: None,
                orgs: None,
                nats: Some(*show),
                pki: None,
                yubikey: None,
            })
        }
        AppMessage::FilterPkiToggled(show) => {
            Some(GraphFilterIntent::ToggleCategoryFilter {
                people: None,
                orgs: None,
                nats: None,
                pki: Some(*show),
                yubikey: None,
            })
        }
        AppMessage::FilterYubikeyToggled(show) => {
            Some(GraphFilterIntent::ToggleCategoryFilter {
                people: None,
                orgs: None,
                nats: None,
                pki: None,
                yubikey: Some(*show),
            })
        }
        _ => None,
    })
}

/// Router for search-related messages
pub fn search_router() -> Route<AppMessage, GraphFilterIntent> {
    Route::new(|msg| match msg {
        AppMessage::SearchQueryChanged(query) => {
            Some(GraphFilterIntent::SearchQueryChanged(query.clone()))
        }
        AppMessage::SearchCleared => Some(GraphFilterIntent::ClearSearch),
        AppMessage::SearchResultSelected(id) => {
            Some(GraphFilterIntent::SelectNode(*id))
        }
        _ => None,
    })
}

/// Router for I/O operations
pub fn io_router() -> Route<AppMessage, IoIntent> {
    Route::new(|msg| match msg {
        AppMessage::ExportRequested { path, include_keys } => {
            Some(IoIntent::Export {
                path: path.clone(),
                include_keys: *include_keys,
            })
        }
        AppMessage::ImportRequested { path } => {
            Some(IoIntent::Import { path: path.clone() })
        }
        _ => None,
    })
}

/// Router for system messages
pub fn system_router() -> Route<AppMessage, SystemIntent> {
    Route::new(|msg| match msg {
        AppMessage::ErrorOccurred(err) => Some(SystemIntent::Error(err.clone())),
        AppMessage::StatusUpdated(status) => Some(SystemIntent::Status(status.clone())),
        _ => None,
    })
}

/// Compose multiple routers that produce the same intent type
///
/// Tries each router in order until one succeeds.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::routing::*;
///
/// // Combine filter and search routers (both produce GraphFilterIntent)
/// let combined = compose_routes(filter_router(), search_router());
///
/// let msg = AppMessage::SearchQueryChanged("alice".to_string());
/// let intent = combined.route(&msg);
/// assert!(intent.is_some());
/// ```
pub fn compose_routes<M, I>(
    first: Route<M, I>,
    second: Route<M, I>,
) -> Route<M, I>
where
    M: 'static,
    I: 'static,
{
    Route::new(move |msg| {
        first.route(msg).or_else(|| second.route(msg))
    })
}

/// Parallel routing: try multiple routers and collect all results
///
/// Unlike compose (which stops at first match), parallel tries all routers
/// and returns all successful results.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::routing::*;
///
/// let msg = AppMessage::StatusUpdated("Ready".to_string());
///
/// // Both routers might handle the same message differently
/// let results = parallel_route(
///     &msg,
///     vec![
///         Box::new(system_router()),
///         // Could have another router that also handles status
///     ]
/// );
/// ```
pub fn parallel_route<M, I>(
    message: &M,
    routers: Vec<Box<Route<M, I>>>,
) -> Vec<I> {
    routers
        .iter()
        .filter_map(|router| router.route(message))
        .collect()
}

/// Fanout: send message to multiple routers with different intent types
///
/// This is useful for cross-cutting concerns like logging, metrics, etc.
///
/// # Example
///
/// ```rust
/// use cim_keys::gui::routing::*;
///
/// let msg = AppMessage::ConceptEntityClicked(uuid::Uuid::now_v7());
///
/// // Route to graph handler AND log it
/// let graph_intent = graph_router().route(&msg);
/// let system_intent = system_router().route(&msg);
/// ```
pub struct Fanout<M> {
    routers: Vec<Box<dyn Fn(&M) -> Vec<String>>>,
}

impl<M> Fanout<M> {
    pub fn new() -> Self {
        Fanout { routers: vec![] }
    }

    pub fn add_router<F>(mut self, router: F) -> Self
    where
        F: Fn(&M) -> Vec<String> + 'static,
    {
        self.routers.push(Box::new(router));
        self
    }

    pub fn route(&self, message: &M) -> Vec<String> {
        self.routers
            .iter()
            .flat_map(|router| router(message))
            .collect()
    }
}

/// Intent classifier: route message to appropriate handler based on category
///
/// This demonstrates a higher-level pattern where we classify messages
/// into categories and dispatch to specialized handlers.
pub struct IntentClassifier {
    pub name: String,
}

impl IntentClassifier {
    pub fn new(name: impl Into<String>) -> Self {
        IntentClassifier { name: name.into() }
    }

    /// Classify a message into a category
    pub fn classify(&self, message: &AppMessage) -> MessageCategory {
        match message {
            AppMessage::ConceptEntityClicked(_)
            | AppMessage::ConceptEntityDragged { .. }
            | AppMessage::ConceptRelationCreated { .. }
            | AppMessage::GraphLayoutChanged(_) => MessageCategory::Graph,

            AppMessage::FilterPeopleToggled(_)
            | AppMessage::FilterOrgsToggled(_)
            | AppMessage::FilterNatsToggled(_)
            | AppMessage::FilterPkiToggled(_)
            | AppMessage::FilterYubikeyToggled(_) => MessageCategory::Filter,

            AppMessage::SearchQueryChanged(_)
            | AppMessage::SearchCleared
            | AppMessage::SearchResultSelected(_) => MessageCategory::Search,

            AppMessage::ExportRequested { .. }
            | AppMessage::ImportRequested { .. } => MessageCategory::Io,

            AppMessage::ErrorOccurred(_)
            | AppMessage::StatusUpdated(_) => MessageCategory::System,
        }
    }

    /// Get the appropriate router for a message category
    pub fn route_by_category(&self, category: MessageCategory) -> String {
        match category {
            MessageCategory::Graph => "GraphRouter".to_string(),
            MessageCategory::Filter => "FilterRouter".to_string(),
            MessageCategory::Search => "SearchRouter".to_string(),
            MessageCategory::Io => "IoRouter".to_string(),
            MessageCategory::System => "SystemRouter".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageCategory {
    Graph,
    Filter,
    Search,
    Io,
    System,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_router() {
        let router = graph_router();
        let node_id = Uuid::now_v7();

        let msg = AppMessage::ConceptEntityClicked(node_id);
        let intent = router.route(&msg);

        assert_eq!(intent, Some(GraphIntent::NodeClicked(node_id)));
    }

    #[test]
    fn test_graph_router_ignores_non_graph_messages() {
        let router = graph_router();
        let msg = AppMessage::SearchQueryChanged("test".to_string());

        let intent = router.route(&msg);
        assert_eq!(intent, None);
    }

    #[test]
    fn test_filter_router() {
        let router = filter_router();
        let msg = AppMessage::FilterPeopleToggled(false);

        let intent = router.route(&msg);
        assert!(intent.is_some());

        if let Some(GraphFilterIntent::ToggleCategoryFilter { people, .. }) = intent {
            assert_eq!(people, Some(false));
        } else {
            panic!("Expected ToggleCategoryFilter intent");
        }
    }

    #[test]
    fn test_search_router() {
        let router = search_router();
        let msg = AppMessage::SearchQueryChanged("alice".to_string());

        let intent = router.route(&msg);
        assert_eq!(
            intent,
            Some(GraphFilterIntent::SearchQueryChanged("alice".to_string()))
        );
    }

    #[test]
    fn test_io_router() {
        let router = io_router();
        let msg = AppMessage::ExportRequested {
            path: "/tmp/export".to_string(),
            include_keys: true,
        };

        let intent = router.route(&msg);
        assert_eq!(
            intent,
            Some(IoIntent::Export {
                path: "/tmp/export".to_string(),
                include_keys: true
            })
        );
    }

    #[test]
    fn test_system_router() {
        let router = system_router();
        let msg = AppMessage::ErrorOccurred("Test error".to_string());

        let intent = router.route(&msg);
        assert_eq!(intent, Some(SystemIntent::Error("Test error".to_string())));
    }

    #[test]
    fn test_compose_routes() {
        let combined = compose_routes(filter_router(), search_router());

        // Should handle filter messages
        let filter_msg = AppMessage::FilterPeopleToggled(true);
        assert!(combined.route(&filter_msg).is_some());

        // Should handle search messages
        let search_msg = AppMessage::SearchQueryChanged("test".to_string());
        assert!(combined.route(&search_msg).is_some());

        // Should reject unhandled messages
        let graph_msg = AppMessage::ConceptEntityClicked(Uuid::now_v7());
        assert!(combined.route(&graph_msg).is_none());
    }

    #[test]
    fn test_intent_classifier() {
        let classifier = IntentClassifier::new("TestClassifier");

        let graph_msg = AppMessage::ConceptEntityClicked(Uuid::now_v7());
        assert_eq!(classifier.classify(&graph_msg), MessageCategory::Graph);

        let filter_msg = AppMessage::FilterPeopleToggled(true);
        assert_eq!(classifier.classify(&filter_msg), MessageCategory::Filter);

        let search_msg = AppMessage::SearchCleared;
        assert_eq!(classifier.classify(&search_msg), MessageCategory::Search);

        let io_msg = AppMessage::ExportRequested {
            path: "/tmp".to_string(),
            include_keys: false,
        };
        assert_eq!(classifier.classify(&io_msg), MessageCategory::Io);

        let system_msg = AppMessage::ErrorOccurred("error".to_string());
        assert_eq!(classifier.classify(&system_msg), MessageCategory::System);
    }

    #[test]
    fn test_route_by_category() {
        let classifier = IntentClassifier::new("TestClassifier");

        assert_eq!(
            classifier.route_by_category(MessageCategory::Graph),
            "GraphRouter"
        );
        assert_eq!(
            classifier.route_by_category(MessageCategory::Filter),
            "FilterRouter"
        );
        assert_eq!(
            classifier.route_by_category(MessageCategory::Search),
            "SearchRouter"
        );
    }

    #[test]
    fn test_or_else_combinator() {
        let filter = filter_router();
        let search = search_router();

        let combined = filter.or_else(move |msg| search.route(msg));

        // Test filter message
        let filter_msg = AppMessage::FilterNatsToggled(false);
        assert!(combined.route(&filter_msg).is_some());

        // Test search message
        let search_msg = AppMessage::SearchCleared;
        assert!(combined.route(&search_msg).is_some());
    }

    #[test]
    fn test_parallel_route() {
        let msg = AppMessage::StatusUpdated("Ready".to_string());

        let results = parallel_route(
            &msg,
            vec![Box::new(system_router())],
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], SystemIntent::Status("Ready".to_string()));
    }
}
