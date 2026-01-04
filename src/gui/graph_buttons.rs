// Copyright (c) 2025 - Cowboy AI, LLC.

//! Graph Button Widget - Unified button component for graph controls
//!
//! Provides consistent button sizing and styling across all graph controls.
//! Buttons are defined as data (label, icon, message) and rendered uniformly.

use iced::{
    widget::{button, container, row, text, Row},
    Element, Length, Padding,
};

use super::cowboy_theme::CowboyAppTheme as CowboyCustomTheme;
use super::view_model::ViewModel;
use super::graph::OrganizationIntent;
use crate::icons::{FONT_BODY, EMOJI_FONT};

// ============================================================================
// BUTTON DEFINITION - Data for a single button
// ============================================================================

/// Definition of a graph button
#[derive(Clone)]
pub struct GraphButtonDef {
    /// Button label text
    pub label: String,
    /// Optional icon (emoji or material icon)
    pub icon: Option<String>,
    /// Message to send when pressed
    pub message: OrganizationIntent,
    /// Button variant for sizing
    pub variant: ButtonVariant,
}

/// Button size variants
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Icon-only button (small, square)
    Icon,
    /// Compact text button
    Compact,
    /// Standard button
    #[default]
    Standard,
}

impl GraphButtonDef {
    /// Create an icon-only button
    pub fn icon(icon: impl Into<String>, message: OrganizationIntent) -> Self {
        Self {
            label: String::new(),
            icon: Some(icon.into()),
            message,
            variant: ButtonVariant::Icon,
        }
    }

    /// Create a compact text button
    pub fn compact(label: impl Into<String>, message: OrganizationIntent) -> Self {
        Self {
            label: label.into(),
            icon: None,
            message,
            variant: ButtonVariant::Compact,
        }
    }

    /// Create a standard button with optional icon
    pub fn standard(label: impl Into<String>, message: OrganizationIntent) -> Self {
        Self {
            label: label.into(),
            icon: None,
            message,
            variant: ButtonVariant::Standard,
        }
    }

    /// Add an icon to the button
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

// ============================================================================
// BUTTON GROUP - Collection of buttons rendered together
// ============================================================================

/// A group of graph buttons with consistent styling
pub struct GraphButtonGroup<'a> {
    buttons: Vec<GraphButtonDef>,
    vm: &'a ViewModel,
    /// Fixed width for all buttons (None = auto-size)
    fixed_width: Option<f32>,
}

impl<'a> GraphButtonGroup<'a> {
    /// Create a new button group
    pub fn new(vm: &'a ViewModel) -> Self {
        Self {
            buttons: Vec::new(),
            vm,
            fixed_width: None,
        }
    }

    /// Add a button definition
    pub fn push(mut self, button: GraphButtonDef) -> Self {
        self.buttons.push(button);
        self
    }

    /// Add multiple buttons from an iterator
    pub fn extend<I: IntoIterator<Item = GraphButtonDef>>(mut self, buttons: I) -> Self {
        self.buttons.extend(buttons);
        self
    }

    /// Set fixed width for all buttons
    pub fn width(mut self, width: f32) -> Self {
        self.fixed_width = Some(width);
        self
    }

    /// Render the button group as a row
    pub fn view(self) -> Element<'a, OrganizationIntent> {
        let vm = self.vm;
        let fixed_width = self.fixed_width;

        let buttons: Vec<Element<'a, OrganizationIntent>> = self
            .buttons
            .into_iter()
            .map(|def| render_button(def, vm, fixed_width))
            .collect();

        Row::with_children(buttons)
            .spacing(vm.spacing_xs)
            .into()
    }
}

// ============================================================================
// BUTTON RENDERING
// ============================================================================

/// Render a single button with consistent styling
fn render_button<'a>(
    def: GraphButtonDef,
    vm: &'a ViewModel,
    fixed_width: Option<f32>,
) -> Element<'a, OrganizationIntent> {
    // Extract owned values to avoid lifetime issues with text widget
    let label = def.label;
    let icon = def.icon;
    let message = def.message;

    // Determine sizing based on variant - ALL values from ViewModel
    let (text_size, min_width, padding) = match def.variant {
        ButtonVariant::Icon => (vm.text_medium, vm.button_width_icon, vm.padding_sm),
        ButtonVariant::Compact => (vm.text_small, vm.button_width_compact, vm.padding_sm),
        ButtonVariant::Standard => (vm.text_small, vm.button_width_standard, vm.padding_md),
    };

    // Use fixed width if provided, otherwise use variant minimum from ViewModel
    let width = fixed_width.unwrap_or(min_width);

    // Build button content - use owned strings to satisfy lifetime requirements
    let content: Element<'a, OrganizationIntent> = if let Some(icon_str) = icon {
        if label.is_empty() {
            // Icon only
            text(icon_str)
                .size(text_size)
                .font(EMOJI_FONT)
                .into()
        } else {
            // Icon + label
            row![
                text(icon_str).size(text_size).font(EMOJI_FONT),
                text(label).size(text_size).font(FONT_BODY),
            ]
            .spacing(vm.spacing_xs)
            .into()
        }
    } else {
        // Label only
        text(label)
            .size(text_size)
            .font(FONT_BODY)
            .into()
    };

    // Wrap in centered container for consistent alignment
    let centered = container(content)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    // Create the button - height from ViewModel for proper scaling
    button(centered)
        .width(Length::Fixed(width))
        .height(Length::Fixed(vm.button_height))
        .padding(Padding::from([padding, padding]))
        .style(CowboyCustomTheme::glass_button())
        .on_press(message)
        .into()
}

