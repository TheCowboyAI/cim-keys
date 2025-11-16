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
        }
    }

    /// Show the context menu at the given position
    pub fn show(&mut self, position: Point) {
        self.position = position;
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

        let menu_items: Column<'_, ContextMenuMessage> = column![
            text("Create Node").size(14),
            button(text("Organization").size(12))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Organization))
                .width(Length::Fill),
            button(text("Organizational Unit").size(12))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::OrganizationalUnit))
                .width(Length::Fill),
            button(text("Person").size(12))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Person))
                .width(Length::Fill),
            button(text("Location").size(12))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Location))
                .width(Length::Fill),
            button(text("Role").size(12))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Role))
                .width(Length::Fill),
            button(text("Policy").size(12))
                .on_press(ContextMenuMessage::CreateNode(NodeCreationType::Policy))
                .width(Length::Fill),
            text("").size(4), // Separator
            text("Other Actions").size(14),
            button(text("Create Edge").size(12))
                .on_press(ContextMenuMessage::CreateEdge)
                .width(Length::Fill),
            button(text("Cancel").size(12))
                .on_press(ContextMenuMessage::Dismiss)
                .width(Length::Fill),
        ]
        .spacing(2)
        .padding(8);

        let menu_container = container(menu_items)
            .width(Length::Fixed(180.0))
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
