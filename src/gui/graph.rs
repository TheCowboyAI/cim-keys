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

use crate::domain::{
    Person, KeyDelegation, KeyOwnerRole, Organization, OrganizationUnit,
    Location, Policy, Role,
};

/// Graph visualization widget for organizational structure
pub struct OrganizationGraph {
    nodes: HashMap<Uuid, GraphNode>,
    edges: Vec<GraphEdge>,
    selected_node: Option<Uuid>,
    dragging_node: Option<Uuid>,  // Node currently being dragged
    drag_offset: Vector,  // Offset from node center to cursor when dragging started
    _viewport: Rectangle,  // Reserved for graph panning/zooming
    zoom: f32,
    pan_offset: Vector,
}

/// A node in the organization graph (represents any domain entity)
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub position: Point,
    pub color: Color,
    pub label: String,
}

/// Type of node in the graph
#[derive(Debug, Clone)]
pub enum NodeType {
    Organization(Organization),
    OrganizationalUnit(OrganizationUnit),
    Person { person: Person, role: KeyOwnerRole },
    Location(Location),
    Role(Role),
    Policy(Policy),
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
    // Organizational hierarchy
    /// Parent-child relationship (Organization → OrganizationalUnit)
    ParentChild,
    /// Manager relationship (Person → OrganizationalUnit)
    ManagesUnit,
    /// Membership (Person → OrganizationalUnit)
    MemberOf,

    // Key relationships
    /// Key ownership (Person → Key)
    OwnsKey,
    /// Key delegation (Person → Person)
    DelegatesKey(KeyDelegation),
    /// Storage location (Key → Location)
    StoredAt,

    // Policy relationships
    /// Role assignment (Person → Role)
    HasRole,
    /// Policy requirement (Role → Policy)
    RoleRequiresPolicy,
    /// Policy governance (Policy → Entity)
    PolicyGovernsEntity,

    // Trust relationships
    /// Trust relationship (Organization → Organization)
    Trusts,
    /// Certificate authority (Key → Key)
    CertifiedBy,

    // Legacy (for backwards compatibility)
    /// Hierarchical relationship (manager -> report)
    Hierarchy,
    /// Trust relationship (CA -> signed cert)
    Trust,
}

