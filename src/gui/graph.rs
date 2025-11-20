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

/// Graph visualization widget for organizational structure
pub struct OrganizationGraph {
    pub nodes: HashMap<Uuid, GraphNode>,
    pub edges: Vec<GraphEdge>,
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

    // NATS Infrastructure (Phase 1)
    /// NATS Operator - root of NATS trust hierarchy
    NatsOperator(NatsIdentityProjection),
    /// NATS Account - corresponds to organizational unit
    NatsAccount(NatsIdentityProjection),
    /// NATS User - corresponds to person
    NatsUser(NatsIdentityProjection),
    /// NATS Service Account - for automated services
    NatsServiceAccount(NatsIdentityProjection),

    // PKI Trust Chain (Phase 2)
    /// Root CA certificate - trust anchor
    RootCertificate {
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        key_usage: Vec<String>,
    },
    /// Intermediate CA certificate - signs leaf certificates
    IntermediateCertificate {
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        key_usage: Vec<String>,
    },
    /// Leaf certificate - end entity certificate
    LeafCertificate {
        cert_id: Uuid,
        subject: String,
        issuer: String,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
        key_usage: Vec<String>,
        san: Vec<String>, // Subject Alternative Names
    },

    // YubiKey Hardware (Phase 3)
    /// YubiKey hardware token
    YubiKey {
        device_id: Uuid,
        serial: String,
        version: String,
        provisioned_at: Option<chrono::DateTime<chrono::Utc>>,
        slots_used: Vec<String>,
    },
    /// PIV slot on a YubiKey
    PivSlot {
        slot_id: Uuid,
        slot_name: String, // e.g., "9A - Authentication"
        yubikey_serial: String,
        has_key: bool,
        certificate_subject: Option<String>,
    },
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
    EdgeSelected(usize),  // Index of selected edge
    EdgeDeleted(usize),   // Index of edge to delete
    EdgeTypeChanged { edge_index: usize, new_type: EdgeType },  // Change edge relationship type
    EdgeCreationStarted(Uuid),  // Start edge creation by dragging from node border
    ZoomIn,
    ZoomOut,
    ResetView,
    Pan(Vector),
    AutoLayout,
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
}

impl Default for OrganizationGraph {
    fn default() -> Self {
        Self::new()
    }
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

impl OrganizationGraph {
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
        use cim_domain::AggregateRoot;
        let node_id = *location.id().as_uuid();
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

    // ===== NATS Infrastructure Nodes (Phase 1) =====

    /// Add a NATS operator node to the graph
    pub fn add_nats_operator_node(&mut self, node_id: Uuid, nats_identity: NatsIdentityProjection, label: String) {
        let node = GraphNode {
            id: node_id,
            node_type: NodeType::NatsOperator(nats_identity),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(1.0, 0.2, 0.0), // Bright red (root of trust)
            label,
        };

        self.nodes.insert(node_id, node);
    }

    /// Add a NATS account node to the graph
    pub fn add_nats_account_node(&mut self, node_id: Uuid, nats_identity: NatsIdentityProjection, label: String) {
        let node = GraphNode {
            id: node_id,
            node_type: NodeType::NatsAccount(nats_identity),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(1.0, 0.5, 0.0), // Orange (intermediate trust)
            label,
        };

        self.nodes.insert(node_id, node);
    }

    /// Add a NATS user node to the graph
    pub fn add_nats_user_node(&mut self, node_id: Uuid, nats_identity: NatsIdentityProjection, label: String) {
        let node = GraphNode {
            id: node_id,
            node_type: NodeType::NatsUser(nats_identity),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(0.2, 0.8, 1.0), // Cyan (leaf user)
            label,
        };

        self.nodes.insert(node_id, node);
    }

