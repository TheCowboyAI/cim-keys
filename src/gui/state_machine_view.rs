// Copyright (c) 2025 - Cowboy AI, LLC.

//! State Machine View Component
//!
//! Displays a Mealy state machine as an interactive graph visualization
//! alongside state and transition details.
//!
//! ## Layout
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │  [State Machine Selector Dropdown]                              │
//! ├──────────────────────────────────┬─────────────────────────────┤
//! │                                  │  State Details               │
//! │     ┌───┐         ┌───┐         │  ─────────────                │
//! │     │ A │ ──────► │ B │         │  Name: Active                 │
//! │     └───┘         └───┘         │  Description: ...             │
//! │       │             │            │  Type: Initial ○ Terminal ○   │
//! │       │             ▼            │                               │
//! │       │           ┌───┐         │  Transitions                  │
//! │       └──────────►│ C │         │  ─────────────                │
//! │                   └───┘         │  • A → B (EventX)            │
//! │                                  │  • B → C (EventY)            │
//! │      [State Machine Graph]       │  • A → C (EventZ)            │
//! └──────────────────────────────────┴─────────────────────────────┘
//! ```

use iced::{
    widget::{canvas, column, container, row, scrollable, pick_list, text, horizontal_space, vertical_space, Canvas},
    Color, Element, Length, Point, Rectangle, mouse, Renderer, Theme,
};
use std::f32::consts::PI;

use super::view_model::ViewModel;
use super::state_machine_graph::{
    StateMachineDefinition, StateMachineType, StateMachineState, StateMachineTransition,
    get_state_machine,
};
use super::cowboy_theme::CowboyAppTheme as CowboyCustomTheme;
use crate::icons::FONT_BODY;

/// Messages from the state machine view
#[derive(Debug, Clone)]
pub enum StateMachineViewMessage {
    /// User selected a different state machine type
    SelectMachine(StateMachineType),
    /// User clicked on a state in the graph
    StateClicked(String),
    /// User clicked on a transition
    TransitionClicked { from: String, to: String },
    /// Canvas interaction
    CanvasEvent(CanvasEvent),
}

/// Canvas-level events
#[derive(Debug, Clone)]
pub enum CanvasEvent {
    MouseMoved(Point),
    MousePressed(Point),
    MouseReleased,
}

/// State machine view component
pub struct StateMachineView {
    /// Currently selected state machine
    pub selected_machine: StateMachineType,
    /// The state machine definition
    pub definition: StateMachineDefinition,
    /// Selected state (if any)
    pub selected_state: Option<String>,
    /// Hovered state (for highlighting)
    pub hovered_state: Option<String>,
    /// Graph cache
    cache: canvas::Cache,
}

impl Default for StateMachineView {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachineView {
    pub fn new() -> Self {
        let selected_machine = StateMachineType::Key;
        let definition = get_state_machine(selected_machine);

        Self {
            selected_machine,
            definition,
            selected_state: None,
            hovered_state: None,
            cache: canvas::Cache::default(),
        }
    }

    /// Update the view based on a message
    pub fn update(&mut self, message: StateMachineViewMessage) {
        match message {
            StateMachineViewMessage::SelectMachine(machine_type) => {
                self.selected_machine = machine_type;
                self.definition = get_state_machine(machine_type);
                self.selected_state = None;
                self.cache.clear();
            }
            StateMachineViewMessage::StateClicked(state_name) => {
                self.selected_state = Some(state_name);
            }
            StateMachineViewMessage::TransitionClicked { from, to } => {
                // Could show transition details
                let _ = (from, to);
            }
            StateMachineViewMessage::CanvasEvent(_event) => {
                // Handle canvas interactions
            }
        }
    }

