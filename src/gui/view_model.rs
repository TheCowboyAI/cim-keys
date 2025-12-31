//! View Model for GUI sizing and layout
//!
//! This module centralizes all sizing, spacing, layout, and color constants
//! to ensure consistent scaling and theming across the entire UI.

use iced::Color;

/// View Model containing all UI sizing, layout, and color parameters
#[derive(Debug, Clone)]
pub struct ViewModel {
    /// Base UI scale factor (1.0 = 100%)
    pub scale: f32,

    /// Color palette
    pub colors: ColorPalette,

    // Text sizes (scaled)
    pub text_tiny: u16,
    pub text_small: u16,
    pub text_normal: u16,
    pub text_medium: u16,
    pub text_large: u16,
    pub text_xlarge: u16,
    pub text_header: u16,
    pub text_title: u16,

    // Padding values (scaled)
    pub padding_xs: u16,
    pub padding_sm: u16,
    pub padding_md: u16,
    pub padding_lg: u16,
    pub padding_xl: u16,

    // Spacing values (scaled)
    pub spacing_xs: u16,
    pub spacing_sm: u16,
    pub spacing_md: u16,
    pub spacing_lg: u16,
    pub spacing_xl: u16,

    // Border radius (scaled)
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub radius_xl: f32,

    // Border width (scaled)
    pub border_thin: f32,
    pub border_normal: f32,
    pub border_thick: f32,

    // Button sizing (scaled)
    pub button_padding: u16,
    pub button_radius: f32,

    // Input sizing (scaled)
    pub input_padding: u16,
    pub input_radius: f32,

    // Card/Container sizing (scaled)
    pub card_padding: u16,
    pub card_radius: f32,

    // Shadow blur (scaled)
    pub shadow_sm: f32,
    pub shadow_md: f32,
    pub shadow_lg: f32,
}

impl ViewModel {
    /// Create a new ViewModel with the given scale factor
    pub fn new(scale: f32) -> Self {
        // Base values (at scale 1.0)
        const BASE_TEXT_TINY: u16 = 10;
        const BASE_TEXT_SMALL: u16 = 12;
        const BASE_TEXT_NORMAL: u16 = 14;
        const BASE_TEXT_MEDIUM: u16 = 16;
        const BASE_TEXT_LARGE: u16 = 18;
        const BASE_TEXT_XLARGE: u16 = 20;
        const BASE_TEXT_HEADER: u16 = 24;
        const BASE_TEXT_TITLE: u16 = 32;

        const BASE_PADDING_XS: u16 = 4;
        const BASE_PADDING_SM: u16 = 8;
        const BASE_PADDING_MD: u16 = 12;
        const BASE_PADDING_LG: u16 = 16;
        const BASE_PADDING_XL: u16 = 20;

        const BASE_SPACING_XS: u16 = 2;
        const BASE_SPACING_SM: u16 = 5;
        const BASE_SPACING_MD: u16 = 10;
        const BASE_SPACING_LG: u16 = 15;
        const BASE_SPACING_XL: u16 = 20;

        const BASE_RADIUS_SM: f32 = 4.0;
        const BASE_RADIUS_MD: f32 = 8.0;
        const BASE_RADIUS_LG: f32 = 12.0;
        const BASE_RADIUS_XL: f32 = 20.0;

        const BASE_BORDER_THIN: f32 = 1.0;
        const BASE_BORDER_NORMAL: f32 = 2.0;
        const BASE_BORDER_THICK: f32 = 3.0;

        const BASE_SHADOW_SM: f32 = 4.0;
        const BASE_SHADOW_MD: f32 = 8.0;
        const BASE_SHADOW_LG: f32 = 16.0;

        Self {
            scale,
            colors: ColorPalette::default(),

            // Text sizes
            text_tiny: (BASE_TEXT_TINY as f32 * scale) as u16,
            text_small: (BASE_TEXT_SMALL as f32 * scale) as u16,
            text_normal: (BASE_TEXT_NORMAL as f32 * scale) as u16,
            text_medium: (BASE_TEXT_MEDIUM as f32 * scale) as u16,
            text_large: (BASE_TEXT_LARGE as f32 * scale) as u16,
            text_xlarge: (BASE_TEXT_XLARGE as f32 * scale) as u16,
            text_header: (BASE_TEXT_HEADER as f32 * scale) as u16,
            text_title: (BASE_TEXT_TITLE as f32 * scale) as u16,

            // Padding
            padding_xs: (BASE_PADDING_XS as f32 * scale) as u16,
            padding_sm: (BASE_PADDING_SM as f32 * scale) as u16,
            padding_md: (BASE_PADDING_MD as f32 * scale) as u16,
            padding_lg: (BASE_PADDING_LG as f32 * scale) as u16,
            padding_xl: (BASE_PADDING_XL as f32 * scale) as u16,

            // Spacing
            spacing_xs: (BASE_SPACING_XS as f32 * scale) as u16,
            spacing_sm: (BASE_SPACING_SM as f32 * scale) as u16,
            spacing_md: (BASE_SPACING_MD as f32 * scale) as u16,
            spacing_lg: (BASE_SPACING_LG as f32 * scale) as u16,
            spacing_xl: (BASE_SPACING_XL as f32 * scale) as u16,

            // Border radius
            radius_sm: BASE_RADIUS_SM * scale,
            radius_md: BASE_RADIUS_MD * scale,
            radius_lg: BASE_RADIUS_LG * scale,
            radius_xl: BASE_RADIUS_XL * scale,

            // Border width
            border_thin: BASE_BORDER_THIN * scale,
            border_normal: BASE_BORDER_NORMAL * scale,
            border_thick: BASE_BORDER_THICK * scale,

            // Button sizing
            button_padding: (BASE_PADDING_MD as f32 * scale) as u16,
            button_radius: BASE_RADIUS_XL * scale,

            // Input sizing
            input_padding: (BASE_PADDING_MD as f32 * scale) as u16,
            input_radius: BASE_RADIUS_MD * scale,

            // Card/Container sizing
            card_padding: (BASE_PADDING_XL as f32 * scale) as u16,
            card_radius: BASE_RADIUS_LG * scale,

            // Shadow blur
            shadow_sm: BASE_SHADOW_SM * scale,
            shadow_md: BASE_SHADOW_MD * scale,
            shadow_lg: BASE_SHADOW_LG * scale,
        }
    }

