//! Property card for editing node properties
//!
//! Displays and allows editing of properties for selected graph nodes.

use iced::{
    widget::{button, checkbox, column, container, row, text, text_input, scrollable, Column, Row},
    Element, Length, Theme,
};
use uuid::Uuid;
use std::collections::HashSet;

use crate::gui::graph::EdgeType;
use crate::gui::domain_node::{DomainNode, DomainNodeData, Injection};
use crate::gui::view_model::ViewModel;
use crate::domain::{PolicyClaim, RoleType, LocationType};
use crate::icons::verified;

/// What is being edited
#[derive(Debug, Clone)]
pub enum EditTarget {
    Node { id: Uuid, domain_node: DomainNode },
    Edge { index: usize, from: Uuid, to: Uuid, edge_type: EdgeType },
}

/// Property card for editing node and edge properties
#[derive(Debug, Clone)]
pub struct PropertyCard {
    edit_target: Option<EditTarget>,
    dirty: bool,
    // Node edit state
    edit_name: String,
    edit_description: String,
    edit_email: String,
    edit_enabled: bool,
    edit_claims: HashSet<PolicyClaim>,
    edit_roles: HashSet<RoleType>,
    // Location-specific fields
    edit_location_type: LocationType,
    edit_address: String,  // Full address as a string
    edit_coordinates: String,  // "lat,long" format
    edit_virtual_location: String,  // URL or platform details
    // Edge edit state
    edit_edge_type: EdgeType,
}

/// Messages emitted by the property card
#[derive(Debug, Clone)]
pub enum PropertyCardMessage {
    // Node editing messages
    /// User changed the name field
    NameChanged(String),
    /// User changed the description field
    DescriptionChanged(String),
    /// User changed the email field
    EmailChanged(String),
    /// User toggled the enabled checkbox
    EnabledToggled(bool),
    /// User toggled a policy claim
    ClaimToggled(PolicyClaim, bool),
    /// User toggled a role
    RoleToggled(RoleType, bool),
    // Location-specific messages
    /// User changed location type
    LocationTypeChanged(LocationType),
    /// User changed address field
    AddressChanged(String),
    /// User changed coordinates field
    CoordinatesChanged(String),
    /// User changed virtual location field
    VirtualLocationChanged(String),
    // Key generation messages (for Person, Organization, and Unit nodes)
    /// User clicked generate root CA button (Organization or Person)
    GenerateRootCA,
    /// User clicked generate intermediate CA button (OrganizationalUnit)
    GenerateIntermediateCA,
    /// User clicked generate personal keys button (Person)
    GeneratePersonalKeys,
    /// User clicked provision YubiKey button (Person)
    ProvisionYubiKey,
    // Edge editing messages
    /// User changed edge type
    EdgeTypeChanged(EdgeType),
    /// User clicked delete edge button
    DeleteEdge,
    // Common messages
    /// User clicked save
    Save,
    /// User clicked cancel
    Cancel,
    /// User clicked close
    Close,
}

impl Default for PropertyCard {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyCard {
    /// Create a new property card
    pub fn new() -> Self {
        Self {
            edit_target: None,
            dirty: false,
            edit_name: String::new(),
            edit_description: String::new(),
            edit_email: String::new(),
            edit_enabled: true,
            edit_claims: HashSet::new(),
            edit_roles: HashSet::new(),
            edit_location_type: LocationType::Physical,
            edit_address: String::new(),
            edit_coordinates: String::new(),
            edit_virtual_location: String::new(),
            edit_edge_type: EdgeType::MemberOf,  // Default edge type
        }
    }