/// Messages for graph interactions
#[derive(Debug, Clone)]
pub enum GraphMessage {
    NodeClicked(Uuid),
    NodeDragStarted { node_id: Uuid, offset: Vector },
    NodeDragged(Point),  // New cursor position
    NodeDragEnded,
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
            dragging_node: None,
            drag_offset: Vector::new(0.0, 0.0),
            _viewport: Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0)),
            zoom: 1.0,
            pan_offset: Vector::new(0.0, 0.0),
        }
    }

    /// Add a person node to the graph
    pub fn add_node(&mut self, person: Person, role: KeyOwnerRole) {
        let node_id = person.id;
        let label = person.name.clone();
        let color = self.role_to_color(&role);

        let node = GraphNode {
            id: node_id,
            node_type: NodeType::Person {
                person: person.clone(),
                role
            },
            position: self.calculate_node_position(node_id),
            color,
            label,
        };

        self.nodes.insert(node_id, node);
    }

    /// Add an organization node to the graph
    pub fn add_organization_node(&mut self, org: Organization) {
        let node_id = org.id;
        let label = org.name.clone();

        let node = GraphNode {
            id: node_id,
            node_type: NodeType::Organization(org),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(0.2, 0.3, 0.6), // Dark blue
            label,
        };

        self.nodes.insert(node_id, node);
    }

    /// Add an organizational unit node to the graph
    pub fn add_org_unit_node(&mut self, unit: OrganizationUnit) {
        let node_id = unit.id;
        let label = unit.name.clone();

        let node = GraphNode {
            id: node_id,
            node_type: NodeType::OrganizationalUnit(unit),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(0.4, 0.5, 0.8), // Light blue
            label,
        };

        self.nodes.insert(node_id, node);
    }

    /// Add a location node to the graph
    pub fn add_location_node(&mut self, location: Location) {
        let node_id = location.id;
        let label = location.name.clone();

        let node = GraphNode {
            id: node_id,
            node_type: NodeType::Location(location),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(0.6, 0.5, 0.4), // Brown/gray
            label,
        };

        self.nodes.insert(node_id, node);
    }

    /// Add a role node to the graph
    pub fn add_role_node(&mut self, role: Role) {
        let node_id = role.id;
        let label = role.name.clone();

        let node = GraphNode {
            id: node_id,
            node_type: NodeType::Role(role),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(0.6, 0.3, 0.8), // Purple
            label,
        };

        self.nodes.insert(node_id, node);
    }

    /// Add a policy node to the graph
    pub fn add_policy_node(&mut self, policy: Policy) {
        let node_id = policy.id;
        let label = policy.name.clone();

        let node = GraphNode {
            id: node_id,
            node_type: NodeType::Policy(policy),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(0.9, 0.7, 0.2), // Gold/yellow
            label,
        };

        self.nodes.insert(node_id, node);
    }

    pub fn add_edge(&mut self, from: Uuid, to: Uuid, edge_type: EdgeType) {
        let color = match &edge_type {
            // Organizational hierarchy - blues
            EdgeType::ParentChild => Color::from_rgb(0.2, 0.4, 0.8),
            EdgeType::ManagesUnit => Color::from_rgb(0.4, 0.2, 0.8),
            EdgeType::MemberOf => Color::from_rgb(0.5, 0.5, 0.5),

            // Key relationships - greens
            EdgeType::OwnsKey => Color::from_rgb(0.2, 0.7, 0.2),
            EdgeType::DelegatesKey(_) => Color::from_rgb(0.9, 0.6, 0.2),
            EdgeType::StoredAt => Color::from_rgb(0.6, 0.5, 0.4),

            // Policy relationships - gold/yellow
            EdgeType::HasRole => Color::from_rgb(0.6, 0.3, 0.8),
            EdgeType::RoleRequiresPolicy => Color::from_rgb(0.9, 0.7, 0.2),
            EdgeType::PolicyGovernsEntity => Color::from_rgb(0.9, 0.7, 0.2),

            // Trust relationships
            EdgeType::Trusts => Color::from_rgb(0.7, 0.5, 0.3),
            EdgeType::CertifiedBy => Color::from_rgb(0.7, 0.5, 0.3),

            // Legacy
            EdgeType::Hierarchy => Color::from_rgb(0.3, 0.3, 0.7),
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
        let node_count = self.nodes.len();
        if node_count == 0 {
            return;
        }

        // Use hierarchical layout based on roles if we have few nodes
        // Otherwise use force-directed layout
        if node_count <= 10 {
            self.hierarchical_layout();
        } else {
            self.force_directed_layout();
        }
    }

    /// Hierarchical layout: organize nodes by type and role
    fn hierarchical_layout(&mut self) {
        let center = Point { x: 400.0, y: 300.0 };

        // Group nodes by type
        let mut type_groups: HashMap<String, Vec<Uuid>> = HashMap::new();
        for (id, node) in &self.nodes {
            let type_key = match &node.node_type {
                NodeType::Organization(_) => "Organization",
                NodeType::OrganizationalUnit(_) => "OrganizationalUnit",
                NodeType::Person { role, .. } => match role {
                    KeyOwnerRole::RootAuthority => "Person_RootAuthority",
                    KeyOwnerRole::SecurityAdmin => "Person_SecurityAdmin",
                    KeyOwnerRole::BackupHolder => "Person_BackupHolder",
                    KeyOwnerRole::Auditor => "Person_Auditor",
                    KeyOwnerRole::Developer => "Person_Developer",
                    KeyOwnerRole::ServiceAccount => "Person_ServiceAccount",
                },
                NodeType::Location(_) => "Location",
                NodeType::Role(_) => "Role",
                NodeType::Policy(_) => "Policy",
            };
            type_groups.entry(type_key.to_string()).or_insert_with(Vec::new).push(*id);
        }

        // Define node type hierarchy (top to bottom)
        let type_order = vec![
            "Organization",
            "OrganizationalUnit",
            "Role",
            "Policy",
            "Person_RootAuthority",
            "Person_SecurityAdmin",
            "Person_BackupHolder",
            "Person_Auditor",
            "Person_Developer",
            "Person_ServiceAccount",
            "Location",
        ];

        let mut y_offset = 100.0;
        let y_spacing = 120.0;

        for type_name in type_order {
            if let Some(node_ids) = type_groups.get(type_name) {
                let x_spacing = 150.0;
                let total_width = (node_ids.len() as f32 - 1.0) * x_spacing;
                let start_x = center.x - total_width / 2.0;

                for (i, &node_id) in node_ids.iter().enumerate() {
                    if let Some(node) = self.nodes.get_mut(&node_id) {
                        node.position = Point {
                            x: start_x + (i as f32) * x_spacing,
                            y: y_offset,
                        };
                    }
                }

                y_offset += y_spacing;
            }
        }
    }

    /// Force-directed layout: physics-based layout for larger graphs
    fn force_directed_layout(&mut self) {
        // Fruchterman-Reingold algorithm
        let width = 800.0;
        let height = 600.0;
        let area = width * height;
        let k = (area / self.nodes.len() as f32).sqrt(); // Optimal distance

        // Initialize random positions if needed
        for node in self.nodes.values_mut() {
            if node.position.x == 0.0 && node.position.y == 0.0 {
                node.position = Point {
                    x: (rand::random::<f32>() * width),
                    y: (rand::random::<f32>() * height),
                };
            }
        }

        // Run simulation for N iterations
        let iterations = 50;
        let mut temperature = width / 10.0;

        for _ in 0..iterations {
            // Calculate repulsive forces between all pairs of nodes
            let node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
            let mut displacements: HashMap<Uuid, Vector> = HashMap::new();

            for &v in &node_ids {
                displacements.insert(v, Vector::new(0.0, 0.0));
            }

            // Repulsive forces (all pairs)
            for i in 0..node_ids.len() {
                for j in (i + 1)..node_ids.len() {
                    let id_v = node_ids[i];
                    let id_u = node_ids[j];

                    if let (Some(node_v), Some(node_u)) = (
                        self.nodes.get(&id_v),
                        self.nodes.get(&id_u),
                    ) {
                        let delta = Vector::new(
                            node_v.position.x - node_u.position.x,
                            node_v.position.y - node_u.position.y,
                        );

                        let distance = (delta.x * delta.x + delta.y * delta.y).sqrt().max(0.01);
                        let repulsion = k * k / distance;

                        let force = Vector::new(
                            (delta.x / distance) * repulsion,
                            (delta.y / distance) * repulsion,
                        );

                        *displacements.get_mut(&id_v).unwrap() =
                            *displacements.get(&id_v).unwrap() + force;
                        *displacements.get_mut(&id_u).unwrap() =
                            *displacements.get(&id_u).unwrap() - force;
                    }
                }
            }

            // Attractive forces (edges only)
            for edge in &self.edges {
                if let (Some(node_from), Some(node_to)) = (
                    self.nodes.get(&edge.from),
                    self.nodes.get(&edge.to),
                ) {
                    let delta = Vector::new(
                        node_to.position.x - node_from.position.x,
                        node_to.position.y - node_from.position.y,
                    );

                    let distance = (delta.x * delta.x + delta.y * delta.y).sqrt().max(0.01);
                    let attraction = distance * distance / k;

                    let force = Vector::new(
                        (delta.x / distance) * attraction,
                        (delta.y / distance) * attraction,
                    );

                    *displacements.get_mut(&edge.from).unwrap() =
                        *displacements.get(&edge.from).unwrap() + force;
                    *displacements.get_mut(&edge.to).unwrap() =
                        *displacements.get(&edge.to).unwrap() - force;
                }
            }

            // Apply displacements with cooling
            for (&id, displacement) in &displacements {
                if let Some(node) = self.nodes.get_mut(&id) {
                    let length = (displacement.x * displacement.x + displacement.y * displacement.y).sqrt();
                    if length > 0.0 {
                        let capped = length.min(temperature);
                        node.position.x += (displacement.x / length) * capped;
                        node.position.y += (displacement.y / length) * capped;

                        // Keep within bounds
                        node.position.x = node.position.x.max(50.0).min(width - 50.0);
                        node.position.y = node.position.y.max(50.0).min(height - 50.0);
                    }
                }
            }

            // Cool down
            temperature *= 0.95;
        }
    }

    pub fn handle_message(&mut self, message: GraphMessage) {
        match message {
            GraphMessage::NodeClicked(id) => self.selected_node = Some(id),
            GraphMessage::NodeDragStarted { node_id, offset } => {
                self.dragging_node = Some(node_id);
                self.drag_offset = offset;
            }
            GraphMessage::NodeDragged(cursor_pos) => {
                if let Some(node_id) = self.dragging_node {
                    if let Some(node) = self.nodes.get_mut(&node_id) {
                        // Adjust for zoom and pan transformations
                        let adjusted_x = (cursor_pos.x - self.pan_offset.x) / self.zoom;
                        let adjusted_y = (cursor_pos.y - self.pan_offset.y) / self.zoom;

                        node.position = Point::new(
                            adjusted_x - self.drag_offset.x,
                            adjusted_y - self.drag_offset.y,
                        );
                    }
                }
            }
            GraphMessage::NodeDragEnded => {
                self.dragging_node = None;
                self.drag_offset = Vector::new(0.0, 0.0);
            }
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

                // Draw edge label
                let edge_label = match &edge.edge_type {
                    // Organizational
                    EdgeType::ParentChild => "parent of",
                    EdgeType::ManagesUnit => "manages",
                    EdgeType::MemberOf => "member of",

                    // Key relationships
                    EdgeType::OwnsKey => "owns",
                    EdgeType::DelegatesKey(_) => "delegates to",
                    EdgeType::StoredAt => "stored at",

                    // Policy relationships
                    EdgeType::HasRole => "has role",
                    EdgeType::RoleRequiresPolicy => "requires",
                    EdgeType::PolicyGovernsEntity => "governs",

                    // Trust
                    EdgeType::Trusts => "trusts",
                    EdgeType::CertifiedBy => "certified by",

                    // Legacy
                    EdgeType::Hierarchy => "reports to",
                    EdgeType::Trust => "trusts",
                };

                // Calculate midpoint for label
                let mid_x = (from_node.position.x + to_node.position.x) / 2.0;
                let mid_y = (from_node.position.y + to_node.position.y) / 2.0;

                // Offset label perpendicular to edge
                let perp_angle = angle + std::f32::consts::PI / 2.0;
                let offset = 15.0;
                let label_position = Point::new(
                    mid_x + offset * perp_angle.cos(),
                    mid_y + offset * perp_angle.sin(),
                );

                // Draw label background
                let label_bg = canvas::Path::rectangle(
                    Point::new(label_position.x - 40.0, label_position.y - 8.0),
                    iced::Size::new(80.0, 16.0),
                );
                frame.fill(&label_bg, Color::from_rgba(1.0, 1.0, 1.0, 0.9));
                frame.stroke(&label_bg, canvas::Stroke::default()
                    .with_color(edge.color)
                    .with_width(1.0));

                // Draw label text
                frame.fill_text(canvas::Text {
                    content: edge_label.to_string(),
                    position: label_position,
                    color: Color::from_rgb(0.2, 0.2, 0.2),
                    size: iced::Pixels(10.0),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    line_height: LineHeight::default(),
                    shaping: Shaping::Advanced,
                });
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

            // Draw node properties as multi-line text based on node type
            let (type_label, primary_text, secondary_text) = match &node.node_type {
                NodeType::Organization(org) => (
                    "Organization",
                    org.name.clone(),
                    org.display_name.clone(),
                ),
                NodeType::OrganizationalUnit(unit) => (
                    "Unit",
                    unit.name.clone(),
                    format!("{:?}", unit.unit_type),
                ),
                NodeType::Person { person, role } => {
                    let role_str = match role {
                        KeyOwnerRole::RootAuthority => "Root CA",
                        KeyOwnerRole::SecurityAdmin => "Security Admin",
                        KeyOwnerRole::Developer => "Developer",
                        KeyOwnerRole::ServiceAccount => "Service",
                        KeyOwnerRole::BackupHolder => "Backup",
                        KeyOwnerRole::Auditor => "Auditor",
                    };
                    (role_str, person.name.clone(), person.email.clone())
                },
                NodeType::Location(loc) => (
                    "Location",
                    loc.name.clone(),
                    format!("{:?}", loc.location_type),
                ),
                NodeType::Role(role) => (
                    "Role",
                    role.name.clone(),
                    role.description.clone(),
                ),
                NodeType::Policy(policy) => (
                    "Policy",
                    policy.name.clone(),
                    format!("{} claims", policy.claims.len()),
                ),
            };

            // Primary text (below node)
            let name_position = Point::new(
                node.position.x,
                node.position.y + radius + 12.0,
            );
            frame.fill_text(canvas::Text {
                content: primary_text,
                position: name_position,
                color: Color::BLACK,
                size: iced::Pixels(13.0),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Top,
                line_height: LineHeight::default(),
                shaping: Shaping::Advanced,
            });

            // Secondary text (below primary)
            let email_position = Point::new(
                node.position.x,
                node.position.y + radius + 27.0,
            );
            frame.fill_text(canvas::Text {
                content: secondary_text,
                position: email_position,
                color: Color::from_rgb(0.4, 0.4, 0.4),
                size: iced::Pixels(10.0),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Top,
                line_height: LineHeight::default(),
                shaping: Shaping::Advanced,
            });

            // Type label (above node)
            let role_position = Point::new(
                node.position.x,
                node.position.y - radius - 8.0,
            );
            frame.fill_text(canvas::Text {
                content: type_label.to_string(),
                position: role_position,
                color: node.color,
                size: iced::Pixels(11.0),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Bottom,
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
                    // Check if click is on a node - start dragging
                    for (node_id, node) in &self.nodes {
                        let distance = ((adjusted_position.x - node.position.x).powi(2)
                            + (adjusted_position.y - node.position.y).powi(2))
                        .sqrt();

                        if distance <= 20.0 {
                            // Calculate offset from node center to cursor
                            let offset = Vector::new(
                                adjusted_position.x - node.position.x,
                                adjusted_position.y - node.position.y,
                            );
                            return (
                                canvas::event::Status::Captured,
                                Some(GraphMessage::NodeDragStarted {
                                    node_id: *node_id,
                                    offset
                                }),
                            );
                        }
                    }
                }
                canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    // End dragging if we were dragging
                    if self.dragging_node.is_some() {
                        return (
                            canvas::event::Status::Captured,
                            Some(GraphMessage::NodeDragEnded),
                        );
                    }
                }
                canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    // Continue dragging if we're dragging a node
                    if self.dragging_node.is_some() {
                        return (
                            canvas::event::Status::Captured,
                            Some(GraphMessage::NodeDragged(cursor_position)),
                        );
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
            let details = match &node.node_type {
                NodeType::Organization(org) => column![
                    text("Selected Organization:").size(16),
                    text(format!("Name: {}", org.name)),
                    text(format!("Display Name: {}", org.display_name)),
                    text(format!("Units: {}", org.units.len())),
                ],
                NodeType::OrganizationalUnit(unit) => column![
                    text("Selected Unit:").size(16),
                    text(format!("Name: {}", unit.name)),
                    text(format!("Type: {:?}", unit.unit_type)),
                ],
                NodeType::Person { person, role } => column![
                    text("Selected Person:").size(16),
                    text(format!("Name: {}", person.name)),
                    text(format!("Email: {}", person.email)),
                    text(format!("Role: {:?}", role)),
                ],
                NodeType::Location(loc) => column![
                    text("Selected Location:").size(16),
                    text(format!("Name: {}", loc.name)),
                    text(format!("Type: {:?}", loc.location_type)),
                    text(format!("Security: {:?}", loc.security_level)),
                ],
                NodeType::Role(role) => column![
                    text("Selected Role:").size(16),
                    text(format!("Name: {}", role.name)),
                    text(format!("Description: {}", role.description)),
                    text(format!("Required Policies: {}", role.required_policies.len())),
                ],
                NodeType::Policy(policy) => column![
                    text("Selected Policy:").size(16),
                    text(format!("Name: {}", policy.name)),
                    text(format!("Claims: {}", policy.claims.len())),
                    text(format!("Conditions: {}", policy.conditions.len())),
                    text(format!("Priority: {}", policy.priority)),
                    text(format!("Enabled: {}", policy.enabled)),
                ],
            };
            items = items.push(details.spacing(5));
        }
    }

    container(items)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}