    /// Render the complete state machine view
    pub fn view<'a>(&'a self, vm: &'a ViewModel) -> Element<'a, StateMachineViewMessage> {
        let machine_types: Vec<StateMachineType> = StateMachineType::all();

        // Header with machine selector
        let selector = pick_list(
            machine_types,
            Some(self.selected_machine),
            StateMachineViewMessage::SelectMachine,
        )
        .width(Length::Fixed(250.0))
        .text_size(vm.text_normal);

        let header = row![
            text("State Machine:").size(vm.text_normal).font(FONT_BODY),
            horizontal_space().width(vm.spacing_md),
            selector,
            horizontal_space(),
            text(self.definition.machine_type.category())
                .size(vm.text_small)
                .font(FONT_BODY),
        ]
        .spacing(vm.spacing_md)
        .padding(vm.padding_md);

        // Main content: graph on left, details on right
        let graph_canvas = Canvas::new(StateMachineCanvas {
            definition: &self.definition,
            selected_state: self.selected_state.as_deref(),
            hovered_state: self.hovered_state.as_deref(),
            vm,
        })
        .width(Length::FillPortion(3))
        .height(Length::Fill);

        let details_panel = self.render_details_panel(vm);

        let content = row![
            container(graph_canvas)
                .style(CowboyCustomTheme::glass_container())
                .padding(vm.padding_sm),
            container(details_panel)
                .width(Length::FillPortion(2))
                .style(CowboyCustomTheme::glass_container())
                .padding(vm.padding_md),
        ]
        .spacing(vm.spacing_md);

        column![
            header,
            content,
        ]
        .spacing(vm.spacing_md)
        .padding(vm.padding_md)
        .into()
    }

    /// Render the details panel (states list and transitions)
    fn render_details_panel<'a>(&self, vm: &'a ViewModel) -> Element<'a, StateMachineViewMessage> {
        let title = text("States & Transitions")
            .size(vm.text_large)
            .font(FONT_BODY);

        // States list
        let states_header = text("States")
            .size(vm.text_medium)
            .font(FONT_BODY);

        let state_items: Vec<Element<'_, StateMachineViewMessage>> = self.definition.states
            .iter()
            .map(|state| {
                let is_selected = self.selected_state.as_ref() == Some(&state.name);
                let status = state_status_text(state);

                let state_text = text(format!("● {}", state.name))
                    .size(vm.text_small)
                    .font(FONT_BODY);

                let status_text = text(status)
                    .size(vm.text_tiny)
                    .font(FONT_BODY);

                let background = if is_selected {
                    Color::from_rgba(0.3, 0.5, 0.7, 0.3)
                } else {
                    Color::TRANSPARENT
                };

                container(
                    column![state_text, status_text]
                        .spacing(2)
                )
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(background)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .padding(vm.padding_sm)
                .into()
            })
            .collect();

        let states_list = column(state_items)
            .spacing(vm.spacing_xs);

        // Transitions list
        let transitions_header = text("Transitions")
            .size(vm.text_medium)
            .font(FONT_BODY);

        let transition_items: Vec<Element<'_, StateMachineViewMessage>> = self.definition.transitions
            .iter()
            .map(|transition| {
                let arrow = if transition.from == transition.to {
                    format!("{} ↻ ({})", transition.from, transition.label)
                } else {
                    format!("{} → {} ({})", transition.from, transition.to, transition.label)
                };

                text(arrow)
                    .size(vm.text_tiny)
                    .font(FONT_BODY)
                    .into()
            })
            .collect();

        let transitions_list = column(transition_items)
            .spacing(vm.spacing_xs);

        // Legend
        let legend = self.render_legend(vm);

        scrollable(
            column![
                title,
                vertical_space().height(vm.spacing_md),
                states_header,
                states_list,
                vertical_space().height(vm.spacing_md),
                transitions_header,
                scrollable(transitions_list).height(Length::Fixed(200.0)),
                vertical_space().height(vm.spacing_md),
                legend,
            ]
            .spacing(vm.spacing_sm)
        )
        .into()
    }

    /// Render the legend
    fn render_legend<'a>(&self, vm: &'a ViewModel) -> Element<'a, StateMachineViewMessage> {
        let legend_items = vec![
            ("◐", "Initial State", Color::from_rgb(0.2, 0.7, 0.3)),
            ("◉", "Terminal State", Color::from_rgb(0.7, 0.2, 0.2)),
            ("●", "Regular State", Color::from_rgb(0.4, 0.5, 0.6)),
            ("◎", "Current State", Color::from_rgb(0.0, 0.6, 1.0)),
        ];

        let items: Vec<Element<'_, StateMachineViewMessage>> = legend_items
            .into_iter()
            .map(|(icon, label, _color)| {
                row![
                    text(icon).size(vm.text_small),
                    text(label).size(vm.text_tiny).font(FONT_BODY),
                ]
                .spacing(vm.spacing_xs)
                .into()
            })
            .collect();

        column![
            text("Legend").size(vm.text_small).font(FONT_BODY),
            column(items).spacing(vm.spacing_xs),
        ]
        .spacing(vm.spacing_xs)
        .into()
    }
}

