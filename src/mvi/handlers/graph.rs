// Copyright (c) 2025 - Cowboy AI, LLC.

//! Graph Intent Handlers
//!
//! Pure handlers for graph manipulation intents.
//!
//! ## Subject Patterns
//!
//! - `ui.graph.node.*` - Node operations
//! - `ui.graph.edge.*` - Edge operations
//! - `ui.graph.properties.*` - Property editing
//! - `ui.graph.layout` - Auto-layout

use super::{Model, HandlerResult};
use iced::Task;

/// Handle graph create node
pub fn handle_create_node(model: Model, node_type: String, position: (f32, f32)) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Creating {} node at ({}, {})",
        node_type, position.0, position.1
    ));
    (updated, Task::none())
}

/// Handle graph edge creation started
pub fn handle_create_edge_started(model: Model, from_node: String) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Edge creation started from node {}",
        from_node
    ));
    (updated, Task::none())
}

/// Handle graph edge creation completed
pub fn handle_create_edge_completed(
    model: Model,
    from: String,
    to: String,
    edge_type: String,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Edge created: {} -> {} ({})",
        from, to, edge_type
    ));
    (updated, Task::none())
}

/// Handle graph edge creation cancelled
pub fn handle_create_edge_cancelled(model: Model) -> HandlerResult {
    let updated = model.with_status_message("Edge creation cancelled".to_string());
    (updated, Task::none())
}

/// Handle concept entity clicked
pub fn handle_entity_clicked(model: Model, node_id: String) -> HandlerResult {
    let updated = model.with_status_message(format!("Node selected: {}", node_id));
    (updated, Task::none())
}

/// Handle graph delete node
pub fn handle_delete_node(model: Model, node_id: String) -> HandlerResult {
    let updated = model.with_status_message(format!("Deleting node: {}", node_id));
    (updated, Task::none())
}

/// Handle graph delete edge
pub fn handle_delete_edge(model: Model, from: String, to: String) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Deleting edge: {} -> {}",
        from, to
    ));
    (updated, Task::none())
}

/// Handle graph edit node properties
pub fn handle_edit_node_properties(model: Model, node_id: String) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Editing properties for node: {}",
        node_id
    ));
    (updated, Task::none())
}

/// Handle graph property changed
pub fn handle_property_changed(
    model: Model,
    node_id: String,
    property: String,
    value: String,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Property changed: {}.{} = {}",
        node_id, property, value
    ));
    (updated, Task::none())
}

/// Handle graph properties saved
pub fn handle_properties_saved(model: Model, node_id: String) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Properties saved for node: {}",
        node_id
    ));
    (updated, Task::none())
}

/// Handle graph properties cancelled
pub fn handle_properties_cancelled(model: Model) -> HandlerResult {
    let updated = model.with_status_message("Property editing cancelled".to_string());
    (updated, Task::none())
}

/// Handle graph auto-layout
pub fn handle_auto_layout(model: Model) -> HandlerResult {
    let updated = model.with_status_message("Auto-layout applied".to_string());
    (updated, Task::none())
}
