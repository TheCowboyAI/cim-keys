//! Organizational graph visualization for key ownership and delegation
//!
//! This module provides a graph view where:
//! - Nodes represent people in the organization
//! - Edges represent relationships and key delegations
//! - Different colors indicate different roles and trust levels

use iced::{
    widget::{container, column, row, text, button, scrollable},
    Color, Element, Length, Point, Rectangle, Size, Theme, Vector,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Person, KeyOwnership, KeyDelegation, KeyOwnerRole};

/// Graph visualization widget for organizational structure
pub struct OrganizationGraph {
    nodes: HashMap<Uuid, GraphNode>,
    edges: Vec<GraphEdge>,
    selected_node: Option<Uuid>,
    viewport: Rectangle,
    zoom: f32,
    pan_offset: Vector,
}

/// A node in the organization graph (represents a person)
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub person: Person,
    pub role: KeyOwnerRole,
    pub position: Point,
    pub color: Color,
    pub keys_owned: usize,
    pub keys_delegated_to: usize,
    pub keys_delegated_from: usize,
}

/// An edge in the organization graph (represents relationship/delegation)
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub edge_type: EdgeType,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    /// Hierarchical relationship (manager -> report)
    Hierarchy,
    /// Key delegation (owner -> delegate)
    Delegation(KeyDelegation),
    /// Trust relationship (CA -> signed cert)
    Trust,
}

/// Messages for graph interactions
#[derive(Debug, Clone)]
pub enum GraphMessage {
    NodeClicked(Uuid),
    EdgeClicked { from: Uuid, to: Uuid },
    ZoomIn,
    ZoomOut,
    ResetView,
    Pan(Vector),
}

impl OrganizationGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            selected_node: None,
            viewport: Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0)),
            zoom: 1.0,
            pan_offset: Vector::new(0.0, 0.0),
        }
    }

    pub fn add_node(&mut self, person: Person, role: KeyOwnerRole) {
        let node = GraphNode {
            person: person.clone(),
            role: role.clone(),
            position: self.calculate_node_position(person.id),
            color: self.role_to_color(&role),
            keys_owned: 0,
            keys_delegated_to: 0,
            keys_delegated_from: 0,
        };

        self.nodes.insert(person.id, node);
    }

    pub fn add_edge(&mut self, from: Uuid, to: Uuid, edge_type: EdgeType) {
        self.edges.push(GraphEdge {
            from,
            to,
            edge_type,
            color: match edge_type {
                EdgeType::Hierarchy => Color::from_rgb(0.3, 0.3, 0.7),
                EdgeType::Delegation(_) => Color::from_rgb(0.3, 0.7, 0.3),
                EdgeType::Trust => Color::from_rgb(0.7, 0.5, 0.3),
            },
        });
    }

    pub fn select_node(&mut self, node_id: Uuid) {
        self.selected_node = Some(node_id);
    }

    pub fn handle_message(&mut self, message: GraphMessage) {
        match message {
            GraphMessage::NodeClicked(id) => self.selected_node = Some(id),
            GraphMessage::ZoomIn => self.zoom = (self.zoom * 1.2).min(3.0),
            GraphMessage::ZoomOut => self.zoom = (self.zoom / 1.2).max(0.3),
            GraphMessage::ResetView => {
                self.zoom = 1.0;
                self.pan_offset = Vector::new(0.0, 0.0);
            }
            GraphMessage::Pan(delta) => {
                self.pan_offset = self.pan_offset + delta;
            }
            _ => {}
        }
    }

    fn calculate_node_position(&self, _id: Uuid) -> Point {
        // Simple circular layout for now
        let index = self.nodes.len() as f32;
        let radius = 200.0;
        let angle = index * 2.0 * std::f32::consts::PI / 10.0;

        Point::new(
            400.0 + radius * angle.cos(),
            300.0 + radius * angle.sin(),
        )
    }

    fn role_to_color(&self, role: &KeyOwnerRole) -> Color {
        match role {
            KeyOwnerRole::SecurityOfficer => Color::from_rgb(0.8, 0.2, 0.2),
            KeyOwnerRole::SystemAdministrator => Color::from_rgb(0.2, 0.2, 0.8),
            KeyOwnerRole::KeyCustodian => Color::from_rgb(0.2, 0.8, 0.2),
            KeyOwnerRole::Auditor => Color::from_rgb(0.8, 0.8, 0.2),
            KeyOwnerRole::Developer => Color::from_rgb(0.5, 0.5, 0.8),
            KeyOwnerRole::BackupOperator => Color::from_rgb(0.8, 0.5, 0.2),
        }
    }
}

/// Create a view element for the graph
///
/// Note: Canvas API has changed in Iced 0.13. For now, we provide a
/// simplified view. Full canvas implementation will be added when the
/// new Canvas API is better documented.
pub fn view_graph(graph: &OrganizationGraph) -> Element<'_, GraphMessage> {
    // Temporary simplified view until canvas API is updated
    let mut items = column![];

    // Add header with graph info
    items = items.push(
        row![
            text(format!("Organization Graph: {} people", graph.nodes.len())),
            button("Zoom In").on_press(GraphMessage::ZoomIn),
            button("Zoom Out").on_press(GraphMessage::ZoomOut),
            button("Reset").on_press(GraphMessage::ResetView),
        ]
        .spacing(10)
    );

    // List nodes as a temporary visualization
    let mut node_list = column![].spacing(5);
    for (id, node) in &graph.nodes {
        let is_selected = graph.selected_node == Some(*id);
        let style = if is_selected { ">>> " } else { "    " };

        node_list = node_list.push(
            button(
                text(format!("{}{} ({})",
                    style,
                    node.person.name,
                    format!("{:?}", node.role)
                ))
            )
            .on_press(GraphMessage::NodeClicked(*id))
        );
    }

    // Add the node list in a scrollable container
    items = items.push(
        scrollable(node_list)
            .height(Length::Fixed(400.0))
    );

    // Show selected node details
    if let Some(selected_id) = graph.selected_node {
        if let Some(node) = graph.nodes.get(&selected_id) {
            items = items.push(
                column![
                    text("Selected Person:").size(16),
                    text(format!("Name: {}", node.person.name)),
                    text(format!("Email: {}", node.person.email)),
                    text(format!("Role: {:?}", node.role)),
                    text(format!("Keys Owned: {}", node.keys_owned)),
                ]
                .spacing(5)
            );
        }
    }

    container(items)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}