    /// Set the node to edit
    pub fn set_node(&mut self, node_id: Uuid, domain_node: DomainNode) {
        self.edit_target = Some(EditTarget::Node {
            id: node_id,
            domain_node: domain_node.clone(),
        });
        self.dirty = false;

        // Initialize edit fields from node data using DomainNode accessors
        match domain_node.data() {
            DomainNodeData::Organization(org) => {
                self.edit_name = org.name.clone();
                self.edit_description = org.description.clone().unwrap_or_default();
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::OrganizationUnit(unit) => {
                self.edit_name = unit.name.clone();
                self.edit_description = String::new();
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::Person { person, .. } => {
                self.edit_name = person.name.clone();
                self.edit_description = String::new();
                self.edit_email = person.email.clone();
                self.edit_enabled = person.active;
                self.edit_roles = person.roles.iter().map(|r| r.role_type.clone()).collect();
            }
            DomainNodeData::Location(loc) => {
                self.edit_name = loc.name.clone();
                self.edit_description = String::new();
                self.edit_email = String::new();
                self.edit_enabled = true;
                // Location-specific fields
                self.edit_location_type = loc.location_type.clone();
                self.edit_address = loc.address.as_ref()
                    .map(|addr| {
                        let mut parts = vec![addr.street1.clone()];
                        if let Some(street2) = &addr.street2 {
                            parts.push(street2.clone());
                        }
                        parts.push(addr.locality.clone());
                        parts.push(addr.region.clone());
                        parts.push(addr.country.clone());
                        parts.push(addr.postal_code.clone());
                        parts.join(", ")
                    })
                    .unwrap_or_default();
                self.edit_coordinates = loc.coordinates.as_ref()
                    .map(|coords| format!("{}, {}", coords.latitude, coords.longitude))
                    .unwrap_or_default();
                self.edit_virtual_location = loc.virtual_location.as_ref()
                    .map(|vl| {
                        if !vl.urls.is_empty() {
                            vl.urls[0].url.clone()
                        } else {
                            vl.primary_identifier.clone()
                        }
                    })
                    .unwrap_or_default();
            }
            DomainNodeData::Role(role) => {
                self.edit_name = role.name.clone();
                self.edit_description = role.description.clone();
                self.edit_email = String::new();
                self.edit_enabled = role.active;
            }
            DomainNodeData::Policy(policy) => {
                self.edit_name = policy.name.clone();
                self.edit_description = policy.description.clone();
                self.edit_email = String::new();
                self.edit_enabled = policy.enabled;
                self.edit_claims = policy.claims.iter().cloned().collect();
            }
            // NATS Infrastructure - read-only, no editing
            DomainNodeData::NatsOperator(identity) => {
                self.edit_name = "NATS Operator".to_string();
                self.edit_description = format!("NKey: {}", identity.nkey.public_key.public_key());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::NatsAccount(identity) => {
                self.edit_name = "NATS Account".to_string();
                self.edit_description = format!("NKey: {}", identity.nkey.public_key.public_key());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::NatsUser(identity) => {
                self.edit_name = "NATS User".to_string();
                self.edit_description = format!("NKey: {}", identity.nkey.public_key.public_key());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::NatsServiceAccount(identity) => {
                self.edit_name = "Service Account".to_string();
                self.edit_description = format!("NKey: {}", identity.nkey.public_key.public_key());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // NATS Infrastructure - Simple variants (visualization only)
            DomainNodeData::NatsOperatorSimple { name, organization_id } => {
                self.edit_name = name.clone();
                self.edit_description = format!("Org: {}", organization_id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string()));
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::NatsAccountSimple { name, unit_id, is_system } => {
                self.edit_name = name.clone();
                self.edit_description = format!("Unit: {}, System: {}", unit_id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string()), is_system);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::NatsUserSimple { name, person_id, account_name } => {
                self.edit_name = name.clone();
                self.edit_description = format!("Account: {}, Person: {}", account_name, person_id.map(|id| id.to_string()).unwrap_or_else(|| "N/A".to_string()));
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // PKI Trust Chain - read-only, no editing
            DomainNodeData::RootCertificate(cert) => {
                self.edit_name = format!("Root CA: {}", cert.subject);
                self.edit_description = format!("Issuer: {}", cert.issuer);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::IntermediateCertificate(cert) => {
                self.edit_name = format!("Intermediate CA: {}", cert.subject);
                self.edit_description = format!("Issuer: {}", cert.issuer);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::LeafCertificate(cert) => {
                self.edit_name = format!("Certificate: {}", cert.subject);
                self.edit_description = format!("Issuer: {}", cert.issuer);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // YubiKey Hardware - read-only, no editing
            DomainNodeData::YubiKey(yk) => {
                self.edit_name = format!("YubiKey {}", yk.serial);
                self.edit_description = format!("Version: {}", yk.version);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::PivSlot(slot) => {
                self.edit_name = slot.slot_name.clone();
                self.edit_description = format!("YubiKey {} - {}", slot.yubikey_serial, if slot.has_key { "In use" } else { "Empty" });
                self.edit_email = String::new();
                self.edit_enabled = slot.has_key;
            }
            DomainNodeData::YubiKeyStatus(status) => {
                self.edit_name = "YubiKey Status".to_string();
                self.edit_description = format!("{}/{} slots - {}",
                    status.slots_provisioned.len(),
                    status.slots_needed.len(),
                    status.yubikey_serial.clone().unwrap_or_else(|| "Not detected".to_string())
                );
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // Cryptographic Keys - read-only, no editing
            DomainNodeData::Key(key) => {
                self.edit_name = format!("Key: {:?}", key.purpose);
                self.edit_description = format!("Algorithm: {:?}, ID: {}", key.algorithm, key.id.as_uuid());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // Export and Manifest - read-only, no editing
            DomainNodeData::Manifest(manifest) => {
                self.edit_name = format!("Manifest: {}", manifest.name);
                self.edit_description = format!("ID: {}, Destination: {}",
                    manifest.id.as_uuid(),
                    manifest.destination.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "None".to_string())
                );
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // Policy Roles from policy-bootstrap.json - read-only
            DomainNodeData::PolicyRole(role) => {
                self.edit_name = role.name.clone();
                self.edit_description = format!("L{} {:?} | {} claims | {}", role.level, role.separation_class, role.claim_count, role.purpose);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // Policy Claims - read-only
            DomainNodeData::PolicyClaim(claim) => {
                self.edit_name = claim.name.clone();
                self.edit_description = format!("Category: {}", claim.category);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // Policy Categories - clickable to expand/collapse
            DomainNodeData::PolicyCategory(category) => {
                self.edit_name = category.name.clone();
                self.edit_description = format!("{} claims | {}", category.claim_count, if category.expanded { "Expanded" } else { "Collapsed" });
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // Separation Class Groups - clickable to expand/collapse
            DomainNodeData::PolicyGroup(group) => {
                self.edit_name = group.name.clone();
                self.edit_description = format!("{} roles | {}", group.role_count, if group.expanded { "Expanded" } else { "Collapsed" });
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // Aggregate Bounded Contexts - read-only state visualization
            DomainNodeData::AggregateOrganization { name, version, people_count, units_count } => {
                self.edit_name = name.clone();
                self.edit_description = format!("v{} | {} people, {} units", version, people_count, units_count);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::AggregatePkiChain { name, version, certificates_count, keys_count } => {
                self.edit_name = name.clone();
                self.edit_description = format!("v{} | {} certs, {} keys", version, certificates_count, keys_count);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::AggregateNatsSecurity { name, version, operators_count, accounts_count, users_count } => {
                self.edit_name = name.clone();
                self.edit_description = format!("v{} | {} ops, {} accts, {} users", version, operators_count, accounts_count, users_count);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            DomainNodeData::AggregateYubiKeyProvisioning { name, version, devices_count, slots_provisioned } => {
                self.edit_name = name.clone();
                self.edit_description = format!("v{} | {} devices, {} slots", version, devices_count, slots_provisioned);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
        }
    }

    /// Set the edge to edit
    pub fn set_edge(&mut self, index: usize, from: Uuid, to: Uuid, edge_type: EdgeType) {
        self.edit_target = Some(EditTarget::Edge {
            index,
            from,
            to,
            edge_type: edge_type.clone(),
        });
        self.dirty = false;
        self.edit_edge_type = edge_type;
    }

    /// Clear the selected node or edge
    pub fn clear(&mut self) {
        self.edit_target = None;
        self.dirty = false;
        self.edit_name.clear();
        self.edit_description.clear();
        self.edit_email.clear();
        self.edit_enabled = true;
        self.edit_claims.clear();
        self.edit_roles.clear();
        self.edit_edge_type = EdgeType::MemberOf;
    }

    /// Check if something is being edited
    pub fn is_editing(&self) -> bool {
        self.edit_target.is_some()
    }

    /// Check if editing a node (vs edge)
    pub fn is_editing_node(&self) -> bool {
        matches!(self.edit_target, Some(EditTarget::Node { .. }))
    }

    /// Check if editing an edge (vs node)
    pub fn is_editing_edge(&self) -> bool {
        matches!(self.edit_target, Some(EditTarget::Edge { .. }))
    }

    /// Check if there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get the node ID being edited (if editing a node)
    pub fn node_id(&self) -> Option<Uuid> {
        match &self.edit_target {
            Some(EditTarget::Node { id, .. }) => Some(*id),
            _ => None,
        }
    }

    /// Get the edge index being edited (if editing an edge)
    pub fn edge_index(&self) -> Option<usize> {
        match &self.edit_target {
            Some(EditTarget::Edge { index, .. }) => Some(*index),
            _ => None,
        }
    }

    /// Get the edited name (for nodes)
    pub fn name(&self) -> &str {
        &self.edit_name
    }

    /// Get the edited description (for nodes)
    pub fn description(&self) -> &str {
        &self.edit_description
    }

    /// Get the edited email (for nodes)
    pub fn email(&self) -> &str {
        &self.edit_email
    }

    /// Get the edited enabled state (for nodes)
    pub fn enabled(&self) -> bool {
        self.edit_enabled
    }

    /// Get the edited claims (for Policy nodes)
    pub fn claims(&self) -> Vec<PolicyClaim> {
        self.edit_claims.iter().cloned().collect()
    }

    /// Get the edited roles (for Person nodes)
    pub fn roles(&self) -> Vec<RoleType> {
        self.edit_roles.iter().cloned().collect()
    }

    /// Get the edited edge type (for edges)
    pub fn edge_type(&self) -> EdgeType {
        self.edit_edge_type.clone()
    }

    /// Handle messages from the property card
    pub fn update(&mut self, message: PropertyCardMessage) {
        match message {
            PropertyCardMessage::NameChanged(name) => {
                self.edit_name = name;
                self.dirty = true;
            }
            PropertyCardMessage::DescriptionChanged(desc) => {
                self.edit_description = desc;
                self.dirty = true;
            }
            PropertyCardMessage::EmailChanged(email) => {
                self.edit_email = email;
                self.dirty = true;
            }
            PropertyCardMessage::EnabledToggled(enabled) => {
                self.edit_enabled = enabled;
                self.dirty = true;
            }
            PropertyCardMessage::ClaimToggled(claim, checked) => {
                if checked {
                    self.edit_claims.insert(claim);
                } else {
                    self.edit_claims.remove(&claim);
                }
                self.dirty = true;
            }
            PropertyCardMessage::RoleToggled(role, checked) => {
                if checked {
                    self.edit_roles.insert(role);
                } else {
                    self.edit_roles.remove(&role);
                }
                self.dirty = true;
            }
            PropertyCardMessage::LocationTypeChanged(location_type) => {
                self.edit_location_type = location_type;
                self.dirty = true;
            }
            PropertyCardMessage::AddressChanged(address) => {
                self.edit_address = address;
                self.dirty = true;
            }
            PropertyCardMessage::CoordinatesChanged(coords) => {
                self.edit_coordinates = coords;
                self.dirty = true;
            }
            PropertyCardMessage::VirtualLocationChanged(vl) => {
                self.edit_virtual_location = vl;
                self.dirty = true;
            }
            PropertyCardMessage::EdgeTypeChanged(edge_type) => {
                self.edit_edge_type = edge_type;
                self.dirty = true;
            }
            PropertyCardMessage::GenerateRootCA => {
                // Handled by parent
            }
            PropertyCardMessage::GenerateIntermediateCA => {
                // Handled by parent
            }
            PropertyCardMessage::GeneratePersonalKeys => {
                // Handled by parent
            }
            PropertyCardMessage::ProvisionYubiKey => {
                // Handled by parent
            }
            PropertyCardMessage::DeleteEdge => {
                // Handled by parent
            }
            PropertyCardMessage::Save => {
                // Handled by parent
            }
            PropertyCardMessage::Cancel => {
                // Handled by parent
            }
            PropertyCardMessage::Close => {
                // Handled by parent
            }
        }
    }

    /// Render the property card
    pub fn view(&self, vm: &ViewModel) -> Element<'_, PropertyCardMessage> {
        if !self.is_editing() {
            return container(text("Select a node or edge to edit").size(vm.text_normal))
                .padding(vm.padding_xl)
                .into();
        }

        match &self.edit_target {
            Some(EditTarget::Node { domain_node, .. }) => self.view_node(domain_node, vm),
            Some(EditTarget::Edge { edge_type, .. }) => self.view_edge(edge_type, vm),
            None => container(text("Select a node or edge to edit").size(vm.text_normal))
                .padding(vm.padding_xl)
                .into(),
        }
    }

    /// Render property card for editing a node
    fn view_node<'a>(&self, domain_node: &'a DomainNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        // For read-only infrastructure types, show detailed info panel instead of edit fields
        let injection = domain_node.injection();
        if injection.is_nats() {
            return self.view_nats_details(domain_node, vm);
        }
        if injection.is_certificate() {
            return self.view_certificate_details(domain_node, vm);
        }
        if matches!(injection, Injection::YubiKey | Injection::PivSlot) {
            return self.view_yubikey_details(domain_node, vm);
        }
        if matches!(injection, Injection::PolicyRole | Injection::PolicyCategory | Injection::PolicyGroup) {
            return self.view_policy_details(domain_node, vm);
        }

        // Get the display label using injection's display_name()
        let node_type_label = injection.display_name();

        let header: Row<'_, PropertyCardMessage> = row![
            text(node_type_label).size(vm.text_large),
            button(verified::icon("close", vm.text_medium))
                .on_press(PropertyCardMessage::Close)
                .style(|theme: &Theme, _status| {
                    button::Style {
                        background: None,
                        text_color: theme.palette().danger,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }),
        ]
        .spacing(vm.spacing_md)
        .align_y(iced::Alignment::Center);

        let mut fields: Column<'_, PropertyCardMessage> = column![]
            .spacing(vm.spacing_md)
            .padding(vm.padding_md);

        // Name field (all types have this)
        fields = fields.push(
            column![
                text("Name:").size(vm.text_small),
                text_input("Enter name", &self.edit_name)
                    .on_input(PropertyCardMessage::NameChanged)
                    .width(Length::Fill),
            ]
            .spacing(vm.spacing_xs)
        );

        // Location-specific fields
        if injection == Injection::Location {
            // Location type selector (Physical, Virtual, Logical, Hybrid)
            fields = fields.push(
                column![
                    text("Location Type:").size(12),
                    row![
                        button(text("Physical").size(10))
                            .on_press(PropertyCardMessage::LocationTypeChanged(LocationType::Physical))
                            .style(if matches!(self.edit_location_type, LocationType::Physical) {
                                |theme: &Theme, _status| {
                                    let palette = theme.extended_palette();
                                    button::Style {
                                        background: Some(iced::Background::Color(palette.primary.strong.color)),
                                        text_color: palette.primary.strong.text,
                                        border: iced::Border::default(),
                                        shadow: iced::Shadow::default(),
                                    }
                                }
                            } else {
                                button::secondary
                            }),
                        button(text("Virtual").size(10))
                            .on_press(PropertyCardMessage::LocationTypeChanged(LocationType::Virtual))
                            .style(if matches!(self.edit_location_type, LocationType::Virtual) {
                                |theme: &Theme, _status| {
                                    let palette = theme.extended_palette();
                                    button::Style {
                                        background: Some(iced::Background::Color(palette.primary.strong.color)),
                                        text_color: palette.primary.strong.text,
                                        border: iced::Border::default(),
                                        shadow: iced::Shadow::default(),
                                    }
                                }
                            } else {
                                button::secondary
                            }),
                        button(text("Logical").size(10))
                            .on_press(PropertyCardMessage::LocationTypeChanged(LocationType::Logical))
                            .style(if matches!(self.edit_location_type, LocationType::Logical) {
                                |theme: &Theme, _status| {
                                    let palette = theme.extended_palette();
                                    button::Style {
                                        background: Some(iced::Background::Color(palette.primary.strong.color)),
                                        text_color: palette.primary.strong.text,
                                        border: iced::Border::default(),
                                        shadow: iced::Shadow::default(),
                                    }
                                }
                            } else {
                                button::secondary
                            }),
                        button(text("Hybrid").size(10))
                            .on_press(PropertyCardMessage::LocationTypeChanged(LocationType::Hybrid))
                            .style(if matches!(self.edit_location_type, LocationType::Hybrid) {
                                |theme: &Theme, _status| {
                                    let palette = theme.extended_palette();
                                    button::Style {
                                        background: Some(iced::Background::Color(palette.primary.strong.color)),
                                        text_color: palette.primary.strong.text,
                                        border: iced::Border::default(),
                                        shadow: iced::Shadow::default(),
                                    }
                                }
                            } else {
                                button::secondary
                            }),
                    ]
                    .spacing(4)
                ]
                .spacing(4)
            );

            // Address field (for physical/hybrid locations)
            if matches!(self.edit_location_type, LocationType::Physical | LocationType::Hybrid) {
                fields = fields.push(
                    column![
                        text("Address:").size(12),
                        text_input("Enter address", &self.edit_address)
                            .on_input(PropertyCardMessage::AddressChanged)
                            .width(Length::Fill),
                    ]
                    .spacing(4)
                );

                // Coordinates field
                fields = fields.push(
                    column![
                        text("Coordinates (lat, long):").size(12),
                        text_input("e.g., 40.7128, -74.0060", &self.edit_coordinates)
                            .on_input(PropertyCardMessage::CoordinatesChanged)
                            .width(Length::Fill),
                    ]
                    .spacing(4)
                );
            }

            // Virtual location field (for virtual/hybrid locations)
            if matches!(self.edit_location_type, LocationType::Virtual | LocationType::Hybrid) {
                fields = fields.push(
                    column![
                        text("Virtual Location (URL/Platform):").size(12),
                        text_input("Enter URL or platform details", &self.edit_virtual_location)
                            .on_input(PropertyCardMessage::VirtualLocationChanged)
                            .width(Length::Fill),
                    ]
                    .spacing(4)
                );
            }
        }

        // Description field (Organization, Role, Policy)
        if matches!(
            injection,
            Injection::Organization | Injection::Role | Injection::Policy
        ) {
            fields = fields.push(
                column![
                    text("Description:").size(12),
                    text_input("Enter description", &self.edit_description)
                        .on_input(PropertyCardMessage::DescriptionChanged)
                        .width(Length::Fill),
                ]
                .spacing(4)
            );
        }

        // Email field (Person only)
        if injection == Injection::Person {
            fields = fields.push(
                column![
                    text("Email:").size(12),
                    text_input("Enter email", &self.edit_email)
                        .on_input(PropertyCardMessage::EmailChanged)
                        .width(Length::Fill),
                ]
                .spacing(4)
            );
        }

        // Roles checkboxes (Person only)
        if injection == Injection::Person {
            fields = fields.push(
                text("Roles:")
                    .size(12)
            );

            // List of all available roles
            let all_roles = vec![
                RoleType::Executive,
                RoleType::Administrator,
                RoleType::Developer,
                RoleType::Operator,
                RoleType::Auditor,
                RoleType::Service,
            ];

            let mut roles_column = Column::new().spacing(2);
            for role in all_roles {
                let is_checked = self.edit_roles.contains(&role);
                let role_name = format!("{:?}", role);
                roles_column = roles_column.push(
                    checkbox(&role_name, is_checked)
                        .on_toggle(move |checked| PropertyCardMessage::RoleToggled(role.clone(), checked))
                        .size(11)
                );
            }

            fields = fields.push(
                scrollable(roles_column)
                    .height(Length::Fixed(150.0))
            );

            // Key Operations section for Person nodes
            fields = fields.push(
                column![
                    text("Key Operations:").size(12),
                    button(text("Generate Root CA").size(11))
                        .on_press(PropertyCardMessage::GenerateRootCA)
                        .width(Length::Fill)
                        .style(|theme: &Theme, _status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(iced::Background::Color(palette.primary.strong.color)),
                                text_color: palette.primary.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                    button(text("Generate Personal Keys").size(11))
                        .on_press(PropertyCardMessage::GeneratePersonalKeys)
                        .width(Length::Fill)
                        .style(|theme: &Theme, _status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(iced::Background::Color(palette.success.strong.color)),
                                text_color: palette.success.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                    button(text("Provision YubiKey").size(11))
                        .on_press(PropertyCardMessage::ProvisionYubiKey)
                        .width(Length::Fill)
                        .style(|theme: &Theme, _status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(iced::Background::Color(palette.secondary.strong.color)),
                                text_color: palette.secondary.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ]
                .spacing(4)
            );
        }

        // Key Operations section for Organization nodes (Root CA only)
        if injection == Injection::Organization {
            fields = fields.push(
                column![
                    text("Key Operations:").size(12),
                    button(text("Generate Root CA").size(11))
                        .on_press(PropertyCardMessage::GenerateRootCA)
                        .width(Length::Fill)
                        .style(|theme: &Theme, _status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(iced::Background::Color(palette.primary.strong.color)),
                                text_color: palette.primary.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ]
                .spacing(4)
            );
        }

        // Key Operations section for OrganizationalUnit nodes (Intermediate CA only)
        if injection == Injection::OrganizationUnit {
            fields = fields.push(
                column![
                    text("Key Operations:").size(12),
                    button(text("Generate Intermediate CA").size(11))
                        .on_press(PropertyCardMessage::GenerateIntermediateCA)
                        .width(Length::Fill)
                        .style(|theme: &Theme, _status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(iced::Background::Color(palette.success.strong.color)),
                                text_color: palette.success.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ]
                .spacing(4)
            );
        }

        // Enabled checkbox (Person, Role, Policy)
        if matches!(
            injection,
            Injection::Person | Injection::Role | Injection::Policy
        ) {
            let label = match injection {
                Injection::Person => "Active",
                Injection::Role => "Active",
                Injection::Policy => "Enabled",
                _ => "Enabled",
            };

            fields = fields.push(
                row![
                    checkbox(label, self.edit_enabled)
                        .on_toggle(PropertyCardMessage::EnabledToggled),
                ]
                .spacing(8)
            );
        }

        // Claims checkboxes (Policy only)
        if injection == Injection::Policy {
            fields = fields.push(
                text("Claims (Permissions):")
                    .size(12)
            );

            // List of all available claims
            let all_claims = vec![
                PolicyClaim::CanGenerateKeys,
                PolicyClaim::CanSignCode,
                PolicyClaim::CanSignCertificates,
                PolicyClaim::CanRevokeKeys,
                PolicyClaim::CanDelegateKeys,
                PolicyClaim::CanExportKeys,
                PolicyClaim::CanBackupKeys,
                PolicyClaim::CanRotateKeys,
                PolicyClaim::CanAccessProduction,
                PolicyClaim::CanAccessStaging,
                PolicyClaim::CanAccessDevelopment,
                PolicyClaim::CanModifyInfrastructure,
                PolicyClaim::CanDeployServices,
            ];

            let mut claims_column = Column::new().spacing(2);
            for claim in all_claims {
                let is_checked = self.edit_claims.contains(&claim);
                let claim_name = format!("{:?}", claim);
                claims_column = claims_column.push(
                    checkbox(&claim_name, is_checked)
                        .on_toggle(move |checked| PropertyCardMessage::ClaimToggled(claim.clone(), checked))
                        .size(11)
                );
            }

            fields = fields.push(
                scrollable(claims_column)
                    .height(Length::Fixed(200.0))
            );
        }

        // Dirty indicator
        if self.dirty {
            fields = fields.push(
                text("* Unsaved changes")
                    .size(11)
                    .style(|theme: &Theme| {
                        text::Style {
                            color: Some(theme.palette().danger),
                        }
                    })
            );
        }

        // Action buttons
        let buttons: Row<'_, PropertyCardMessage> = row![
            button(text("Save").size(12))
                .on_press(PropertyCardMessage::Save)
                .style(|theme: &Theme, _status| {
                    button::Style {
                        background: Some(iced::Background::Color(theme.palette().success)),
                        text_color: iced::Color::WHITE,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }),
            button(text("Cancel").size(12))
                .on_press(PropertyCardMessage::Cancel)
                .style(|theme: &Theme, _status| {
                    button::Style {
                        background: Some(iced::Background::Color(theme.palette().danger)),
                        text_color: iced::Color::WHITE,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }),
        ]
        .spacing(10);

        use crate::gui::cowboy_theme::CowboyAppTheme as CowboyCustomTheme;

        let content: Column<'_, PropertyCardMessage> = column![
            header,
            fields,
            buttons,
        ]
        .spacing(12)
        .padding(16);

        container(content)
            .width(Length::Fixed(300.0))
            .style(CowboyCustomTheme::pastel_teal_card())
            .into()
    }

    /// Render property card for editing an edge
    fn view_edge(&self, _edge_type: &EdgeType, vm: &ViewModel) -> Element<'_, PropertyCardMessage> {
        let header: Row<'_, PropertyCardMessage> = row![
            text("Edge Relationship").size(vm.text_large),
            button(verified::icon("close", vm.text_medium))
                .on_press(PropertyCardMessage::Close)
                .style(|theme: &Theme, _status| {
                    button::Style {
                        background: None,
                        text_color: theme.palette().danger,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }),
        ]
        .spacing(vm.spacing_md)
        .align_y(iced::Alignment::Center);

        let mut fields: Column<'_, PropertyCardMessage> = column![]
            .spacing(vm.spacing_sm)
            .padding(vm.padding_sm);

        // Edge type label
        fields = fields.push(
            text("Relationship Type:").size(vm.text_small)
        );

        // Edge type picker (list of buttons for each type)
        let edge_types = vec![
            ("Parent-Child", EdgeType::ParentChild),
            ("Manages Unit", EdgeType::ManagesUnit),
            ("Member Of", EdgeType::MemberOf),
            ("Owns Key", EdgeType::OwnsKey),
            ("Stored At", EdgeType::StoredAt),
            ("Has Role", EdgeType::HasRole { valid_from: chrono::Utc::now(), valid_until: None }),
            ("Hierarchy", EdgeType::Hierarchy),
            ("Trust", EdgeType::Trust),
        ];

        for (label, et) in edge_types {
            let is_selected = matches!(
                (&self.edit_edge_type, &et),
                (EdgeType::ParentChild, EdgeType::ParentChild) |
                (EdgeType::ManagesUnit, EdgeType::ManagesUnit) |
                (EdgeType::MemberOf, EdgeType::MemberOf) |
                (EdgeType::OwnsKey, EdgeType::OwnsKey) |
                (EdgeType::StoredAt, EdgeType::StoredAt) |
                (EdgeType::HasRole { .. }, EdgeType::HasRole { .. }) |
                (EdgeType::Hierarchy, EdgeType::Hierarchy) |
                (EdgeType::Trust, EdgeType::Trust)
            );

            fields = fields.push(
                button(text(label).size(vm.text_small))
                    .on_press(PropertyCardMessage::EdgeTypeChanged(et))
                    .width(Length::Fill)
                    .style(move |theme: &Theme, _status| {
                        if is_selected {
                            button::Style {
                                background: Some(iced::Background::Color(theme.palette().primary)),
                                text_color: iced::Color::WHITE,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        } else {
                            button::Style::default()
                        }
                    })
            );
        }

        // Dirty indicator
        if self.dirty {
            fields = fields.push(
                text("* Unsaved changes")
                    .size(vm.text_tiny)
                    .style(|theme: &Theme| {
                        text::Style {
                            color: Some(theme.palette().danger),
                        }
                    })
            );
        }

        // Action buttons
        let buttons: Row<'_, PropertyCardMessage> = row![
            button(text("Save").size(vm.text_small))
                .on_press(PropertyCardMessage::Save)
                .style(|theme: &Theme, _status| {
                    button::Style {
                        background: Some(iced::Background::Color(theme.palette().success)),
                        text_color: iced::Color::WHITE,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }),
            button(text("Delete").size(vm.text_small))
                .on_press(PropertyCardMessage::DeleteEdge)
                .style(|theme: &Theme, _status| {
                    button::Style {
                        background: Some(iced::Background::Color(theme.palette().danger)),
                        text_color: iced::Color::WHITE,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }),
            button(text("Cancel").size(vm.text_small))
                .on_press(PropertyCardMessage::Cancel)
                .style(|theme: &Theme, _status| {
                    button::Style {
                        background: None,
                        text_color: theme.palette().text,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }),
        ]
        .spacing(vm.spacing_md);

        use crate::gui::cowboy_theme::CowboyAppTheme as CowboyCustomTheme;

        let content: Column<'_, PropertyCardMessage> = column![
            header,
            fields,
            buttons,
        ]
        .spacing(vm.spacing_md)
        .padding(vm.padding_lg);

        container(content)
            .width(Length::Fixed(300.0))
            .style(CowboyCustomTheme::pastel_teal_card())
            .into()
    }

    /// Render detailed NATS infrastructure information (read-only)
    fn view_nats_details<'a>(&self, domain_node: &'a DomainNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        let (title, details) = match domain_node.data() {
            DomainNodeData::NatsOperator(identity) => (
                "NATS Operator",
                column![
                    self.detail_row("Type:", "Root Authority", vm),
                    self.detail_row("Public Key:", &identity.nkey.public_key.public_key()[..32], vm),
                    self.detail_row("Key Type:", &format!("{:?}", identity.nkey.key_type), vm),
                    text("JWT Token:").size(vm.text_small).color(vm.colors.text_secondary),
                    scrollable(
                        text(identity.jwt.token())
                            .size(vm.text_tiny)
                            .font(iced::Font::MONOSPACE)
                    ).height(Length::Fixed(100.0)),
                    self.detail_row("Has Credential:", if identity.credential.is_some() { "Yes" } else { "No" }, vm),
                ].spacing(vm.spacing_sm)
            ),
            DomainNodeData::NatsAccount(identity) => (
                "NATS Account",
                column![
                    self.detail_row("Type:", "Account (Organizational Unit)", vm),
                    self.detail_row("Public Key:", &identity.nkey.public_key.public_key()[..32], vm),
                    self.detail_row("Key Type:", &format!("{:?}", identity.nkey.key_type), vm),
                    text("JWT Token:").size(vm.text_small).color(vm.colors.text_secondary),
                    scrollable(
                        text(identity.jwt.token())
                            .size(vm.text_tiny)
                            .font(iced::Font::MONOSPACE)
                    ).height(Length::Fixed(100.0)),
                    self.detail_row("Has Credential:", if identity.credential.is_some() { "Yes" } else { "No" }, vm),
                ].spacing(vm.spacing_sm)
            ),
            DomainNodeData::NatsUser(identity) => (
                "NATS User",
                column![
                    self.detail_row("Type:", "User (Person)", vm),
                    self.detail_row("Public Key:", &identity.nkey.public_key.public_key()[..32], vm),
                    self.detail_row("Key Type:", &format!("{:?}", identity.nkey.key_type), vm),
                    text("JWT Token:").size(vm.text_small).color(vm.colors.text_secondary),
                    scrollable(
                        text(identity.jwt.token())
                            .size(vm.text_tiny)
                            .font(iced::Font::MONOSPACE)
                    ).height(Length::Fixed(100.0)),
                    self.detail_row("Has Credential:", if identity.credential.is_some() { "Yes" } else { "No" }, vm),
                ].spacing(vm.spacing_sm)
            ),
            DomainNodeData::NatsServiceAccount(identity) => (
                "NATS Service Account",
                column![
                    self.detail_row("Type:", "Service (Automation)", vm),
                    self.detail_row("Public Key:", &identity.nkey.public_key.public_key()[..32], vm),
                    self.detail_row("Key Type:", &format!("{:?}", identity.nkey.key_type), vm),
                    text("JWT Token:").size(vm.text_small).color(vm.colors.text_secondary),
                    scrollable(
                        text(identity.jwt.token())
                            .size(vm.text_tiny)
                            .font(iced::Font::MONOSPACE)
                    ).height(Length::Fixed(100.0)),
                    self.detail_row("Has Credential:", if identity.credential.is_some() { "Yes" } else { "No" }, vm),
                ].spacing(vm.spacing_sm)
            ),
            _ => ("Unknown", column![].spacing(vm.spacing_sm)),
        };

        container(
            column![
                row![
                    text(title).size(vm.text_large),
                    button(verified::icon("close", vm.text_medium))
                        .on_press(PropertyCardMessage::Close)
                        .style(|theme: &Theme, _status| {
                            button::Style {
                                background: None,
                                text_color: theme.palette().danger,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ].spacing(vm.spacing_md).align_y(iced::Alignment::Center),
                scrollable(details).height(Length::Fill),
            ].spacing(vm.spacing_md).padding(vm.padding_md)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render detailed PKI certificate information (read-only)
    fn view_certificate_details<'a>(&self, domain_node: &'a DomainNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        let text_tiny = vm.text_tiny;
        let spacing_xs = vm.spacing_xs;
        let (title, details) = match domain_node.data() {
            DomainNodeData::RootCertificate(cert) => (
                "Root CA Certificate",
                column![
                    self.detail_row("Certificate Type:", "Root Certificate Authority", vm),
                    self.detail_row("Subject:", &cert.subject, vm),
                    self.detail_row("Issuer:", &cert.issuer, vm),
                    self.detail_row("Valid From:", &cert.not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string(), vm),
                    self.detail_row("Valid Until:", &cert.not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string(), vm),
                    self.detail_row("Validity:", &format!("{} days", (cert.not_after.signed_duration_since(cert.not_before).num_days())), vm),
                    text("Key Usage:").size(vm.text_small).color(vm.colors.text_secondary),
                    column(cert.key_usage.iter().map(|usage| {
                        text(format!("  • {}", usage)).size(text_tiny).into()
                    }).collect::<Vec<_>>()).spacing(spacing_xs),
                    self.detail_row("Trust Level:", "Root (Highest)", vm),
                    self.detail_row("Path Length:", "Unlimited", vm),
                ].spacing(vm.spacing_sm)
            ),
            DomainNodeData::IntermediateCertificate(cert) => (
                "Intermediate CA Certificate",
                column![
                    self.detail_row("Certificate Type:", "Intermediate Certificate Authority", vm),
                    self.detail_row("Subject:", &cert.subject, vm),
                    self.detail_row("Issuer:", &cert.issuer, vm),
                    self.detail_row("Valid From:", &cert.not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string(), vm),
                    self.detail_row("Valid Until:", &cert.not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string(), vm),
                    self.detail_row("Validity:", &format!("{} days", (cert.not_after.signed_duration_since(cert.not_before).num_days())), vm),
                    text("Key Usage:").size(vm.text_small).color(vm.colors.text_secondary),
                    column(cert.key_usage.iter().map(|usage| {
                        text(format!("  • {}", usage)).size(text_tiny).into()
                    }).collect::<Vec<_>>()).spacing(spacing_xs),
                    self.detail_row("Trust Level:", "Intermediate", vm),
                    self.detail_row("Can Sign:", "Leaf Certificates", vm),
                ].spacing(vm.spacing_sm)
            ),
            DomainNodeData::LeafCertificate(cert) => (
                "Leaf Certificate",
                column![
                    self.detail_row("Certificate Type:", "End Entity Certificate", vm),
                    self.detail_row("Subject:", &cert.subject, vm),
                    self.detail_row("Issuer:", &cert.issuer, vm),
                    self.detail_row("Valid From:", &cert.not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string(), vm),
                    self.detail_row("Valid Until:", &cert.not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string(), vm),
                    self.detail_row("Validity:", &format!("{} days", (cert.not_after.signed_duration_since(cert.not_before).num_days())), vm),
                    text("Key Usage:").size(vm.text_small).color(vm.colors.text_secondary),
                    column(cert.key_usage.iter().map(|usage| {
                        text(format!("  • {}", usage)).size(text_tiny).into()
                    }).collect::<Vec<_>>()).spacing(spacing_xs),
                    if !cert.san.is_empty() {
                        column![
                            text("Subject Alternative Names:").size(vm.text_small).color(vm.colors.text_secondary),
                            column(cert.san.iter().map(|name| {
                                text(format!("  • {}", name)).size(text_tiny).into()
                            }).collect::<Vec<_>>()).spacing(spacing_xs),
                        ].spacing(vm.spacing_xs)
                    } else {
                        column![].into()
                    },
                    self.detail_row("Trust Level:", "Leaf (End Entity)", vm),
                ].spacing(vm.spacing_sm)
            ),
            _ => ("Unknown", column![].spacing(vm.spacing_sm)),
        };

        container(
            column![
                row![
                    text(title).size(vm.text_large),
                    button(verified::icon("close", vm.text_medium))
                        .on_press(PropertyCardMessage::Close)
                        .style(|theme: &Theme, _status| {
                            button::Style {
                                background: None,
                                text_color: theme.palette().danger,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ].spacing(vm.spacing_md).align_y(iced::Alignment::Center),
                scrollable(details).height(Length::Fill),
            ].spacing(vm.spacing_md).padding(vm.padding_md)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render detailed YubiKey hardware information (read-only)
    fn view_yubikey_details<'a>(&self, domain_node: &'a DomainNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        let text_tiny = vm.text_tiny;
        let spacing_xs = vm.spacing_xs;
        let (title, details) = match domain_node.data() {
            DomainNodeData::YubiKey(yk) => (
                "YubiKey Hardware Token",
                column![
                    self.detail_row("Device Type:", "YubiKey Hardware Security Module", vm),
                    self.detail_row("Serial Number:", &yk.serial, vm),
                    self.detail_row("Firmware Version:", &yk.version, vm),
                    self.detail_row("Provisioned:",
                        &yk.provisioned_at
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or_else(|| "Not provisioned".to_string())
                    , vm),
                    self.detail_row("Slots Used:", &format!("{} / 4", yk.slots_used.len()), vm),
                    text("Active PIV Slots:").size(vm.text_small).color(vm.colors.text_secondary),
                    if !yk.slots_used.is_empty() {
                        column(yk.slots_used.iter().map(|slot| {
                            text(format!("  • Slot {}", slot)).size(text_tiny).into()
                        }).collect::<Vec<_>>()).spacing(spacing_xs)
                    } else {
                        column![text("  No slots in use").size(text_tiny).color(vm.colors.text_disabled)].into()
                    },
                    text("Available Slots:").size(vm.text_small).color(vm.colors.text_secondary),
                    column![
                        text("  • 9A - Authentication (PIV)").size(text_tiny),
                        text("  • 9C - Digital Signature (PIV)").size(text_tiny),
                        text("  • 9D - Key Management (PIV)").size(text_tiny),
                        text("  • 9E - Card Authentication (PIV)").size(text_tiny),
                    ].spacing(spacing_xs),
                ].spacing(vm.spacing_sm)
            ),
            DomainNodeData::PivSlot(slot) => (
                "PIV Slot",
                column![
                    self.detail_row("Slot:", &slot.slot_name, vm),
                    self.detail_row("YubiKey:", &slot.yubikey_serial, vm),
                    self.detail_row("Status:", if slot.has_key { "In Use" } else { "Empty" }, vm),
                    self.detail_row("Purpose:", {
                        if slot.slot_name.contains("9A") {
                            "Authentication - SSH, VPN, System Login"
                        } else if slot.slot_name.contains("9C") {
                            "Digital Signature - Code Signing, Email Signing"
                        } else if slot.slot_name.contains("9D") {
                            "Key Management - Encryption, Decryption"
                        } else if slot.slot_name.contains("9E") {
                            "Card Authentication - Physical Access"
                        } else {
                            "Unknown"
                        }
                    }, vm),
                    if slot.has_key {
                        self.detail_row("Certificate Subject:",
                            slot.certificate_subject.as_deref().unwrap_or("No certificate loaded"), vm)
                    } else {
                        self.detail_row("Certificate:", "None (empty slot)", vm)
                    },
                    self.detail_row("Algorithm:", if slot.has_key { "RSA 2048 or ECC P-256" } else { "N/A" }, vm),
                    self.detail_row("Touch Policy:", "Not configured", vm),
                    self.detail_row("PIN Policy:", "Default (once per session)", vm),
                ].spacing(vm.spacing_sm)
            ),
            _ => ("Unknown", column![].spacing(vm.spacing_sm)),
        };

        container(
            column![
                row![
                    text(title).size(vm.text_large),
                    button(verified::icon("close", vm.text_medium))
                        .on_press(PropertyCardMessage::Close)
                        .style(|theme: &Theme, _status| {
                            button::Style {
                                background: None,
                                text_color: theme.palette().danger,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ].spacing(vm.spacing_md).align_y(iced::Alignment::Center),
                scrollable(details).height(Length::Fill),
            ].spacing(vm.spacing_md).padding(vm.padding_md)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render detailed policy information (read-only)
    fn view_policy_details<'a>(&self, domain_node: &'a DomainNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        use crate::gui::cowboy_theme::CowboyAppTheme as CowboyCustomTheme;

        let (title, details) = match domain_node.data() {
            DomainNodeData::PolicyRole(role) => (
                "Role",
                column![
                    self.detail_row("Name:", &role.name, vm),
                    self.detail_row("Purpose:", &role.purpose, vm),
                    self.detail_row("Level:", &format!("{} ({})", role.level, match role.level {
                        0 => "Entry",
                        1 => "Junior",
                        2 => "Mid-level",
                        3 => "Senior",
                        4 => "Staff/Principal",
                        5 => "Executive",
                        _ => "Custom",
                    }), vm),
                    self.detail_row("Separation Class:", &format!("{:?}", role.separation_class), vm),
                    self.detail_row("Claims:", &format!("{} permissions", role.claim_count), vm),
                ].spacing(vm.spacing_sm)
            ),
            DomainNodeData::PolicyCategory(category) => (
                "Claim Category",
                column![
                    self.detail_row("Category:", &category.name, vm),
                    self.detail_row("Total Claims:", &format!("{}", category.claim_count), vm),
                    text("Claim categories group related permissions.").size(vm.text_tiny).color(vm.colors.text_hint),
                ].spacing(vm.spacing_sm)
            ),
            DomainNodeData::PolicyGroup(group) => (
                "Separation Class",
                column![
                    self.detail_row("Class:", &group.name, vm),
                    self.detail_row("Type:", &format!("{:?}", group.separation_class), vm),
                    self.detail_row("Roles:", &format!("{} roles", group.role_count), vm),
                    text("Separation classes enforce duty segregation.").size(vm.text_tiny).color(vm.colors.text_hint),
                    text(match group.separation_class {
                        crate::policy::SeparationClass::Operational => "Operational roles handle day-to-day tasks.",
                        crate::policy::SeparationClass::Administrative => "Administrative roles manage users and policies.",
                        crate::policy::SeparationClass::Audit => "Audit roles monitor and review system activity.",
                        crate::policy::SeparationClass::Emergency => "Emergency roles provide break-glass access.",
                        crate::policy::SeparationClass::Financial => "Financial roles manage budgets and spending.",
                        crate::policy::SeparationClass::Personnel => "Personnel roles handle HR and staffing.",
                    }).size(vm.text_tiny).color(vm.colors.text_subtle_success),
                ].spacing(vm.spacing_sm)
            ),
            _ => ("Policy", column![].spacing(vm.spacing_sm)),
        };

        container(
            column![
                row![
                    text(title).size(vm.text_large),
                    button(verified::icon("close", vm.text_medium))
                        .on_press(PropertyCardMessage::Close)
                        .style(|theme: &Theme, _status| {
                            button::Style {
                                background: None,
                                text_color: theme.palette().danger,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ].spacing(vm.spacing_md).align_y(iced::Alignment::Center),
                scrollable(details).height(Length::Fill),
            ].spacing(vm.spacing_md).padding(vm.padding_md)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(CowboyCustomTheme::pastel_coral_card())
        .into()
    }

    /// Helper to create a detail row (label: value)
    fn detail_row(&self, label: impl Into<String>, value: impl Into<String>, vm: &ViewModel) -> Row<'static, PropertyCardMessage> {
        let label_str = label.into();
        let value_str = value.into();
        row![
            text(label_str).size(vm.text_small).color(vm.colors.text_secondary).width(Length::Fixed(150.0)),
            text(value_str).size(vm.text_small),
        ]
        .spacing(vm.spacing_md)
        .align_y(iced::Alignment::Center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Organization;
    use std::collections::HashMap;

    #[test]
    fn test_property_card_creation() {
        let card = PropertyCard::new();
        assert!(!card.is_editing());
        assert!(!card.is_dirty());
    }

    #[test]
    fn test_property_card_set_node() {
        let mut card = PropertyCard::new();

        let org = Organization {
            id: Uuid::new_v4(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("A test org".to_string()),
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let domain_node = DomainNode::inject_organization(org.clone());
        card.set_node(org.id, domain_node);

        assert!(card.is_editing());
        assert!(!card.is_dirty());
        assert_eq!(card.name(), "Test Org");
        assert_eq!(card.description(), "A test org");
    }

    #[test]
    fn test_property_card_dirty_state() {
        let mut card = PropertyCard::new();

        let org = Organization {
            id: Uuid::new_v4(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("A test org".to_string()),
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let domain_node = DomainNode::inject_organization(org.clone());
        card.set_node(org.id, domain_node);
        assert!(!card.is_dirty());

        card.update(PropertyCardMessage::NameChanged("New Name".to_string()));
        assert!(card.is_dirty());
        assert_eq!(card.name(), "New Name");
    }

    #[test]
    fn test_property_card_clear() {
        let mut card = PropertyCard::new();

        let org = Organization {
            id: Uuid::new_v4(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("A test org".to_string()),
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        };

        let domain_node = DomainNode::inject_organization(org.clone());
        card.set_node(org.id, domain_node);
        card.update(PropertyCardMessage::NameChanged("Modified".to_string()));

        assert!(card.is_editing());
        assert!(card.is_dirty());

        card.clear();

        assert!(!card.is_editing());
        assert!(!card.is_dirty());
        assert_eq!(card.name(), "");
    }
}
