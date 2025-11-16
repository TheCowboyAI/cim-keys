//! Property card for editing node properties
//!
//! Displays and allows editing of properties for selected graph nodes.

use iced::{
    widget::{button, checkbox, column, container, row, text, text_input, scrollable, Column, Row},
    Element, Length, Theme,
};
use uuid::Uuid;
use std::collections::HashSet;

use crate::gui::graph::{NodeType, EdgeType};
use crate::domain::PolicyClaim;

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
            PropertyCardMessage::EdgeTypeChanged(edge_type) => {
                self.edit_edge_type = edge_type;
                self.dirty = true;
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
    fn view_node(&self, node_type: &NodeType) -> Element<'_, PropertyCardMessage> {
        let node_type_label = match node_type {
            NodeType::Organization(_) => "Organization",
            NodeType::OrganizationalUnit(_) => "Organizational Unit",
            NodeType::Person { .. } => "Person",
            NodeType::Location(_) => "Location",
            NodeType::Role(_) => "Role",
            NodeType::Policy(_) => "Policy",
        };

        let header: Row<'_, PropertyCardMessage> = row![
            text(node_type_label).size(18),
            button(text("✕").size(16))
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

        let content: Column<'_, PropertyCardMessage> = column![
            header,
            fields,
            buttons,
        ]
        .spacing(15)
        .padding(20);

        container(content)
            .width(Length::Fixed(300.0))
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(iced::Background::Color(theme.palette().background)),
                    border: iced::Border {
                        color: theme.palette().text,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    text_color: Some(theme.palette().text),
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                        offset: iced::Vector::new(4.0, 4.0),
                        blur_radius: 8.0,
                    },
                }
            })
            .into()
    }

    /// Render property card for editing an edge
    fn view_edge(&self, _edge_type: &EdgeType) -> Element<'_, PropertyCardMessage> {
        let header: Row<'_, PropertyCardMessage> = row![
            text("Edge Relationship").size(18),
            button(text("✕").size(16))
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

        let content: Column<'_, PropertyCardMessage> = column![
            header,
            fields,
            buttons,
        ]
        .spacing(15)
        .padding(20);

        container(content)
            .width(Length::Fixed(300.0))
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(iced::Background::Color(theme.palette().background)),
                    border: iced::Border {
                        color: theme.palette().text,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    text_color: Some(theme.palette().text),
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                        offset: iced::Vector::new(4.0, 4.0),
                        blur_radius: 8.0,
                    },
                }
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, Person};
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
