// Copyright (c) 2025 - Cowboy AI, LLC.

//! State Machine SVG Generator
//!
//! Generates SVG visualizations of Mealy state machines.
//!
//! ## Mealy Machine Representation
//!
//! A Mealy machine is a 6-tuple (S, S₀, Σ, Λ, T, G) where:
//! - S: Finite set of states (circles in SVG)
//! - S₀: Initial state (double circle)
//! - Σ: Input alphabet (transition labels)
//! - Λ: Output alphabet (transition outputs - currently events)
//! - T: Transition function S × Σ → S (arrows)
//! - G: Output function S × Σ → Λ (arrow labels)
//!
//! ## SVG Structure
//!
//! ```xml
//! <svg>
//!   <defs>
//!     <marker id="arrowhead">...</marker>
//!   </defs>
//!   <g class="states">
//!     <circle class="state" />
//!     <text class="state-label" />
//!   </g>
//!   <g class="transitions">
//!     <path class="transition" />
//!     <text class="transition-label" />
//!   </g>
//! </svg>
//! ```

use std::f32::consts::PI;
use iced::Color;

use super::state_machine_graph::{StateMachineDefinition, StateMachineState, StateMachineTransition};
use super::view_model::ViewModel;

/// Configuration for SVG generation derived from ViewModel
#[derive(Debug, Clone)]
pub struct SvgConfig {
    /// SVG width
    pub width: f32,
    /// SVG height
    pub height: f32,
    /// Center X
    pub center_x: f32,
    /// Center Y
    pub center_y: f32,
    /// Radius for circular layout
    pub layout_radius: f32,
    /// Node (state) radius
    pub node_radius: f32,
    /// Font size for state labels
    pub state_font_size: f32,
    /// Font size for transition labels
    pub transition_font_size: f32,
    /// Arrow marker size
    pub arrow_size: f32,
    /// Stroke width for edges
    pub stroke_width: f32,
    /// Color scheme
    pub colors: SvgColors,
}

/// Color scheme for SVG elements
#[derive(Debug, Clone)]
pub struct SvgColors {
    pub background: String,
    pub state_fill: String,
    pub state_stroke: String,
    pub initial_fill: String,
    pub terminal_fill: String,
    pub current_fill: String,
    pub transition_stroke: String,
    pub text_fill: String,
    pub active_transition: String,
}

impl SvgConfig {
    /// Create SVG config from ViewModel
    pub fn from_view_model(vm: &ViewModel, width: f32, height: f32) -> Self {
        let scale = vm.scale;

        Self {
            width,
            height,
            center_x: width / 2.0,
            center_y: height / 2.0,
            layout_radius: (width.min(height) / 2.0 - 80.0) * scale.min(1.5),
            node_radius: 35.0 * scale,
            state_font_size: vm.text_small as f32,
            transition_font_size: vm.text_tiny as f32,
            arrow_size: 8.0 * scale,
            stroke_width: vm.border_normal,
            colors: SvgColors::default(),
        }
    }
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            width: 600.0,
            height: 500.0,
            center_x: 300.0,
            center_y: 250.0,
            layout_radius: 180.0,
            node_radius: 35.0,
            state_font_size: 11.0,
            transition_font_size: 9.0,
            arrow_size: 8.0,
            stroke_width: 1.5,
            colors: SvgColors::default(),
        }
    }
}

impl Default for SvgColors {
    fn default() -> Self {
        Self {
            background: "transparent".to_string(),
            state_fill: "#2a3a4a".to_string(),
            state_stroke: "#4a6a8a".to_string(),
            initial_fill: "#2d5a3d".to_string(),
            terminal_fill: "#5a2d2d".to_string(),
            current_fill: "#0066cc".to_string(),
            transition_stroke: "#6a8aaa".to_string(),
            text_fill: "#e0e0e0".to_string(),
            active_transition: "#00cc66".to_string(),
        }
    }
}

/// Position for a state in the layout
#[derive(Debug, Clone, Copy)]
struct StatePosition {
    x: f32,
    y: f32,
}

