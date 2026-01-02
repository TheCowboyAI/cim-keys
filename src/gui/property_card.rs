//! Property card for editing node properties
//!
//! Displays and allows editing of properties for selected graph nodes.
//!
//! This module uses LiftedNode for property card operations.

use iced::{
    widget::{button, checkbox, column, container, row, text, text_input, scrollable, Column, Row},
    Element, Length, Theme,
};
use uuid::Uuid;
use std::collections::HashSet;

use crate::gui::graph::EdgeType;
use crate::gui::folds::query::extract_edit_fields_from_lifted;
use crate::gui::view_model::ViewModel;
use crate::domain::{PolicyClaim, RoleType, LocationType};
use crate::lifting::LiftedNode;
use crate::icons::verified;

/// What is being edited
#[derive(Debug, Clone)]
pub enum EditTarget {
    Node { id: Uuid, lifted_node: LiftedNode },
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
    ///
    /// Uses the LiftedNode downcast pattern to extract edit data,
    /// replacing the deprecated DomainNode fold pattern.
    pub fn set_node(&mut self, node_id: Uuid, lifted_node: LiftedNode) {
        self.edit_target = Some(EditTarget::Node {
            id: node_id,
            lifted_node: lifted_node.clone(),
        });
        self.dirty = false;

        // Extract edit fields using the new downcast-based extraction
        let edit_data = extract_edit_fields_from_lifted(&lifted_node);

        // Apply the extracted data to our edit fields
        self.edit_name = edit_data.name;
        self.edit_description = edit_data.description;
        self.edit_email = edit_data.email;
        self.edit_enabled = edit_data.enabled;

        // Roles (for Person nodes)
        self.edit_roles = edit_data.role_types.into_iter().collect();

        // Claims (for Policy nodes)
        self.edit_claims = edit_data.policy_claims.into_iter().collect();

        // Location-specific fields
        if let Some(loc_type) = edit_data.location_type {
            self.edit_location_type = loc_type;
        }
        self.edit_address = edit_data.address.unwrap_or_default();
        self.edit_coordinates = edit_data.coordinates.unwrap_or_default();
        self.edit_virtual_location = edit_data.virtual_location.unwrap_or_default();
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
            Some(EditTarget::Node { lifted_node, .. }) => self.view_node(lifted_node, vm),
            Some(EditTarget::Edge { edge_type, .. }) => self.view_edge(edge_type, vm),
            None => container(text("Select a node or edge to edit").size(vm.text_normal))
                .padding(vm.padding_xl)
                .into(),
        }
    }

    /// Render property card for editing a node
    fn view_node<'a>(&self, lifted_node: &'a LiftedNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        // For read-only infrastructure types, show detailed info panel instead of edit fields
        let injection = lifted_node.injection();
        if injection.is_nats() {
            return self.view_nats_details(lifted_node, vm);
        }
        if injection.is_certificate() {
            return self.view_certificate_details(lifted_node, vm);
        }
        if injection.is_yubikey_or_slot() {
            return self.view_yubikey_details(lifted_node, vm);
        }
        if injection.is_policy_variant() {
            return self.view_policy_details(lifted_node, vm);
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
        if injection.is_location() {
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
        if injection.is_organization() || injection.is_role() || injection.is_policy() {
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
        if injection.is_person() {
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
        if injection.is_person() {
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
        if injection.is_organization() {
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
        if injection.is_organization_unit() {
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
        if injection.is_person() || injection.is_role() || injection.is_policy() {
            let label = if injection.is_person() || injection.is_role() {
                "Active"
            } else {
                "Enabled"
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
        if injection.is_policy() {
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
    fn view_nats_details<'a>(&self, lifted_node: &'a LiftedNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        use crate::domain_projections::NatsIdentityProjection;

        // Try to downcast to get detailed NATS identity info
        let (title, details) = if let Some(identity) = lifted_node.downcast::<NatsIdentityProjection>() {
            let type_label = lifted_node.injection().display_name();
            (
                type_label,
                column![
                    self.detail_row("Type:", type_label, vm),
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
            )
        } else {
            // Fallback: use label and secondary from LiftedNode
            let title = lifted_node.injection().display_name();
            let details = column![
                self.detail_row("Name:", &lifted_node.label, vm),
                self.detail_row("Info:", lifted_node.secondary.as_deref().unwrap_or("N/A"), vm),
            ].spacing(vm.spacing_sm);
            (title, details)
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
    fn view_certificate_details<'a>(&self, lifted_node: &'a LiftedNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        use crate::domain::pki::{Certificate, CertificateType};

        let text_tiny = vm.text_tiny;
        let spacing_xs = vm.spacing_xs;

        // Try downcasting to Certificate - all certificate types use the same struct
        // with cert_type field distinguishing them
        let (title, details) = if let Some(cert) = lifted_node.downcast::<Certificate>() {
            match cert.cert_type {
                CertificateType::Root => (
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
                CertificateType::Intermediate => (
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
                CertificateType::Leaf => (
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
                CertificateType::Policy => (
                    "Policy Certificate",
                    column![
                        self.detail_row("Certificate Type:", "Policy-Specific CA", vm),
                        self.detail_row("Subject:", &cert.subject, vm),
                        self.detail_row("Issuer:", &cert.issuer, vm),
                        self.detail_row("Valid From:", &cert.not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string(), vm),
                        self.detail_row("Valid Until:", &cert.not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string(), vm),
                        self.detail_row("Validity:", &format!("{} days", (cert.not_after.signed_duration_since(cert.not_before).num_days())), vm),
                        text("Key Usage:").size(vm.text_small).color(vm.colors.text_secondary),
                        column(cert.key_usage.iter().map(|usage| {
                            text(format!("  • {}", usage)).size(text_tiny).into()
                        }).collect::<Vec<_>>()).spacing(spacing_xs),
                        self.detail_row("Trust Level:", "Policy-Specific", vm),
                    ].spacing(vm.spacing_sm)
                ),
            }
        } else {
            // Fallback: show basic info from LiftedNode
            (
                lifted_node.injection().display_name(),
                column![
                    self.detail_row("Name:", &lifted_node.label, vm),
                    self.detail_row("Info:", lifted_node.secondary.as_deref().unwrap_or("N/A"), vm),
                ].spacing(vm.spacing_sm)
            )
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
    fn view_yubikey_details<'a>(&self, lifted_node: &'a LiftedNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        use crate::domain::yubikey::{YubiKeyDevice, PivSlotView};

        let text_tiny = vm.text_tiny;
        let spacing_xs = vm.spacing_xs;

        // Try downcasting to YubiKeyDevice or PivSlotView
        let (title, details) = if let Some(yk) = lifted_node.downcast::<YubiKeyDevice>() {
            (
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
            )
        } else if let Some(slot) = lifted_node.downcast::<PivSlotView>() {
            (
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
            )
        } else {
            // Fallback: show basic info from LiftedNode
            (
                lifted_node.injection().display_name(),
                column![
                    self.detail_row("Name:", &lifted_node.label, vm),
                    self.detail_row("Info:", lifted_node.secondary.as_deref().unwrap_or("N/A"), vm),
                ].spacing(vm.spacing_sm)
            )
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
    fn view_policy_details<'a>(&self, lifted_node: &'a LiftedNode, vm: &ViewModel) -> Element<'a, PropertyCardMessage> {
        use crate::gui::cowboy_theme::CowboyAppTheme as CowboyCustomTheme;
        use crate::domain::visualization::{PolicyRole, PolicyCategory, PolicyGroup};

        // Try downcasting to each policy type
        let (title, details) = if let Some(role) = lifted_node.downcast::<PolicyRole>() {
            (
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
            )
        } else if let Some(category) = lifted_node.downcast::<PolicyCategory>() {
            (
                "Claim Category",
                column![
                    self.detail_row("Category:", &category.name, vm),
                    self.detail_row("Total Claims:", &format!("{}", category.claim_count), vm),
                    text("Claim categories group related permissions.").size(vm.text_tiny).color(vm.colors.text_hint),
                ].spacing(vm.spacing_sm)
            )
        } else if let Some(group) = lifted_node.downcast::<PolicyGroup>() {
            (
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
            )
        } else {
            // Fallback: show basic info from LiftedNode
            (
                lifted_node.injection().display_name(),
                column![
                    self.detail_row("Name:", &lifted_node.label, vm),
                    self.detail_row("Info:", lifted_node.secondary.as_deref().unwrap_or("N/A"), vm),
                ].spacing(vm.spacing_sm)
            )
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
    use crate::domain::ids::BootstrapOrgId;
    use crate::lifting::LiftableDomain;
    use std::collections::HashMap;

    fn make_test_org() -> Organization {
        Organization {
            id: BootstrapOrgId::new(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("A test org".to_string()),
            parent_id: None,
            units: vec![],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_property_card_creation() {
        let card = PropertyCard::new();
        assert!(!card.is_editing());
        assert!(!card.is_dirty());
    }

    #[test]
    fn test_property_card_set_node() {
        let mut card = PropertyCard::new();

        let org = make_test_org();

        // Use LiftableDomain trait to lift Organization to LiftedNode
        let lifted_node = org.lift();
        card.set_node(org.id.as_uuid(), lifted_node);

        assert!(card.is_editing());
        assert!(!card.is_dirty());
        assert_eq!(card.name(), "Test Org");
        assert_eq!(card.description(), "A test org");
    }

    #[test]
    fn test_property_card_dirty_state() {
        let mut card = PropertyCard::new();

        let org = make_test_org();

        // Use LiftableDomain trait to lift Organization to LiftedNode
        let lifted_node = org.lift();
        card.set_node(org.id.as_uuid(), lifted_node);
        assert!(!card.is_dirty());

        card.update(PropertyCardMessage::NameChanged("New Name".to_string()));
        assert!(card.is_dirty());
        assert_eq!(card.name(), "New Name");
    }

    #[test]
    fn test_property_card_clear() {
        let mut card = PropertyCard::new();

        let org = make_test_org();

        // Use LiftableDomain trait to lift Organization to LiftedNode
        let lifted_node = org.lift();
        card.set_node(org.id.as_uuid(), lifted_node);
        card.update(PropertyCardMessage::NameChanged("Modified".to_string()));

        assert!(card.is_editing());
        assert!(card.is_dirty());

        card.clear();

        assert!(!card.is_editing());
        assert!(!card.is_dirty());
        assert_eq!(card.name(), "");
    }
}