    /// Update scale and recalculate all values
    pub fn set_scale(&mut self, new_scale: f32) {
        *self = Self::new(new_scale);
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// Centralized color palette for the application
#[derive(Debug, Clone)]
pub struct ColorPalette {
    // Primary text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_disabled: Color,
    pub text_light: Color,
    pub text_dark: Color,
    pub text_hint: Color,           // Subtle hint text (blue-tinted, info ontology)
    pub text_subtle_success: Color, // Subtle positive text (green-tinted, life ontology)

    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // UI element colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub surface: Color,
    pub border: Color,

    // Specific UI colors
    pub glass_background: Color,
    pub card_background: Color,
    pub button_primary: Color,
    pub button_secondary: Color,
    pub button_security: Color,
    pub button_disabled: Color,
    pub button_disabled_text: Color,

    // Button/interactive states (for closures)
    pub surface_hover: Color,
    pub surface_pressed: Color,
    pub panel_background: Color,
    pub border_subtle: Color,

    // Graph/visualization colors
    pub node_default: Color,
    pub node_selected: Color,
    pub edge_default: Color,
    pub edge_selected: Color,

    // Node type colors
    pub node_organization: Color,
    pub node_unit: Color,
    pub node_person: Color,
    pub node_location: Color,
    pub node_role: Color,
    pub node_policy: Color,
    pub node_edge_highlight: Color,

    // Separation class colors (domain ontology for role segregation)
    pub class_operational: Color,     // Blue - day-to-day tasks
    pub class_administrative: Color,  // Purple - user/policy management
    pub class_audit: Color,           // Teal - monitoring/review
    pub class_emergency: Color,       // Red - break-glass access
    pub class_financial: Color,       // Gold - budgets/spending
    pub class_personnel: Color,       // Rose - HR/staffing

    // Semantic colors
    pub blue_bright: Color,
    pub blue_glow: Color,
    pub green_success: Color,
    pub red_error: Color,
    pub yellow_warning: Color,
    pub orange_caution: Color,  // Solid orange for "fair" states (H40°, S75%, L50%)
    pub orange_warning: Color,

    // Overlay/Modal colors
    pub overlay_background: Color,
    pub modal_background: Color,
    pub modal_border: Color,
    pub modal_content_background: Color,

    // Shadow colors
    pub shadow_default: Color,
    pub shadow_medium: Color,
    pub shadow_blue: Color,
    pub shadow_yellow: Color,

    // Strength indicator background
    pub strength_bar_background: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            // Primary text colors
            text_primary: Color::from_rgb(0.9, 0.9, 0.9),
            text_secondary: Color::from_rgb(0.7, 0.7, 0.8),
            text_tertiary: Color::from_rgb(0.6, 0.6, 0.6),
            text_disabled: Color::from_rgb(0.5, 0.5, 0.5),
            text_light: Color::from_rgb(1.0, 1.0, 1.0),
            text_dark: Color::from_rgb(0.1, 0.1, 0.1),
            text_hint: Color::from_rgb(0.6, 0.6, 0.7),           // Info ontology (blue-tint)
            text_subtle_success: Color::from_rgb(0.5, 0.6, 0.5), // Life ontology (green-tint)

            // Status colors
            success: Color::from_rgb(0.3, 0.8, 0.3),
            warning: Color::from_rgb(1.0, 0.8, 0.0),
            error: Color::from_rgb(0.9, 0.2, 0.2),
            info: Color::from_rgb(0.3, 0.6, 1.0),

            // UI element colors
            primary: Color::from_rgb(0.3, 0.6, 1.0),
            secondary: Color::from_rgb(0.5, 0.5, 0.6),
            accent: Color::from_rgb(0.8, 0.4, 0.9),
            background: Color::from_rgb(0.0, 0.0, 0.0),
            surface: Color::from_rgba(0.1, 0.1, 0.15, 0.8),
            border: Color::from_rgba(0.5, 0.5, 0.6, 0.7),

            // Specific UI colors
            glass_background: Color::from_rgba(0.2, 0.2, 0.3, 0.5),
            card_background: Color::from_rgba(0.15, 0.15, 0.2, 0.6),
            button_primary: Color::from_rgb(0.3, 0.6, 1.0),
            button_secondary: Color::from_rgb(0.5, 0.5, 0.6),
            button_security: Color::from_rgb(0.8, 0.3, 0.3),
            button_disabled: Color::from_rgb(0.3, 0.3, 0.3),       // L30% neutral
            button_disabled_text: Color::from_rgb(0.5, 0.5, 0.5),  // L50% neutral

            // Button/interactive states
            surface_hover: Color::from_rgba(0.3, 0.3, 0.35, 0.9),
            surface_pressed: Color::from_rgba(0.25, 0.25, 0.3, 0.9),
            panel_background: Color::from_rgba(0.15, 0.15, 0.2, 0.9),
            border_subtle: Color::from_rgba(0.3, 0.3, 0.35, 0.5),

            // Graph/visualization colors
            node_default: Color::from_rgba(0.2, 0.2, 0.3, 0.9),
            node_selected: Color::from_rgba(0.3, 0.6, 1.0, 0.95),
            edge_default: Color::from_rgba(0.8, 0.8, 0.9, 0.7),
            edge_selected: Color::from_rgba(0.3, 0.6, 1.0, 0.9),

            // Node type colors
            node_organization: Color::from_rgb(0.2, 0.3, 0.6),
            node_unit: Color::from_rgb(0.4, 0.5, 0.8),
            node_person: Color::from_rgb(0.5, 0.7, 0.3),
            node_location: Color::from_rgb(0.6, 0.5, 0.4),
            node_role: Color::from_rgb(0.6, 0.3, 0.8),
            node_policy: Color::from_rgb(0.8, 0.6, 0.2),
            node_edge_highlight: Color::from_rgb(0.3, 0.3, 0.7),

            // Separation class colors (domain ontology)
            class_operational: Color::from_rgb(0.3, 0.6, 0.9),     // Blue - H210°
            class_administrative: Color::from_rgb(0.6, 0.4, 0.8),  // Purple - H270°
            class_audit: Color::from_rgb(0.2, 0.7, 0.5),           // Teal - H160°
            class_emergency: Color::from_rgb(0.9, 0.3, 0.2),       // Red - H10°
            class_financial: Color::from_rgb(0.9, 0.7, 0.2),       // Gold - H45°
            class_personnel: Color::from_rgb(0.8, 0.4, 0.6),       // Rose - H340°

            // Semantic colors
            blue_bright: Color::from_rgba(0.3, 0.6, 1.0, 0.8),
            blue_glow: Color::from_rgba(0.3, 0.6, 1.0, 0.6),
            green_success: Color::from_rgb(0.2, 0.9, 0.2),
            red_error: Color::from_rgb(0.9, 0.2, 0.2),
            yellow_warning: Color::from_rgb(1.0, 0.8, 0.0),
            orange_caution: Color::from_rgb(0.8, 0.6, 0.2),  // H40°, solid orange for "fair"
            orange_warning: Color::from_rgba(0.8, 0.6, 0.0, 0.15),

            // Overlay/Modal colors
            overlay_background: Color::from_rgba(0.0, 0.0, 0.0, 0.7),
            modal_background: Color::from_rgba(0.1, 0.1, 0.15, 0.95),
            modal_border: Color::from_rgb(0.4, 0.6, 0.8),    // H210°, soft blue border
            modal_content_background: Color::from_rgba(0.12, 0.12, 0.16, 0.98),

            // Shadow colors
            shadow_default: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            shadow_medium: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            shadow_blue: Color::from_rgba(0.3, 0.6, 1.0, 0.3),
            shadow_yellow: Color::from_rgba(1.0, 0.8, 0.0, 0.3),

            // Strength indicator background
            strength_bar_background: Color::from_rgb(0.2, 0.2, 0.2),
        }
    }
}

