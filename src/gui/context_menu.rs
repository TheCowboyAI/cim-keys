//! Context menu for graph interactions
//!
//! Provides a right-click context menu for creating nodes and edges in the graph.

use iced::{
    widget::{button, column, container, text, Column},
    Element, Length, Point, Theme,
};

use crate::mvi::intent::NodeCreationType;
use super::view_model::ViewModel;

/// Context menu for graph operations
#[derive(Debug, Clone)]
pub struct ContextMenu {
    position: Point,
    visible: bool,
    ui_scale: f32,
}

/// Messages emitted by the context menu
#[derive(Debug, Clone)]
pub enum ContextMenuMessage {
    /// User selected to create a node
    CreateNode(NodeCreationType),
    /// User selected to create an edge
    CreateEdge,
    /// User clicked outside the menu (dismiss)
    Dismiss,
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenu {
    /// Create a new context menu
    pub fn new() -> Self {
        Self {
            position: Point::ORIGIN,
            visible: false,
            ui_scale: 1.0,  // Will be updated when shown
        }
    }

    /// Show the context menu at the given position with UI scale
    pub fn show(&mut self, position: Point, ui_scale: f32) {
        self.position = position;
        self.ui_scale = ui_scale;
        self.visible = true;
    }

    /// Hide the context menu
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the context menu is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the position of the context menu
    pub fn position(&self) -> Point {
        self.position
    }

    /// Render the context menu positioned at the stored location
    pub fn view(&self, vm: &ViewModel) -> Element<'_, ContextMenuMessage> {
        if !self.visible {
            return container(column![]).into();
        }

        // Use ViewModel for consistent sizing across the UI
        let header_size = vm.text_normal;
        let item_size = vm.text_small;

        let menu_items: Column<'_, ContextMenuMessage> = column![
            text("Create Node").size(header_size),
            button(text("Organization").size(item_size))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Organization))
                .width(Length::Fill),
            button(text("Organizational Unit").size(item_size))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::OrganizationalUnit))
                .width(Length::Fill),
            button(text("Person").size(item_size))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Person))
                .width(Length::Fill),
            button(text("Location").size(item_size))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Location))
                .width(Length::Fill),
            button(text("Role").size(item_size))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Role))
                .width(Length::Fill),
            button(text("Policy").size(item_size))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Policy))
                .width(Length::Fill),
            text("").size(vm.spacing_xs), // Separator
            text("Other Actions").size(header_size),
            button(text("Create Edge").size(item_size))
                .on_press(ContextMenuMessage::CreateEdge)
                .width(Length::Fill),
            button(text("Cancel").size(item_size))
                .on_press(ContextMenuMessage::Dismiss)
                .width(Length::Fill),
        ]
        .spacing(vm.spacing_xs)
        .padding(vm.padding_sm);

        // Menu styling with ontological color mapping
        let menu_width = 180.0 * vm.scale;
        let shadow_color = vm.colors.shadow_default;
        let border_width = vm.border_thin;
        let border_radius = vm.radius_sm;
        let shadow_blur = vm.shadow_sm;

        let menu_container = container(menu_items)
            .width(Length::Fixed(menu_width))
            .style(move |theme: &Theme| {
                container::Style {
                    background: Some(iced::Background::Color(theme.palette().background)),
                    border: iced::Border {
                        color: theme.palette().text,
                        width: border_width,
                        radius: border_radius.into(),
                    },
                    text_color: Some(theme.palette().text),
                    shadow: iced::Shadow {
                        color: shadow_color,
                        offset: iced::Vector::new(2.0, 2.0),
                        blur_radius: shadow_blur,
                    },
                }
            });

        // Position is now handled by the parent using vertical_space/horizontal_space
        // Don't apply positioning here or it will be doubled
        menu_container.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_creation() {
        let menu = ContextMenu::new();
        assert!(!menu.is_visible());
        assert_eq!(menu.position(), Point::ORIGIN);
    }

    #[test]
    fn test_context_menu_show_hide() {
        let mut menu = ContextMenu::new();

        menu.show(Point::new(100.0, 200.0), 1.0);
        assert!(menu.is_visible());
        assert_eq!(menu.position(), Point::new(100.0, 200.0));

        menu.hide();
        assert!(!menu.is_visible());
    }
}