/// Get status text for a state
fn state_status_text(state: &StateMachineState) -> String {
    let mut parts = Vec::new();
    if state.is_initial {
        parts.push("initial");
    }
    if state.is_terminal {
        parts.push("terminal");
    }
    if parts.is_empty() {
        state.description.clone()
    } else {
        format!("[{}] {}", parts.join(", "), state.description)
    }
}

// ============================================================================
// Canvas Rendering for State Machine Graph
// ============================================================================

/// Canvas program for rendering the state machine graph
struct StateMachineCanvas<'a> {
    definition: &'a StateMachineDefinition,
    selected_state: Option<&'a str>,
    hovered_state: Option<&'a str>,
    vm: &'a ViewModel,
}

impl<'a> canvas::Program<StateMachineViewMessage> for StateMachineCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Layout configuration
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let layout_radius = (bounds.width.min(bounds.height) / 2.0 - 60.0).max(100.0);
        let node_radius = (25.0 * self.vm.scale).clamp(20.0, 40.0);

        // Calculate positions
        let positions = calculate_positions(&self.definition.states, center, layout_radius);

        // Draw title
        let title = canvas::Text {
            content: self.definition.machine_type.display_name().to_string(),
            position: Point::new(center.x, 20.0),
            color: Color::from_rgb(0.9, 0.9, 0.9),
            size: iced::Pixels(self.vm.text_medium as f32),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Top,
            ..Default::default()
        };
        frame.fill_text(title);

        // Draw transitions first (behind states)
        for transition in &self.definition.transitions {
            if let (Some(&from_pos), Some(&to_pos)) = (
                positions.get(&transition.from),
                positions.get(&transition.to),
            ) {
                draw_transition(
                    &mut frame,
                    transition,
                    from_pos,
                    to_pos,
                    node_radius,
                    self.vm,
                );
            }
        }

        // Draw states
        for state in &self.definition.states {
            if let Some(&pos) = positions.get(&state.name) {
                let is_selected = self.selected_state == Some(&state.name);
                let is_hovered = self.hovered_state == Some(&state.name);
                let is_current = self.definition.current_state.as_ref() == Some(&state.name);

                draw_state(
                    &mut frame,
                    state,
                    pos,
                    node_radius,
                    is_selected,
                    is_hovered,
                    is_current,
                    self.vm,
                );
            }
        }

        vec![frame.into_geometry()]
    }
}

/// Calculate circular layout positions for states
fn calculate_positions(
    states: &[StateMachineState],
    center: Point,
    radius: f32,
) -> std::collections::HashMap<String, Point> {
    let mut positions = std::collections::HashMap::new();
    let count = states.len();

    if count == 0 {
        return positions;
    }

    let angle_step = 2.0 * PI / count as f32;

    for (i, state) in states.iter().enumerate() {
        // Start from top (-PI/2)
        let angle = angle_step * i as f32 - PI / 2.0;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        positions.insert(state.name.clone(), Point::new(x, y));
    }

    positions
}

/// Draw a single state node
fn draw_state(
    frame: &mut canvas::Frame,
    state: &StateMachineState,
    pos: Point,
    radius: f32,
    is_selected: bool,
    is_hovered: bool,
    is_current: bool,
    vm: &ViewModel,
) {
    // Determine colors
    let fill_color = if is_current {
        Color::from_rgb(0.0, 0.5, 0.9)
    } else if is_selected {
        Color::from_rgb(0.3, 0.6, 0.9)
    } else if is_hovered {
        Color::from_rgb(0.4, 0.5, 0.6)
    } else if state.is_terminal {
        Color::from_rgb(0.6, 0.2, 0.2)
    } else if state.is_initial {
        Color::from_rgb(0.2, 0.5, 0.3)
    } else {
        Color::from_rgb(0.25, 0.35, 0.45)
    };

    let stroke_color = if is_current {
        Color::from_rgb(0.0, 0.7, 1.0)
    } else {
        Color::from_rgb(0.4, 0.6, 0.8)
    };

    let stroke_width = if is_current || is_selected {
        vm.border_thick
    } else {
        vm.border_normal
    };

    // Initial state: outer circle
    if state.is_initial && !is_current {
        let outer_circle = canvas::Path::circle(pos, radius + 5.0);
        frame.stroke(
            &outer_circle,
            canvas::Stroke::default()
                .with_color(stroke_color)
                .with_width(vm.border_thin),
        );
    }

    // Main circle
    let circle = canvas::Path::circle(pos, radius);
    frame.fill(&circle, fill_color);
    frame.stroke(
        &circle,
        canvas::Stroke::default()
            .with_color(stroke_color)
            .with_width(stroke_width),
    );

    // Terminal state: inner circle
    if state.is_terminal {
        let inner_circle = canvas::Path::circle(pos, radius - 6.0);
        frame.stroke(
            &inner_circle,
            canvas::Stroke::default()
                .with_color(stroke_color)
                .with_width(vm.border_thin),
        );
    }

    // State name
    let display_name = if state.name.len() > 12 {
        format!("{}…", &state.name[..11])
    } else {
        state.name.clone()
    };

    let text_content = canvas::Text {
        content: display_name,
        position: pos,
        color: Color::from_rgb(0.95, 0.95, 0.95),
        size: iced::Pixels(vm.text_tiny as f32),
        horizontal_alignment: iced::alignment::Horizontal::Center,
        vertical_alignment: iced::alignment::Vertical::Center,
        ..Default::default()
    };
    frame.fill_text(text_content);
}