impl ColorPalette {
    /// Get color for password/passphrase strength indicator
    ///
    /// Ontological mapping:
    /// - Weak (< 0.3): Red → Danger/Error ontology
    /// - Fair (< 0.6): Orange → Caution ontology
    /// - Good (< 0.8): Yellow → Warning/Progress ontology
    /// - Strong (>= 0.8): Green → Success/Life ontology
    pub fn strength_color(&self, strength: f32) -> Color {
        if strength < 0.3 {
            self.red_error
        } else if strength < 0.6 {
            self.orange_caution
        } else if strength < 0.8 {
            self.yellow_warning
        } else {
            self.green_success
        }
    }

    /// Get color for separation class (duty segregation ontology)
    ///
    /// Each class maps to a distinct hue family for clear visual distinction:
    /// - Operational: Blue (trust, routine)
    /// - Administrative: Purple (authority, elevation)
    /// - Audit: Teal (observation, neutrality)
    /// - Emergency: Red (urgency, danger)
    /// - Financial: Gold (value, caution)
    /// - Personnel: Rose (people, warmth)
    pub fn separation_class_color(&self, class: &crate::policy::SeparationClass) -> Color {
        match class {
            crate::policy::SeparationClass::Operational => self.class_operational,
            crate::policy::SeparationClass::Administrative => self.class_administrative,
            crate::policy::SeparationClass::Audit => self.class_audit,
            crate::policy::SeparationClass::Emergency => self.class_emergency,
            crate::policy::SeparationClass::Financial => self.class_financial,
            crate::policy::SeparationClass::Personnel => self.class_personnel,
        }
    }