// ============================================================================
// STANDARD GRAPH BUTTON SETS
// ============================================================================

/// Zoom controls for graph navigation
/// - +/- are universal zoom symbols (icon-only)
/// - Reset uses 🎯 target icon with text for clarity
pub fn zoom_controls() -> Vec<GraphButtonDef> {
    vec![
        GraphButtonDef::icon("+", OrganizationIntent::ZoomIn),
        GraphButtonDef::icon("−", OrganizationIntent::ZoomOut),  // proper minus sign
        GraphButtonDef::compact("Reset", OrganizationIntent::ResetView)
            .with_icon("🎯"),
    ]
}

/// Layout algorithm buttons for graph arrangement
/// Icons reflect the mathematical/visual nature of each algorithm:
/// - ✨ Auto: intelligent/automatic arrangement
/// - 📐 Tutte: precise mathematical embedding (barycentric)
/// - ⚡ Force: force-directed energy minimization
/// - ⭕ Circle: circular arrangement
/// - 🌳 Tree: hierarchical parent-child structure
pub fn layout_buttons() -> Vec<GraphButtonDef> {
    use super::graph::LayoutAlgorithm;

    vec![
        GraphButtonDef::compact("Auto", OrganizationIntent::AutoLayout)
            .with_icon("✨"),
        GraphButtonDef::compact("Tutte", OrganizationIntent::ApplyLayout(LayoutAlgorithm::Tutte))
            .with_icon("📐"),
        GraphButtonDef::compact("Force", OrganizationIntent::ApplyLayout(LayoutAlgorithm::FruchtermanReingold))
            .with_icon("⚡"),
        GraphButtonDef::compact("Circle", OrganizationIntent::ApplyLayout(LayoutAlgorithm::Circular))
            .with_icon("⭕"),
        GraphButtonDef::compact("Tree", OrganizationIntent::ApplyLayout(LayoutAlgorithm::Hierarchical))
            .with_icon("🌳"),
    ]
}

/// Domain-specific layout buttons for CIM PKI context
/// Icons reflect the domain entities being grouped:
/// - 🔑 Keys: hardware security key grouping (visible when keys present)
/// - 📡 NATS: messaging/event infrastructure hierarchy (visible when NATS present)
///
/// Visibility computed from graph content at view time (pure function)
pub fn domain_layout_buttons(has_keys: bool, has_nats: bool) -> Vec<GraphButtonDef> {
    use super::graph::LayoutAlgorithm;

    let mut buttons = Vec::new();

    if has_keys {
        buttons.push(
            GraphButtonDef::compact("Keys", OrganizationIntent::ApplyLayout(LayoutAlgorithm::YubiKeyGrouped))
                .with_icon("🔑")
        );
    }

    if has_nats {
        buttons.push(
            GraphButtonDef::compact("NATS", OrganizationIntent::ApplyLayout(LayoutAlgorithm::NatsHierarchical))
                .with_icon("📡")
        );
    }

    buttons
}

// ============================================================================
// CONVENIENCE FUNCTION
// ============================================================================

/// Create the complete graph control bar
/// All sizing comes from ViewModel - buttons scale with UI
/// Visibility flags passed in by caller (no graph knowledge here)
pub fn graph_controls<'a>(
    vm: &'a ViewModel,
    has_keys: bool,
    has_nats: bool,
) -> Element<'a, OrganizationIntent> {
    let mut controls = row![
        // Zoom controls: +/- icons + Reset with icon+text
        GraphButtonGroup::new(vm)
            .extend(zoom_controls())
            .view(),
        // Layout algorithms: icon + text buttons (always visible)
        GraphButtonGroup::new(vm)
            .extend(layout_buttons())
            .view(),
    ]
    .spacing(vm.spacing_md);

    // Domain layouts: only add if relevant entities exist
    let domain_buttons = domain_layout_buttons(has_keys, has_nats);
    if !domain_buttons.is_empty() {
        controls = controls.push(
            GraphButtonGroup::new(vm)
                .extend(domain_buttons)
                .view()
        );
    }

    controls.into()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_def_creation() {
        let icon_btn = GraphButtonDef::icon("+", OrganizationIntent::ZoomIn);
        assert!(icon_btn.icon.is_some());
        assert!(icon_btn.label.is_empty());
        assert_eq!(icon_btn.variant, ButtonVariant::Icon);

        let compact_btn = GraphButtonDef::compact("Test", OrganizationIntent::AutoLayout);
        assert!(compact_btn.icon.is_none());
        assert_eq!(compact_btn.label, "Test");
        assert_eq!(compact_btn.variant, ButtonVariant::Compact);
    }

    #[test]
    fn test_button_with_icon() {
        let btn = GraphButtonDef::standard("Layout", OrganizationIntent::AutoLayout)
            .with_icon("📐");
        assert!(btn.icon.is_some());
        assert_eq!(btn.label, "Layout");
    }

    #[test]
    fn test_zoom_controls_count() {
        let controls = zoom_controls();
        assert_eq!(controls.len(), 3);
    }

    #[test]
    fn test_layout_buttons_count() {
        let buttons = layout_buttons();
        assert_eq!(buttons.len(), 5);
    }
}