/// Draw a transition arrow
fn draw_transition(
    frame: &mut canvas::Frame,
    transition: &StateMachineTransition,
    from: Point,
    to: Point,
    node_radius: f32,
    vm: &ViewModel,
) {
    let stroke_color = if transition.is_active {
        Color::from_rgb(0.0, 0.8, 0.4)
    } else {
        Color::from_rgb(0.5, 0.6, 0.7)
    };

    let is_self_loop = transition.from == transition.to;

    if is_self_loop {
        // Self-loop: draw arc above the state
        draw_self_loop(frame, from, node_radius, &transition.label, stroke_color, vm);
    } else {
        // Regular transition: curved arrow
        draw_curved_arrow(frame, from, to, node_radius, &transition.label, stroke_color, vm);
    }
}

/// Draw a self-loop (transition to same state)
fn draw_self_loop(
    frame: &mut canvas::Frame,
    pos: Point,
    radius: f32,
    label: &str,
    color: Color,
    vm: &ViewModel,
) {
    let loop_radius = 20.0;
    let loop_center = Point::new(pos.x, pos.y - radius - loop_radius);

    // Draw the loop arc
    let path = canvas::Path::new(|builder| {
        builder.arc(canvas::path::Arc {
            center: loop_center,
            radius: loop_radius,
            start_angle: iced::Radians(0.5 * PI),
            end_angle: iced::Radians(2.5 * PI),
        });
    });

    frame.stroke(
        &path,
        canvas::Stroke::default()
            .with_color(color)
            .with_width(vm.border_normal),
    );

    // Draw arrowhead at end of loop
    let arrow_pos = Point::new(pos.x + 8.0, pos.y - radius - 3.0);
    draw_arrowhead(frame, arrow_pos, PI / 4.0, color, vm);

    // Label
    let label_pos = Point::new(pos.x, pos.y - radius - loop_radius * 2.0 - 8.0);
    let text_content = canvas::Text {
        content: truncate_label(label, 15),
        position: label_pos,
        color,
        size: iced::Pixels(vm.text_tiny as f32),
        horizontal_alignment: iced::alignment::Horizontal::Center,
        vertical_alignment: iced::alignment::Vertical::Bottom,
        ..Default::default()
    };
    frame.fill_text(text_content);
}