    /// Create a lighter variant of any color (for hover states, etc.)
    pub fn lighten(&self, color: Color, amount: f32) -> Color {
        Color::from_rgba(
            (color.r + amount).min(1.0),
            (color.g + amount).min(1.0),
            (color.b + amount).min(1.0),
            color.a,
        )
    }

    /// Create a darker variant of any color
    pub fn darken(&self, color: Color, amount: f32) -> Color {
        Color::from_rgba(
            (color.r - amount).max(0.0),
            (color.g - amount).max(0.0),
            (color.b - amount).max(0.0),
            color.a,
        )
    }

    /// Adjust alpha/transparency of any color
    pub fn with_alpha(&self, color: Color, alpha: f32) -> Color {
        Color::from_rgba(color.r, color.g, color.b, alpha)
    }
}

// ============================================================================
// GRAPH NODE VIEW MODEL - DDD SEPARATION OF UI CONCERNS
// ============================================================================

use iced::Point;
use uuid::Uuid;

// Import conceptual-spaces types for semantic positioning
#[cfg(feature = "conceptual-spaces")]
use cim_domain_spaces::{Point3, KnowledgeLevel, EvidenceScore};

/// Stereographic projection from 3D unit sphere to 2D plane.
///
/// Projects a point on the unit sphere (semantic 3D position) to a 2D screen
/// position using stereographic projection from the south pole.
///
/// # Algorithm
///
/// Given a point (x, y, z) on the unit sphere where z ≠ -1:
/// - Projected x' = x / (1 + z)
/// - Projected y' = y / (1 + z)
///
/// The result is then scaled to fit the canvas dimensions.
///
/// # Arguments
///
/// * `p3` - A point on the unit sphere (semantic position)
/// * `center` - Center of the 2D canvas
/// * `scale` - Scale factor for the projection
///
/// # Returns
///
/// A 2D point suitable for screen rendering.
#[cfg(feature = "conceptual-spaces")]
pub fn stereographic_projection(p3: &Point3<f64>, center: Point, scale: f32) -> Point {
    // Avoid division by zero at south pole
    let denom: f64 = 1.0 + p3.z;
    let denom = if denom.abs() < 0.001 { 0.001 } else { denom };

    let x = (p3.x / denom) as f32 * scale + center.x;
    let y = (p3.y / denom) as f32 * scale + center.y;

    Point::new(x, y)
}

