// Copyright (c) 2025 - Cowboy AI, LLC.
//! Role Palette - Visual picker for drag-and-drop role assignment
//!
//! This module implements: **Roles from policy-bootstrap.json displayed as a draggable palette**
//!
//! ## Design
//!
//! The RolePalette is displayed as a collapsible sidebar that shows roles grouped by
//! separation class. Users can drag roles from this palette and drop them onto person
//! nodes in the organization graph to assign roles.
//!
//! ## Separation Classes (color-coded)
//!
//! - Operational (Blue) - Day-to-day operations
//! - Administrative (Purple) - System administration
//! - Audit (Teal) - Audit and compliance
//! - Emergency (Red) - Emergency access
//! - Financial (Gold) - Financial operations
//! - Personnel (Rose) - HR and personnel management
//!
//! ## Interaction Model
//!
//! 1. User expands a separation class category
//! 2. User drags a role badge from the palette
//! 3. Ghost node follows cursor with role color
//! 4. Person nodes highlight when hoverable
//! 5. SoD conflicts show as red border on ghost node
//! 6. Dropping on valid person assigns the role

use iced::{
    widget::{button, container, row, scrollable, text, Column},
    Color, Element, Length, Padding, Theme,
};

use crate::policy::SeparationClass;
use crate::policy_loader::{PolicyBootstrapData, StandardRoleEntry};
use super::graph::DragSource;
use super::cowboy_theme::CowboyAppTheme as CowboyCustomTheme;
use super::view_model::ViewModel;
use crate::icons;

/// Message emitted by the role palette
#[derive(Debug, Clone)]
pub enum RolePaletteMessage {
    /// Toggle expansion of a separation class category
    ToggleCategory(SeparationClass),
    /// Start dragging a role
    StartDrag {
        role_name: String,
        separation_class: SeparationClass,
    },
    /// Cancel drag operation
    CancelDrag,
}

/// State for the role palette widget
#[derive(Debug, Clone)]
pub struct RolePalette {
    /// Which categories are expanded
    expanded: std::collections::HashSet<SeparationClass>,
    /// Whether the entire palette is collapsed
    collapsed: bool,
}

impl Default for RolePalette {
    fn default() -> Self {
        Self {
            expanded: std::collections::HashSet::new(),
            collapsed: false,
        }
    }
}

impl RolePalette {
    /// Create a new role palette
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle whether the entire palette is collapsed
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Toggle expansion of a category
    pub fn toggle_category(&mut self, class: SeparationClass) {
        if self.expanded.contains(&class) {
            self.expanded.remove(&class);
        } else {
            self.expanded.insert(class);
        }
    }

    /// Check if a category is expanded
    pub fn is_expanded(&self, class: &SeparationClass) -> bool {
        self.expanded.contains(class)
    }

