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
use crate::domain_projections::NatsIdentityProjection;
use super::edge_indicator::EdgeCreationIndicator;
use super::graph_events::{EventStack, GraphEvent};
use super::GraphLayout;
use super::cowboy_theme::CowboyAppTheme as CowboyCustomTheme;
use super::domain_node::{DomainNode, DomainNodeData, Injection};

/// Role badge for compact display mode (shown on person nodes)
#[derive(Debug, Clone)]
pub struct RoleBadge {
    pub name: String,
    pub separation_class: crate::policy::SeparationClass,
    pub level: u8,
}

impl RoleBadge {
    /// Get the badge color based on separation class
    pub fn color(&self) -> Color {
        match self.separation_class {
            crate::policy::SeparationClass::Operational => Color::from_rgb(0.3, 0.6, 0.9), // Blue
            crate::policy::SeparationClass::Administrative => Color::from_rgb(0.6, 0.4, 0.8), // Purple
            crate::policy::SeparationClass::Audit => Color::from_rgb(0.2, 0.7, 0.5), // Teal
            crate::policy::SeparationClass::Emergency => Color::from_rgb(0.9, 0.3, 0.2), // Red
            crate::policy::SeparationClass::Financial => Color::from_rgb(0.9, 0.7, 0.2), // Gold
            crate::policy::SeparationClass::Personnel => Color::from_rgb(0.8, 0.4, 0.6), // Rose
        }
    }

    /// Get abbreviated name (max 3 chars)
    pub fn abbrev(&self) -> String {
        if self.name.len() <= 3 {
            self.name.clone()
        } else {
            // Take first letters of each word or first 3 chars
            let words: Vec<&str> = self.name.split_whitespace().collect();
            if words.len() >= 2 {
                words.iter().take(3).map(|w| w.chars().next().unwrap_or(' ')).collect()
            } else {
                self.name.chars().take(3).collect()
            }
        }
    }
}

/// Person role badges storage (for compact mode)
#[derive(Debug, Clone, Default)]
pub struct PersonRoleBadges {
    pub badges: Vec<RoleBadge>,
    pub has_more: bool, // True if there are more roles than shown
}

/// Graph visualization widget for organizational structure
#[derive(Clone)]
pub struct OrganizationConcept {
    pub nodes: HashMap<Uuid, ConceptEntity>,
    pub edges: Vec<ConceptRelation>,
    pub selected_node: Option<Uuid>,
    pub selected_edge: Option<usize>,  // Index of selected edge in edges Vec
    pub dragging_node: Option<Uuid>,  // Node currently being dragged
    pub drag_offset: Vector,  // Offset from node center to cursor when dragging started
    pub drag_start_position: Option<Point>,  // Original position when drag started (for NodeMoved event)
    _viewport: Rectangle,  // Reserved for graph panning/zooming
    pub zoom: f32,
    pub pan_offset: Vector,
    // Phase 4: Edge creation indicator
    pub edge_indicator: EdgeCreationIndicator,
    // Phase 4: Event sourcing for undo/redo
    pub event_stack: EventStack,
    // Phase 8: Node/edge type filtering
    pub filter_show_people: bool,
    pub filter_show_orgs: bool,
    pub filter_show_nats: bool,
    pub filter_show_pki: bool,
    pub filter_show_yubikey: bool,
    // Animation state for smooth layout transitions
    pub animation_targets: HashMap<Uuid, Point>,
    pub animation_start: HashMap<Uuid, Point>,
    pub animation_progress: f32,  // 0.0 to 1.0
    pub animating: bool,
    pub animation_start_time: Option<std::time::Instant>,
    // Role badges for compact mode (person_id -> badges)
    pub role_badges: HashMap<Uuid, PersonRoleBadges>,
    // Role drag-and-drop state
    pub dragging_role: Option<DraggingRole>,
}

/// Source of the role being dragged
#[derive(Debug, Clone)]
pub enum DragSource {
    /// Dragging FROM the role palette TO a person
    RoleFromPalette {
        role_name: String,
        separation_class: crate::policy::SeparationClass,
    },
    /// Dragging FROM a person's existing role badge TO remove/reassign
    RoleFromPerson {
        person_id: Uuid,
        role_name: String,
        separation_class: crate::policy::SeparationClass,
    },
}

/// SoD (Separation of Duties) conflict during drag
#[derive(Debug, Clone)]
pub struct SoDConflict {
    pub conflicting_role: String,
    pub reason: String,
}

/// State for dragging a role onto or from a person
#[derive(Debug, Clone)]
pub struct DraggingRole {
    pub source: DragSource,
    pub cursor_position: Point,
    pub hover_person: Option<Uuid>,  // Person node we're hovering over
    pub sod_conflicts: Vec<SoDConflict>,  // Real-time SoD validation
}

/// A node in the organization graph (represents any domain entity)
///
/// Graph node representing a domain entity with visualization data.
///
/// Uses the categorical coproduct pattern via `DomainNode` for type-safe
/// node representation. The `visualization()` method provides rendering data.
#[derive(Debug, Clone)]
pub struct ConceptEntity {
    pub id: Uuid,
    /// Domain node using categorical coproduct pattern
    pub domain_node: super::domain_node::DomainNode,
    pub position: Point,
    /// Computed from domain_node.fold(&FoldVisualization).color
    pub color: Color,
    /// Computed from domain_node.fold(&FoldVisualization).primary_text
    pub label: String,
}

impl ConceptEntity {
    /// Create a new ConceptEntity from a DomainNode
    ///
    /// Uses the categorical coproduct pattern - domain_node carries all type
    /// information. Visualization data (color, label) is derived via FoldVisualization.
    pub fn from_domain_node(id: Uuid, domain_node: super::domain_node::DomainNode, position: Point) -> Self {
        use super::domain_node::FoldVisualization;

        let viz = domain_node.fold(&FoldVisualization);

        Self {
            id,
            domain_node,
            position,
            color: viz.color,
            label: viz.primary_text,
        }
    }

    /// Get visualization data from the domain node
    ///
    /// This is the preferred way to get color, label, icon, etc.
    /// Instead of matching on node_type, use this method.
    pub fn visualization(&self) -> super::domain_node::VisualizationData {
        use super::domain_node::FoldVisualization;
        self.domain_node.fold(&FoldVisualization)
    }

    /// Get the injection type (what kind of domain entity this is)
    pub fn injection(&self) -> super::domain_node::Injection {
        self.domain_node.injection()
    }
}

/// An edge in the organization graph (represents relationship/delegation)
#[derive(Debug, Clone)]
pub struct ConceptRelation {
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
    /// Person responsible for unit (Person → OrganizationalUnit)
    ResponsibleFor,
    /// General management relationship (Person → Entity)
    Manages,
    /// Resource management (Organization/Unit → Resource)
    ManagesResource,
    /// Managed by relationship (Entity → Organization/Unit)
    ManagedBy,

    // Key relationships
    /// Key ownership (Person → Key)
    OwnsKey,
    /// Key delegation (Person → Person)
    DelegatesKey(KeyDelegation),
    /// Storage location (Key → Location)
    StoredAt,
    /// Key rotation chain (OldKey → NewKey)
    KeyRotation,
    /// Certificate uses key (Certificate → Key)
    CertificateUsesKey,
    /// Key stored in YubiKey slot (Key → YubiKey) with slot name
    StoredInYubiKeySlot(String),

    // Policy relationships
    /// Role assignment (Person → Role) with temporal validity
    HasRole {
        valid_from: chrono::DateTime<chrono::Utc>,
        valid_until: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// Separation of duties - roles that cannot be held simultaneously
    IncompatibleWith,
    /// Role contains claim (Role → Claim)
    RoleContainsClaim,
    /// Category contains claim (PolicyCategory → PolicyClaim)
    CategoryContainsClaim,
    /// Separation class contains role (PolicyGroup → PolicyRole)
    ClassContainsRole,
    /// Policy requirement (Role → Policy)
    RoleRequiresPolicy,
    /// Policy governance (Policy → Entity)
    PolicyGovernsEntity,
    /// Organization defines role (Organization → Role)
    DefinesRole,
    /// Organization defines policy (Organization → Policy)
    DefinesPolicy,

    // Trust and Access relationships
    /// Trust relationship (Organization → Organization)
    Trusts,
    /// Certificate authority (Key → Key)
    CertifiedBy,
    /// Access permission (Person → Location/Resource)
    HasAccess,

    // NATS Infrastructure (Phase 1)
    /// JWT signing relationship (Operator → Account, Account → User)
    Signs,
    /// Account membership (User → Account)
    BelongsToAccount,
    /// Account mapped to organizational unit (Account → OrganizationalUnit)
    MapsToOrgUnit,
    /// User mapped to person (User → Person)
    MapsToPerson,

    // PKI Trust Chain (Phase 2)
    /// Certificate signing relationship (CA cert → signed cert)
    SignedBy,
    /// Certificate certifies a key (Certificate → Key/Person)
    CertifiesKey,
    /// Certificate issued to an entity (Certificate → Person/Service/Organization)
    IssuedTo,

    // YubiKey Hardware (Phase 3)
    /// YubiKey ownership (Person → YubiKey)
    OwnsYubiKey,
    /// YubiKey assigned to person (YubiKey → Person)
    AssignedTo,
    /// PIV slot on YubiKey (YubiKey → PivSlot)
    HasSlot,
    /// Key stored in slot (PivSlot → Key)
    StoresKey,
    /// Certificate loaded in slot (PivSlot → Certificate)
    LoadedCertificate,
    /// Person requires YubiKey (Person → YubiKeyStatus)
    Requires,

    // Export and Manifest (Phase 4)
    /// Entity exported to manifest (Manifest → Entity)
    ExportedTo,
    /// Export signed by person (Manifest → Person)
    SignedByPerson,

    // Legacy (for backwards compatibility)
    /// Hierarchical relationship (manager -> report)
    Hierarchy,
    /// Trust relationship (CA -> signed cert)
    Trust,
}

/// Available graph layout algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutAlgorithm {
    /// Tutte's barycentric embedding (1963) - best for planar graphs
    Tutte,
    /// Fruchterman-Reingold force-directed (1991) - general purpose
    FruchtermanReingold,
    /// Circular layout - nodes on a circle
    Circular,
    /// Hierarchical/Sugiyama - for DAGs
    Hierarchical,
    /// Combined: Tutte + F-R refinement
    TuttePlusFR,
    /// YubiKey grouped - bipartite layout with slots grouped under YubiKeys
    YubiKeyGrouped,
    /// NATS hierarchical - Operator at top, Accounts in middle, Users at bottom
    NatsHierarchical,
}

#[derive(Debug, Clone)]
pub enum OrganizationIntent {
    NodeClicked(Uuid),
    /// Click on +/- expansion indicator (separate from node click)
    ExpandIndicatorClicked(Uuid),
    NodeDragStarted { node_id: Uuid, offset: Vector },
    NodeDragged(Point),  // New cursor position
    NodeDragEnded,
    EdgeClicked { from: Uuid, to: Uuid },
    EdgeSelected(usize),  // Index of selected edge
    EdgeDeleted(usize),   // Index of edge to delete
    EdgeTypeChanged { edge_index: usize, new_type: EdgeType },  // Change edge relationship type
    EdgeCreationStarted(Uuid),  // Start edge creation by dragging from node border
    ZoomIn,
    ZoomOut,
    /// Smooth zoom by delta (for touchpad gestures)
    ZoomBy(f32),
    ResetView,
    Pan(Vector),
    AutoLayout,
    /// Apply a specific layout algorithm
    ApplyLayout(LayoutAlgorithm),
    /// Animation tick (called ~60fps during animation)
    AnimationTick,
    AddEdge { from: Uuid, to: Uuid, edge_type: EdgeType },
    // Phase 4: Context menu trigger
    RightClick(Point),  // Right-click position (adjusted for zoom/pan)
    // Phase 4: Cursor movement for edge indicator
    CursorMoved(Point),  // Cursor position (adjusted for zoom/pan)
    // Phase 4: Keyboard shortcuts
    CancelEdgeCreation,  // Esc key - cancel edge creation
    DeleteSelected,      // Delete key - delete selected node or edge
    Undo,                // Ctrl+Z - undo last action
    Redo,                // Ctrl+Y or Ctrl+Shift+Z - redo last undone action
    // Canvas interaction
    CanvasClicked(Point),  // Click on empty canvas (not on a node)
    // Role drag-and-drop (from palette or from person's badge)
    RoleDragStarted(DragSource),
    RoleDragMoved(Point),  // Cursor position during drag
    RoleDragCancelled,
    RoleDragDropped,  // Drop at current hover position
    /// Role successfully assigned to person
    RoleAssigned { person_id: Uuid, role_name: String },
    /// Role removed from person
    RoleRemoved { person_id: Uuid, role_name: String },
}

impl Default for OrganizationConcept {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two line segments intersect
/// Returns true if segment (p1, p2) crosses segment (p3, p4)
fn segments_intersect(p1: Point, p2: Point, p3: Point, p4: Point) -> bool {
    // Cross product helper
    fn cross(o: Point, a: Point, b: Point) -> f32 {
        (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
    }

    let d1 = cross(p3, p4, p1);
    let d2 = cross(p3, p4, p2);
    let d3 = cross(p1, p2, p3);
    let d4 = cross(p1, p2, p4);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0)) &&
       ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0)) {
        return true;
    }

    // Collinear cases (endpoints touch)
    if d1.abs() < 0.0001 && on_segment(p3, p1, p4) { return true; }
    if d2.abs() < 0.0001 && on_segment(p3, p2, p4) { return true; }
    if d3.abs() < 0.0001 && on_segment(p1, p3, p2) { return true; }
    if d4.abs() < 0.0001 && on_segment(p1, p4, p2) { return true; }

    false
}

/// Check if point q lies on segment pr
fn on_segment(p: Point, q: Point, r: Point) -> bool {
    q.x <= p.x.max(r.x) && q.x >= p.x.min(r.x) &&
    q.y <= p.y.max(r.y) && q.y >= p.y.min(r.y)
}

// Helper function to calculate distance from point to line segment
fn distance_to_line_segment(point: Point, line_start: Point, line_end: Point) -> f32 {
    let px = point.x;
    let py = point.y;
    let x1 = line_start.x;
    let y1 = line_start.y;
    let x2 = line_end.x;
    let y2 = line_end.y;

    let dx = x2 - x1;
    let dy = y2 - y1;

    if dx == 0.0 && dy == 0.0 {
        // Line segment is a point
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }

    // Parameter t represents position along line segment (0 = start, 1 = end)
    let t = ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy);
    let t = t.clamp(0.0, 1.0);  // Clamp to line segment

    // Closest point on line segment
    let closest_x = x1 + t * dx;
    let closest_y = y1 + t * dy;

    // Distance from point to closest point
    ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt()
}

