//! Property card for editing node properties
//!
//! Displays and allows editing of properties for selected graph nodes.

use iced::{
    widget::{button, checkbox, column, container, row, text, text_input, scrollable, Column, Row},
    Color, Element, Length, Theme,
};
use uuid::Uuid;
use std::collections::HashSet;

use crate::gui::graph::{NodeType, EdgeType};
use crate::domain::{PolicyClaim, RoleType};
use crate::icons::{self, ICON_CLOSE};

/// What is being edited
#[derive(Debug, Clone)]
pub enum EditTarget {
    Node { id: Uuid, node_type: NodeType },
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
    // Key generation messages (for Person nodes)
    /// User clicked generate root CA button
    GenerateRootCA,
    /// User clicked generate personal keys button
    GeneratePersonalKeys,
    /// User clicked provision YubiKey button
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
            edit_edge_type: EdgeType::MemberOf,  // Default edge type
        }
    }

    /// Set the node to edit
    pub fn set_node(&mut self, node_id: Uuid, node_type: NodeType) {
        self.edit_target = Some(EditTarget::Node {
            id: node_id,
            node_type: node_type.clone(),
        });
        self.dirty = false;

        // Initialize edit fields from node data
        match &node_type {
            NodeType::Organization(org) => {
                self.edit_name = org.name.clone();
                self.edit_description = org.description.clone().unwrap_or_default();
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::OrganizationalUnit(unit) => {
                self.edit_name = unit.name.clone();
                self.edit_description = String::new();
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::Person { person, .. } => {
                self.edit_name = person.name.clone();
                self.edit_description = String::new();
                self.edit_email = person.email.clone();
                self.edit_enabled = person.active;
                self.edit_roles = person.roles.iter().map(|r| r.role_type.clone()).collect();
            }
            NodeType::Location(loc) => {
                self.edit_name = loc.name.clone();
                self.edit_description = String::new();
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::Role(role) => {
                self.edit_name = role.name.clone();
                self.edit_description = role.description.clone();
                self.edit_email = String::new();
                self.edit_enabled = role.active;
            }
            NodeType::Policy(policy) => {
                self.edit_name = policy.name.clone();
                self.edit_description = policy.description.clone();
                self.edit_email = String::new();
                self.edit_enabled = policy.enabled;
                self.edit_claims = policy.claims.iter().cloned().collect();
            }
            // NATS Infrastructure - read-only, no editing
            NodeType::NatsOperator(identity) => {
                self.edit_name = "NATS Operator".to_string();
                self.edit_description = format!("NKey: {}", identity.nkey.public_key.public_key());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::NatsAccount(identity) => {
                self.edit_name = "NATS Account".to_string();
                self.edit_description = format!("NKey: {}", identity.nkey.public_key.public_key());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::NatsUser(identity) => {
                self.edit_name = "NATS User".to_string();
                self.edit_description = format!("NKey: {}", identity.nkey.public_key.public_key());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::NatsServiceAccount(identity) => {
                self.edit_name = "Service Account".to_string();
                self.edit_description = format!("NKey: {}", identity.nkey.public_key.public_key());
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // PKI Trust Chain - read-only, no editing
            NodeType::RootCertificate { subject, issuer, .. } => {
                self.edit_name = format!("Root CA: {}", subject);
                self.edit_description = format!("Issuer: {}", issuer);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::IntermediateCertificate { subject, issuer, .. } => {
                self.edit_name = format!("Intermediate CA: {}", subject);
                self.edit_description = format!("Issuer: {}", issuer);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::LeafCertificate { subject, issuer, .. } => {
                self.edit_name = format!("Certificate: {}", subject);
                self.edit_description = format!("Issuer: {}", issuer);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            // YubiKey Hardware - read-only, no editing
            NodeType::YubiKey { serial, version, .. } => {
                self.edit_name = format!("YubiKey {}", serial);
                self.edit_description = format!("Version: {}", version);
                self.edit_email = String::new();
                self.edit_enabled = true;
            }
            NodeType::PivSlot { slot_name, yubikey_serial, has_key, .. } => {
                self.edit_name = slot_name.clone();
                self.edit_description = format!("YubiKey {} - {}", yubikey_serial, if *has_key { "In use" } else { "Empty" });
                self.edit_email = String::new();
                self.edit_enabled = *has_key;
            }
            NodeType::YubiKeyStatus { yubikey_serial, slots_provisioned, slots_needed, .. } => {
                self.edit_name = "YubiKey Status".to_string();
                self.edit_description = format!("{}/{} slots - {}",
                    slots_provisioned.len(),
                    slots_needed.len(),
                    yubikey_serial.clone().unwrap_or_else(|| "Not detected".to_string())
                );
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
            PropertyCardMessage::EdgeTypeChanged(edge_type) => {
                self.edit_edge_type = edge_type;
                self.dirty = true;
            }
            PropertyCardMessage::GenerateRootCA => {
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
    pub fn view(&self) -> Element<'_, PropertyCardMessage> {
        if !self.is_editing() {
            return container(text("Select a node or edge to edit"))
                .padding(20)
                .into();
        }

        match &self.edit_target {
            Some(EditTarget::Node { node_type, .. }) => self.view_node(node_type),
            Some(EditTarget::Edge { edge_type, .. }) => self.view_edge(edge_type),
            None => container(text("Select a node or edge to edit"))
                .padding(20)
                .into(),
        }
    }

    /// Render property card for editing a node
    fn view_node<'a>(&self, node_type: &'a NodeType) -> Element<'a, PropertyCardMessage> {
        // For read-only infrastructure types, show detailed info panel instead of edit fields
        match node_type {
            NodeType::NatsOperator(_) | NodeType::NatsAccount(_) |
            NodeType::NatsUser(_) | NodeType::NatsServiceAccount(_) => {
                return self.view_nats_details(node_type);
            }
            NodeType::RootCertificate { .. } | NodeType::IntermediateCertificate { .. } |
            NodeType::LeafCertificate { .. } => {
                return self.view_certificate_details(node_type);
            }
            NodeType::YubiKey { .. } | NodeType::PivSlot { .. } => {
                return self.view_yubikey_details(node_type);
            }
            _ => {}
        }

        let node_type_label = match node_type {
            NodeType::Organization(_) => "Organization",
            NodeType::OrganizationalUnit(_) => "Organizational Unit",
            NodeType::Person { .. } => "Person",
            NodeType::Location(_) => "Location",
            NodeType::Role(_) => "Role",
            NodeType::Policy(_) => "Policy",
            _ => "Unknown",
        };

        let header: Row<'_, PropertyCardMessage> = row![
            text(node_type_label).size(18),
            button(icons::icon_sized(ICON_CLOSE, 16))
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
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let mut fields: Column<'_, PropertyCardMessage> = column![]
            .spacing(10)
            .padding(10);

        // Name field (all types have this)
        fields = fields.push(
            column![
                text("Name:").size(12),
                text_input("Enter name", &self.edit_name)
                    .on_input(PropertyCardMessage::NameChanged)
                    .width(Length::Fill),
            ]
            .spacing(4)
        );

        // Description field (Organization, Role, Policy)
        if matches!(
            node_type,
            NodeType::Organization(_) | NodeType::Role(_) | NodeType::Policy(_)
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
        if matches!(node_type, NodeType::Person { .. }) {
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
        if matches!(node_type, NodeType::Person { .. }) {
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

        // Enabled checkbox (Person, Role, Policy)
        if matches!(
            node_type,
            NodeType::Person { .. } | NodeType::Role(_) | NodeType::Policy(_)
        ) {
            let label = match node_type {
                NodeType::Person { .. } => "Active",
                NodeType::Role(_) => "Active",
                NodeType::Policy(_) => "Enabled",
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
        if matches!(node_type, NodeType::Policy(_)) {
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
    fn view_edge(&self, _edge_type: &EdgeType) -> Element<'_, PropertyCardMessage> {
        let header: Row<'_, PropertyCardMessage> = row![
            text("Edge Relationship").size(18),
            button(icons::icon_sized(ICON_CLOSE, 16))
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
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let mut fields: Column<'_, PropertyCardMessage> = column![]
            .spacing(5)
            .padding(5);

        // Edge type label
        fields = fields.push(
            text("Relationship Type:").size(12)
        );

        // Edge type picker (list of buttons for each type)
        let edge_types = vec![
            ("Parent-Child", EdgeType::ParentChild),
            ("Manages Unit", EdgeType::ManagesUnit),
            ("Member Of", EdgeType::MemberOf),
            ("Owns Key", EdgeType::OwnsKey),
            ("Stored At", EdgeType::StoredAt),
            ("Has Role", EdgeType::HasRole),
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
                (EdgeType::HasRole, EdgeType::HasRole) |
                (EdgeType::Hierarchy, EdgeType::Hierarchy) |
                (EdgeType::Trust, EdgeType::Trust)
            );

            fields = fields.push(
                button(text(label).size(12))
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
            button(text("Delete").size(12))
                .on_press(PropertyCardMessage::DeleteEdge)
                .style(|theme: &Theme, _status| {
                    button::Style {
                        background: Some(iced::Background::Color(theme.palette().danger)),
                        text_color: iced::Color::WHITE,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }),
            button(text("Cancel").size(12))
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

    /// Render detailed NATS infrastructure information (read-only)
    fn view_nats_details<'a>(&self, node_type: &'a NodeType) -> Element<'a, PropertyCardMessage> {
        let (title, details) = match node_type {
            NodeType::NatsOperator(identity) => (
                "NATS Operator",
                column![
                    self.detail_row("Type:", "Root Authority"),
                    self.detail_row("Public Key:", &identity.nkey.public_key.public_key()[..32]),
                    self.detail_row("Key Type:", &format!("{:?}", identity.nkey.key_type)),
                    text("JWT Token:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    scrollable(
                        text(identity.jwt.token())
                            .size(10)
                            .font(iced::Font::MONOSPACE)
                    ).height(Length::Fixed(100.0)),
                    self.detail_row("Has Credential:", if identity.credential.is_some() { "Yes" } else { "No" }),
                ].spacing(8)
            ),
            NodeType::NatsAccount(identity) => (
                "NATS Account",
                column![
                    self.detail_row("Type:", "Account (Organizational Unit)"),
                    self.detail_row("Public Key:", &identity.nkey.public_key.public_key()[..32]),
                    self.detail_row("Key Type:", &format!("{:?}", identity.nkey.key_type)),
                    text("JWT Token:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    scrollable(
                        text(identity.jwt.token())
                            .size(10)
                            .font(iced::Font::MONOSPACE)
                    ).height(Length::Fixed(100.0)),
                    self.detail_row("Has Credential:", if identity.credential.is_some() { "Yes" } else { "No" }),
                ].spacing(8)
            ),
            NodeType::NatsUser(identity) => (
                "NATS User",
                column![
                    self.detail_row("Type:", "User (Person)"),
                    self.detail_row("Public Key:", &identity.nkey.public_key.public_key()[..32]),
                    self.detail_row("Key Type:", &format!("{:?}", identity.nkey.key_type)),
                    text("JWT Token:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    scrollable(
                        text(identity.jwt.token())
                            .size(10)
                            .font(iced::Font::MONOSPACE)
                    ).height(Length::Fixed(100.0)),
                    self.detail_row("Has Credential:", if identity.credential.is_some() { "Yes" } else { "No" }),
                ].spacing(8)
            ),
            NodeType::NatsServiceAccount(identity) => (
                "NATS Service Account",
                column![
                    self.detail_row("Type:", "Service (Automation)"),
                    self.detail_row("Public Key:", &identity.nkey.public_key.public_key()[..32]),
                    self.detail_row("Key Type:", &format!("{:?}", identity.nkey.key_type)),
                    text("JWT Token:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    scrollable(
                        text(identity.jwt.token())
                            .size(10)
                            .font(iced::Font::MONOSPACE)
                    ).height(Length::Fixed(100.0)),
                    self.detail_row("Has Credential:", if identity.credential.is_some() { "Yes" } else { "No" }),
                ].spacing(8)
            ),
            _ => ("Unknown", column![].spacing(8)),
        };

        container(
            column![
                row![
                    text(title).size(18),
                    button(icons::icon_sized(ICON_CLOSE, 16))
                        .on_press(PropertyCardMessage::Close)
                        .style(|theme: &Theme, _status| {
                            button::Style {
                                background: None,
                                text_color: theme.palette().danger,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ].spacing(10).align_y(iced::Alignment::Center),
                scrollable(details).height(Length::Fill),
            ].spacing(10).padding(15)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render detailed PKI certificate information (read-only)
    fn view_certificate_details<'a>(&self, node_type: &'a NodeType) -> Element<'a, PropertyCardMessage> {
        let (title, details) = match node_type {
            NodeType::RootCertificate { subject, issuer, not_before, not_after, key_usage, .. } => (
                "Root CA Certificate",
                column![
                    self.detail_row("Certificate Type:", "Root Certificate Authority"),
                    self.detail_row("Subject:", subject),
                    self.detail_row("Issuer:", issuer),
                    self.detail_row("Valid From:", &not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                    self.detail_row("Valid Until:", &not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                    self.detail_row("Validity:", &format!("{} days", (not_after.signed_duration_since(*not_before).num_days()))),
                    text("Key Usage:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    column(key_usage.iter().map(|usage| {
                        text(format!("  • {}", usage)).size(11).into()
                    }).collect::<Vec<_>>()).spacing(2),
                    self.detail_row("Trust Level:", "Root (Highest)"),
                    self.detail_row("Path Length:", "Unlimited"),
                ].spacing(8)
            ),
            NodeType::IntermediateCertificate { subject, issuer, not_before, not_after, key_usage, .. } => (
                "Intermediate CA Certificate",
                column![
                    self.detail_row("Certificate Type:", "Intermediate Certificate Authority"),
                    self.detail_row("Subject:", subject),
                    self.detail_row("Issuer:", issuer),
                    self.detail_row("Valid From:", &not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                    self.detail_row("Valid Until:", &not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                    self.detail_row("Validity:", &format!("{} days", (not_after.signed_duration_since(*not_before).num_days()))),
                    text("Key Usage:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    column(key_usage.iter().map(|usage| {
                        text(format!("  • {}", usage)).size(11).into()
                    }).collect::<Vec<_>>()).spacing(2),
                    self.detail_row("Trust Level:", "Intermediate"),
                    self.detail_row("Can Sign:", "Leaf Certificates"),
                ].spacing(8)
            ),
            NodeType::LeafCertificate { subject, issuer, not_before, not_after, key_usage, san, .. } => (
                "Leaf Certificate",
                column![
                    self.detail_row("Certificate Type:", "End Entity Certificate"),
                    self.detail_row("Subject:", subject),
                    self.detail_row("Issuer:", issuer),
                    self.detail_row("Valid From:", &not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                    self.detail_row("Valid Until:", &not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                    self.detail_row("Validity:", &format!("{} days", (not_after.signed_duration_since(*not_before).num_days()))),
                    text("Key Usage:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    column(key_usage.iter().map(|usage| {
                        text(format!("  • {}", usage)).size(11).into()
                    }).collect::<Vec<_>>()).spacing(2),
                    if !san.is_empty() {
                        column![
                            text("Subject Alternative Names:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                            column(san.iter().map(|name| {
                                text(format!("  • {}", name)).size(11).into()
                            }).collect::<Vec<_>>()).spacing(2),
                        ].spacing(4)
                    } else {
                        column![].into()
                    },
                    self.detail_row("Trust Level:", "Leaf (End Entity)"),
                ].spacing(8)
            ),
            _ => ("Unknown", column![].spacing(8)),
        };

        container(
            column![
                row![
                    text(title).size(18),
                    button(icons::icon_sized(ICON_CLOSE, 16))
                        .on_press(PropertyCardMessage::Close)
                        .style(|theme: &Theme, _status| {
                            button::Style {
                                background: None,
                                text_color: theme.palette().danger,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ].spacing(10).align_y(iced::Alignment::Center),
                scrollable(details).height(Length::Fill),
            ].spacing(10).padding(15)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render detailed YubiKey hardware information (read-only)
    fn view_yubikey_details<'a>(&self, node_type: &'a NodeType) -> Element<'a, PropertyCardMessage> {
        let (title, details) = match node_type {
            NodeType::YubiKey { serial, version, provisioned_at, slots_used, .. } => (
                "YubiKey Hardware Token",
                column![
                    self.detail_row("Device Type:", "YubiKey Hardware Security Module"),
                    self.detail_row("Serial Number:", serial),
                    self.detail_row("Firmware Version:", version),
                    self.detail_row("Provisioned:",
                        &provisioned_at
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or_else(|| "Not provisioned".to_string())
                    ),
                    self.detail_row("Slots Used:", &format!("{} / 4", slots_used.len())),
                    text("Active PIV Slots:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    if !slots_used.is_empty() {
                        column(slots_used.iter().map(|slot| {
                            text(format!("  • Slot {}", slot)).size(11).into()
                        }).collect::<Vec<_>>()).spacing(2)
                    } else {
                        column![text("  No slots in use").size(11).color(Color::from_rgb(0.5, 0.5, 0.5))].into()
                    },
                    text("Available Slots:").size(12).color(Color::from_rgb(0.7, 0.7, 0.8)),
                    column![
                        text("  • 9A - Authentication (PIV)").size(11),
                        text("  • 9C - Digital Signature (PIV)").size(11),
                        text("  • 9D - Key Management (PIV)").size(11),
                        text("  • 9E - Card Authentication (PIV)").size(11),
                    ].spacing(2),
                ].spacing(8)
            ),
            NodeType::PivSlot { slot_name, yubikey_serial, has_key, certificate_subject, .. } => (
                "PIV Slot",
                column![
                    self.detail_row("Slot:", slot_name),
                    self.detail_row("YubiKey:", yubikey_serial),
                    self.detail_row("Status:", if *has_key { "In Use" } else { "Empty" }),
                    self.detail_row("Purpose:", {
                        if slot_name.contains("9A") {
                            "Authentication - SSH, VPN, System Login"
                        } else if slot_name.contains("9C") {
                            "Digital Signature - Code Signing, Email Signing"
                        } else if slot_name.contains("9D") {
                            "Key Management - Encryption, Decryption"
                        } else if slot_name.contains("9E") {
                            "Card Authentication - Physical Access"
                        } else {
                            "Unknown"
                        }
                    }),
                    if *has_key {
                        self.detail_row("Certificate Subject:",
                            certificate_subject.as_deref().unwrap_or("No certificate loaded"))
                    } else {
                        self.detail_row("Certificate:", "None (empty slot)")
                    },
                    self.detail_row("Algorithm:", if *has_key { "RSA 2048 or ECC P-256" } else { "N/A" }),
                    self.detail_row("Touch Policy:", "Not configured"),
                    self.detail_row("PIN Policy:", "Default (once per session)"),
                ].spacing(8)
            ),
            _ => ("Unknown", column![].spacing(8)),
        };

        container(
            column![
                row![
                    text(title).size(18),
                    button(icons::icon_sized(ICON_CLOSE, 16))
                        .on_press(PropertyCardMessage::Close)
                        .style(|theme: &Theme, _status| {
                            button::Style {
                                background: None,
                                text_color: theme.palette().danger,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            }
                        }),
                ].spacing(10).align_y(iced::Alignment::Center),
                scrollable(details).height(Length::Fill),
            ].spacing(10).padding(15)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Helper to create a detail row (label: value)
    fn detail_row(&self, label: impl Into<String>, value: impl Into<String>) -> Row<'static, PropertyCardMessage> {
        let label_str = label.into();
        let value_str = value.into();
        row![
            text(label_str).size(12).color(Color::from_rgb(0.7, 0.7, 0.8)).width(Length::Fixed(150.0)),
            text(value_str).size(12),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Organization;
    use chrono::Utc;
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
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        card.set_node(org.id, NodeType::Organization(org.clone()));

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
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        card.set_node(org.id, NodeType::Organization(org.clone()));
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
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        card.set_node(org.id, NodeType::Organization(org));
        card.update(PropertyCardMessage::NameChanged("Modified".to_string()));

        assert!(card.is_editing());
        assert!(card.is_dirty());

        card.clear();

        assert!(!card.is_editing());
        assert!(!card.is_dirty());
        assert_eq!(card.name(), "");
    }
}