/// Generate complete SVG for a state machine
pub fn generate_state_machine_svg(
    definition: &StateMachineDefinition,
    config: &SvgConfig,
) -> String {
    let mut svg = String::new();

    // Calculate positions for all states
    let positions = calculate_circular_layout(&definition.states, config);

    // SVG header
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
        config.width, config.height, config.width, config.height
    ));
    svg.push('\n');

    // Definitions (arrow markers)
    svg.push_str(&generate_defs(config));

    // Background
    svg.push_str(&format!(
        r#"  <rect width="100%" height="100%" fill="{}"/>"#,
        config.colors.background
    ));
    svg.push('\n');

    // Draw transitions first (so they're behind states)
    svg.push_str("  <g class=\"transitions\">\n");
    for transition in &definition.transitions {
        if let (Some(from_pos), Some(to_pos)) = (
            positions.get(&transition.from),
            positions.get(&transition.to),
        ) {
            svg.push_str(&generate_transition_svg(
                transition,
                *from_pos,
                *to_pos,
                config,
                &transition.from == &transition.to, // Self-loop
            ));
        }
    }
    svg.push_str("  </g>\n");

    // Draw states
    svg.push_str("  <g class=\"states\">\n");
    for state in &definition.states {
        if let Some(pos) = positions.get(&state.name) {
            let is_current = definition.current_state.as_ref() == Some(&state.name);
            svg.push_str(&generate_state_svg(state, *pos, config, is_current));
        }
    }
    svg.push_str("  </g>\n");

    // Title
    svg.push_str(&format!(
        r#"  <text x="{}" y="30" text-anchor="middle" fill="{}" font-size="16" font-weight="bold">{}</text>"#,
        config.center_x,
        config.colors.text_fill,
        definition.machine_type.display_name()
    ));
    svg.push('\n');

    // Legend
    svg.push_str(&generate_legend(config));

    svg.push_str("</svg>");

    svg
}

/// Calculate circular layout positions for states
fn calculate_circular_layout(
    states: &[StateMachineState],
    config: &SvgConfig,
) -> std::collections::HashMap<String, StatePosition> {
    let mut positions = std::collections::HashMap::new();
    let state_count = states.len();

    if state_count == 0 {
        return positions;
    }

    let angle_step = 2.0 * PI / state_count as f32;

    for (i, state) in states.iter().enumerate() {
        // Start from top (-PI/2) and go clockwise
        let angle = angle_step * i as f32 - PI / 2.0;
        let x = config.center_x + config.layout_radius * angle.cos();
        let y = config.center_y + config.layout_radius * angle.sin();

        positions.insert(state.name.clone(), StatePosition { x, y });
    }

    positions
}

/// Generate SVG defs (markers, gradients)
fn generate_defs(config: &SvgConfig) -> String {
    format!(
        r#"  <defs>
    <marker id="arrowhead" markerWidth="{size}" markerHeight="{size}"
            refX="{refx}" refY="{refy}" orient="auto" markerUnits="strokeWidth">
      <polygon points="0 0, {size} {half}, 0 {size}" fill="{color}"/>
    </marker>
    <marker id="arrowhead-active" markerWidth="{size}" markerHeight="{size}"
            refX="{refx}" refY="{refy}" orient="auto" markerUnits="strokeWidth">
      <polygon points="0 0, {size} {half}, 0 {size}" fill="{active}"/>
    </marker>
  </defs>
"#,
        size = config.arrow_size,
        half = config.arrow_size / 2.0,
        refx = config.arrow_size,
        refy = config.arrow_size / 2.0,
        color = config.colors.transition_stroke,
        active = config.colors.active_transition,
    )
}

/// Generate SVG for a single state
fn generate_state_svg(
    state: &StateMachineState,
    pos: StatePosition,
    config: &SvgConfig,
    is_current: bool,
) -> String {
    let fill = if is_current {
        &config.colors.current_fill
    } else if state.is_terminal {
        &config.colors.terminal_fill
    } else if state.is_initial {
        &config.colors.initial_fill
    } else {
        &config.colors.state_fill
    };

    let stroke = if is_current {
        "#00aaff"
    } else {
        &config.colors.state_stroke
    };

    let stroke_width = if is_current {
        config.stroke_width * 2.0
    } else {
        config.stroke_width
    };

    let mut svg = String::new();

    // Initial state: double circle
    if state.is_initial && !is_current {
        svg.push_str(&format!(
            r#"    <circle cx="{}" cy="{}" r="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
            pos.x, pos.y, config.node_radius + 5.0, stroke, config.stroke_width
        ));
        svg.push('\n');
    }

    // Main state circle
    svg.push_str(&format!(
        r#"    <circle cx="{}" cy="{}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        pos.x, pos.y, config.node_radius, fill, stroke, stroke_width
    ));
    svg.push('\n');

    // Terminal state: inner circle
    if state.is_terminal {
        svg.push_str(&format!(
            r#"    <circle cx="{}" cy="{}" r="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
            pos.x, pos.y, config.node_radius - 6.0, stroke, config.stroke_width
        ));
        svg.push('\n');
    }

    // State name (may need to wrap for long names)
    let display_name = truncate_name(&state.name, 12);
    svg.push_str(&format!(
        r#"    <text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" fill="{}" font-size="{}" font-family="sans-serif">{}</text>"#,
        pos.x, pos.y, config.colors.text_fill, config.state_font_size, display_name
    ));
    svg.push('\n');

    svg
}

