//! Edge creation indicator for visual feedback
//!
//! Shows a dashed line from source node to cursor during edge creation.

use iced::Point;
use iced::widget::canvas;
use iced::Color;
use uuid::Uuid;

use super::graph::OrganizationConcept;

/// Edge creation indicator state
#[derive(Debug, Clone)]
pub struct EdgeCreationIndicator {
    from_node: Option<Uuid>,
    current_position: Point,
    active: bool,
}

impl Default for EdgeCreationIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeCreationIndicator {
    /// Create a new edge creation indicator
    pub fn new() -> Self {
        Self {
            from_node: None,
            current_position: Point::ORIGIN,
            active: false,
        }
    }

    /// Start edge creation from a node
    pub fn start(&mut self, from_node: Uuid, position: Point) {
        self.from_node = Some(from_node);
        self.current_position = position;
        self.active = true;
    }

    /// Update the current cursor position
    pub fn update_position(&mut self, position: Point) {
        if self.active {
            self.current_position = position;
        }
    }

    /// Complete edge creation
    pub fn complete(&mut self) {
        self.active = false;
        self.from_node = None;
        self.current_position = Point::ORIGIN;
    }

    /// Cancel edge creation
    pub fn cancel(&mut self) {
        self.active = false;
        self.from_node = None;
        self.current_position = Point::ORIGIN;
    }

    /// Check if edge creation is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the source node ID
    pub fn from_node(&self) -> Option<Uuid> {
        self.from_node
    }

    /// Draw the edge creation indicator on the canvas
    pub fn draw(&self, frame: &mut canvas::Frame, graph: &OrganizationConcept) {
        if !self.active || self.from_node.is_none() {
            return;
        }

        let from_id = self.from_node.unwrap();
        if let Some(from_view) = graph.node_views.get(&from_id) {
            // Draw dashed line from source node to cursor
            let path = canvas::Path::line(from_view.position, self.current_position);

            let stroke = canvas::Stroke {
                style: canvas::stroke::Style::Solid(Color::from_rgb(0.5, 0.5, 1.0)),
                width: 2.0,
                line_cap: canvas::LineCap::Round,
                line_join: canvas::LineJoin::Round,
                line_dash: canvas::LineDash {
                    segments: &[10.0, 5.0],
                    offset: 0,
                },
            };

            frame.stroke(&path, stroke);

            // Draw arrow at cursor position to show direction
            let dx = self.current_position.x - from_view.position.x;
            let dy = self.current_position.y - from_view.position.y;
            let angle = dy.atan2(dx);

            let arrow_size = 12.0;
            let arrow_point1 = Point::new(
                self.current_position.x - arrow_size * (angle - 0.5).cos(),
                self.current_position.y - arrow_size * (angle - 0.5).sin(),
            );
            let arrow_point2 = Point::new(
                self.current_position.x - arrow_size * (angle + 0.5).cos(),
                self.current_position.y - arrow_size * (angle + 0.5).sin(),
            );

            let arrow = canvas::Path::new(|builder| {
                builder.move_to(self.current_position);
                builder.line_to(arrow_point1);
                builder.move_to(self.current_position);
                builder.line_to(arrow_point2);
            });

            let arrow_stroke = canvas::Stroke::default()
                .with_color(Color::from_rgb(0.5, 0.5, 1.0))
                .with_width(2.0);

            frame.stroke(&arrow, arrow_stroke);

            // Draw instruction text near cursor
            let text_position = Point::new(
                self.current_position.x + 15.0,
                self.current_position.y - 10.0,
            );

            frame.fill_text(canvas::Text {
                content: "Click target node".to_string(),
                position: text_position,
                color: Color::from_rgb(0.5, 0.5, 1.0),
                size: iced::Pixels(12.0),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Top,
                line_height: iced::widget::text::LineHeight::default(),
                shaping: iced::widget::text::Shaping::Advanced,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_indicator_creation() {
        let indicator = EdgeCreationIndicator::new();
        assert!(!indicator.is_active());
        assert!(indicator.from_node().is_none());
    }

    #[test]
    fn test_edge_indicator_start() {
        let mut indicator = EdgeCreationIndicator::new();
        let node_id = Uuid::new_v4();
        let position = Point::new(100.0, 200.0);

        indicator.start(node_id, position);

        assert!(indicator.is_active());
        assert_eq!(indicator.from_node(), Some(node_id));
    }

    #[test]
    fn test_edge_indicator_update_position() {
        let mut indicator = EdgeCreationIndicator::new();
        let node_id = Uuid::new_v4();

        indicator.start(node_id, Point::new(100.0, 200.0));
        indicator.update_position(Point::new(150.0, 250.0));

        assert!(indicator.is_active());
        assert_eq!(indicator.from_node(), Some(node_id));
    }

    #[test]
    fn test_edge_indicator_complete() {
        let mut indicator = EdgeCreationIndicator::new();
        let node_id = Uuid::new_v4();

        indicator.start(node_id, Point::new(100.0, 200.0));
        assert!(indicator.is_active());

        indicator.complete();
        assert!(!indicator.is_active());
        assert!(indicator.from_node().is_none());
    }

    #[test]
    fn test_edge_indicator_cancel() {
        let mut indicator = EdgeCreationIndicator::new();
        let node_id = Uuid::new_v4();

        indicator.start(node_id, Point::new(100.0, 200.0));
        assert!(indicator.is_active());

        indicator.cancel();
        assert!(!indicator.is_active());
        assert!(indicator.from_node().is_none());
    }
}