    /// Add a NATS service account node to the graph
    pub fn add_nats_service_account_node(&mut self, node_id: Uuid, nats_identity: NatsIdentityProjection, label: String) {
        let node = GraphNode {
            id: node_id,
            node_type: NodeType::NatsServiceAccount(nats_identity),
            position: self.calculate_node_position(node_id),
            color: Color::from_rgb(0.8, 0.2, 0.8), // Magenta (service account)
            label,
        };

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
        let node = GraphNode {
            id: cert_id,
            node_type: NodeType::RootCertificate {
                cert_id,
                subject: subject.clone(),
                issuer,
                not_before,
                not_after,
                key_usage,
            },
            position: self.calculate_node_position(cert_id),
            color: Color::from_rgb(0.0, 0.6, 0.4), // Dark teal (root trust)
            label: format!("Root CA: {}", subject),
        };

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
        let node = GraphNode {
            id: cert_id,
            node_type: NodeType::IntermediateCertificate {
                cert_id,
                subject: subject.clone(),
                issuer,
                not_before,
                not_after,
                key_usage,
            },
            position: self.calculate_node_position(cert_id),
            color: Color::from_rgb(0.2, 0.8, 0.6), // Medium teal (intermediate trust)
            label: format!("Intermediate CA: {}", subject),
        };

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
        let node = GraphNode {
            id: cert_id,
            node_type: NodeType::LeafCertificate {
                cert_id,
                subject: subject.clone(),
                issuer,
                not_before,
                not_after,
                key_usage,
                san,
            },
            position: self.calculate_node_position(cert_id),
            color: Color::from_rgb(0.4, 1.0, 0.8), // Light teal (leaf certificate)
            label: format!("Certificate: {}", subject),
        };

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
        let node = GraphNode {
            id: device_id,
            node_type: NodeType::YubiKey {
                device_id,
                serial: serial.clone(),
                version: version.clone(),
                provisioned_at,
                slots_used: slots_used.clone(),
            },
            position: self.calculate_node_position(device_id),
            color: Color::from_rgb(0.8, 0.3, 0.8), // Magenta (hardware)
            label: format!("YubiKey {}", serial),
        };

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
        let node = GraphNode {
            id: slot_id,
            node_type: NodeType::PivSlot {
                slot_id,
                slot_name: slot_name.clone(),
                yubikey_serial,
                has_key,
                certificate_subject,
            },
            position: self.calculate_node_position(slot_id),
            color: Color::from_rgb(0.9, 0.5, 0.9), // Light magenta (slot)
            label: slot_name,
        };

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
            GraphEvent::NodeCreated { node_id, node_type, position, color, label, .. } => {
                let node = GraphNode {
                    id: *node_id,
                    node_type: node_type.clone(),
                    position: *position,
                    color: *color,
                    label: label.clone(),
                };
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
            GraphEvent::NodePropertiesChanged { node_id, new_node_type, new_label, .. } => {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.node_type = new_node_type.clone();
                    node.label = new_label.clone();
                }
            }
            GraphEvent::NodeMoved { node_id, new_position, .. } => {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.position = *new_position;
                }
            }
            GraphEvent::EdgeCreated { from, to, edge_type, color, .. } => {
                self.edges.push(GraphEdge {
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
                // NATS Infrastructure
                NodeType::NatsOperator(_) => "NatsOperator",
                NodeType::NatsAccount(_) => "NatsAccount",
                NodeType::NatsUser(_) => "NatsUser",
                NodeType::NatsServiceAccount(_) => "NatsServiceAccount",
                // PKI Trust Chain
                NodeType::RootCertificate { .. } => "RootCertificate",
                NodeType::IntermediateCertificate { .. } => "IntermediateCertificate",
                NodeType::LeafCertificate { .. } => "LeafCertificate",
                // YubiKey Hardware
                NodeType::YubiKey { .. } => "YubiKey",
                NodeType::PivSlot { .. } => "PivSlot",
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
            GraphMessage::NodeClicked(id) => {
                self.selected_node = Some(id);
                // Clear dragging state (click without significant movement)
                self.dragging_node = None;
                self.drag_offset = Vector::new(0.0, 0.0);
                self.drag_start_position = None;
            },
            GraphMessage::NodeDragStarted { node_id, offset } => {
                self.dragging_node = Some(node_id);
                self.drag_offset = offset;
                // Capture starting position for NodeMoved event
                if let Some(node) = self.nodes.get(&node_id) {
                    self.drag_start_position = Some(node.position);
                }
            }
            GraphMessage::NodeDragged(cursor_pos) => {
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
            GraphMessage::NodeDragEnded => {
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
            GraphMessage::EdgeClicked { from: _, to: _ } => {}
            GraphMessage::ZoomIn => self.zoom = (self.zoom * 1.2).min(10.0),  // Max zoom 10.0 (zoom in closer)
            GraphMessage::ZoomOut => self.zoom = (self.zoom / 1.2).max(0.1),  // Min zoom 0.1 (zoom out much further)
            GraphMessage::ResetView => {
                self.zoom = 1.0;  // Reset to 1:1 scale
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
                // Complete edge indicator if it's active
                if self.edge_indicator.is_active() {
                    self.edge_indicator.complete();
                }
            }
            // Phase 4: Right-click handled in main GUI (shows context menu)
            GraphMessage::RightClick(_) => {}
            // Phase 4: Update edge indicator position during edge creation
            GraphMessage::CursorMoved(position) => {
                self.edge_indicator.update_position(position);
            }
            // Phase 4: Cancel edge creation with Esc key
            GraphMessage::CancelEdgeCreation => {
                self.edge_indicator.cancel();
            }
            // Phase 4: Delete selected node with Delete key
            GraphMessage::DeleteSelected => {
                // Deletion now handled via events in GUI layer
                // The event application will handle node removal and edge cleanup
            }
            // Phase 4: Undo last action
            GraphMessage::Undo => {
                if let Some(compensating_event) = self.event_stack.undo() {
                    self.apply_event(&compensating_event);
                }
            }
            // Phase 4: Redo last undone action
            GraphMessage::Redo => {
                if let Some(compensating_event) = self.event_stack.redo() {
                    self.apply_event(&compensating_event);
                }
            }
            // Edge editing messages
            GraphMessage::EdgeSelected(index) => {
                self.selected_edge = Some(index);
                // Clear node selection when edge is selected
                self.selected_node = None;
            }
            GraphMessage::EdgeCreationStarted(from_node) => {
                // Start edge creation indicator from this node
                if let Some(node) = self.nodes.get(&from_node) {
                    self.edge_indicator.start(from_node, node.position);
                }
            }
            GraphMessage::EdgeDeleted(index) => {
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
            GraphMessage::EdgeTypeChanged { edge_index, new_type } => {
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

    /// Check if a node should be visible based on current filter settings
    pub fn should_show_node(&self, node: &GraphNode) -> bool {
        match &node.node_type {
            NodeType::Person { .. } => self.filter_show_people,
            NodeType::Organization(_) | NodeType::OrganizationalUnit(_) |
            NodeType::Location(_) | NodeType::Role(_) | NodeType::Policy(_) => {
                self.filter_show_orgs
            }
            NodeType::NatsOperator(_) | NodeType::NatsAccount(_) |
            NodeType::NatsUser(_) | NodeType::NatsServiceAccount(_) => {
                self.filter_show_nats
            }
            NodeType::RootCertificate { .. } | NodeType::IntermediateCertificate { .. } |
            NodeType::LeafCertificate { .. } => {
                self.filter_show_pki
            }
            NodeType::YubiKey { .. } | NodeType::PivSlot { .. } => {
                self.filter_show_yubikey
            }
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
            match &node.node_type {
                NodeType::Person { .. } => person_nodes.push(*id),
                NodeType::Organization(_) | NodeType::OrganizationalUnit(_) |
                NodeType::Location(_) | NodeType::Role(_) | NodeType::Policy(_) => {
                    org_nodes.push(*id);
                }
                NodeType::NatsOperator(_) | NodeType::NatsAccount(_) |
                NodeType::NatsUser(_) | NodeType::NatsServiceAccount(_) => {
                    nats_nodes.push(*id);
                }
                NodeType::RootCertificate { .. } | NodeType::IntermediateCertificate { .. } |
                NodeType::LeafCertificate { .. } => {
                    pki_nodes.push(*id);
                }
                NodeType::YubiKey { .. } | NodeType::PivSlot { .. } => {
                    yubikey_nodes.push(*id);
                }
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
}

/// Implementation of canvas::Program for graph rendering
/// Canvas state for tracking interaction during event processing
#[derive(Debug, Default, Clone)]
pub struct CanvasState {
    dragging_node: Option<Uuid>,
    drag_start_pos: Option<Point>,
    // Panning state
    panning: bool,
    pan_start_pos: Option<Point>,
    pan_start_offset: Vector,
}

impl canvas::Program<GraphMessage> for OrganizationGraph {
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

            // Draw node properties as multi-line text based on node type
            let (type_icon, type_font, primary_text, secondary_text) = match &node.node_type {
                NodeType::Organization(org) => (
                    crate::icons::ICON_BUSINESS,
                    crate::icons::MATERIAL_ICONS,
                    org.name.clone(),
                    org.display_name.clone(),
                ),
                NodeType::OrganizationalUnit(unit) => (
                    crate::icons::ICON_GROUP,
                    crate::icons::MATERIAL_ICONS,
                    unit.name.clone(),
                    format!("{:?}", unit.unit_type),
                ),
                NodeType::Person { person, role: _ } => {
                    (crate::icons::ICON_PERSON, crate::icons::MATERIAL_ICONS, person.name.clone(), person.email.clone())
                },
                NodeType::Location(loc) => (
                    crate::icons::ICON_LOCATION,
                    crate::icons::MATERIAL_ICONS,
                    loc.name.clone(),
                    format!("{:?}", loc.location_type),
                ),
                NodeType::Role(role) => (
                    crate::icons::ICON_SECURITY,
                    crate::icons::MATERIAL_ICONS,
                    role.name.clone(),
                    role.description.clone(),
                ),
                NodeType::Policy(policy) => (
                    crate::icons::ICON_VERIFIED,
                    crate::icons::MATERIAL_ICONS,
                    policy.name.clone(),
                    format!("{} claims", policy.claims.len()),
                ),
                // NATS Infrastructure
                NodeType::NatsOperator(identity) => (
                    crate::icons::ICON_CLOUD,
                    crate::icons::MATERIAL_ICONS,
                    "NATS Operator".to_string(),
                    identity.nkey.public_key.public_key()[..8].to_string(), // First 8 chars of NKey
                ),
                NodeType::NatsAccount(identity) => (
                    crate::icons::ICON_ACCOUNT_CIRCLE,
                    crate::icons::MATERIAL_ICONS,
                    "NATS Account".to_string(),
                    identity.nkey.public_key.public_key()[..8].to_string(),
                ),
                NodeType::NatsUser(identity) => (
                    crate::icons::ICON_PERSON,
                    crate::icons::MATERIAL_ICONS,
                    "NATS User".to_string(),
                    identity.nkey.public_key.public_key()[..8].to_string(),
                ),
                NodeType::NatsServiceAccount(identity) => (
                    crate::icons::ICON_SETTINGS,
                    crate::icons::MATERIAL_ICONS,
                    "Service Account".to_string(),
                    identity.nkey.public_key.public_key()[..8].to_string(),
                ),
                // PKI Trust Chain
                NodeType::RootCertificate { subject, not_after, .. } => (
                    crate::icons::ICON_VERIFIED,
                    crate::icons::MATERIAL_ICONS,
                    "Root CA".to_string(),
                    format!("{} (expires {})", subject, not_after.format("%Y-%m-%d")),
                ),
                NodeType::IntermediateCertificate { subject, not_after, .. } => (
                    crate::icons::ICON_VERIFIED,
                    crate::icons::MATERIAL_ICONS,
                    "Intermediate CA".to_string(),
                    format!("{} (expires {})", subject, not_after.format("%Y-%m-%d")),
                ),
                NodeType::LeafCertificate { subject, not_after, san, .. } => (
                    crate::icons::ICON_LOCK,
                    crate::icons::MATERIAL_ICONS,
                    format!("Certificate: {}", subject),
                    if !san.is_empty() {
                        format!("SAN: {} (expires {})", san[0], not_after.format("%Y-%m-%d"))
                    } else {
                        format!("expires {}", not_after.format("%Y-%m-%d"))
                    },
                ),
                // YubiKey Hardware
                NodeType::YubiKey { serial, version, slots_used, .. } => (
                    crate::icons::ICON_SECURITY,
                    crate::icons::MATERIAL_ICONS,
                    format!("YubiKey {}", serial),
                    format!("v{} ({} slots used)", version, slots_used.len()),
                ),
                NodeType::PivSlot { slot_name, has_key, certificate_subject, .. } => (
                    crate::icons::ICON_LOCK,
                    crate::icons::MATERIAL_ICONS,
                    slot_name.clone(),
                    if *has_key {
                        certificate_subject.clone().unwrap_or_else(|| "Key loaded".to_string())
                    } else {
                        "Empty slot".to_string()
                    },
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

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,  // Canvas widget bounds for coordinate conversion
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<GraphMessage>) {
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
                    state.pan_start_pos = Some(canvas_relative);
                    state.pan_start_offset = self.pan_offset;
                    return (canvas::event::Status::Captured, None);
                }
                canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                    state.panning = false;
                    state.pan_start_pos = None;
                    return (canvas::event::Status::Captured, None);
                }
                canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    // Check if click is on a node
                    for (node_id, node) in &self.nodes {
                        let distance = ((adjusted_position.x - node.position.x).powi(2)
                            + (adjusted_position.y - node.position.y).powi(2))
                        .sqrt();

                        if distance <= 20.0 {
                            // Check if click is on border (outer ring) vs center
                            // Border: 12-20 pixels from center → start edge creation
                            // Center: 0-12 pixels from center → start node drag
                            if distance >= 12.0 {
                                // Border click - start edge creation
                                return (
                                    canvas::event::Status::Captured,
                                    Some(GraphMessage::EdgeCreationStarted(*node_id)),
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
                                    Some(GraphMessage::NodeDragStarted {
                                        node_id: *node_id,
                                        offset
                                    }),
                                );
                            }
                        }
                    }
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
                                        Some(GraphMessage::AddEdge {
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
                            Some(GraphMessage::CancelEdgeCreation),
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
                                Some(GraphMessage::NodeDragEnded),
                            );
                        } else {
                            // This was a click (no significant movement)
                            return (
                                canvas::event::Status::Captured,
                                Some(GraphMessage::NodeClicked(node_id)),
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
                                        Some(GraphMessage::EdgeSelected(index)),
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
                        Some(GraphMessage::RightClick(canvas_relative)),  // Use canvas-relative coords!
                    );
                }
                canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    // Handle panning with middle mouse button
                    if state.panning {
                        if let Some(pan_start) = state.pan_start_pos {
                            let delta = Vector::new(
                                canvas_relative.x - pan_start.x,
                                canvas_relative.y - pan_start.y,
                            );
                            let new_offset = state.pan_start_offset + delta;
                            return (
                                canvas::event::Status::Captured,
                                Some(GraphMessage::Pan(new_offset - self.pan_offset)),
                            );
                        }
                    }
                    // Update edge indicator if active
                    if self.edge_indicator.is_active() {
                        return (
                            canvas::event::Status::Captured,
                            Some(GraphMessage::CursorMoved(adjusted_position)),
                        );
                    }
                    // Continue dragging if we're dragging a node
                    if state.dragging_node.is_some() {
                        return (
                            canvas::event::Status::Captured,
                            Some(GraphMessage::NodeDragged(canvas_relative)),  // Use canvas-relative, not window coords!
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
        .height(Length::Fill);

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
    .spacing(10)
    .height(Length::Fill);


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
                // NATS Infrastructure
                NodeType::NatsOperator(identity) => column![
                    text("Selected NATS Operator:").size(16),
                    text(format!("Public Key: {}", identity.nkey.public_key.public_key())),
                    text(format!("JWT Token: {}...", &identity.jwt.token()[..20])),
                    text(format!("Has Credential: {}", identity.credential.is_some())),
                ],
                NodeType::NatsAccount(identity) => column![
                    text("Selected NATS Account:").size(16),
                    text(format!("Public Key: {}", identity.nkey.public_key.public_key())),
                    text(format!("JWT Token: {}...", &identity.jwt.token()[..20])),
                    text(format!("Has Credential: {}", identity.credential.is_some())),
                ],
                NodeType::NatsUser(identity) => column![
                    text("Selected NATS User:").size(16),
                    text(format!("Public Key: {}", identity.nkey.public_key.public_key())),
                    text(format!("JWT Token: {}...", &identity.jwt.token()[..20])),
                    text(format!("Has Credential: {}", identity.credential.is_some())),
                ],
                NodeType::NatsServiceAccount(identity) => column![
                    text("Selected Service Account:").size(16),
                    text(format!("Public Key: {}", identity.nkey.public_key.public_key())),
                    text(format!("JWT Token: {}...", &identity.jwt.token()[..20])),
                    text(format!("Has Credential: {}", identity.credential.is_some())),
                ],
                // PKI Trust Chain
                NodeType::RootCertificate { subject, issuer, not_before, not_after, key_usage, .. } => column![
                    text("Selected Root CA Certificate:").size(16),
                    text(format!("Subject: {}", subject)),
                    text(format!("Issuer: {}", issuer)),
                    text(format!("Valid From: {}", not_before.format("%Y-%m-%d %H:%M:%S UTC"))),
                    text(format!("Valid Until: {}", not_after.format("%Y-%m-%d %H:%M:%S UTC"))),
                    text(format!("Key Usage: {}", key_usage.join(", "))),
                ],
                NodeType::IntermediateCertificate { subject, issuer, not_before, not_after, key_usage, .. } => column![
                    text("Selected Intermediate CA Certificate:").size(16),
                    text(format!("Subject: {}", subject)),
                    text(format!("Issuer: {}", issuer)),
                    text(format!("Valid From: {}", not_before.format("%Y-%m-%d %H:%M:%S UTC"))),
                    text(format!("Valid Until: {}", not_after.format("%Y-%m-%d %H:%M:%S UTC"))),
                    text(format!("Key Usage: {}", key_usage.join(", "))),
                ],
                NodeType::LeafCertificate { subject, issuer, not_before, not_after, key_usage, san, .. } => column![
                    text("Selected Leaf Certificate:").size(16),
                    text(format!("Subject: {}", subject)),
                    text(format!("Issuer: {}", issuer)),
                    text(format!("Valid From: {}", not_before.format("%Y-%m-%d %H:%M:%S UTC"))),
                    text(format!("Valid Until: {}", not_after.format("%Y-%m-%d %H:%M:%S UTC"))),
                    text(format!("Key Usage: {}", key_usage.join(", "))),
                    text(format!("Subject Alt Names: {}", if san.is_empty() { "none".to_string() } else { san.join(", ") })),
                ],
                // YubiKey Hardware
                NodeType::YubiKey { serial, version, provisioned_at, slots_used, .. } => column![
                    text("Selected YubiKey:").size(16),
                    text(format!("Serial: {}", serial)),
                    text(format!("Version: {}", version)),
                    text(format!("Provisioned: {}", provisioned_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_else(|| "Not provisioned".to_string()))),
                    text(format!("Slots Used: {}", slots_used.join(", "))),
                ],
                NodeType::PivSlot { slot_name, yubikey_serial, has_key, certificate_subject, .. } => column![
                    text("Selected PIV Slot:").size(16),
                    text(format!("Slot: {}", slot_name)),
                    text(format!("YubiKey: {}", yubikey_serial)),
                    text(format!("Status: {}", if *has_key { "Key loaded" } else { "Empty" })),
                    text(format!("Certificate: {}", certificate_subject.clone().unwrap_or_else(|| "None".to_string()))),
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