/// Generate SVG for a transition
fn generate_transition_svg(
    transition: &StateMachineTransition,
    from: StatePosition,
    to: StatePosition,
    config: &SvgConfig,
    is_self_loop: bool,
) -> String {
    let mut svg = String::new();

    let stroke_color = if transition.is_active {
        &config.colors.active_transition
    } else {
        &config.colors.transition_stroke
    };

    let marker = if transition.is_active {
        "arrowhead-active"
    } else {
        "arrowhead"
    };

    if is_self_loop {
        // Self-loop: draw a loop above the state
        let loop_offset = config.node_radius + 30.0;
        svg.push_str(&format!(
            r#"    <path d="M {} {} C {} {} {} {} {} {}" fill="none" stroke="{}" stroke-width="{}" marker-end="url(#{})"/>"#,
            from.x - 10.0, from.y - config.node_radius, // Start left of top
            from.x - 40.0, from.y - loop_offset, // Control point left
            from.x + 40.0, from.y - loop_offset, // Control point right
            from.x + 10.0, from.y - config.node_radius, // End right of top
            stroke_color, config.stroke_width, marker
        ));
        svg.push('\n');

        // Label for self-loop
        svg.push_str(&format!(
            r#"    <text x="{}" y="{}" text-anchor="middle" fill="{}" font-size="{}">{}</text>"#,
            from.x, from.y - config.node_radius - 35.0,
            config.colors.text_fill, config.transition_font_size,
            truncate_name(&transition.label, 15)
        ));
        svg.push('\n');
    } else {
        // Regular transition: curved arrow
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Calculate start and end points on circle edges
        let unit_x = dx / dist;
        let unit_y = dy / dist;

        let start_x = from.x + unit_x * config.node_radius;
        let start_y = from.y + unit_y * config.node_radius;
        let end_x = to.x - unit_x * (config.node_radius + config.arrow_size);
        let end_y = to.y - unit_y * (config.node_radius + config.arrow_size);

        // Curve offset perpendicular to the line
        let perp_x = -unit_y;
        let perp_y = unit_x;
        let curve_offset = 25.0;

        let mid_x = (from.x + to.x) / 2.0 + perp_x * curve_offset;
        let mid_y = (from.y + to.y) / 2.0 + perp_y * curve_offset;

        svg.push_str(&format!(
            r#"    <path d="M {} {} Q {} {} {} {}" fill="none" stroke="{}" stroke-width="{}" marker-end="url(#{})"/>"#,
            start_x, start_y,
            mid_x, mid_y,
            end_x, end_y,
            stroke_color, config.stroke_width, marker
        ));
        svg.push('\n');

        // Label at curve midpoint
        let label_x = mid_x;
        let label_y = mid_y - 5.0;
        svg.push_str(&format!(
            r#"    <text x="{}" y="{}" text-anchor="middle" fill="{}" font-size="{}" font-style="italic">{}</text>"#,
            label_x, label_y,
            config.colors.text_fill, config.transition_font_size,
            truncate_name(&transition.label, 18)
        ));
        svg.push('\n');
    }

    svg
}

