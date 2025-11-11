//! Organizational graph visualization for key ownership and delegation
//!
//! This module provides a graph view where:
//! - Nodes represent people in the organization
//! - Edges represent relationships and key delegations
//! - Different colors indicate different roles and trust levels

use iced::{
    widget::{canvas, container, column, row, text, button, Canvas},
    Color, Element, Length, Point, Rectangle, Size, Theme, Vector,
    mouse, Renderer,
};
use iced::widget::text::{LineHeight, Shaping};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Person, KeyDelegation, KeyOwnerRole};

/// Graph visualization widget for organizational structure
pub struct OrganizationGraph {
    nodes: HashMap<Uuid, GraphNode>,
    edges: Vec<GraphEdge>,
    selected_node: Option<Uuid>,
    _viewport: Rectangle,  // Reserved for graph panning/zooming
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
    AutoLayout,
    AddEdge { from: Uuid, to: Uuid, edge_type: EdgeType },
}

impl Default for OrganizationGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganizationGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            selected_node: None,
            _viewport: Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0)),
            zoom: 1.0,
            pan_offset: Vector::new(0.0, 0.0),
        }
    }

    pub fn add_node(&mut self, person: Person, role: KeyOwnerRole) {
        let node = GraphNode {
            person: person.clone(),
            role: role,
            position: self.calculate_node_position(person.id),
            color: self.role_to_color(&role),
            keys_owned: 0,
            keys_delegated_to: 0,
            keys_delegated_from: 0,
        };

        self.nodes.insert(person.id, node);
    }

    pub fn add_edge(&mut self, from: Uuid, to: Uuid, edge_type: EdgeType) {
        let color = match &edge_type {
            EdgeType::Hierarchy => Color::from_rgb(0.3, 0.3, 0.7),
            EdgeType::Delegation(_) => Color::from_rgb(0.3, 0.7, 0.3),
            EdgeType::Trust => Color::from_rgb(0.7, 0.5, 0.3),
        };
        self.edges.push(GraphEdge {
            from,
            to,
            edge_type,
            color,
        });
    }

    pub fn select_node(&mut self, node_id: Uuid) {
        self.selected_node = Some(node_id);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn auto_layout(&mut self) {
        // Simple circular layout for nodes
        let node_count = self.nodes.len();
        if node_count == 0 {
            return;
        }

        let center = Point { x: 400.0, y: 300.0 };
        let radius = 200.0;

        for (i, node) in self.nodes.values_mut().enumerate() {
            let angle = (i as f32) * (2.0 * std::f32::consts::PI) / (node_count as f32);
            node.position = Point {
                x: center.x + radius * angle.cos(),
                y: center.y + radius * angle.sin(),
            };
        }
    }

    pub fn handle_message(&mut self, message: GraphMessage) {
        match message {
            GraphMessage::NodeClicked(id) => self.selected_node = Some(id),
            GraphMessage::EdgeClicked { from: _, to: _ } => {}
            GraphMessage::ZoomIn => self.zoom = (self.zoom * 1.2).min(3.0),
            GraphMessage::ZoomOut => self.zoom = (self.zoom / 1.2).max(0.3),
            GraphMessage::ResetView => {
                self.zoom = 1.0;
                self.pan_offset = Vector::new(0.0, 0.0);
            }
            GraphMessage::Pan(delta) => {
                self.pan_offset = self.pan_offset + delta;
            }
            GraphMessage::AutoLayout => {
                self.auto_layout();
            }
            GraphMessage::AddEdge { from, to, edge_type } => {
                self.add_edge(from, to, edge_type);
            }
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
            KeyOwnerRole::RootAuthority => Color::from_rgb(0.8, 0.2, 0.2),
            KeyOwnerRole::SecurityAdmin => Color::from_rgb(0.2, 0.2, 0.8),
            KeyOwnerRole::Developer => Color::from_rgb(0.5, 0.5, 0.8),
            KeyOwnerRole::ServiceAccount => Color::from_rgb(0.2, 0.8, 0.2),
            KeyOwnerRole::BackupHolder => Color::from_rgb(0.8, 0.5, 0.2),
            KeyOwnerRole::Auditor => Color::from_rgb(0.8, 0.8, 0.2),
        }
    }
}