    /// Get display name for a separation class
    fn class_name(class: &SeparationClass) -> &'static str {
        match class {
            SeparationClass::Operational => "Operational",
            SeparationClass::Administrative => "Administrative",
            SeparationClass::Audit => "Audit",
            SeparationClass::Emergency => "Emergency",
            SeparationClass::Financial => "Financial",
            SeparationClass::Personnel => "Personnel",
        }
    }

    /// Group roles by separation class
    fn group_roles_by_class(policy_data: &PolicyBootstrapData) -> std::collections::HashMap<SeparationClass, Vec<&StandardRoleEntry>> {
        let mut groups: std::collections::HashMap<SeparationClass, Vec<&StandardRoleEntry>> = std::collections::HashMap::new();

        for role in &policy_data.standard_roles {
            let class = role.separation_class_enum();
            groups.entry(class).or_default().push(role);
        }

        // Sort roles within each group by level (highest first), then by name
        for roles in groups.values_mut() {
            roles.sort_by(|a, b| {
                b.level.cmp(&a.level).then(a.name.cmp(&b.name))
            });
        }

        groups
    }

    /// Create a role badge element
    fn role_badge<'a>(role: &'a StandardRoleEntry, class: SeparationClass, vm: &ViewModel) -> Element<'a, RolePaletteMessage> {
        // Get colors from ontological palette
        let color = vm.colors.separation_class_color(&class);
        let text_tertiary = vm.colors.text_tertiary;
        let text_disabled = vm.colors.text_disabled;
        let text_light = vm.colors.text_light;
        let surface_bg = vm.colors.surface;
        let border_radius = vm.radius_sm;
        let claim_count = role.claims.len();

        let badge_content = row![
            // Colored circle indicator
            container(text(" ").size(vm.padding_sm))
                .width(12)
                .height(12)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(color)),
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            // Role name
            text(&role.name)
                .size(vm.text_small)
                .font(icons::FONT_BODY),
            // Level indicator
            text(format!(" L{}", role.level))
                .size(vm.text_tiny)
                .font(icons::FONT_BODY)
                .color(text_tertiary),
            // Claim count
            text(format!(" ({})", claim_count))
                .size(vm.text_tiny)
                .font(icons::FONT_BODY)
                .color(text_disabled),
        ]
        .spacing(vm.padding_xs)
        .align_y(iced::Alignment::Center);

        let role_name = role.name.clone();
        button(badge_content)
            .on_press(RolePaletteMessage::StartDrag {
                role_name,
                separation_class: class,
            })
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color::from_rgba(color.r, color.g, color.b, 0.2),
                    button::Status::Pressed => Color::from_rgba(color.r, color.g, color.b, 0.3),
                    _ => surface_bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: text_light,
                    border: iced::Border {
                        radius: border_radius.into(),
                        width: 1.0,
                        color: Color::from_rgba(color.r, color.g, color.b, 0.4),
                    },
                    ..Default::default()
                }
            })
            .padding([4, 8])
            .into()
    }

    /// Create a category header element
    fn category_header<'a>(class: SeparationClass, is_expanded: bool, role_count: usize, vm: &ViewModel) -> Element<'a, RolePaletteMessage> {
        // Get colors from ontological palette
        let color = vm.colors.separation_class_color(&class);
        let text_light = vm.colors.text_light;
        let surface_bg = vm.colors.surface;
        let secondary_bg = vm.colors.secondary;
        let border_radius = vm.radius_sm;

        let name = Self::class_name(&class);
        let arrow = if is_expanded { "▼" } else { "▶" };

        let header_content = row![
            // Expansion arrow
            text(arrow)
                .size(vm.text_tiny)
                .font(icons::FONT_BODY),
            // Colored circle
            container(text(" ").size(vm.spacing_xs))
                .width(10)
                .height(10)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(color)),
                    border: iced::Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            // Category name
            text(name)
                .size(vm.text_small)
                .font(icons::FONT_BODY)
                .color(color),
            // Role count badge
            container(
                text(format!("{}", role_count))
                    .size(vm.text_tiny)
                    .font(icons::FONT_BODY)
                    .color(text_light)
            )
            .padding([1, 4])
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(secondary_bg)),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ]
        .spacing(vm.spacing_sm)
        .align_y(iced::Alignment::Center);

        let surface_hover = vm.colors.surface_hover;
        let surface_pressed = vm.colors.surface_pressed;

        button(header_content)
            .on_press(RolePaletteMessage::ToggleCategory(class))
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered => surface_hover,
                    button::Status::Pressed => surface_pressed,
                    _ => surface_bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: text_light,
                    border: iced::Border {
                        radius: border_radius.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding([6, 8])
            .width(Length::Fill)
            .into()
    }

    /// Render the role palette as a widget
    pub fn view<'a>(&self, policy_data: Option<&'a PolicyBootstrapData>, vm: &ViewModel) -> Element<'a, RolePaletteMessage> {
        // Get colors from ontological palette
        let text_disabled = vm.colors.text_disabled;
        let text_primary = vm.colors.text_primary;
        let text_tertiary = vm.colors.text_tertiary;
        let panel_bg = vm.colors.panel_background;
        let modal_bg = vm.colors.modal_background;
        let border_subtle = vm.colors.border_subtle;
        let border_radius = vm.radius_sm;
        let border_radius_lg = vm.radius_md;

        let Some(policy) = policy_data else {
            return container(
                text("No policy data loaded")
                    .size(vm.text_small)
                    .font(icons::FONT_BODY)
                    .color(text_disabled)
            )
            .padding(vm.padding_md)
            .into();
        };

        if self.collapsed {
            // Collapsed state - just show a small expand button
            return container(
                button(text("▶ Roles").size(vm.text_tiny).font(icons::FONT_BODY))
                    .on_press(RolePaletteMessage::ToggleCategory(SeparationClass::Operational)) // Any will do, handled externally
                    .style(CowboyCustomTheme::glass_button())
                    .padding([4, 8])
            )
            .padding(vm.padding_xs)
            .into();
        }

        // Group roles by separation class
        let groups = Self::group_roles_by_class(policy);

        // Define the order of categories
        let category_order = [
            SeparationClass::Operational,
            SeparationClass::Administrative,
            SeparationClass::Financial,
            SeparationClass::Audit,
            SeparationClass::Personnel,
            SeparationClass::Emergency,
        ];

        let mut content = Column::new()
            .spacing(vm.padding_xs)
            .width(Length::Fill);

        // Title
        content = content.push(
            container(
                row![
                    text("Role Palette")
                        .size(vm.text_normal)
                        .font(icons::FONT_HEADING)
                        .color(text_primary),
                    text(format!("  ({})", policy.standard_roles.len()))
                        .size(vm.text_tiny)
                        .font(icons::FONT_BODY)
                        .color(text_tertiary),
                ]
                .align_y(iced::Alignment::Center)
            )
            .padding([8, 10])
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(panel_bg)),
                border: iced::Border {
                    radius: border_radius.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
        );

        // Hint text
        content = content.push(
            container(
                text("Drag roles onto people to assign")
                    .size(vm.text_tiny)
                    .font(icons::FONT_BODY)
                    .color(text_disabled)
            )
            .padding([4, 10])
        );

        // Categories
        for class in category_order.iter() {
            let roles = groups.get(class).cloned().unwrap_or_default();
            if roles.is_empty() {
                continue;
            }

            let is_expanded = self.is_expanded(class);

            // Category header
            content = content.push(Self::category_header(*class, is_expanded, roles.len(), vm));

            // Role badges (if expanded)
            if is_expanded {
                let mut role_column = Column::new()
                    .spacing(vm.spacing_xs)
                    .padding(Padding::default().left(16)); // Left indent

                for role in roles {
                    role_column = role_column.push(Self::role_badge(role, *class, vm));
                }

                content = content.push(role_column);
            }
        }

        // Wrap in scrollable container
        let scrollable_content = scrollable(content)
            .height(Length::Fill)
            .width(Length::Fixed(220.0));

        container(scrollable_content)
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(modal_bg)),
                border: iced::Border {
                    radius: border_radius_lg.into(),
                    width: 1.0,
                    color: border_subtle,
                },
                ..Default::default()
            })
            .padding(vm.padding_xs)
            .into()
    }
}

/// Convert RolePaletteMessage to a DragSource for the graph
impl From<RolePaletteMessage> for Option<DragSource> {
    fn from(msg: RolePaletteMessage) -> Self {
        match msg {
            RolePaletteMessage::StartDrag { role_name, separation_class } => {
                Some(DragSource::RoleFromPalette { role_name, separation_class })
            }
            _ => None,
        }
    }
}