/// Generate legend for the SVG
fn generate_legend(config: &SvgConfig) -> String {
    let y_base = config.height - 60.0;
    let x_start = 20.0;
    let spacing = 110.0;

    format!(
        r#"  <g class="legend" transform="translate({}, {})">
    <circle cx="8" cy="0" r="6" fill="{}" stroke="{}"/>
    <text x="18" y="4" fill="{}" font-size="10">Initial</text>

    <circle cx="{}" cy="0" r="6" fill="{}" stroke="{}"/>
    <text x="{}" y="4" fill="{}" font-size="10">Active</text>

    <circle cx="{}" cy="0" r="6" fill="{}" stroke="{}"/>
    <circle cx="{}" cy="0" r="4" fill="none" stroke="{}"/>
    <text x="{}" y="4" fill="{}" font-size="10">Terminal</text>

    <circle cx="{}" cy="0" r="6" fill="{}" stroke="{}" stroke-width="2"/>
    <text x="{}" y="4" fill="{}" font-size="10">Current</text>
  </g>
"#,
        x_start, y_base,
        config.colors.initial_fill, config.colors.state_stroke,
        config.colors.text_fill,
        spacing, config.colors.state_fill, config.colors.state_stroke,
        spacing + 10.0, config.colors.text_fill,
        spacing * 2.0, config.colors.terminal_fill, config.colors.state_stroke,
        spacing * 2.0, config.colors.state_stroke,
        spacing * 2.0 + 10.0, config.colors.text_fill,
        spacing * 3.0, config.colors.current_fill, "#00aaff",
        spacing * 3.0 + 10.0, config.colors.text_fill,
    )
}

/// Truncate a name if too long
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}…", &name[..max_len - 1])
    }
}

/// Convert iced Color to SVG hex color string
#[allow(dead_code)]
pub fn color_to_hex(color: Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::state_machine_graph::{
        build_key_state_machine, build_certificate_state_machine,
        build_person_state_machine, StateMachineType,
    };

    #[test]
    fn test_generate_key_state_machine_svg() {
        let definition = build_key_state_machine();
        let config = SvgConfig::default();

        let svg = generate_state_machine_svg(&definition, &config);

        // Should contain SVG elements
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("<path"));
        assert!(svg.contains("Key Lifecycle"));
    }

    #[test]
    fn test_generate_certificate_state_machine_svg() {
        let definition = build_certificate_state_machine();
        let config = SvgConfig::default();

        let svg = generate_state_machine_svg(&definition, &config);

        assert!(svg.contains("Certificate Lifecycle"));
        // Should have states
        assert!(svg.contains("Active"));
        assert!(svg.contains("Revoked"));
    }

    #[test]
    fn test_circular_layout_positions() {
        let definition = build_key_state_machine();
        let config = SvgConfig::default();

        let positions = calculate_circular_layout(&definition.states, &config);

        // Should have position for each state
        assert_eq!(positions.len(), definition.states.len());

        // All positions should be different
        let unique: std::collections::HashSet<_> = positions.values()
            .map(|p| ((p.x * 10.0) as i32, (p.y * 10.0) as i32))
            .collect();
        assert_eq!(unique.len(), positions.len());
    }

    #[test]
    fn test_current_state_highlighting() {
        let mut definition = build_person_state_machine();
        definition.current_state = Some("Active".to_string());
        let config = SvgConfig::default();

        let svg = generate_state_machine_svg(&definition, &config);

        // Should contain current state fill color
        assert!(svg.contains(&config.colors.current_fill));
    }

    #[test]
    fn test_legend_present() {
        let definition = build_key_state_machine();
        let config = SvgConfig::default();

        let svg = generate_state_machine_svg(&definition, &config);

        assert!(svg.contains("class=\"legend\""));
        assert!(svg.contains("Initial"));
        assert!(svg.contains("Terminal"));
        assert!(svg.contains("Current"));
    }

    #[test]
    fn test_self_loop_transition() {
        // Create a state machine with self-loop (if any exist)
        let mut definition = StateMachineDefinition::new(StateMachineType::Key);
        definition = definition
            .with_state(StateMachineState::new("Test", "Test state").initial())
            .with_transition(StateMachineTransition::new("Test", "Test", "SelfLoop"));

        let config = SvgConfig::default();
        let svg = generate_state_machine_svg(&definition, &config);

        // Should have a curve path for self-loop
        assert!(svg.contains("path"));
        assert!(svg.contains("SelfLoop"));
    }

    #[test]
    fn test_truncate_name() {
        assert_eq!(truncate_name("Short", 10), "Short");
        assert_eq!(truncate_name("VeryLongStateName", 10), "VeryLongS…");
    }

    #[test]
    fn test_config_from_view_model() {
        let vm = ViewModel::default();
        let config = SvgConfig::from_view_model(&vm, 800.0, 600.0);

        assert_eq!(config.width, 800.0);
        assert_eq!(config.height, 600.0);
        assert_eq!(config.center_x, 400.0);
        assert_eq!(config.center_y, 300.0);
    }
}