/// Implementation of canvas::Program for graph rendering
impl canvas::Program<GraphMessage> for OrganizationGraph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Apply zoom and pan transformations
        frame.translate(self.pan_offset);
        frame.scale(self.zoom);

        // Draw edges first (so they appear behind nodes)
        for edge in &self.edges {
            if let (Some(from_node), Some(to_node)) = (
                self.nodes.get(&edge.from),
                self.nodes.get(&edge.to),
            ) {
                let edge_path = canvas::Path::line(
                    from_node.position,
                    to_node.position,
                );

                let stroke = canvas::Stroke::default()
                    .with_color(edge.color)
                    .with_width(2.0);

                frame.stroke(&edge_path, stroke);

                // Draw arrow head for directed edges
                let dx = to_node.position.x - from_node.position.x;
                let dy = to_node.position.y - from_node.position.y;
                let angle = dy.atan2(dx);

                let arrow_size = 10.0;
                let arrow_point1 = Point::new(
                    to_node.position.x - arrow_size * (angle - 0.5).cos(),
                    to_node.position.y - arrow_size * (angle - 0.5).sin(),
                );
                let arrow_point2 = Point::new(
                    to_node.position.x - arrow_size * (angle + 0.5).cos(),
                    to_node.position.y - arrow_size * (angle + 0.5).sin(),
                );

                let arrow = canvas::Path::new(|builder| {
                    builder.move_to(to_node.position);
                    builder.line_to(arrow_point1);
                    builder.move_to(to_node.position);
                    builder.line_to(arrow_point2);
                });

                frame.stroke(&arrow, stroke);
            }
        }

        // Draw nodes
        for (node_id, node) in &self.nodes {
            let is_selected = self.selected_node == Some(*node_id);
            let radius = if is_selected { 25.0 } else { 20.0 };

            // Draw node circle
            let circle = canvas::Path::circle(node.position, radius);
            frame.fill(&circle, node.color);

            // Draw selection ring if selected
            if is_selected {
                let selection_ring = canvas::Path::circle(node.position, radius + 3.0);
                let stroke = canvas::Stroke::default()
                    .with_color(Color::from_rgb(1.0, 1.0, 0.0))
                    .with_width(3.0);
                frame.stroke(&selection_ring, stroke);
            }

            // Draw border
            let border_stroke = canvas::Stroke::default()
                .with_color(Color::BLACK)
                .with_width(if is_selected { 2.0 } else { 1.0 });
            frame.stroke(&circle, border_stroke);

            // Draw text label
            let label_position = Point::new(
                node.position.x,
                node.position.y + radius + 15.0,
            );

            frame.fill_text(canvas::Text {
                content: node.person.name.clone(),
                position: label_position,
                color: Color::BLACK,
                size: iced::Pixels(12.0),
                font: iced::Font::default(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Top,
                line_height: LineHeight::default(),
                shaping: Shaping::Advanced,
            });
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        _bounds: Rectangle,  // Reserved for hit testing within bounds
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<GraphMessage>) {
        if let mouse::Cursor::Available(cursor_position) = cursor {
            // Adjust cursor position for zoom and pan
            let adjusted_position = Point::new(
                (cursor_position.x - self.pan_offset.x) / self.zoom,
                (cursor_position.y - self.pan_offset.y) / self.zoom,
            );

            match event {
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    // Check if click is on a node
                    for (node_id, node) in &self.nodes {
                        let distance = ((adjusted_position.x - node.position.x).powi(2)
                            + (adjusted_position.y - node.position.y).powi(2))
                        .sqrt();

                        if distance <= 20.0 {
                            return (
                                canvas::event::Status::Captured,
                                Some(GraphMessage::NodeClicked(*node_id)),
                            );
                        }
                    }
                }
                canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            if y > 0.0 {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(GraphMessage::ZoomIn),
                                );
                            } else if y < 0.0 {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(GraphMessage::ZoomOut),
                                );
                            }
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => {
                            if y > 0.0 {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(GraphMessage::ZoomIn),
                                );
                            } else if y < 0.0 {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(GraphMessage::ZoomOut),
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        (canvas::event::Status::Ignored, None)
    }

}

/// Create a view element for the graph
pub fn view_graph(graph: &OrganizationGraph) -> Element<'_, GraphMessage> {
    // Full Canvas-based graph visualization
    let canvas = Canvas::new(graph)
        .width(Length::Fill)
        .height(Length::Fixed(500.0));

    let controls = row![
        button("Zoom In").on_press(GraphMessage::ZoomIn),
        button("Zoom Out").on_press(GraphMessage::ZoomOut),
        button("Reset").on_press(GraphMessage::ResetView),
        button("Auto Layout").on_press(GraphMessage::AutoLayout),
    ]
    .spacing(10);

    let mut items = column![
        controls,
        canvas,
    ]
    .spacing(10);


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