impl OrganizationConcept {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            selected_node: None,
            selected_edge: None,
            dragging_node: None,
            drag_offset: Vector::new(0.0, 0.0),
            drag_start_position: None,
            _viewport: Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0)),
            zoom: 1.0,  // Default 1:1 scale
            pan_offset: Vector::new(0.0, 0.0),
            edge_indicator: EdgeCreationIndicator::new(),
            event_stack: EventStack::default(),
            // Phase 8: Node/edge type filtering - all enabled by default
            filter_show_people: true,
            filter_show_orgs: true,
            filter_show_nats: true,
            filter_show_pki: true,
            filter_show_yubikey: true,
            // Animation state
            animation_targets: HashMap::new(),
            animation_start: HashMap::new(),
            animation_progress: 1.0,  // Start fully complete (no animation)
            animating: false,
            animation_start_time: None,
            // Role badges for compact mode
            role_badges: HashMap::new(),
            // Role drag state
            dragging_role: None,
        }
    }

    /// Clear all nodes and edges from the graph
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.selected_node = None;
        self.selected_edge = None;
        self.dragging_node = None;
        self.event_stack.clear();
        self.role_badges.clear();
    }

    /// Set role badges for a person (compact mode display)
    pub fn set_person_role_badges(&mut self, person_id: Uuid, badges: Vec<RoleBadge>, has_more: bool) {
        self.role_badges.insert(person_id, PersonRoleBadges { badges, has_more });
    }

    /// Get role badges for a person (for compact mode rendering)
    pub fn get_person_role_badges(&self, person_id: &Uuid) -> Option<&PersonRoleBadges> {
        self.role_badges.get(person_id)
    }

    /// Clear all role badges
    pub fn clear_role_badges(&mut self) {
        self.role_badges.clear();
    }

    /// Start dragging a role
    pub fn start_role_drag(&mut self, source: DragSource, cursor_position: Point) {
        self.dragging_role = Some(DraggingRole {
            source,
            cursor_position,
            hover_person: None,
            sod_conflicts: Vec::new(),
        });
    }

    /// Update drag position and find hover target
    pub fn update_role_drag(&mut self, cursor_position: Point) {
        // Compute hover person before mutable borrow (borrow checker fix)
        let hover_person = self.find_person_at_position(cursor_position);
        if let Some(ref mut drag) = self.dragging_role {
            drag.cursor_position = cursor_position;
            drag.hover_person = hover_person;
        }
    }

    /// Cancel the drag operation
    pub fn cancel_role_drag(&mut self) {
        self.dragging_role = None;
    }

    /// Find person node at the given position (within node radius)
    pub fn find_person_at_position(&self, position: Point) -> Option<Uuid> {
        let node_radius = 25.0;  // Standard node radius

        for (id, node) in &self.nodes {
            if node.injection() == super::domain_node::Injection::Person {
                let dx = position.x - node.position.x;
                let dy = position.y - node.position.y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= node_radius {
                    return Some(*id);
                }
            }
        }
        None
    }

    /// Get the role name from current drag source
    pub fn get_dragging_role_name(&self) -> Option<&str> {
        self.dragging_role.as_ref().map(|drag| {
            match &drag.source {
                DragSource::RoleFromPalette { role_name, .. } => role_name.as_str(),
                DragSource::RoleFromPerson { role_name, .. } => role_name.as_str(),
            }
        })
    }

    /// Get the separation class color for the dragging role
    pub fn get_dragging_role_color(&self) -> Option<Color> {
        self.dragging_role.as_ref().map(|drag| {
            let sep_class = match &drag.source {
                DragSource::RoleFromPalette { separation_class, .. } => separation_class,
                DragSource::RoleFromPerson { separation_class, .. } => separation_class,
            };
            match sep_class {
                crate::policy::SeparationClass::Operational => Color::from_rgb(0.3, 0.6, 0.9),
                crate::policy::SeparationClass::Administrative => Color::from_rgb(0.6, 0.4, 0.8),
                crate::policy::SeparationClass::Audit => Color::from_rgb(0.2, 0.7, 0.5),
                crate::policy::SeparationClass::Emergency => Color::from_rgb(0.9, 0.3, 0.2),
                crate::policy::SeparationClass::Financial => Color::from_rgb(0.9, 0.7, 0.2),
                crate::policy::SeparationClass::Personnel => Color::from_rgb(0.8, 0.4, 0.6),
            }
        })
    }

    /// Add a person node to the graph
    pub fn add_node(&mut self, person: Person, role: KeyOwnerRole) {
        let node_id = person.id;
        let domain_node = DomainNode::inject_person(person, role);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    /// Add an organization node to the graph
    pub fn add_organization_node(&mut self, org: Organization) {
        let node_id = org.id;
        let domain_node = DomainNode::inject_organization(org);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    /// Add an organizational unit node to the graph
    pub fn add_org_unit_node(&mut self, unit: OrganizationUnit) {
        let node_id = unit.id;
        let domain_node = DomainNode::inject_organization_unit(unit);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    /// Add a location node to the graph
    pub fn add_location_node(&mut self, location: Location) {
        use cim_domain::AggregateRoot;
        let node_id = *location.id().as_uuid();
        let domain_node = DomainNode::inject_location(location);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    /// Add a role node to the graph (domain Role type)
    pub fn add_domain_role_node(&mut self, role: Role) {
        let node_id = role.id;
        let domain_node = DomainNode::inject_role(role);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    /// Add a policy-based role node to the graph
    /// Color is derived from FoldVisualization based on separation class
    pub fn add_role_node(
        &mut self,
        role_id: Uuid,
        name: String,
        purpose: String,
        level: u8,
        separation_class: crate::policy::SeparationClass,
        claim_count: usize,
    ) {
        let domain_node = DomainNode::inject_policy_role_uuid(
            role_id, name, purpose, level, separation_class, claim_count
        );
        let position = self.calculate_node_position(role_id);
        let node = ConceptEntity::from_domain_node(role_id, domain_node, position);

        self.nodes.insert(role_id, node);
    }

    /// Add a claim node to the graph
    /// Color is derived from FoldVisualization based on category
    pub fn add_claim_node(
        &mut self,
        claim_id: Uuid,
        name: String,
        category: String,
    ) {
        let domain_node = DomainNode::inject_policy_claim_uuid(claim_id, name, category);
        let position = self.calculate_node_position(claim_id);
        let node = ConceptEntity::from_domain_node(claim_id, domain_node, position);

        self.nodes.insert(claim_id, node);
    }

    /// Add a policy category node to the graph (for progressive disclosure)
    pub fn add_category_node(
        &mut self,
        category_id: Uuid,
        name: String,
        claim_count: usize,
        expanded: bool,
    ) {
        let domain_node = DomainNode::inject_policy_category_uuid(category_id, name, claim_count, expanded);
        let position = self.calculate_node_position(category_id);
        let node = ConceptEntity::from_domain_node(category_id, domain_node, position);

        self.nodes.insert(category_id, node);
    }

    /// Add a separation class group node to the graph (for progressive disclosure)
    pub fn add_separation_class_node(
        &mut self,
        class_id: Uuid,
        name: String,
        separation_class: crate::policy::SeparationClass,
        role_count: usize,
        expanded: bool,
    ) {
        let domain_node = DomainNode::inject_policy_group_uuid(
            class_id, name, separation_class, role_count, expanded
        );
        let position = self.calculate_node_position(class_id);
        let node = ConceptEntity::from_domain_node(class_id, domain_node, position);

        self.nodes.insert(class_id, node);
    }

    /// Add a policy node to the graph
    pub fn add_policy_node(&mut self, policy: Policy) {
        let node_id = policy.id;
        let domain_node = DomainNode::inject_policy(policy);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    // ===== NATS Infrastructure Nodes (Phase 1) =====

    /// Add a NATS operator node to the graph
    pub fn add_nats_operator_node(&mut self, node_id: Uuid, nats_identity: NatsIdentityProjection, _label: String) {
        let domain_node = DomainNode::inject_nats_operator(nats_identity);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    /// Add a NATS account node to the graph
    pub fn add_nats_account_node(&mut self, node_id: Uuid, nats_identity: NatsIdentityProjection, _label: String) {
        let domain_node = DomainNode::inject_nats_account(nats_identity);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    /// Add a NATS user node to the graph
    pub fn add_nats_user_node(&mut self, node_id: Uuid, nats_identity: NatsIdentityProjection, _label: String) {
        let domain_node = DomainNode::inject_nats_user(nats_identity);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    /// Add a NATS service account node to the graph
    pub fn add_nats_service_account_node(&mut self, node_id: Uuid, nats_identity: NatsIdentityProjection, _label: String) {
        let domain_node = DomainNode::inject_nats_service_account(nats_identity);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);

        self.nodes.insert(node_id, node);
    }

    // ===== Simple NATS Nodes (Visualization Only) =====

    /// Add a simple NATS operator node (visualization without crypto)
    pub fn add_nats_operator_simple(&mut self, node_id: Uuid, name: String, organization_id: Option<Uuid>) {
        let domain_node = DomainNode::inject_nats_operator_simple(name, organization_id);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);
        self.nodes.insert(node_id, node);
    }

    /// Add a simple NATS account node (visualization without crypto)
    pub fn add_nats_account_simple(&mut self, node_id: Uuid, name: String, unit_id: Option<Uuid>, is_system: bool) {
        let domain_node = DomainNode::inject_nats_account_simple(name, unit_id, is_system);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);
        self.nodes.insert(node_id, node);
    }

    /// Add a simple NATS user node (visualization without crypto)
    pub fn add_nats_user_simple(&mut self, node_id: Uuid, name: String, person_id: Option<Uuid>, account_name: String) {
        let domain_node = DomainNode::inject_nats_user_simple(name, person_id, account_name);
        let position = self.calculate_node_position(node_id);
        let node = ConceptEntity::from_domain_node(node_id, domain_node, position);
        self.nodes.insert(node_id, node);
    }

    // ===== PKI Trust Chain Nodes (Phase 2) =====

    /// Add a root CA certificate node to the graph
    pub fn add_root_certificate_node(
        &mut self,
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        key_usage: Vec<String>,
    ) {
        let domain_node = DomainNode::inject_root_certificate_uuid(
            cert_id, subject, issuer, not_before, not_after, key_usage
        );
        let position = self.calculate_node_position(cert_id);
        let node = ConceptEntity::from_domain_node(cert_id, domain_node, position);

        self.nodes.insert(cert_id, node);
    }

    /// Add an intermediate CA certificate node to the graph
    pub fn add_intermediate_certificate_node(
        &mut self,
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        key_usage: Vec<String>,
    ) {
        let domain_node = DomainNode::inject_intermediate_certificate_uuid(
            cert_id, subject, issuer, not_before, not_after, key_usage
        );
        let position = self.calculate_node_position(cert_id);
        let node = ConceptEntity::from_domain_node(cert_id, domain_node, position);

        self.nodes.insert(cert_id, node);
    }

    /// Add a leaf certificate node to the graph
    pub fn add_leaf_certificate_node(
        &mut self,
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        key_usage: Vec<String>,
        san: Vec<String>,
    ) {
        let domain_node = DomainNode::inject_leaf_certificate_uuid(
            cert_id, subject, issuer, not_before, not_after, key_usage, san
        );
        let position = self.calculate_node_position(cert_id);
        let node = ConceptEntity::from_domain_node(cert_id, domain_node, position);

        self.nodes.insert(cert_id, node);
    }

    // ===== NATS Infrastructure Graph Population (Phase 1) =====

    /// Populate the graph from NATS OrganizationBootstrap
    ///
    /// Creates nodes for operator, accounts, users, and service accounts,
    /// and edges showing the JWT signing hierarchy.
    pub fn populate_nats_infrastructure(&mut self, bootstrap: &crate::domain_projections::OrganizationBootstrap) {
        // 1. Create operator node
        let operator_id = bootstrap.operator.nkey.id;
        self.add_nats_operator_node(
            operator_id,
            bootstrap.operator.clone(),
            format!("{} Operator", bootstrap.organization.name),
        );

        // 2. Create account nodes and operator→account edges
        for (unit_id, (unit, account_identity)) in &bootstrap.accounts {
            let account_id = account_identity.nkey.id;

            // Add account node
            self.add_nats_account_node(
                account_id,
                account_identity.clone(),
                format!("{} Account", unit.name),
            );

            // Add operator→account "signs" edge
            self.add_edge(operator_id, account_id, EdgeType::Signs);

            // Add account→orgunit "maps to" edge (if orgunit node exists in graph)
            self.add_edge(account_id, *unit_id, EdgeType::MapsToOrgUnit);
        }

        // 3. Create user nodes and account→user edges
        for (person_id, (person, user_identity)) in &bootstrap.users {
            let user_id = user_identity.nkey.id;

            // Add user node
            self.add_nats_user_node(
                user_id,
                user_identity.clone(),
                person.name.clone(),
            );

            // Find which account this user belongs to (based on organizational unit)
            // For now, we'll connect to the first account as a simplified version
            // TODO: In the future, connect based on person's organizational unit membership
            if let Some((_unit_id, (_unit, account_identity))) = bootstrap.accounts.iter().next() {
                let account_id = account_identity.nkey.id;
                self.add_edge(account_id, user_id, EdgeType::Signs);
                self.add_edge(user_id, account_id, EdgeType::BelongsToAccount);
            }

            // Add user→person "maps to" edge (if person node exists in graph)
            self.add_edge(user_id, *person_id, EdgeType::MapsToPerson);
        }

        // 4. Create service account nodes
        for (_service_id, (service, service_identity)) in &bootstrap.service_accounts {
            let service_account_id = service_identity.nkey.id;

            // Add service account node
            self.add_nats_service_account_node(
                service_account_id,
                service_identity.clone(),
                service.name.clone(),
            );

            // Find which account this service belongs to
            // For now, connect to the first account
            // TODO: In the future, connect based on service's organizational unit
            if let Some((_unit_id, (_unit, account_identity))) = bootstrap.accounts.iter().next() {
                let account_id = account_identity.nkey.id;
                self.add_edge(account_id, service_account_id, EdgeType::Signs);
                self.add_edge(service_account_id, account_id, EdgeType::BelongsToAccount);
            }
        }
    }

    // ===== YubiKey Hardware Nodes (Phase 3) =====

    /// Add a YubiKey hardware token node to the graph
    pub fn add_yubikey_node(
        &mut self,
        device_id: Uuid,
        serial: String,
        version: String,
        provisioned_at: Option<chrono::DateTime<chrono::Utc>>,
        slots_used: Vec<String>,
    ) {
        let domain_node = DomainNode::inject_yubikey_uuid(
            device_id, serial, version, provisioned_at, slots_used
        );
        let position = self.calculate_node_position(device_id);
        let node = ConceptEntity::from_domain_node(device_id, domain_node, position);

        self.nodes.insert(device_id, node);
    }

    /// Add a PIV slot node to the graph
    pub fn add_piv_slot_node(
        &mut self,
        slot_id: Uuid,
        slot_name: String,
        yubikey_serial: String,
        has_key: bool,
        certificate_subject: Option<String>,
    ) {
        let domain_node = DomainNode::inject_piv_slot_uuid(
            slot_id, slot_name, yubikey_serial, has_key, certificate_subject
        );
        let position = self.calculate_node_position(slot_id);
        let node = ConceptEntity::from_domain_node(slot_id, domain_node, position);

        self.nodes.insert(slot_id, node);
    }

    // ===== PKI Trust Chain Graph Population (Phase 2) =====

    /// Populate the graph from PKI certificate hierarchy
    ///
    /// Creates nodes for root CAs, intermediate CAs, and leaf certificates,
    /// and edges showing the signing chain.
    ///
    /// # Arguments
    /// * `certificates` - Slice of certificate entries from the projection
    pub fn populate_pki_trust_chain(&mut self, certificates: &[crate::projections::CertificateEntry]) {
        use std::collections::HashMap;

        // Group certificates by type (root CA, intermediate CA, leaf)
        let mut root_cas: Vec<&crate::projections::CertificateEntry> = Vec::new();
        let mut intermediate_cas: Vec<&crate::projections::CertificateEntry> = Vec::new();
        let mut leaf_certs: Vec<&crate::projections::CertificateEntry> = Vec::new();

        // Separate certificates by type based on is_ca flag and issuer
        for cert in certificates {
            if cert.is_ca {
                // CA certificate
                if cert.issuer.is_none() || cert.issuer.as_ref() == Some(&cert.subject) {
                    // Self-signed = root CA
                    root_cas.push(cert);
                } else {
                    // Signed by another CA = intermediate CA
                    intermediate_cas.push(cert);
                }
            } else {
                // Not a CA = leaf certificate
                leaf_certs.push(cert);
            }
        }

        // Create mapping from subject to cert_id for edge creation
        let mut subject_to_id: HashMap<String, Uuid> = HashMap::new();
        for cert in certificates {
            subject_to_id.insert(cert.subject.clone(), cert.cert_id);
        }

        // 1. Create root CA nodes
        for cert in &root_cas {
            self.add_root_certificate_node(
                cert.cert_id,
                cert.subject.clone(),
                cert.issuer.clone().unwrap_or_else(|| cert.subject.clone()),
                cert.not_before,
                cert.not_after,
                vec!["KeyCertSign".to_string(), "CRLSign".to_string()], // Default CA key usage
            );
        }

        // 2. Create intermediate CA nodes and edges to their signing CAs
        for cert in &intermediate_cas {
            self.add_intermediate_certificate_node(
                cert.cert_id,
                cert.subject.clone(),
                cert.issuer.clone().unwrap_or_else(|| "Unknown".to_string()),
                cert.not_before,
                cert.not_after,
                vec!["KeyCertSign".to_string(), "CRLSign".to_string()],
            );

            // Add SignedBy edge from intermediate to signing CA
            if let Some(issuer) = &cert.issuer {
                if let Some(&signing_ca_id) = subject_to_id.get(issuer) {
                    self.add_edge(cert.cert_id, signing_ca_id, EdgeType::SignedBy);
                }
            }
        }

        // 3. Create leaf certificate nodes and edges to their signing CAs
        for cert in &leaf_certs {
            // Extract SANs if available (would need to be added to CertificateEntry in the future)
            let san = Vec::new(); // TODO: Add SAN extraction from certificate extensions

            self.add_leaf_certificate_node(
                cert.cert_id,
                cert.subject.clone(),
                cert.issuer.clone().unwrap_or_else(|| "Unknown".to_string()),
                cert.not_before,
                cert.not_after,
                vec!["DigitalSignature".to_string()], // Default leaf key usage
                san,
            );

            // Add SignedBy edge from leaf to signing CA
            if let Some(issuer) = &cert.issuer {
                if let Some(&signing_ca_id) = subject_to_id.get(issuer) {
                    self.add_edge(cert.cert_id, signing_ca_id, EdgeType::SignedBy);
                }
            }

            // Add IssuedTo edge from certificate to key owner (if key_id matches a person node)
            // This would require person nodes to already be in the graph
            self.add_edge(cert.cert_id, cert.key_id, EdgeType::CertifiesKey);
        }
    }

    // ===== YubiKey Hardware Graph Population (Phase 3) =====

    /// Populate the graph from YubiKey hardware data
    ///
    /// Creates nodes for YubiKeys and their PIV slots,
    /// and edges showing ownership and key storage.
    ///
    /// # Arguments
    /// * `yubikeys` - Slice of YubiKey entries from the projection
    /// * `people` - Slice of person entries to show ownership
    pub fn populate_yubikey_graph(
        &mut self,
        yubikeys: &[crate::projections::YubiKeyEntry],
        people: &[crate::projections::PersonEntry],
    ) {
        use std::collections::HashMap;

        // Create mapping from person_id to person for ownership edges
        let _person_map: HashMap<Uuid, &crate::projections::PersonEntry> =
            people.iter().map(|p| (p.person_id, p)).collect();

        // Common PIV slot names
        let piv_slots = vec![
            ("9A", "Authentication"),
            ("9C", "Digital Signature"),
            ("9D", "Key Management"),
            ("9E", "Card Authentication"),
        ];

        for yubikey in yubikeys {
            // Create YubiKey device node
            let yubikey_id = Uuid::now_v7(); // Generate ID for YubiKey device
            self.add_yubikey_node(
                yubikey_id,
                yubikey.serial.clone(),
                "5.x".to_string(), // Default version (would need to be added to YubiKeyEntry)
                Some(yubikey.provisioned_at),
                yubikey.slots_used.clone(),
            );

            // Create PIV slot nodes and edges
            for (slot_num, slot_desc) in &piv_slots {
                let slot_id = Uuid::now_v7();
                let slot_name = format!("{} - {}", slot_num, slot_desc);
                let has_key = yubikey.slots_used.contains(&slot_num.to_string());

                self.add_piv_slot_node(
                    slot_id,
                    slot_name,
                    yubikey.serial.clone(),
                    has_key,
                    None, // Certificate subject would need to be added to projection
                );

                // Add YubiKey → PivSlot edge
                self.add_edge(yubikey_id, slot_id, EdgeType::HasSlot);

                // If slot has a key, add edge to indicate storage
                if has_key {
                    // In a real implementation, we'd link to the actual key node
                    // For now, just show that the slot stores a key
                    // self.add_edge(slot_id, key_id, EdgeType::StoresKey);
                }
            }

            // Find person who owns this YubiKey (based on naming convention in serial or config)
            // In a real implementation, there should be a direct mapping in the projection
            // For now, we'll create ownership edges if we can determine ownership

            // Example: If we have a way to determine ownership, create the edge
            // This would typically be stored in the YubiKeyEntry
            // For demonstration, we'll just show the pattern:
            // if let Some(owner_id) = yubikey.owner_person_id {
            //     if person_map.contains_key(&owner_id) {
            //         self.add_edge(owner_id, yubikey_id, EdgeType::OwnsYubiKey);
            //         self.add_edge(yubikey_id, owner_id, EdgeType::AssignedTo);
            //     }
            // }
        }
    }

    pub fn add_edge(&mut self, from: Uuid, to: Uuid, edge_type: EdgeType) {
        let color = match &edge_type {
            // Organizational hierarchy - blues
            EdgeType::ParentChild => Color::from_rgb(0.2, 0.4, 0.8),
            EdgeType::ManagesUnit => Color::from_rgb(0.4, 0.2, 0.8),
            EdgeType::MemberOf => Color::from_rgb(0.5, 0.5, 0.5),
            EdgeType::ResponsibleFor => Color::from_rgb(0.3, 0.5, 0.9),
            EdgeType::Manages => Color::from_rgb(0.4, 0.3, 0.7),
            EdgeType::ManagesResource => Color::from_rgb(0.5, 0.4, 0.8),
            EdgeType::ManagedBy => Color::from_rgb(0.4, 0.5, 0.7),

            // Key relationships - greens
            EdgeType::OwnsKey => Color::from_rgb(0.2, 0.7, 0.2),
            EdgeType::DelegatesKey(_) => Color::from_rgb(0.9, 0.6, 0.2),
            EdgeType::StoredAt => Color::from_rgb(0.6, 0.5, 0.4),
            EdgeType::KeyRotation => Color::from_rgb(0.3, 0.8, 0.3),
            EdgeType::CertificateUsesKey => Color::from_rgb(0.4, 0.7, 0.4),
            EdgeType::StoredInYubiKeySlot(_) => Color::from_rgb(0.7, 0.4, 0.7),

            // Policy relationships - gold/yellow
            EdgeType::HasRole { .. } => Color::from_rgb(0.6, 0.3, 0.8),
            EdgeType::IncompatibleWith => Color::from_rgb(0.9, 0.2, 0.2), // Red for conflicts
            EdgeType::RoleContainsClaim => Color::from_rgb(0.5, 0.7, 0.3), // Green for role→claim
            EdgeType::CategoryContainsClaim => Color::from_rgb(0.4, 0.6, 0.4), // Muted green
            EdgeType::ClassContainsRole => Color::from_rgb(0.5, 0.5, 0.7), // Purple-gray
            EdgeType::RoleRequiresPolicy => Color::from_rgb(0.9, 0.7, 0.2),
            EdgeType::PolicyGovernsEntity => Color::from_rgb(0.9, 0.7, 0.2),
            EdgeType::DefinesRole => Color::from_rgb(0.8, 0.6, 0.2),
            EdgeType::DefinesPolicy => Color::from_rgb(0.9, 0.6, 0.3),

            // Trust and Access relationships
            EdgeType::Trusts => Color::from_rgb(0.7, 0.5, 0.3),
            EdgeType::CertifiedBy => Color::from_rgb(0.7, 0.5, 0.3),
            EdgeType::HasAccess => Color::from_rgb(0.5, 0.6, 0.7),

            // NATS Infrastructure - orange/red (JWT signing chains)
            EdgeType::Signs => Color::from_rgb(1.0, 0.4, 0.0), // Bright orange
            EdgeType::BelongsToAccount => Color::from_rgb(0.8, 0.5, 0.2), // Brown-orange
            EdgeType::MapsToOrgUnit => Color::from_rgb(0.6, 0.6, 0.8), // Light purple
            EdgeType::MapsToPerson => Color::from_rgb(0.5, 0.7, 0.9), // Light blue

            // PKI Trust Chain - green/teal (certificate chains)
            EdgeType::SignedBy => Color::from_rgb(0.2, 0.8, 0.6), // Teal (trust chain)
            EdgeType::CertifiesKey => Color::from_rgb(0.3, 0.9, 0.3), // Bright green (certification)
            EdgeType::IssuedTo => Color::from_rgb(0.4, 0.7, 0.9), // Sky blue (issuance)

            // YubiKey Hardware - purple/magenta (physical hardware)
            EdgeType::OwnsYubiKey => Color::from_rgb(0.8, 0.3, 0.8), // Magenta (ownership)
            EdgeType::AssignedTo => Color::from_rgb(0.7, 0.2, 0.7), // Dark magenta (assignment)
            EdgeType::HasSlot => Color::from_rgb(0.9, 0.4, 0.9), // Light magenta (slot)
            EdgeType::StoresKey => Color::from_rgb(0.6, 0.2, 0.9), // Purple (key storage)
            EdgeType::LoadedCertificate => Color::from_rgb(0.7, 0.5, 0.9), // Light purple (cert)
            EdgeType::Requires => Color::from_rgb(0.6, 0.4, 0.8), // Purple (requirement)

            // Export and Manifest - brown/tan
            EdgeType::ExportedTo => Color::from_rgb(0.6, 0.5, 0.3), // Brown (export)
            EdgeType::SignedByPerson => Color::from_rgb(0.7, 0.6, 0.4), // Tan (signature)

            // Legacy
            EdgeType::Hierarchy => Color::from_rgb(0.3, 0.3, 0.7),
            EdgeType::Trust => Color::from_rgb(0.7, 0.5, 0.3),
        };
        self.edges.push(ConceptRelation {
            from,
            to,
            edge_type,
            color,
        });
    }

    pub fn select_node(&mut self, node_id: Uuid) {
        self.selected_node = Some(node_id);
    }

    /// Delete a node and all edges connected to it
    pub fn delete_node(&mut self, node_id: Uuid) {
        // Remove the node
        self.nodes.remove(&node_id);

        // Remove all edges connected to this node
        self.edges.retain(|edge| edge.from != node_id && edge.to != node_id);

        // Clear selection if this was the selected node
        if self.selected_node == Some(node_id) {
            self.selected_node = None;
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Apply a graph event to update the graph state
    /// This is the ONLY way to change graph state for undo/redo to work correctly
    pub fn apply_event(&mut self, event: &GraphEvent) {
        match event {
            GraphEvent::NodeCreated { node_id, domain_node, position, color, label, .. } => {
                // Use from_domain_node - the preferred constructor
                // Override color/label with event values for event-sourced consistency
                let mut node = ConceptEntity::from_domain_node(*node_id, domain_node.clone(), *position);
                node.color = *color;
                node.label = label.clone();
                self.nodes.insert(*node_id, node);
            }
            GraphEvent::NodeDeleted { node_id, .. } => {
                self.nodes.remove(node_id);
                // Remove all edges connected to this node
                self.edges.retain(|edge| edge.from != *node_id && edge.to != *node_id);
                // Clear selection if this was the selected node
                if self.selected_node == Some(*node_id) {
                    self.selected_node = None;
                }
            }
            GraphEvent::NodePropertiesChanged { node_id, new_domain_node, new_label, .. } => {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.domain_node = new_domain_node.clone();
                    node.label = new_label.clone();
                }
            }
            GraphEvent::NodeMoved { node_id, new_position, .. } => {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.position = *new_position;
                }
            }
            GraphEvent::EdgeCreated { from, to, edge_type, color, .. } => {
                self.edges.push(ConceptRelation {
                    from: *from,
                    to: *to,
                    edge_type: edge_type.clone(),
                    color: *color,
                });
            }
            GraphEvent::EdgeDeleted { from, to, edge_type, .. } => {
                self.edges.retain(|edge| {
                    !(edge.from == *from && edge.to == *to && edge.edge_type == *edge_type)
                });
            }
            GraphEvent::EdgeTypeChanged { from, to, new_type, .. } => {
                for edge in &mut self.edges {
                    if edge.from == *from && edge.to == *to {
                        edge.edge_type = new_type.clone();
                        break;
                    }
                }
            }
        }
    }

    pub fn auto_layout(&mut self) {
        let node_count = self.nodes.len();
        if node_count == 0 {
            return;
        }

        // Always use crossing-aware force-directed layout for best results
        self.crossing_aware_layout();
    }

    /// Main crossing-aware layout - applies Tutte's barycentric embedding
    /// followed by Fruchterman-Reingold refinement
    fn crossing_aware_layout(&mut self) {
        let width = 1200.0;
        let height = 900.0;

        let node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        let n = node_ids.len();
        if n == 0 {
            return;
        }

        // Apply Tutte's barycentric embedding (produces planar layout if graph is planar)
        self.tutte_barycentric_layout(width, height);

        // Refine with Fruchterman-Reingold to optimize aesthetics while preserving planarity
        self.fruchterman_reingold_layout(width, height, 50);

        let final_crossings = self.count_all_crossings();
        tracing::info!("Layout complete with {} crossings (Tutte + F-R)", final_crossings);
    }

    /// Tutte's Barycentric Embedding (1963)
    /// For a 3-connected planar graph, fixes outer face vertices on a convex polygon
    /// and places interior vertices at the barycenter of their neighbors.
    /// This is guaranteed to produce a planar, crossing-free drawing for planar graphs.
    fn tutte_barycentric_layout(&mut self, width: f32, height: f32) {
        let center = Point { x: width / 2.0, y: height / 2.0 };
        let node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        let n = node_ids.len();

        if n == 0 {
            return;
        }

        // Create node index mapping
        let id_to_idx: HashMap<Uuid, usize> = node_ids.iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        // Build adjacency list
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for edge in &self.edges {
            if let (Some(&i), Some(&j)) = (id_to_idx.get(&edge.from), id_to_idx.get(&edge.to)) {
                if !adj[i].contains(&j) {
                    adj[i].push(j);
                }
                if !adj[j].contains(&i) {
                    adj[j].push(i);
                }
            }
        }

        // Find boundary nodes (nodes with highest degree, or use all for small graphs)
        // For Tutte's method, we fix some nodes on a convex boundary
        let num_boundary = (n / 3).max(3).min(n);
        let mut degrees: Vec<(usize, usize)> = adj.iter()
            .enumerate()
            .map(|(i, neighbors)| (i, neighbors.len()))
            .collect();
        degrees.sort_by(|a, b| b.1.cmp(&a.1));  // Sort by degree descending

        let boundary_indices: Vec<usize> = degrees.iter()
            .take(num_boundary)
            .map(|(i, _)| *i)
            .collect();

        let is_boundary: Vec<bool> = (0..n)
            .map(|i| boundary_indices.contains(&i))
            .collect();

        // Place boundary nodes on a circle (convex polygon)
        let radius = (width.min(height) / 2.0 - 100.0).max(100.0);
        for (bi, &idx) in boundary_indices.iter().enumerate() {
            let angle = 2.0 * std::f32::consts::PI * (bi as f32) / (num_boundary as f32);
            if let Some(node) = self.nodes.get_mut(&node_ids[idx]) {
                node.position = Point {
                    x: center.x + radius * angle.cos(),
                    y: center.y + radius * angle.sin(),
                };
            }
        }

        // Solve for interior nodes using barycentric coordinates
        // Each interior node is placed at the average position of its neighbors
        // This is an iterative relaxation (Jacobi iteration)
        for _iteration in 0..100 {
            let mut max_move = 0.0_f32;

            for i in 0..n {
                if is_boundary[i] || adj[i].is_empty() {
                    continue;
                }

                // Calculate barycenter of neighbors
                let mut sum_x = 0.0;
                let mut sum_y = 0.0;
                let neighbor_count = adj[i].len() as f32;

                for &j in &adj[i] {
                    let neighbor_pos = self.nodes[&node_ids[j]].position;
                    sum_x += neighbor_pos.x;
                    sum_y += neighbor_pos.y;
                }

                let new_x = sum_x / neighbor_count;
                let new_y = sum_y / neighbor_count;

                if let Some(node) = self.nodes.get_mut(&node_ids[i]) {
                    let dx = new_x - node.position.x;
                    let dy = new_y - node.position.y;
                    max_move = max_move.max((dx * dx + dy * dy).sqrt());

                    node.position.x = new_x;
                    node.position.y = new_y;
                }
            }

            // Converged if movement is small
            if max_move < 0.1 {
                break;
            }
        }
    }

    /// Fruchterman-Reingold Force-Directed Layout (1991)
    /// Uses attractive spring forces along edges and repulsive forces between all node pairs.
    /// Formula: f_a(d) = d²/k, f_r(d) = k²/d where k = sqrt(area/|V|)
    fn fruchterman_reingold_layout(&mut self, width: f32, height: f32, iterations: usize) {
        let node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        let n = node_ids.len();
        if n == 0 {
            return;
        }

        let area = width * height;
        let k = (area / n as f32).sqrt();  // Optimal distance between nodes

        let mut temperature = width / 10.0;  // Initial temperature for simulated annealing

        for _iter in 0..iterations {
            let mut displacements: HashMap<Uuid, (f32, f32)> = HashMap::new();
            for &id in &node_ids {
                displacements.insert(id, (0.0, 0.0));
            }

            // Calculate repulsive forces: f_r(d) = k²/d
            for i in 0..n {
                for j in (i + 1)..n {
                    let id_i = node_ids[i];
                    let id_j = node_ids[j];

                    let pos_i = self.nodes[&id_i].position;
                    let pos_j = self.nodes[&id_j].position;

                    let dx = pos_i.x - pos_j.x;
                    let dy = pos_i.y - pos_j.y;
                    let dist = (dx * dx + dy * dy).sqrt().max(0.01);

                    // Repulsive force: k²/d
                    let force = (k * k) / dist;
                    let fx = (dx / dist) * force;
                    let fy = (dy / dist) * force;

                    let (dxi, dyi) = displacements[&id_i];
                    displacements.insert(id_i, (dxi + fx, dyi + fy));

                    let (dxj, dyj) = displacements[&id_j];
                    displacements.insert(id_j, (dxj - fx, dyj - fy));
                }
            }

            // Calculate attractive forces: f_a(d) = d²/k
            for edge in &self.edges {
                let pos_from = self.nodes[&edge.from].position;
                let pos_to = self.nodes[&edge.to].position;

                let dx = pos_to.x - pos_from.x;
                let dy = pos_to.y - pos_from.y;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);

                // Attractive force: d²/k (but we use d/k to get displacement)
                let force = dist / k;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;

                let (dx_from, dy_from) = displacements[&edge.from];
                displacements.insert(edge.from, (dx_from + fx, dy_from + fy));

                let (dx_to, dy_to) = displacements[&edge.to];
                displacements.insert(edge.to, (dx_to - fx, dy_to - fy));
            }

            // Apply displacements with temperature limiting
            for &id in &node_ids {
                let (dx, dy) = displacements[&id];
                let mag = (dx * dx + dy * dy).sqrt();

                if mag > 0.0 {
                    let capped_mag = mag.min(temperature);
                    let ratio = capped_mag / mag;

                    if let Some(node) = self.nodes.get_mut(&id) {
                        node.position.x += dx * ratio;
                        node.position.y += dy * ratio;

                        // Keep within bounds
                        node.position.x = node.position.x.max(50.0).min(width - 50.0);
                        node.position.y = node.position.y.max(50.0).min(height - 50.0);
                    }
                }
            }

            // Cool down (annealing schedule)
            temperature *= 0.95;
        }
    }

    /// Circular layout - place all nodes evenly on a circle
    fn circular_layout(&mut self, width: f32, height: f32) {
        let center = Point { x: width / 2.0, y: height / 2.0 };
        let node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        let n = node_ids.len();

        if n == 0 {
            return;
        }

        let radius = (width.min(height) / 2.0 - 80.0).max(100.0);

        for (i, &id) in node_ids.iter().enumerate() {
            if let Some(node) = self.nodes.get_mut(&id) {
                let angle = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
                node.position = Point {
                    x: center.x + radius * angle.cos(),
                    y: center.y + radius * angle.sin(),
                };
            }
        }
    }

    /// Count all edge crossings in the graph
    fn count_all_crossings(&self) -> usize {
        let mut count = 0;
        for i in 0..self.edges.len() {
            for j in (i + 1)..self.edges.len() {
                let e1 = &self.edges[i];
                let e2 = &self.edges[j];

                // Skip if edges share a node
                if e1.from == e2.from || e1.from == e2.to ||
                   e1.to == e2.from || e1.to == e2.to {
                    continue;
                }

                if let (Some(n1), Some(n2), Some(n3), Some(n4)) = (
                    self.nodes.get(&e1.from),
                    self.nodes.get(&e1.to),
                    self.nodes.get(&e2.from),
                    self.nodes.get(&e2.to),
                ) {
                    if segments_intersect(n1.position, n2.position, n3.position, n4.position) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Count crossings involving edges connected to a specific node
    #[allow(dead_code)]
    fn count_crossings_for_node(&self, node_id: Uuid) -> usize {
        let mut count = 0;
        let connected_edges: Vec<usize> = self.edges.iter()
            .enumerate()
            .filter(|(_, e)| e.from == node_id || e.to == node_id)
            .map(|(i, _)| i)
            .collect();

        for &edge_idx in &connected_edges {
            let e1 = &self.edges[edge_idx];
            for (j, e2) in self.edges.iter().enumerate() {
                if edge_idx >= j {
                    continue;
                }
                if e1.from == e2.from || e1.from == e2.to ||
                   e1.to == e2.from || e1.to == e2.to {
                    continue;
                }

                if let (Some(n1), Some(n2), Some(n3), Some(n4)) = (
                    self.nodes.get(&e1.from),
                    self.nodes.get(&e1.to),
                    self.nodes.get(&e2.from),
                    self.nodes.get(&e2.to),
                ) {
                    if segments_intersect(n1.position, n2.position, n3.position, n4.position) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Find all pairs of crossing edges
    #[allow(dead_code)]
    fn find_crossing_edges(&self) -> Vec<(usize, usize)> {
        let mut crossings = Vec::new();
        for i in 0..self.edges.len() {
            for j in (i + 1)..self.edges.len() {
                let e1 = &self.edges[i];
                let e2 = &self.edges[j];

                if e1.from == e2.from || e1.from == e2.to ||
                   e1.to == e2.from || e1.to == e2.to {
                    continue;
                }

                if let (Some(n1), Some(n2), Some(n3), Some(n4)) = (
                    self.nodes.get(&e1.from),
                    self.nodes.get(&e1.to),
                    self.nodes.get(&e2.from),
                    self.nodes.get(&e2.to),
                ) {
                    if segments_intersect(n1.position, n2.position, n3.position, n4.position) {
                        crossings.push((i, j));
                    }
                }
            }
        }
        crossings
    }

    /// Detect if graph contains cycles using DFS
    #[allow(dead_code)]
    fn detect_cycles(&self) -> bool {
        use std::collections::HashSet;

        let mut visited: HashSet<Uuid> = HashSet::new();

        // Build adjacency list (treat as undirected for cycle detection)
        let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from).or_default().push(edge.to);
            adj.entry(edge.to).or_default().push(edge.from);
        }

        fn has_cycle_dfs(
            node: Uuid,
            parent: Option<Uuid>,
            adj: &HashMap<Uuid, Vec<Uuid>>,
            visited: &mut HashSet<Uuid>,
        ) -> bool {
            visited.insert(node);

            if let Some(neighbors) = adj.get(&node) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        if has_cycle_dfs(neighbor, Some(node), adj, visited) {
                            return true;
                        }
                    } else if parent != Some(neighbor) {
                        // Found a back edge (cycle)
                        return true;
                    }
                }
            }
            false
        }

        for &node_id in self.nodes.keys() {
            if !visited.contains(&node_id) {
                if has_cycle_dfs(node_id, None, &adj, &mut visited) {
                    return true;
                }
            }
        }
        false
    }

    /// Hierarchical layout: organize nodes by type and role with crossing minimization
    #[allow(dead_code)]
    fn hierarchical_layout(&mut self) {
        let center = Point { x: 400.0, y: 300.0 };

        // Group nodes by type using DomainNode
        let mut type_groups: HashMap<String, Vec<Uuid>> = HashMap::new();
        for (id, node) in &self.nodes {
            let type_key = match node.domain_node.data() {
                DomainNodeData::Person { role, .. } => match role {
                    KeyOwnerRole::RootAuthority => "Person_RootAuthority",
                    KeyOwnerRole::SecurityAdmin => "Person_SecurityAdmin",
                    KeyOwnerRole::BackupHolder => "Person_BackupHolder",
                    KeyOwnerRole::Auditor => "Person_Auditor",
                    KeyOwnerRole::Developer => "Person_Developer",
                    KeyOwnerRole::ServiceAccount => "Person_ServiceAccount",
                },
                _ => node.domain_node.injection().display_name(),
            };
            type_groups.entry(type_key.to_string()).or_insert_with(Vec::new).push(*id);
        }

        // Define node type hierarchy (top to bottom)
        // Includes all node types for proper tiered layout
        let type_order = vec![
            // Organization hierarchy
            "Organization",
            "OrganizationalUnit",
            "Role",
            "Policy",
            // Person roles (by authority level)
            "Person_RootAuthority",
            "Person_SecurityAdmin",
            "Person_BackupHolder",
            "Person_Auditor",
            "Person_Developer",
            "Person_ServiceAccount",
            "Location",
            // NATS infrastructure hierarchy
            "NatsOperator",
            "NatsAccount",
            "NatsUser",
            "NatsServiceAccount",
            // PKI trust chain hierarchy
            "RootCertificate",
            "IntermediateCertificate",
            "LeafCertificate",
            "Key",
            // YubiKey hierarchy
            "YubiKey",
            "PivSlot",
            "YubiKeyStatus",
            // Manifest
            "Manifest",
        ];

        // Build ordered tiers for crossing minimization
        let mut tiers: Vec<Vec<Uuid>> = Vec::new();
        for type_name in &type_order {
            if let Some(node_ids) = type_groups.get(*type_name) {
                if !node_ids.is_empty() {
                    tiers.push(node_ids.clone());
                }
            }
        }

        // Build node-to-tier index for quick lookup
        let mut node_tier: HashMap<Uuid, usize> = HashMap::new();
        for (tier_idx, tier) in tiers.iter().enumerate() {
            for &node_id in tier {
                node_tier.insert(node_id, tier_idx);
            }
        }

        // Initial positioning to calculate barycenters
        let y_spacing = 120.0;
        let x_spacing = 150.0;
        let mut y_offset = 100.0;
        for tier in &tiers {
            let total_width = (tier.len() as f32 - 1.0) * x_spacing;
            let start_x = center.x - total_width / 2.0;
            for (i, &node_id) in tier.iter().enumerate() {
                if let Some(node) = self.nodes.get_mut(&node_id) {
                    node.position = Point {
                        x: start_x + (i as f32) * x_spacing,
                        y: y_offset,
                    };
                }
            }
            y_offset += y_spacing;
        }

        // Multi-pass barycenter ordering considering ALL connected neighbors
        for _ in 0..8 {
            // Forward pass
            for tier_idx in 0..tiers.len() {
                self.order_tier_by_global_barycenter(&mut tiers, tier_idx, &node_tier);
            }
            // Backward pass
            for tier_idx in (0..tiers.len()).rev() {
                self.order_tier_by_global_barycenter(&mut tiers, tier_idx, &node_tier);
            }
        }

        // Update positions after barycenter ordering
        y_offset = 100.0;
        for tier in &tiers {
            let total_width = (tier.len() as f32 - 1.0) * x_spacing;
            let start_x = center.x - total_width / 2.0;
            for (i, &node_id) in tier.iter().enumerate() {
                if let Some(node) = self.nodes.get_mut(&node_id) {
                    node.position = Point {
                        x: start_x + (i as f32) * x_spacing,
                        y: y_offset,
                    };
                }
            }
            y_offset += y_spacing;
        }

        // Local swap optimization - swap adjacent nodes if it reduces crossings
        let mut improved = true;
        let mut iterations = 0;
        while improved && iterations < 20 {
            improved = false;
            iterations += 1;

            for tier_idx in 0..tiers.len() {
                let tier_len = tiers[tier_idx].len();
                if tier_len < 2 {
                    continue;
                }

                for i in 0..(tier_len - 1) {
                    // Count crossings with current order
                    let crossings_before = self.count_crossings_for_pair(&tiers, tier_idx, i, i + 1);

                    // Swap and count again
                    tiers[tier_idx].swap(i, i + 1);
                    let crossings_after = self.count_crossings_for_pair(&tiers, tier_idx, i, i + 1);

                    if crossings_after < crossings_before {
                        // Keep the swap
                        improved = true;
                    } else {
                        // Revert
                        tiers[tier_idx].swap(i, i + 1);
                    }
                }
            }
        }

        // Final positioning
        y_offset = 100.0;
        for tier in &tiers {
            let total_width = (tier.len() as f32 - 1.0) * x_spacing;
            let start_x = center.x - total_width / 2.0;
            for (i, &node_id) in tier.iter().enumerate() {
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

    /// Order nodes in a tier by barycenter considering ALL connected neighbors
    fn order_tier_by_global_barycenter(&self, tiers: &mut [Vec<Uuid>], tier_idx: usize, node_tier: &HashMap<Uuid, usize>) {
        if tier_idx >= tiers.len() {
            return;
        }

        // Build position map for ALL nodes based on their current tier position
        let mut all_positions: HashMap<Uuid, f32> = HashMap::new();
        for tier in tiers.iter() {
            for (i, &node_id) in tier.iter().enumerate() {
                all_positions.insert(node_id, i as f32);
            }
        }

        // Calculate weighted barycenter for each node
        let mut barycenters: Vec<(Uuid, f32)> = Vec::new();
        for &node_id in &tiers[tier_idx] {
            let mut sum = 0.0;
            let mut weight_sum = 0.0;

            for edge in &self.edges {
                let neighbor_id = if edge.from == node_id {
                    Some(edge.to)
                } else if edge.to == node_id {
                    Some(edge.from)
                } else {
                    None
                };

                if let Some(neighbor) = neighbor_id {
                    if let Some(&pos) = all_positions.get(&neighbor) {
                        // Weight by tier distance (closer tiers have more influence)
                        let neighbor_tier = node_tier.get(&neighbor).copied().unwrap_or(tier_idx);
                        let tier_dist = (tier_idx as i32 - neighbor_tier as i32).abs() as f32;
                        let weight = 1.0 / (1.0 + tier_dist);
                        sum += pos * weight;
                        weight_sum += weight;
                    }
                }
            }

            let barycenter = if weight_sum > 0.0 {
                sum / weight_sum
            } else {
                // No neighbors - keep current position
                all_positions.get(&node_id).copied().unwrap_or(0.0)
            };

            barycenters.push((node_id, barycenter));
        }

        // Sort by barycenter
        barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        tiers[tier_idx] = barycenters.into_iter().map(|(id, _)| id).collect();
    }

    /// Count edge crossings involving nodes at positions i and i+1 in a tier
    fn count_crossings_for_pair(&self, tiers: &[Vec<Uuid>], tier_idx: usize, i: usize, j: usize) -> usize {
        let node_a = tiers[tier_idx][i];
        let node_b = tiers[tier_idx][j];

        // Get temporary positions based on tier ordering
        let mut positions: HashMap<Uuid, (f32, f32)> = HashMap::new();
        let x_spacing = 150.0;
        let y_spacing = 120.0;
        let center_x = 400.0;
        let mut y = 100.0;

        for tier in tiers {
            let total_width = (tier.len() as f32 - 1.0) * x_spacing;
            let start_x = center_x - total_width / 2.0;
            for (idx, &node_id) in tier.iter().enumerate() {
                positions.insert(node_id, (start_x + (idx as f32) * x_spacing, y));
            }
            y += y_spacing;
        }

        // Count crossings for edges connected to node_a or node_b
        let mut crossings = 0;
        let relevant_edges: Vec<_> = self.edges.iter()
            .filter(|e| e.from == node_a || e.to == node_a || e.from == node_b || e.to == node_b)
            .collect();

        for edge1 in relevant_edges.iter() {
            for edge2 in self.edges.iter() {
                // Skip same edge or edges sharing a node
                if edge1.from == edge2.from || edge1.from == edge2.to ||
                   edge1.to == edge2.from || edge1.to == edge2.to {
                    continue;
                }

                if let (Some(&p1), Some(&p2), Some(&p3), Some(&p4)) = (
                    positions.get(&edge1.from),
                    positions.get(&edge1.to),
                    positions.get(&edge2.from),
                    positions.get(&edge2.to),
                ) {
                    let pt1 = Point::new(p1.0, p1.1);
                    let pt2 = Point::new(p2.0, p2.1);
                    let pt3 = Point::new(p3.0, p3.1);
                    let pt4 = Point::new(p4.0, p4.1);

                    if segments_intersect(pt1, pt2, pt3, pt4) {
                        crossings += 1;
                    }
                }
            }
        }

        crossings
    }

    /// Force-directed layout: physics-based layout for larger graphs
    #[allow(dead_code)]
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

            // Edge crossing penalty - push nodes apart when their edges cross
            let crossing_strength = k * 0.5;  // Strength of crossing avoidance
            for i in 0..self.edges.len() {
                for j in (i + 1)..self.edges.len() {
                    let edge1 = &self.edges[i];
                    let edge2 = &self.edges[j];

                    // Skip if edges share a node
                    if edge1.from == edge2.from || edge1.from == edge2.to ||
                       edge1.to == edge2.from || edge1.to == edge2.to {
                        continue;
                    }

                    if let (Some(n1), Some(n2), Some(n3), Some(n4)) = (
                        self.nodes.get(&edge1.from),
                        self.nodes.get(&edge1.to),
                        self.nodes.get(&edge2.from),
                        self.nodes.get(&edge2.to),
                    ) {
                        // Check if edges cross
                        if segments_intersect(n1.position, n2.position, n3.position, n4.position) {
                            // Push edge midpoints apart
                            let mid1 = Point::new(
                                (n1.position.x + n2.position.x) / 2.0,
                                (n1.position.y + n2.position.y) / 2.0,
                            );
                            let mid2 = Point::new(
                                (n3.position.x + n4.position.x) / 2.0,
                                (n3.position.y + n4.position.y) / 2.0,
                            );

                            let delta = Vector::new(mid1.x - mid2.x, mid1.y - mid2.y);
                            let dist = (delta.x * delta.x + delta.y * delta.y).sqrt().max(0.01);
                            let push = crossing_strength / dist;

                            // Push edge1 nodes away from edge2 midpoint
                            let force1 = Vector::new((delta.x / dist) * push, (delta.y / dist) * push);
                            *displacements.get_mut(&edge1.from).unwrap() =
                                *displacements.get(&edge1.from).unwrap() + force1;
                            *displacements.get_mut(&edge1.to).unwrap() =
                                *displacements.get(&edge1.to).unwrap() + force1;

                            // Push edge2 nodes opposite direction
                            *displacements.get_mut(&edge2.from).unwrap() =
                                *displacements.get(&edge2.from).unwrap() - force1;
                            *displacements.get_mut(&edge2.to).unwrap() =
                                *displacements.get(&edge2.to).unwrap() - force1;
                        }
                    }
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

    pub fn handle_message(&mut self, message: OrganizationIntent) {
        match message {
            OrganizationIntent::NodeClicked(id) => {
                self.selected_node = Some(id);
                // Clear dragging state (click without significant movement)
                self.dragging_node = None;
                self.drag_offset = Vector::new(0.0, 0.0);
                self.drag_start_position = None;
            },
            OrganizationIntent::ExpandIndicatorClicked(_id) => {
                // This is handled at the gui.rs level where we have access to
                // expanded_separation_classes and expanded_categories
                // The graph just receives this message for routing
            },
            OrganizationIntent::NodeDragStarted { node_id, offset } => {
                self.dragging_node = Some(node_id);
                self.drag_offset = offset;
                // Capture starting position for NodeMoved event
                if let Some(node) = self.nodes.get(&node_id) {
                    self.drag_start_position = Some(node.position);
                }
            }
            OrganizationIntent::NodeDragged(cursor_pos) => {
                if let Some(node_id) = self.dragging_node {
                    if let Some(node) = self.nodes.get_mut(&node_id) {
                        // Adjust for zoom and pan transformations
                        let adjusted_x = (cursor_pos.x - self.pan_offset.x) / self.zoom;
                        let adjusted_y = (cursor_pos.y - self.pan_offset.y) / self.zoom;

                        // Temporary position update during drag (no event yet)
                        node.position = Point::new(
                            adjusted_x - self.drag_offset.x,
                            adjusted_y - self.drag_offset.y,
                        );
                    }
                }
            }
            OrganizationIntent::NodeDragEnded => {
                // Create NodeMoved event when drag completes
                if let (Some(node_id), Some(old_position)) = (self.dragging_node, self.drag_start_position) {
                    if let Some(node) = self.nodes.get(&node_id) {
                        let new_position = node.position;

                        // Only create event if position actually changed
                        if (new_position.x - old_position.x).abs() > 0.1
                            || (new_position.y - old_position.y).abs() > 0.1 {
                            use chrono::Utc;

                            let event = GraphEvent::NodeMoved {
                                node_id,
                                old_position,
                                new_position,
                                timestamp: Utc::now(),
                            };

                            self.event_stack.push(event);
                        }
                    }
                }

                self.dragging_node = None;
                self.drag_offset = Vector::new(0.0, 0.0);
                self.drag_start_position = None;
            }
            OrganizationIntent::EdgeClicked { from: _, to: _ } => {}
            OrganizationIntent::ZoomIn => self.zoom = (self.zoom * 1.2).min(10.0),  // Max zoom 10.0 (zoom in closer)
            OrganizationIntent::ZoomOut => self.zoom = (self.zoom / 1.2).max(0.1),  // Min zoom 0.1 (zoom out much further)
            OrganizationIntent::ZoomBy(delta) => {
                // Smooth zoom using exponential scaling
                // delta > 0 = zoom in, delta < 0 = zoom out
                // 0.01 factor gives good responsiveness on high-res touchpads
                let zoom_factor = 1.0 + delta * 0.01;
                self.zoom = (self.zoom * zoom_factor).clamp(0.1, 10.0);
            }
            OrganizationIntent::ResetView => {
                self.zoom = 1.0;  // Reset to 1:1 scale
                self.pan_offset = Vector::new(0.0, 0.0);
            }
            OrganizationIntent::Pan(delta) => {
                self.pan_offset = self.pan_offset + delta;
            }
            OrganizationIntent::AutoLayout => {
                self.auto_layout();
            }
            OrganizationIntent::ApplyLayout(algorithm) => {
                let width = 1200.0;
                let height = 900.0;

                // Store CURRENT visual positions as animation start (before any changes)
                // This captures mid-animation positions if animation is in progress
                let current_positions: HashMap<Uuid, Point> = self.nodes.iter()
                    .map(|(id, node)| (*id, node.position))
                    .collect();

                // If animation was in progress, set nodes to their TARGET positions
                // so the layout algorithm has stable positions to work with
                if self.animating {
                    for (id, target) in &self.animation_targets {
                        if let Some(node) = self.nodes.get_mut(id) {
                            node.position = *target;
                        }
                    }
                    self.animation_targets.clear();
                    self.animation_start.clear();
                    self.animating = false;
                    self.animation_start_time = None;
                }

                // Calculate target positions based on algorithm
                // (layout algorithms modify node.position directly)
                match algorithm {
                    LayoutAlgorithm::Tutte => {
                        self.tutte_barycentric_layout(width, height);
                    }
                    LayoutAlgorithm::FruchtermanReingold => {
                        self.fruchterman_reingold_layout(width, height, 100);
                    }
                    LayoutAlgorithm::Circular => {
                        self.circular_layout(width, height);
                    }
                    LayoutAlgorithm::Hierarchical => {
                        self.hierarchical_layout();
                    }
                    LayoutAlgorithm::TuttePlusFR => {
                        self.tutte_barycentric_layout(width, height);
                        self.fruchterman_reingold_layout(width, height, 50);
                    }
                    LayoutAlgorithm::YubiKeyGrouped => {
                        self.layout_yubikey_grouped();
                    }
                    LayoutAlgorithm::NatsHierarchical => {
                        self.layout_nats_hierarchical();
                    }
                }

                // Store new layout positions as animation targets
                self.animation_targets = self.nodes.iter()
                    .map(|(id, node)| (*id, node.position))
                    .collect();

                // Restore the CURRENT visual positions (including mid-animation positions)
                // so animation starts from where nodes visually are, not where they would end up
                self.animation_start = current_positions.clone();
                for (id, pos) in &current_positions {
                    if let Some(node) = self.nodes.get_mut(id) {
                        node.position = *pos;
                    }
                }

                // Start animation with time-based progress
                self.animation_progress = 0.0;
                self.animating = true;
                self.animation_start_time = Some(std::time::Instant::now());

                let crossings = self.count_all_crossings();
                tracing::info!("Starting animated {:?} layout (will have {} crossings)", algorithm, crossings);
            }
            OrganizationIntent::AnimationTick => {
                // Animation duration in seconds (0.7s feels smooth without being slow)
                const ANIMATION_DURATION: f32 = 0.7;

                if let Some(start_time) = self.animation_start_time {
                    let elapsed = start_time.elapsed().as_secs_f32();
                    let raw_progress = elapsed / ANIMATION_DURATION;

                    if raw_progress >= 1.0 {
                        // Animation complete - set final positions
                        for (id, target) in &self.animation_targets {
                            if let Some(node) = self.nodes.get_mut(id) {
                                node.position = *target;
                            }
                        }
                        self.animation_targets.clear();
                        self.animation_start.clear();
                        self.animating = false;
                        self.animation_start_time = None;
                        self.animation_progress = 1.0;
                    } else {
                        // Apply ease-out cubic: 1 - (1-t)^3
                        let eased = 1.0 - (1.0 - raw_progress).powi(3);
                        self.animation_progress = raw_progress;

                        // Interpolate positions
                        for (id, target) in &self.animation_targets {
                            if let (Some(start), Some(node)) = (
                                self.animation_start.get(id),
                                self.nodes.get_mut(id)
                            ) {
                                node.position = Point::new(
                                    start.x + (target.x - start.x) * eased,
                                    start.y + (target.y - start.y) * eased,
                                );
                            }
                        }
                    }
                }
            }
            OrganizationIntent::AddEdge { from, to, edge_type } => {
                self.add_edge(from, to, edge_type);
                // Complete edge indicator if it's active
                if self.edge_indicator.is_active() {
                    self.edge_indicator.complete();
                }
            }
            // Phase 4: Right-click handled in main GUI (shows context menu)
            OrganizationIntent::RightClick(_) => {}
            // Phase 4: Update edge indicator position during edge creation
            OrganizationIntent::CursorMoved(position) => {
                self.edge_indicator.update_position(position);
            }
            // Phase 4: Cancel edge creation with Esc key
            OrganizationIntent::CancelEdgeCreation => {
                self.edge_indicator.cancel();
            }
            // Phase 4: Delete selected node with Delete key
            OrganizationIntent::DeleteSelected => {
                // Deletion now handled via events in GUI layer
                // The event application will handle node removal and edge cleanup
            }
            // Phase 4: Undo last action
            OrganizationIntent::Undo => {
                if let Some(compensating_event) = self.event_stack.undo() {
                    self.apply_event(&compensating_event);
                }
            }
            // Phase 4: Redo last undone action
            OrganizationIntent::Redo => {
                if let Some(compensating_event) = self.event_stack.redo() {
                    self.apply_event(&compensating_event);
                }
            }
            // Edge editing messages
            OrganizationIntent::EdgeSelected(index) => {
                self.selected_edge = Some(index);
                // Clear node selection when edge is selected
                self.selected_node = None;
            }
            OrganizationIntent::EdgeCreationStarted(from_node) => {
                // Start edge creation indicator from this node
                if let Some(node) = self.nodes.get(&from_node) {
                    self.edge_indicator.start(from_node, node.position);
                }
            }
            OrganizationIntent::EdgeDeleted(index) => {
                if index < self.edges.len() {
                    let edge = self.edges[index].clone();
                    use chrono::Utc;

                    let event = GraphEvent::EdgeDeleted {
                        from: edge.from,
                        to: edge.to,
                        edge_type: edge.edge_type,
                        color: edge.color,
                        timestamp: Utc::now(),
                    };

                    self.event_stack.push(event);
                    self.edges.remove(index);
                    self.selected_edge = None;
                }
            }
            OrganizationIntent::EdgeTypeChanged { edge_index, new_type } => {
                if edge_index < self.edges.len() {
                    let old_type = self.edges[edge_index].edge_type.clone();
                    use chrono::Utc;

                    let event = GraphEvent::EdgeTypeChanged {
                        from: self.edges[edge_index].from,
                        to: self.edges[edge_index].to,
                        old_type,
                        new_type: new_type.clone(),
                        timestamp: Utc::now(),
                    };

                    self.event_stack.push(event);
                    self.edges[edge_index].edge_type = new_type;
                }
            }
            // Canvas clicked - handled in main GUI layer for node creation
            OrganizationIntent::CanvasClicked(_) => {}
            // Role drag-and-drop operations
            OrganizationIntent::RoleDragStarted(source) => {
                // Get initial cursor position from current context (will be updated by RoleDragMoved)
                self.start_role_drag(source, Point::ORIGIN);
            }
            OrganizationIntent::RoleDragMoved(position) => {
                self.update_role_drag(position);
            }
            OrganizationIntent::RoleDragCancelled => {
                self.cancel_role_drag();
            }
            OrganizationIntent::RoleDragDropped => {
                // Drop is handled in main GUI layer which has access to policy data
                // for SoD validation and event generation
                self.cancel_role_drag();
            }
            OrganizationIntent::RoleAssigned { person_id, role_name } => {
                // Role assignment is handled through events, this is just for notification
                tracing::info!("Role '{}' assigned to person {:?}", role_name, person_id);
            }
            OrganizationIntent::RoleRemoved { person_id, role_name } => {
                // Role removal is handled through events
                tracing::info!("Role '{}' removed from person {:?}", role_name, person_id);
            }
        }
    }

    fn calculate_node_position(&self, _id: Uuid) -> Point {
        // Default position - will be repositioned by layout_hierarchical()
        Point::new(400.0, 300.0)
    }

    /// Calculate position for a node based on its tier and index within that tier
    fn calculate_tiered_position(&self, tier: usize, index_in_tier: usize, count_in_tier: usize) -> Point {
        // Tier 0: Organization (top) - Y = 80
        // Tier 1: Units (middle) - Y = 220
        // Tier 2: People (bottom) - Y = 360
        // Tier 3: Infrastructure (NATS, PKI, YubiKey) - Y = 500
        // Tier 4: Certificates/Keys - Y = 640

        let tier_y = match tier {
            0 => 80.0,   // Organization
            1 => 220.0,  // Units
            2 => 360.0,  // People
            3 => 500.0,  // Infrastructure (NATS Operator/Accounts, YubiKeys)
            4 => 640.0,  // Leaf items (NATS Users, Certificates, Keys)
            _ => 780.0,  // Overflow
        };

        // Spread nodes horizontally within their tier
        let canvas_width = 800.0;
        let margin = 100.0;
        let usable_width = canvas_width - (2.0 * margin);

        let x = if count_in_tier <= 1 {
            canvas_width / 2.0  // Center single node
        } else {
            let spacing = usable_width / (count_in_tier - 1) as f32;
            margin + (index_in_tier as f32 * spacing)
        };

        Point::new(x, tier_y)
    }

    /// Reorganize all nodes into a hierarchical tiered layout
    /// Call this after adding all nodes to reposition them properly
    pub fn layout_hierarchical(&mut self) {
        // Collect nodes by tier using the DomainNode fold pattern
        // This replaces the 30-line match statement with a single method call
        let mut tier_0: Vec<Uuid> = Vec::new(); // Root entities (Organization, NATS Operator, Root CA, YubiKey)
        let mut tier_1: Vec<Uuid> = Vec::new(); // Intermediate entities (OrgUnit, NATS Account, Intermediate CA, Role, Policy)
        let mut tier_2: Vec<Uuid> = Vec::new(); // Leaf entities (Person, Location, NATS User, Leaf Cert, Key)
        let tier_3: Vec<Uuid> = Vec::new(); // Reserved for future expansion
        let tier_4: Vec<Uuid> = Vec::new(); // Reserved for future expansion

        for (id, node) in &self.nodes {
            // Use the categorical fold pattern via injection().layout_tier()
            match node.injection().layout_tier() {
                0 => tier_0.push(*id),
                1 => tier_1.push(*id),
                _ => tier_2.push(*id), // Tier 2 and above
            }
        }

        // Reposition each tier
        let tiers = [
            (0, tier_0),
            (1, tier_1),
            (2, tier_2),
            (3, tier_3),
            (4, tier_4),
        ];

        // Calculate all positions first (to avoid borrow conflicts)
        let mut position_updates: Vec<(Uuid, Point)> = Vec::new();

        for (tier_num, tier_nodes) in tiers {
            let count = tier_nodes.len();
            for (index, id) in tier_nodes.into_iter().enumerate() {
                let pos = self.calculate_tiered_position(tier_num, index, count);
                position_updates.push((id, pos));
            }
        }

        // Apply all position updates
        for (id, pos) in position_updates {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.position = pos;
            }
        }
    }

    /// Specialized layout for YubiKey graphs
    /// Groups PIV slots under their parent YubiKey with better spacing
    pub fn layout_yubikey_grouped(&mut self) {
        // Collect YubiKeys and PIV slots using the DomainNode accessor pattern
        // This replaces the match statement with injection() checks and accessor methods
        let mut yubikeys: Vec<(Uuid, String)> = Vec::new();  // (node_id, serial)
        let mut piv_slots: Vec<(Uuid, String)> = Vec::new(); // (node_id, yubikey_serial)

        for (id, node) in &self.nodes {
            let injection = node.injection();
            if injection == super::domain_node::Injection::YubiKey {
                if let Some(serial) = node.domain_node.yubikey_serial() {
                    yubikeys.push((*id, serial.to_string()));
                }
            } else if injection == super::domain_node::Injection::PivSlot {
                if let Some(serial) = node.domain_node.yubikey_serial() {
                    piv_slots.push((*id, serial.to_string()));
                }
            }
        }

        // Sort YubiKeys by serial for consistent ordering
        yubikeys.sort_by(|a, b| a.1.cmp(&b.1));

        // Layout constants - wider spacing for YubiKeys
        let yubikey_spacing = 220.0;  // Horizontal space per YubiKey column
        let start_x = 120.0;          // Left margin
        let yubikey_y = 100.0;        // Y position for YubiKey row
        let slot_start_y = 200.0;     // Y position for first row of slots
        let slot_spacing_x = 90.0;    // Horizontal spacing between slots in a group
        let slot_spacing_y = 80.0;    // Vertical spacing between slot rows

        let mut position_updates: Vec<(Uuid, Point)> = Vec::new();

        // Position YubiKeys horizontally
        for (idx, (yubikey_id, serial)) in yubikeys.iter().enumerate() {
            let x = start_x + (idx as f32 * yubikey_spacing);
            position_updates.push((*yubikey_id, Point::new(x, yubikey_y)));

            // Find PIV slots for this YubiKey
            let slots_for_yubikey: Vec<Uuid> = piv_slots
                .iter()
                .filter(|(_, yk_serial)| yk_serial == serial)
                .map(|(id, _)| *id)
                .collect();

            // Position slots in a 2x2 grid under this YubiKey
            // Centered under the parent YubiKey
            for (slot_idx, slot_id) in slots_for_yubikey.iter().enumerate() {
                let col = slot_idx % 2;
                let row = slot_idx / 2;
                let slot_x = x - (slot_spacing_x / 2.0) + (col as f32 * slot_spacing_x);
                let slot_y = slot_start_y + (row as f32 * slot_spacing_y);
                position_updates.push((*slot_id, Point::new(slot_x, slot_y)));
            }
        }

        // Apply all position updates
        for (id, pos) in position_updates {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.position = pos;
            }
        }
    }

    /// Specialized layout for NATS infrastructure graphs
    /// Arranges: Operator (top) -> Accounts (middle row) -> Users (bottom rows under their accounts)
    pub fn layout_nats_hierarchical(&mut self) {
        // Collect NATS nodes by type using the DomainNode accessor pattern
        // This replaces the match statement with injection() checks and accessor methods
        use super::domain_node::Injection;

        let mut operators: Vec<Uuid> = Vec::new();
        let mut accounts: Vec<(Uuid, String)> = Vec::new(); // (id, name)
        let mut users: Vec<(Uuid, String)> = Vec::new();    // (id, account_name)

        for (id, node) in &self.nodes {
            let injection = node.injection();
            match injection {
                Injection::NatsOperator | Injection::NatsOperatorSimple => {
                    operators.push(*id);
                }
                Injection::NatsAccount | Injection::NatsAccountSimple => {
                    // Use the nats_account_name accessor, fall back to label
                    let name = node.domain_node.nats_account_name()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| node.label.clone());
                    accounts.push((*id, name));
                }
                Injection::NatsUser | Injection::NatsUserSimple => {
                    // Use the nats_user_account_name accessor
                    let account_name = node.domain_node.nats_user_account_name()
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    users.push((*id, account_name));
                }
                Injection::NatsServiceAccount => {
                    users.push((*id, String::new()));
                }
                _ => {}
            }
        }

        // Layout constants
        let center_x = 500.0;
        let operator_y = 80.0;
        let account_y = 220.0;
        let user_start_y = 360.0;
        let account_spacing = 250.0;
        let user_spacing_x = 120.0;
        let user_spacing_y = 100.0;

        let mut position_updates: Vec<(Uuid, Point)> = Vec::new();

        // Position operators at top center
        let op_count = operators.len();
        for (idx, op_id) in operators.iter().enumerate() {
            let x = center_x + ((idx as f32) - (op_count as f32 - 1.0) / 2.0) * 200.0;
            position_updates.push((*op_id, Point::new(x, operator_y)));
        }

        // Position accounts in a row
        let account_count = accounts.len();
        let accounts_start_x = center_x - ((account_count as f32 - 1.0) / 2.0) * account_spacing;

        for (idx, (account_id, account_name)) in accounts.iter().enumerate() {
            let x = accounts_start_x + (idx as f32) * account_spacing;
            position_updates.push((*account_id, Point::new(x, account_y)));

            // Find users for this account and position them below
            let account_users: Vec<Uuid> = users.iter()
                .filter(|(_, acct)| acct == account_name || acct.is_empty())
                .map(|(id, _)| *id)
                .collect();

            for (user_idx, user_id) in account_users.iter().enumerate() {
                let col = user_idx % 3;
                let row = user_idx / 3;
                let user_x = x - user_spacing_x + (col as f32) * user_spacing_x;
                let user_y = user_start_y + (row as f32) * user_spacing_y;
                position_updates.push((*user_id, Point::new(user_x, user_y)));
            }
        }

        // Apply all position updates
        for (id, pos) in position_updates {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.position = pos;
            }
        }
    }

    /// Check if a node should be visible based on current filter settings
    pub fn should_show_node(&self, node: &ConceptEntity) -> bool {
        let injection = node.domain_node.injection();
        match injection {
            Injection::Person => self.filter_show_people,
            Injection::Organization | Injection::OrganizationUnit |
            Injection::Location | Injection::Role | Injection::Policy |
            Injection::PolicyRole | Injection::PolicyClaim |
            Injection::PolicyCategory | Injection::PolicyGroup |
            Injection::Manifest => self.filter_show_orgs,
            _ if injection.is_nats() => self.filter_show_nats,
            _ if injection.is_certificate() => self.filter_show_pki,
            _ if injection.is_yubikey() => self.filter_show_yubikey,
            Injection::Key => self.filter_show_pki,
            _ => true,
        }
    }

    /// Apply the specified layout algorithm to reposition all nodes
    pub fn apply_layout(&mut self, layout: GraphLayout) {
        match layout {
            GraphLayout::Manual => {
                // Manual layout - do nothing, keep current positions
            }
            GraphLayout::Hierarchical => {
                self.apply_hierarchical_layout();
            }
            GraphLayout::ForceDirected => {
                self.apply_force_directed_layout();
            }
            GraphLayout::Circular => {
                self.apply_circular_layout();
            }
        }
    }

    /// Hierarchical tree layout - arranges nodes in layers based on relationships
    fn apply_hierarchical_layout(&mut self) {
        let mut layers: Vec<Vec<Uuid>> = Vec::new();
        let mut visited: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

        // Find root nodes (nodes with no incoming edges)
        let mut root_nodes: Vec<Uuid> = Vec::new();
        for node_id in self.nodes.keys() {
            let has_incoming = self.edges.iter().any(|e| e.to == *node_id);
            if !has_incoming {
                root_nodes.push(*node_id);
            }
        }

        if !root_nodes.is_empty() {
            layers.push(root_nodes.clone());
            visited.extend(root_nodes.iter());
        }

        // Build layers using BFS
        while let Some(current_layer) = layers.last().cloned() {
            let mut next_layer = Vec::new();
            for node_id in current_layer {
                // Find all children (outgoing edges)
                for edge in &self.edges {
                    if edge.from == node_id && !visited.contains(&edge.to) {
                        next_layer.push(edge.to);
                        visited.insert(edge.to);
                    }
                }
            }
            if next_layer.is_empty() {
                break;
            }
            layers.push(next_layer);
        }

        // Position nodes in layers
        let layer_height = 150.0;
        let node_spacing = 120.0;

        for (layer_index, layer) in layers.iter().enumerate() {
            let y = 100.0 + (layer_index as f32) * layer_height;
            let layer_width = (layer.len() as f32) * node_spacing;
            let start_x = -layer_width / 2.0;

            for (node_index, node_id) in layer.iter().enumerate() {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.position = Point::new(
                        start_x + (node_index as f32) * node_spacing,
                        y,
                    );
                }
            }
        }
    }

    /// Force-directed layout using simple spring simulation
    fn apply_force_directed_layout(&mut self) {
        // Simple force-directed algorithm
        let iterations = 100;
        let k = 80.0; // Optimal distance
        let c = 0.01; // Cooling factor

        for iteration in 0..iterations {
            let mut forces: HashMap<Uuid, Vector> = HashMap::new();

            // Calculate repulsive forces between all nodes
            let node_ids: Vec<Uuid> = self.nodes.keys().cloned().collect();
            for i in 0..node_ids.len() {
                for j in (i + 1)..node_ids.len() {
                    let id1 = node_ids[i];
                    let id2 = node_ids[j];

                    if let (Some(node1), Some(node2)) = (self.nodes.get(&id1), self.nodes.get(&id2)) {
                        let dx = node2.position.x - node1.position.x;
                        let dy = node2.position.y - node1.position.y;
                        let distance = (dx * dx + dy * dy).sqrt().max(0.1);

                        let repulsion = (k * k) / distance;
                        let fx = (dx / distance) * repulsion;
                        let fy = (dy / distance) * repulsion;

                        let force1 = forces.entry(id1).or_insert(Vector::new(0.0, 0.0));
                        *force1 = Vector::new(force1.x - fx, force1.y - fy);
                        let force2 = forces.entry(id2).or_insert(Vector::new(0.0, 0.0));
                        *force2 = Vector::new(force2.x + fx, force2.y + fy);
                    }
                }
            }

            // Calculate attractive forces for connected nodes
            for edge in &self.edges {
                if let (Some(node1), Some(node2)) = (self.nodes.get(&edge.from), self.nodes.get(&edge.to)) {
                    let dx = node2.position.x - node1.position.x;
                    let dy = node2.position.y - node1.position.y;
                    let distance = (dx * dx + dy * dy).sqrt().max(0.1);

                    let attraction = (distance * distance) / k;
                    let fx = (dx / distance) * attraction;
                    let fy = (dy / distance) * attraction;

                    let force_from = forces.entry(edge.from).or_insert(Vector::new(0.0, 0.0));
                    *force_from = Vector::new(force_from.x + fx, force_from.y + fy);
                    let force_to = forces.entry(edge.to).or_insert(Vector::new(0.0, 0.0));
                    *force_to = Vector::new(force_to.x - fx, force_to.y - fy);
                }
            }

            // Apply forces with cooling
            let temperature = 1.0 - (iteration as f32 / iterations as f32);
            for (node_id, force) in forces {
                if let Some(node) = self.nodes.get_mut(&node_id) {
                    node.position.x += force.x * c * temperature;
                    node.position.y += force.y * c * temperature;
                }
            }
        }
    }

    /// Circular layout - arranges nodes in concentric circles by type
    fn apply_circular_layout(&mut self) {
        

        // Group nodes by type
        let mut person_nodes = Vec::new();
        let mut org_nodes = Vec::new();
        let mut nats_nodes = Vec::new();
        let mut pki_nodes = Vec::new();
        let mut yubikey_nodes = Vec::new();

        for (id, node) in &self.nodes {
            let injection = node.domain_node.injection();
            match injection {
                Injection::Person => person_nodes.push(*id),
                Injection::Organization | Injection::OrganizationUnit |
                Injection::Location | Injection::Role | Injection::Policy |
                Injection::Manifest | Injection::PolicyRole | Injection::PolicyClaim |
                Injection::PolicyCategory | Injection::PolicyGroup => {
                    org_nodes.push(*id);
                }
                _ if injection.is_nats() => nats_nodes.push(*id),
                _ if injection.is_certificate() => pki_nodes.push(*id),
                _ if injection.is_yubikey() => yubikey_nodes.push(*id),
                Injection::Key => pki_nodes.push(*id),
                _ => org_nodes.push(*id),
            }
        }

        // Position each group in concentric circles
        self.position_circle(&person_nodes, 80.0);
        self.position_circle(&org_nodes, 160.0);
        self.position_circle(&nats_nodes, 240.0);
        self.position_circle(&pki_nodes, 320.0);
        self.position_circle(&yubikey_nodes, 400.0);
    }

    /// Helper to position nodes in a circle
    fn position_circle(&mut self, node_ids: &[Uuid], radius: f32) {
        use std::f32::consts::PI;

        if node_ids.is_empty() {
            return;
        }

        let angle_step = 2.0 * PI / node_ids.len() as f32;

        for (index, node_id) in node_ids.iter().enumerate() {
            let angle = index as f32 * angle_step;
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.position = Point::new(
                    radius * angle.cos(),
                    radius * angle.sin(),
                );
            }
        }
    }

    /// Subscription for animation ticks - only active when animating
    pub fn subscription(&self) -> iced::Subscription<OrganizationIntent> {
        use std::time::Duration;

        if self.animating {
            // 60fps = ~16.67ms per frame
            iced::time::every(Duration::from_millis(16))
                .map(|_| OrganizationIntent::AnimationTick)
        } else {
            iced::Subscription::none()
        }
    }
}

/// Implementation of canvas::Program for graph rendering
/// Canvas state for tracking interaction during event processing
#[derive(Debug, Default, Clone)]
pub struct CanvasState {
    dragging_node: Option<Uuid>,
    drag_start_pos: Option<Point>,
    // Panning state - track last cursor position for smooth frame-to-frame delta
    panning: bool,
    last_pan_pos: Option<Point>,
}

impl canvas::Program<OrganizationIntent> for OrganizationConcept {
    type State = CanvasState;

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
                // Only draw edge if both nodes are visible
                if !self.should_show_node(from_node) || !self.should_show_node(to_node) {
                    continue;
                }
                let edge_path = canvas::Path::line(
                    from_node.position,
                    to_node.position,
                );

                // Use dashed line for IncompatibleWith (SoD) edges
                let stroke = if matches!(edge.edge_type, EdgeType::IncompatibleWith) {
                    // Create base stroke with color and width
                    let base = canvas::Stroke::default()
                        .with_color(edge.color)
                        .with_width(2.5);
                    // Use struct update syntax to add dashed line pattern
                    canvas::Stroke {
                        line_dash: canvas::LineDash {
                            segments: &[8.0, 4.0], // 8px dash, 4px gap
                            offset: 0,
                        },
                        ..base
                    }
                } else {
                    canvas::Stroke::default()
                        .with_color(edge.color)
                        .with_width(2.0)
                };

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
                    EdgeType::HasRole { .. } => "has role",
                    EdgeType::IncompatibleWith => "incompatible with",
                    EdgeType::RoleContainsClaim => "grants",
                    EdgeType::CategoryContainsClaim => "contains",
                    EdgeType::ClassContainsRole => "includes",
                    EdgeType::RoleRequiresPolicy => "requires",
                    EdgeType::PolicyGovernsEntity => "governs",

                    // Trust
                    EdgeType::Trusts => "trusts",
                    EdgeType::CertifiedBy => "certified by",

                    // NATS Infrastructure
                    EdgeType::Signs => "signs",
                    EdgeType::BelongsToAccount => "belongs to",
                    EdgeType::MapsToOrgUnit => "maps to",
                    EdgeType::MapsToPerson => "maps to",

                    // PKI Trust Chain
                    EdgeType::SignedBy => "signed by",
                    EdgeType::CertifiesKey => "certifies",
                    EdgeType::IssuedTo => "issued to",

                    // YubiKey Hardware
                    EdgeType::OwnsYubiKey => "owns",
                    EdgeType::AssignedTo => "assigned to",
                    EdgeType::HasSlot => "has slot",
                    EdgeType::StoresKey => "stores key",
                    EdgeType::LoadedCertificate => "has cert",
                    EdgeType::Requires => "requires",

                    // Key relationships (Phase 4)
                    EdgeType::KeyRotation => "rotates",
                    EdgeType::CertificateUsesKey => "uses key",
                    EdgeType::StoredInYubiKeySlot(_) => "in slot",

                    // Organizational hierarchy (Phase 4)
                    EdgeType::ResponsibleFor => "responsible for",
                    EdgeType::Manages => "manages",
                    EdgeType::ManagesResource => "manages",
                    EdgeType::ManagedBy => "managed by",

                    // Policy relationships (Phase 4)
                    EdgeType::DefinesRole => "defines",
                    EdgeType::DefinesPolicy => "defines",

                    // Trust and Access (Phase 4)
                    EdgeType::HasAccess => "has access",

                    // Export and Manifest (Phase 4)
                    EdgeType::ExportedTo => "exported to",
                    EdgeType::SignedByPerson => "signed by",

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
                frame.fill(&label_bg, Color::from_rgba(0.15, 0.15, 0.15, 0.8));  // Dark semi-transparent for black bg
                frame.stroke(&label_bg, canvas::Stroke::default()
                    .with_color(edge.color)
                    .with_width(1.0));

                // Draw label text
                frame.fill_text(canvas::Text {
                    content: edge_label.to_string(),
                    position: label_position,
                    color: Color::from_rgb(0.9, 0.9, 0.9),  // Light text for visibility
                    size: iced::Pixels(10.0),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    line_height: LineHeight::default(),
                    shaping: Shaping::Advanced,
                });
            }
        }

        // Draw drop target indicators for ALL person nodes when dragging a role
        // This provides visual feedback about valid drop targets
        if self.dragging_role.is_some() {
            for (node_id, node) in &self.nodes {
                if !self.should_show_node(node) {
                    continue;
                }
                // Only highlight Person nodes as valid drop targets
                if node.injection() == super::domain_node::Injection::Person {
                    let is_hovered = self.dragging_role.as_ref()
                        .map(|d| d.hover_person == Some(*node_id))
                        .unwrap_or(false);

                    // Skip the hovered node - it gets special highlighting later
                    if is_hovered {
                        continue;
                    }

                    // Draw a subtle pulsing ring around valid drop targets
                    let indicator_radius = 30.0;
                    let indicator_circle = canvas::Path::circle(node.position, indicator_radius);
                    let indicator_stroke = canvas::Stroke::default()
                        .with_color(Color::from_rgba(0.4, 0.7, 0.9, 0.4)) // Light blue, semi-transparent
                        .with_width(2.0);
                    frame.stroke(&indicator_circle, indicator_stroke);

                    // Draw small "+" indicator to show this is a drop target
                    frame.fill_text(canvas::Text {
                        content: "+".to_string(),
                        position: Point::new(node.position.x + 25.0, node.position.y - 20.0),
                        color: Color::from_rgba(0.4, 0.7, 0.9, 0.7),
                        size: iced::Pixels(12.0),
                        font: iced::Font::DEFAULT,
                        horizontal_alignment: iced::alignment::Horizontal::Center,
                        vertical_alignment: iced::alignment::Vertical::Center,
                        line_height: LineHeight::default(),
                        shaping: Shaping::Advanced,
                    });
                }
            }
        }

        // Draw nodes with 3D disc effect (tiddlywinks/necco wafer style)
        for (node_id, node) in &self.nodes {
            // Only draw node if it matches current filter settings
            if !self.should_show_node(node) {
                continue;
            }

            let is_selected = self.selected_node == Some(*node_id);
            let radius = if is_selected { 25.0 } else { 20.0 };

            // === Phase 4: 3D Disc Effect ===

            // 1. Drop shadow (offset down-right for depth)
            let shadow_offset = Point::new(
                node.position.x + 2.0,
                node.position.y + 2.0,
            );
            let shadow_circle = canvas::Path::circle(shadow_offset, radius);
            frame.fill(&shadow_circle, Color::from_rgba(0.0, 0.0, 0.0, 0.3));

            // 2. Base disc with gradient effect (concentric circles)
            // Outer layer (darker edge for depth)
            let outer_color = Color {
                r: node.color.r * 0.7,
                g: node.color.g * 0.7,
                b: node.color.b * 0.7,
                a: 1.0,
            };
            let outer_circle = canvas::Path::circle(node.position, radius);
            frame.fill(&outer_circle, outer_color);

            // Middle layer (base color)
            let mid_circle = canvas::Path::circle(node.position, radius * 0.85);
            frame.fill(&mid_circle, node.color);

            // Inner highlight (lighter center for 3D effect)
            let highlight_color = Color {
                r: (node.color.r + 0.3).min(1.0),
                g: (node.color.g + 0.3).min(1.0),
                b: (node.color.b + 0.3).min(1.0),
                a: 1.0,
            };
            let inner_circle = canvas::Path::circle(node.position, radius * 0.5);
            frame.fill(&inner_circle, highlight_color);

            // 3. Top highlight (glossy effect - small bright spot)
            let highlight_pos = Point::new(
                node.position.x - radius * 0.3,
                node.position.y - radius * 0.3,
            );
            let highlight = canvas::Path::circle(highlight_pos, radius * 0.25);
            frame.fill(&highlight, Color::from_rgba(1.0, 1.0, 1.0, 0.5));

            // 4. Selection ring if selected
            if is_selected {
                let selection_ring = canvas::Path::circle(node.position, radius + 3.0);
                let stroke = canvas::Stroke::default()
                    .with_color(Color::from_rgb(1.0, 0.84, 0.0)) // Gold color
                    .with_width(3.0);
                frame.stroke(&selection_ring, stroke);
            }

            // 5. Border (defines the disc edge) - bright for visibility on black
            let circle = canvas::Path::circle(node.position, radius);
            let border_stroke = canvas::Stroke::default()
                .with_color(Color::from_rgba(0.8, 0.8, 0.9, 0.7))  // Light border for black bg
                .with_width(if is_selected { 2.5 } else { 2.0 });  // Slightly thicker
            frame.stroke(&circle, border_stroke);

            // Draw node properties using the DomainNode fold pattern
            // This replaces the 180-line match statement with a single method call
            let viz = node.visualization();
            let (type_icon, type_font, primary_text, secondary_text) = (
                viz.icon,
                viz.icon_font,
                viz.primary_text,
                viz.secondary_text,
            );

            // Primary text (below node)
            let name_position = Point::new(
                node.position.x,
                node.position.y + radius + 12.0,
            );
            frame.fill_text(canvas::Text {
                content: primary_text,
                position: name_position,
                color: Color::from_rgb(1.0, 1.0, 1.0),  // White for black bg
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
                color: Color::from_rgb(0.7, 0.7, 0.8),  // Light gray for black bg
                size: iced::Pixels(10.0),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Top,
                line_height: LineHeight::default(),
                shaping: Shaping::Advanced,
            });

            // Draw +/- expansion indicator for expandable nodes (PolicyGroup, PolicyCategory)
            // Using fold pattern: viz.expandable and viz.expanded from VisualizationData
            if viz.expandable {
                let expanded = viz.expanded;

                // Position indicator below secondary text (with extra spacing)
                let indicator_y = node.position.y + radius + 52.0;
                let indicator_center = Point::new(node.position.x, indicator_y);
                let indicator_radius = 10.0;

                // Draw indicator circle (blue background)
                let indicator_circle = canvas::Path::circle(indicator_center, indicator_radius);
                frame.fill(&indicator_circle, Color::from_rgb(0.3, 0.5, 0.8));

                // Draw border
                let border = canvas::Path::circle(indicator_center, indicator_radius);
                frame.stroke(&border, canvas::Stroke::default()
                    .with_color(Color::from_rgb(0.5, 0.7, 1.0))
                    .with_width(1.5));

                // Draw + or - symbol
                let symbol = if expanded { "−" } else { "+" };
                frame.fill_text(canvas::Text {
                    content: symbol.to_string(),
                    position: indicator_center,
                    color: Color::WHITE,
                    size: iced::Pixels(16.0),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    line_height: LineHeight::default(),
                    shaping: Shaping::Advanced,
                });
            }

            // Draw role badges for Person nodes (compact mode)
            // Using injection() to check node type
            if node.injection() == super::domain_node::Injection::Person {
                if let Some(badges) = self.role_badges.get(&node.id) {
                    let badge_y = node.position.y + radius + 42.0;
                    let badge_spacing = 24.0;
                    let total_width = (badges.badges.len() as f32) * badge_spacing;
                    let start_x = node.position.x - total_width / 2.0 + badge_spacing / 2.0;

                    for (i, badge) in badges.badges.iter().enumerate() {
                        let badge_x = start_x + (i as f32) * badge_spacing;
                        let badge_center = Point::new(badge_x, badge_y);

                        // Draw badge circle
                        let badge_circle = canvas::Path::circle(badge_center, 8.0);
                        frame.fill(&badge_circle, badge.color());

                        // Draw badge abbreviation
                        frame.fill_text(canvas::Text {
                            content: badge.abbrev(),
                            position: badge_center,
                            color: Color::WHITE,
                            size: iced::Pixels(7.0),
                            font: iced::Font::DEFAULT,
                            horizontal_alignment: iced::alignment::Horizontal::Center,
                            vertical_alignment: iced::alignment::Vertical::Center,
                            line_height: LineHeight::default(),
                            shaping: Shaping::Advanced,
                        });
                    }

                    // Draw "+N more" indicator if there are more roles
                    if badges.has_more {
                        let more_x = start_x + (badges.badges.len() as f32) * badge_spacing;
                        frame.fill_text(canvas::Text {
                            content: "+".to_string(),
                            position: Point::new(more_x, badge_y),
                            color: Color::from_rgb(0.6, 0.6, 0.7),
                            size: iced::Pixels(9.0),
                            font: iced::Font::DEFAULT,
                            horizontal_alignment: iced::alignment::Horizontal::Center,
                            vertical_alignment: iced::alignment::Vertical::Center,
                            line_height: LineHeight::default(),
                            shaping: Shaping::Advanced,
                        });
                    }
                }
            }

            // Type icon (above node)
            let icon_position = Point::new(
                node.position.x,
                node.position.y - radius - 8.0,
            );
            frame.fill_text(canvas::Text {
                content: type_icon.to_string(),
                position: icon_position,
                color: node.color,
                size: iced::Pixels(18.0),
                font: type_font,
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Bottom,
                line_height: LineHeight::default(),
                shaping: Shaping::Advanced,
            });
        }

        // Phase 4: Draw edge creation indicator (if active)
        self.edge_indicator.draw(&mut frame, self);

        // Draw ghost node during role drag
        if let Some(ref drag) = self.dragging_role {
            let ghost_pos = drag.cursor_position;
            let role_name = match &drag.source {
                DragSource::RoleFromPalette { role_name, .. } => role_name,
                DragSource::RoleFromPerson { role_name, .. } => role_name,
            };
            let role_color = self.get_dragging_role_color().unwrap_or(Color::from_rgb(0.5, 0.5, 0.5));

            // Determine ghost node border color based on SoD conflicts and hover state
            let (border_color, border_width) = if drag.hover_person.is_some() {
                if drag.sod_conflicts.is_empty() {
                    (Color::from_rgb(0.2, 0.8, 0.3), 3.0) // Green - valid drop
                } else {
                    (Color::from_rgb(0.9, 0.3, 0.2), 3.0) // Red - conflicts
                }
            } else {
                (Color::from_rgb(0.6, 0.6, 0.7), 2.0) // Gray - no target
            };

            // Draw ghost node circle with transparency
            let ghost_radius = 20.0;
            let ghost_circle = canvas::Path::circle(ghost_pos, ghost_radius);
            frame.fill(&ghost_circle, Color::from_rgba8(
                (role_color.r * 255.0) as u8,
                (role_color.g * 255.0) as u8,
                (role_color.b * 255.0) as u8,
                180.0, // Semi-transparent (alpha as f32, 0.0-255.0 range)
            ));

            // Draw border
            let border_stroke = canvas::Stroke::default()
                .with_color(border_color)
                .with_width(border_width);
            frame.stroke(&ghost_circle, border_stroke);

            // Draw role name abbreviation
            let abbrev: String = if role_name.len() <= 3 {
                role_name.clone()
            } else {
                let words: Vec<&str> = role_name.split_whitespace().collect();
                if words.len() >= 2 {
                    words.iter().take(3).filter_map(|w| w.chars().next()).collect()
                } else {
                    role_name.chars().take(3).collect()
                }
            };

            frame.fill_text(canvas::Text {
                content: abbrev,
                position: ghost_pos,
                color: Color::WHITE,
                size: iced::Pixels(10.0),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                line_height: LineHeight::default(),
                shaping: Shaping::Advanced,
            });

            // Draw conflict indicator if hovering over person with conflicts
            if drag.hover_person.is_some() && !drag.sod_conflicts.is_empty() {
                // Draw X mark
                frame.fill_text(canvas::Text {
                    content: "✗".to_string(),
                    position: Point::new(ghost_pos.x + ghost_radius - 2.0, ghost_pos.y - ghost_radius + 2.0),
                    color: Color::from_rgb(0.9, 0.2, 0.2),
                    size: iced::Pixels(14.0),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    line_height: LineHeight::default(),
                    shaping: Shaping::Advanced,
                });
            }

            // Highlight the hovered person node
            if let Some(hover_id) = drag.hover_person {
                if let Some(hover_node) = self.nodes.get(&hover_id) {
                    let highlight_color = if drag.sod_conflicts.is_empty() {
                        Color::from_rgba(0.2, 0.8, 0.3, 0.3) // Green glow
                    } else {
                        Color::from_rgba(0.9, 0.3, 0.2, 0.3) // Red glow
                    };
                    let highlight_circle = canvas::Path::circle(hover_node.position, 35.0);
                    frame.fill(&highlight_circle, highlight_color);
                }
            }

            // Draw tooltip overlay with role details and conflicts
            let tooltip_pos = Point::new(ghost_pos.x + 30.0, ghost_pos.y - 20.0);
            let tooltip_width = 180.0;
            let tooltip_line_height = 14.0;

            // Calculate tooltip height based on content
            let conflict_count = drag.sod_conflicts.len();
            let tooltip_height = if conflict_count > 0 {
                30.0 + (conflict_count.min(3) as f32) * tooltip_line_height + 10.0
            } else if drag.hover_person.is_some() {
                40.0 // "Drop to assign" message
            } else {
                30.0 // Just role name
            };

            // Draw tooltip background
            let tooltip_bg = canvas::Path::rectangle(
                tooltip_pos,
                iced::Size::new(tooltip_width, tooltip_height),
            );
            frame.fill(&tooltip_bg, Color::from_rgba(0.1, 0.1, 0.15, 0.95));
            frame.stroke(&tooltip_bg, canvas::Stroke::default()
                .with_color(Color::from_rgba(0.5, 0.5, 0.6, 0.8))
                .with_width(1.0));

            // Draw role name header
            frame.fill_text(canvas::Text {
                content: format!("📋 {}", role_name),
                position: Point::new(tooltip_pos.x + 8.0, tooltip_pos.y + 12.0),
                color: Color::WHITE,
                size: iced::Pixels(11.0),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Center,
                line_height: LineHeight::default(),
                shaping: Shaping::Advanced,
            });

            // Show action hint or conflicts
            if !drag.sod_conflicts.is_empty() {
                // Show "SoD Violation" warning
                frame.fill_text(canvas::Text {
                    content: "⚠️ SoD Conflicts:".to_string(),
                    position: Point::new(tooltip_pos.x + 8.0, tooltip_pos.y + 26.0),
                    color: Color::from_rgb(0.9, 0.6, 0.2),
                    size: iced::Pixels(10.0),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    line_height: LineHeight::default(),
                    shaping: Shaping::Advanced,
                });

                // List conflicting roles (max 3)
                for (idx, conflict) in drag.sod_conflicts.iter().take(3).enumerate() {
                    frame.fill_text(canvas::Text {
                        content: format!("• {}", conflict.conflicting_role),
                        position: Point::new(
                            tooltip_pos.x + 12.0,
                            tooltip_pos.y + 40.0 + (idx as f32) * tooltip_line_height,
                        ),
                        color: Color::from_rgb(0.9, 0.4, 0.3),
                        size: iced::Pixels(9.0),
                        font: iced::Font::DEFAULT,
                        horizontal_alignment: iced::alignment::Horizontal::Left,
                        vertical_alignment: iced::alignment::Vertical::Center,
                        line_height: LineHeight::default(),
                        shaping: Shaping::Advanced,
                    });
                }

                if drag.sod_conflicts.len() > 3 {
                    frame.fill_text(canvas::Text {
                        content: format!("  +{} more...", drag.sod_conflicts.len() - 3),
                        position: Point::new(
                            tooltip_pos.x + 12.0,
                            tooltip_pos.y + 40.0 + 3.0 * tooltip_line_height,
                        ),
                        color: Color::from_rgb(0.7, 0.5, 0.4),
                        size: iced::Pixels(9.0),
                        font: iced::Font::DEFAULT,
                        horizontal_alignment: iced::alignment::Horizontal::Left,
                        vertical_alignment: iced::alignment::Vertical::Center,
                        line_height: LineHeight::default(),
                        shaping: Shaping::Advanced,
                    });
                }
            } else if drag.hover_person.is_some() {
                // Show "Drop to assign" hint
                frame.fill_text(canvas::Text {
                    content: "✓ Drop to assign role".to_string(),
                    position: Point::new(tooltip_pos.x + 8.0, tooltip_pos.y + 26.0),
                    color: Color::from_rgb(0.3, 0.8, 0.4),
                    size: iced::Pixels(10.0),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    line_height: LineHeight::default(),
                    shaping: Shaping::Advanced,
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,  // Canvas widget bounds for coordinate conversion
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<OrganizationIntent>) {
        if let mouse::Cursor::Available(cursor_position) = cursor {
            // Convert window coordinates to canvas-relative coordinates
            let canvas_relative_x = cursor_position.x - bounds.x;
            let canvas_relative_y = cursor_position.y - bounds.y;
            let canvas_relative = Point::new(canvas_relative_x, canvas_relative_y);

            // Adjust cursor position for zoom and pan (for node hit detection)
            let adjusted_position = Point::new(
                (canvas_relative_x - self.pan_offset.x) / self.zoom,
                (canvas_relative_y - self.pan_offset.y) / self.zoom,
            );

            match event {
                // Middle mouse button for panning
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                    state.panning = true;
                    state.last_pan_pos = Some(canvas_relative);
                    return (canvas::event::Status::Captured, None);
                }
                canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                    state.panning = false;
                    state.last_pan_pos = None;
                    return (canvas::event::Status::Captured, None);
                }
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    // First, check if click is on a +/- expansion indicator for expandable nodes
                    let node_radius = 20.0;
                    let indicator_offset = node_radius + 52.0;  // Must match drawing code
                    let indicator_radius = 10.0;

                    for (node_id, node) in &self.nodes {
                        // Check if this is an expandable node
                        let is_expandable = matches!(
                            node.domain_node.injection(),
                            Injection::PolicyGroup | Injection::PolicyCategory
                        );

                        if is_expandable {
                            // Check if click is on the +/- indicator
                            let indicator_center_y = node.position.y + indicator_offset;
                            let indicator_distance = ((adjusted_position.x - node.position.x).powi(2)
                                + (adjusted_position.y - indicator_center_y).powi(2))
                            .sqrt();

                            if indicator_distance <= indicator_radius + 2.0 {  // Small tolerance
                                // Click on expansion indicator - expand/collapse
                                return (
                                    canvas::event::Status::Captured,
                                    Some(OrganizationIntent::ExpandIndicatorClicked(*node_id)),
                                );
                            }
                        }
                    }

                    // Check if click is on a node (not indicator)
                    for (node_id, node) in &self.nodes {
                        let distance = ((adjusted_position.x - node.position.x).powi(2)
                            + (adjusted_position.y - node.position.y).powi(2))
                        .sqrt();

                        if distance <= node_radius {
                            // Check if click is on border (outer ring) vs center
                            // Border: 12-20 pixels from center → start edge creation
                            // Center: 0-12 pixels from center → start node drag
                            if distance >= 12.0 {
                                // Border click - start edge creation
                                return (
                                    canvas::event::Status::Captured,
                                    Some(OrganizationIntent::EdgeCreationStarted(*node_id)),
                                );
                            } else {
                                // Center click - start node drag
                                // Track drag in canvas state
                                state.dragging_node = Some(*node_id);
                                state.drag_start_pos = Some(node.position);

                                // Calculate offset from node center to cursor
                                let offset = Vector::new(
                                    adjusted_position.x - node.position.x,
                                    adjusted_position.y - node.position.y,
                                );
                                return (
                                    canvas::event::Status::Captured,
                                    Some(OrganizationIntent::NodeDragStarted {
                                        node_id: *node_id,
                                        offset
                                    }),
                                );
                            }
                        }
                    }

                    // Click on empty canvas - emit CanvasClicked for node placement
                    return (
                        canvas::event::Status::Captured,
                        Some(OrganizationIntent::CanvasClicked(adjusted_position)),
                    );
                }
                canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    // Check if we're creating an edge
                    if self.edge_indicator.is_active() {
                        if let Some(from_node_id) = self.edge_indicator.from_node() {
                            // Check if we released over a different node
                            for (node_id, node) in &self.nodes {
                                if *node_id == from_node_id {
                                    continue; // Skip the source node
                                }

                                let distance = ((adjusted_position.x - node.position.x).powi(2)
                                    + (adjusted_position.y - node.position.y).powi(2))
                                .sqrt();

                                if distance <= 20.0 {
                                    // Released over a target node - create the edge
                                    // Use MemberOf as default edge type (user can change later)
                                    return (
                                        canvas::event::Status::Captured,
                                        Some(OrganizationIntent::AddEdge {
                                            from: from_node_id,
                                            to: *node_id,
                                            edge_type: EdgeType::MemberOf,
                                        }),
                                    );
                                }
                            }
                        }
                        // Released but not over a target node - cancel edge creation
                        return (
                            canvas::event::Status::Captured,
                            Some(OrganizationIntent::CancelEdgeCreation),
                        );
                    }

                    // End dragging if we were dragging
                    if let Some(node_id) = state.dragging_node {
                        // Check if node actually moved significantly (more than 5 pixels)
                        let moved_significantly = if let (Some(start_pos), Some(node)) = (state.drag_start_pos, self.nodes.get(&node_id)) {
                            let distance = ((node.position.x - start_pos.x).powi(2) + (node.position.y - start_pos.y).powi(2)).sqrt();
                            distance > 5.0
                        } else {
                            false
                        };

                        // Clear drag state
                        state.dragging_node = None;
                        state.drag_start_pos = None;

                        if moved_significantly {
                            // This was a drag operation
                            return (
                                canvas::event::Status::Captured,
                                Some(OrganizationIntent::NodeDragEnded),
                            );
                        } else {
                            // This was a click (no significant movement)
                            return (
                                canvas::event::Status::Captured,
                                Some(OrganizationIntent::NodeClicked(node_id)),
                            );
                        }
                    } else {
                        // Not dragging - check if we clicked on an edge
                        const EDGE_CLICK_THRESHOLD: f32 = 10.0;  // pixels
                        for (index, edge) in self.edges.iter().enumerate() {
                            if let (Some(from_node), Some(to_node)) = (self.nodes.get(&edge.from), self.nodes.get(&edge.to)) {
                                let distance = distance_to_line_segment(
                                    adjusted_position,
                                    from_node.position,
                                    to_node.position
                                );
                                if distance <= EDGE_CLICK_THRESHOLD {
                                    return (
                                        canvas::event::Status::Captured,
                                        Some(OrganizationIntent::EdgeSelected(index)),
                                    );
                                }
                            }
                        }
                    }
                }
                // Phase 4: Right-click to show context menu
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                    return (
                        canvas::event::Status::Captured,
                        Some(OrganizationIntent::RightClick(canvas_relative)),  // Use canvas-relative coords!
                    );
                }
                canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    // Handle panning with middle mouse button - use frame-to-frame delta for smoothness
                    if state.panning {
                        if let Some(last_pos) = state.last_pan_pos {
                            let delta = Vector::new(
                                canvas_relative.x - last_pos.x,
                                canvas_relative.y - last_pos.y,
                            );
                            // Update last position for next frame
                            state.last_pan_pos = Some(canvas_relative);
                            return (
                                canvas::event::Status::Captured,
                                Some(OrganizationIntent::Pan(delta)),
                            );
                        }
                    }
                    // Update edge indicator if active
                    if self.edge_indicator.is_active() {
                        return (
                            canvas::event::Status::Captured,
                            Some(OrganizationIntent::CursorMoved(adjusted_position)),
                        );
                    }
                    // Continue dragging if we're dragging a node
                    if state.dragging_node.is_some() {
                        return (
                            canvas::event::Status::Captured,
                            Some(OrganizationIntent::NodeDragged(canvas_relative)),  // Use canvas-relative, not window coords!
                        );
                    }
                }
                canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            // Mouse wheel - use discrete zoom steps
                            if y > 0.0 {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(OrganizationIntent::ZoomIn),
                                );
                            } else if y < 0.0 {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(OrganizationIntent::ZoomOut),
                                );
                            }
                        }
                        mouse::ScrollDelta::Pixels { x, y } => {
                            // Touchpad gestures:
                            // - Vertical scroll (y) = smooth zoom (pinch-to-zoom also maps to this)
                            // - Horizontal scroll (x) = horizontal pan (two-finger swipe)
                            //
                            // Pan controls:
                            // - Horizontal touchpad swipe = pan
                            // - Middle mouse button + drag = free pan (X and Y)
                            //
                            // Priority: horizontal pan first, then zoom
                            if x.abs() > y.abs() && x.abs() > 0.5 {
                                // Primarily horizontal - pan
                                return (
                                    canvas::event::Status::Captured,
                                    Some(OrganizationIntent::Pan(Vector::new(-x * 1.5, 0.0))),
                                );
                            }
                            if y.abs() > 0.5 {
                                // Primarily vertical - zoom
                                return (
                                    canvas::event::Status::Captured,
                                    Some(OrganizationIntent::ZoomBy(y)),
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
pub fn view_graph(graph: &OrganizationConcept) -> Element<'_, OrganizationIntent> {
    // Full Canvas-based graph visualization
    let canvas = Canvas::new(graph)
        .width(Length::Fill)
        .height(Length::Fill);

    let controls = row![
        button(text("+").size(16).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::ZoomIn)
            .style(CowboyCustomTheme::glass_button())
            .padding(6),
        button(text("-").size(16).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::ZoomOut)
            .style(CowboyCustomTheme::glass_button())
            .padding(6),
        button(text("🔄").size(14).font(crate::icons::EMOJI_FONT))
            .on_press(OrganizationIntent::ResetView)
            .style(CowboyCustomTheme::glass_button())
            .padding(6),
        button(text("Auto Layout").size(12).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::AutoLayout)
            .style(CowboyCustomTheme::glass_button())
            .padding(6),
        // Layout algorithm buttons
        button(text("Tutte").size(11).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::ApplyLayout(LayoutAlgorithm::Tutte))
            .style(CowboyCustomTheme::glass_button())
            .padding(4),
        button(text("F-R").size(11).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::ApplyLayout(LayoutAlgorithm::FruchtermanReingold))
            .style(CowboyCustomTheme::glass_button())
            .padding(4),
        button(text("Circle").size(11).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::ApplyLayout(LayoutAlgorithm::Circular))
            .style(CowboyCustomTheme::glass_button())
            .padding(4),
        button(text("Tree").size(11).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::ApplyLayout(LayoutAlgorithm::Hierarchical))
            .style(CowboyCustomTheme::glass_button())
            .padding(4),
        button(text("YubiKey").size(11).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::ApplyLayout(LayoutAlgorithm::YubiKeyGrouped))
            .style(CowboyCustomTheme::glass_button())
            .padding(4),
        button(text("NATS").size(11).font(crate::icons::FONT_BODY))
            .on_press(OrganizationIntent::ApplyLayout(LayoutAlgorithm::NatsHierarchical))
            .style(CowboyCustomTheme::glass_button())
            .padding(4),
    ]
    .spacing(8);

    let mut items = column![
        controls,
        canvas,
    ]
    .spacing(10)
    .height(Length::Fill);


    // Show selected node details using the DomainNode fold pattern
    // This replaces the 220-line match statement with the FoldDetailPanel catamorphism
    if let Some(selected_id) = graph.selected_node {
        if let Some(node) = graph.nodes.get(&selected_id) {
            // Get detail panel data from the DomainNode fold
            let detail_data = node.domain_node.detail_panel();

            // Build the column from DetailPanelData
            let mut details = column![
                text(detail_data.title).size(16),
            ];

            for (label, value) in detail_data.fields {
                details = details.push(text(format!("{}: {}", label, value)));
            }

            // Special handling for Person nodes: show role badges
            if node.injection() == super::domain_node::Injection::Person {
                if let Some(badges) = graph.role_badges.get(&selected_id) {
                    details = details.push(text("").size(6)); // Spacer
                    details = details.push(text("ASSIGNED ROLES:").size(14));

                    for badge in &badges.badges {
                        let level_indicator = "●".repeat(badge.level as usize).chars().take(5).collect::<String>();
                        let empty_indicator = "○".repeat(5_usize.saturating_sub(badge.level as usize));
                        details = details.push(
                            text(format!("  {} {} {}{}",
                                match badge.separation_class {
                                    crate::policy::SeparationClass::Operational => "🔵",
                                    crate::policy::SeparationClass::Administrative => "🟣",
                                    crate::policy::SeparationClass::Audit => "🟢",
                                    crate::policy::SeparationClass::Emergency => "🔴",
                                    crate::policy::SeparationClass::Financial => "🟡",
                                    crate::policy::SeparationClass::Personnel => "🟠",
                                },
                                badge.name,
                                level_indicator,
                                empty_indicator,
                            )).size(12)
                        );
                    }

                    if badges.has_more {
                        details = details.push(text("  (+more roles...)").size(11));
                    }
                }
            }

            items = items.push(details.spacing(5));
        }
    }

    container(items)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(CowboyCustomTheme::glass_container())
        .padding(10)
        .into()
}