/// View model for individual graph node visualization.
///
/// Separates UI concerns (position, selection, sizing) from domain data.
/// The domain entity (DomainNode) remains pure while the view layer
/// handles all rendering-specific properties.
///
/// ## DDD Compliance
///
/// This separation achieves:
/// - Domain layer purity (no UI concerns in entities)
/// - Clear responsibility boundaries
/// - Easier testing (domain logic vs UI logic)
#[derive(Debug, Clone)]
pub struct NodeView {
    /// Reference to the domain entity
    pub entity_id: Uuid,

    /// Position in the graph canvas (UI concern - 2D projection)
    pub position: Point,

    /// Semantic 3D position on unit sphere (conceptual-spaces integration)
    ///
    /// When set, the 2D `position` is derived via stereographic projection.
    /// This enables semantic similarity to be visualized spatially.
    #[cfg(feature = "conceptual-spaces")]
    pub semantic_position: Option<Point3<f64>>,

    /// Knowledge level from conceptual-spaces
    ///
    /// Indicates our epistemic state about this entity:
    /// - Unknown: We don't know if it exists
    /// - Suspected: We suspect it exists based on patterns
    /// - KnownUnknown: We know we don't know the details
    /// - Known: We have verified information
    #[cfg(feature = "conceptual-spaces")]
    pub knowledge_level: Option<KnowledgeLevel>,

    /// Evidence score for confidence visualization
    ///
    /// Range [0.0, 1.0] indicating confidence in the entity's data.
    /// Affects visual rendering (e.g., opacity, border style).
    #[cfg(feature = "conceptual-spaces")]
    pub evidence_score: Option<EvidenceScore>,

    /// Display color (derived from domain node type)
    pub color: Color,

    /// Display label (derived from domain node data)
    pub label: String,

    /// Secondary text for additional info
    pub secondary_text: Option<String>,

    /// Whether this node is currently selected
    pub selected: bool,

    /// Whether this node is being dragged
    pub dragging: bool,

    /// Radius for rendering
    pub radius: f32,
}

impl NodeView {
    /// Create a new node view with default state
    pub fn new(entity_id: Uuid, position: Point, color: Color, label: String) -> Self {
        Self {
            entity_id,
            position,
            #[cfg(feature = "conceptual-spaces")]
            semantic_position: None,
            #[cfg(feature = "conceptual-spaces")]
            knowledge_level: None,
            #[cfg(feature = "conceptual-spaces")]
            evidence_score: None,
            color,
            label,
            secondary_text: None,
            selected: false,
            dragging: false,
            radius: 30.0,
        }
    }

    /// Create a node view with semantic 3D position
    ///
    /// The 2D screen position is derived via stereographic projection.
    #[cfg(feature = "conceptual-spaces")]
    pub fn with_semantic_position(
        entity_id: Uuid,
        semantic_pos: Point3<f64>,
        canvas_center: Point,
        scale: f32,
        color: Color,
        label: String,
    ) -> Self {
        let position = stereographic_projection(&semantic_pos, canvas_center, scale);
        Self {
            entity_id,
            position,
            semantic_position: Some(semantic_pos),
            knowledge_level: None,
            evidence_score: None,
            color,
            label,
            secondary_text: None,
            selected: false,
            dragging: false,
            radius: 30.0,
        }
    }

    /// Set the knowledge level for this node
    #[cfg(feature = "conceptual-spaces")]
    pub fn with_knowledge_level(mut self, level: KnowledgeLevel) -> Self {
        self.knowledge_level = Some(level);
        self
    }

