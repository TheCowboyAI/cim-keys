//! Context menu for graph interactions
//!
//! Provides a right-click context menu for creating nodes and edges in the graph.

use iced::{
    widget::{button, column, container, text, Column},
    Element, Length, Point, Theme,
};

use crate::mvi::intent::NodeCreationType;

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
    pub fn view(&self) -> Element<'_, ContextMenuMessage> {
        if !self.visible {
            return container(column![]).into();
        }

        // Scale text sizes and spacing based on ui_scale
        let header_size = (14.0 * self.ui_scale) as u16;
        let item_size = (12.0 * self.ui_scale) as u16;
        let spacing = (2.0 * self.ui_scale) as u16;
        let padding = (8.0 * self.ui_scale) as u16;

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
            text("").size(4), // Separator
            text("Other Actions").size(header_size),
            button(text("Create Edge").size(item_size))
                .on_press(ContextMenuMessage::CreateEdge)
                .width(Length::Fill),
            button(text("Cancel").size(item_size))
                .on_press(ContextMenuMessage::Dismiss)
                .width(Length::Fill),
        ]
        .spacing(spacing)
        .padding(padding);

        let menu_width = 180.0 * self.ui_scale;
        let menu_container = container(menu_items)
            .width(Length::Fixed(menu_width))
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(iced::Background::Color(theme.palette().background)),
                    border: iced::Border {
                        color: theme.palette().text,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    text_color: Some(theme.palette().text),
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                        offset: iced::Vector::new(2.0, 2.0),
                        blur_radius: 4.0,
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

        menu.show(Point::new(100.0, 200.0));
        assert!(menu.is_visible());
        assert_eq!(menu.position(), Point::new(100.0, 200.0));

        menu.hide();
        assert!(!menu.is_visible());
    }
}