/// Draw a curved arrow between two points
fn draw_curved_arrow(
    frame: &mut canvas::Frame,
    from: Point,
    to: Point,
    node_radius: f32,
    label: &str,
    color: Color,
    vm: &ViewModel,
) {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 0.001 {
        return;
    }

    // Unit vector
    let ux = dx / dist;
    let uy = dy / dist;

    // Perpendicular for curve
    let px = -uy;
    let py = ux;
    let curve_offset = 20.0;

    // Start and end points on circle edges
    let start = Point::new(
        from.x + ux * node_radius,
        from.y + uy * node_radius,
    );
    let end = Point::new(
        to.x - ux * (node_radius + 8.0),
        to.y - uy * (node_radius + 8.0),
    );

    // Control point for quadratic bezier
    let mid = Point::new(
        (from.x + to.x) / 2.0 + px * curve_offset,
        (from.y + to.y) / 2.0 + py * curve_offset,
    );

    // Draw the curve
    let path = canvas::Path::new(|builder| {
        builder.move_to(start);
        builder.quadratic_curve_to(mid, end);
    });

    frame.stroke(
        &path,
        canvas::Stroke::default()
            .with_color(color)
            .with_width(vm.border_normal),
    );

    // Calculate arrow direction at end of curve
    // Derivative of quadratic bezier at t=1: 2*(P2-P1) where P1=mid, P2=end
    let arrow_dx = end.x - mid.x;
    let arrow_dy = end.y - mid.y;
    let arrow_angle = arrow_dy.atan2(arrow_dx);

    draw_arrowhead(frame, end, arrow_angle, color, vm);

    // Label at curve midpoint
    let label_pos = Point::new(mid.x, mid.y - 8.0);
    let text_content = canvas::Text {
        content: truncate_label(label, 18),
        position: label_pos,
        color,
        size: iced::Pixels(vm.text_tiny as f32),
        horizontal_alignment: iced::alignment::Horizontal::Center,
        vertical_alignment: iced::alignment::Vertical::Bottom,
        ..Default::default()
    };
    frame.fill_text(text_content);
}

/// Draw an arrowhead
fn draw_arrowhead(
    frame: &mut canvas::Frame,
    tip: Point,
    angle: f32,
    color: Color,
    vm: &ViewModel,
) {
    let size = 8.0 * vm.scale;
    let half_angle = PI / 6.0; // 30 degrees

    let left_angle = angle + PI - half_angle;
    let right_angle = angle + PI + half_angle;

    let left = Point::new(
        tip.x + size * left_angle.cos(),
        tip.y + size * left_angle.sin(),
    );
    let right = Point::new(
        tip.x + size * right_angle.cos(),
        tip.y + size * right_angle.sin(),
    );

    let path = canvas::Path::new(|builder| {
        builder.move_to(tip);
        builder.line_to(left);
        builder.line_to(right);
        builder.close();
    });

    frame.fill(&path, color);
}

/// Truncate a label if too long
fn truncate_label(label: &str, max_len: usize) -> String {
    if label.len() <= max_len {
        label.to_string()
    } else {
        format!("{}…", &label[..max_len - 1])
    }
}

// ============================================================================
// Display impl for StateMachineType (needed for pick_list)
// ============================================================================

impl std::fmt::Display for StateMachineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_view_creation() {
        let view = StateMachineView::new();
        assert_eq!(view.selected_machine, StateMachineType::Key);
        assert!(view.selected_state.is_none());
    }

    #[test]
    fn test_select_machine_updates_definition() {
        let mut view = StateMachineView::new();

        view.update(StateMachineViewMessage::SelectMachine(StateMachineType::Certificate));

        assert_eq!(view.selected_machine, StateMachineType::Certificate);
        assert_eq!(view.definition.machine_type, StateMachineType::Certificate);
    }

    #[test]
    fn test_state_click_updates_selection() {
        let mut view = StateMachineView::new();

        view.update(StateMachineViewMessage::StateClicked("Active".to_string()));

        assert_eq!(view.selected_state, Some("Active".to_string()));
    }

    #[test]
    fn test_calculate_positions_circular() {
        let states = vec![
            StateMachineState::new("A", "State A"),
            StateMachineState::new("B", "State B"),
            StateMachineState::new("C", "State C"),
            StateMachineState::new("D", "State D"),
        ];
        let center = Point::new(100.0, 100.0);
        let radius = 50.0;

        let positions = calculate_positions(&states, center, radius);

        assert_eq!(positions.len(), 4);

        // First state should be at top (center.y - radius)
        let pos_a = positions.get("A").unwrap();
        assert!((pos_a.y - (center.y - radius)).abs() < 1.0);
    }

    #[test]
    fn test_truncate_label() {
        assert_eq!(truncate_label("Short", 10), "Short");
        assert_eq!(truncate_label("VeryLongLabel", 10), "VeryLongL…");
    }

    #[test]
    fn test_state_status_text() {
        let initial = StateMachineState::new("Test", "Description").initial();
        let terminal = StateMachineState::new("End", "Final").terminal();
        let normal = StateMachineState::new("Mid", "Middle");

        assert!(state_status_text(&initial).contains("initial"));
        assert!(state_status_text(&terminal).contains("terminal"));
        assert_eq!(state_status_text(&normal), "Middle");
    }
}