    /// Set the evidence score for this node
    #[cfg(feature = "conceptual-spaces")]
    pub fn with_evidence_score(mut self, score: EvidenceScore) -> Self {
        self.evidence_score = Some(score);
        self
    }

    /// Get the visual opacity based on knowledge level
    ///
    /// - Known: 1.0 (fully opaque)
    /// - KnownUnknown: 0.8
    /// - Suspected: 0.6
    /// - Unknown: 0.4
    #[cfg(feature = "conceptual-spaces")]
    pub fn knowledge_opacity(&self) -> f32 {
        match self.knowledge_level {
            Some(KnowledgeLevel::Known) => 1.0,
            Some(KnowledgeLevel::KnownUnknown) => 0.8,
            Some(KnowledgeLevel::Suspected) => 0.6,
            Some(KnowledgeLevel::Unknown) => 0.4,
            None => 1.0, // Default to fully opaque
        }
    }

    /// Get the border style based on knowledge level
    ///
    /// Returns (border_width, is_dashed) tuple.
    #[cfg(feature = "conceptual-spaces")]
    pub fn knowledge_border_style(&self) -> (f32, bool) {
        match self.knowledge_level {
            Some(KnowledgeLevel::Known) => (2.0, false),        // Solid, normal
            Some(KnowledgeLevel::KnownUnknown) => (2.0, true),  // Dashed
            Some(KnowledgeLevel::Suspected) => (1.5, true),     // Thin dashed
            Some(KnowledgeLevel::Unknown) => (1.0, true),       // Thin dashed
            None => (2.0, false),
        }
    }

    /// Update position during drag
    pub fn with_position(mut self, position: Point) -> Self {
        self.position = position;
        self
    }

    /// Set selection state
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set dragging state
    pub fn with_dragging(mut self, dragging: bool) -> Self {
        self.dragging = dragging;
        self
    }

    /// Check if a point is within this node's bounds
    pub fn contains_point(&self, point: Point) -> bool {
        let dx = point.x - self.position.x;
        let dy = point.y - self.position.y;
        (dx * dx + dy * dy).sqrt() <= self.radius
    }

    /// Get the effective radius (larger when selected)
    pub fn effective_radius(&self) -> f32 {
        if self.selected {
            self.radius * 1.2
        } else {
            self.radius
        }
    }

    /// Get the border color (different when selected or dragging)
    pub fn border_color(&self) -> Color {
        if self.dragging {
            Color::from_rgb(0.0, 0.8, 0.0)
        } else if self.selected {
            Color::from_rgb(0.2, 0.6, 1.0)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.3)
        }
    }
}

/// View model for graph edge visualization.
#[derive(Debug, Clone)]
pub struct EdgeView {
    /// Source node ID
    pub from_id: Uuid,

    /// Target node ID
    pub to_id: Uuid,

    /// Edge color
    pub color: Color,

    /// Edge style (solid, dashed, etc.)
    pub style: EdgeStyle,

    /// Optional label on the edge
    pub label: Option<String>,
}

/// Style of edge line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeStyle {
    /// Solid line
    #[default]
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
}

impl EdgeView {
    /// Create a new edge view
    pub fn new(from_id: Uuid, to_id: Uuid, color: Color) -> Self {
        Self {
            from_id,
            to_id,
            color,
            style: EdgeStyle::Solid,
            label: None,
        }
    }

    /// Set edge style
    pub fn with_style(mut self, style: EdgeStyle) -> Self {
        self.style = style;
        self
    }

    /// Set edge label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_view_contains_point() {
        let view = NodeView::new(
            Uuid::now_v7(),
            Point::new(100.0, 100.0),
            Color::WHITE,
            "Test".to_string(),
        );

        // Point inside
        assert!(view.contains_point(Point::new(100.0, 100.0)));
        assert!(view.contains_point(Point::new(110.0, 110.0)));

        // Point outside
        assert!(!view.contains_point(Point::new(200.0, 200.0)));
    }

    #[test]
    fn test_node_view_effective_radius() {
        let view = NodeView::new(
            Uuid::now_v7(),
            Point::new(0.0, 0.0),
            Color::WHITE,
            "Test".to_string(),
        );

        assert_eq!(view.effective_radius(), 30.0);

        let selected = view.with_selected(true);
        assert_eq!(selected.effective_radius(), 36.0); // 30 * 1.2
    }
}
