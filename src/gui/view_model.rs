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

    // Semantic colors
    pub blue_bright: Color,
    pub blue_glow: Color,
    pub green_success: Color,
    pub red_error: Color,
    pub yellow_warning: Color,
    pub orange_warning: Color,

    // Overlay/Modal colors
    pub overlay_background: Color,
    pub modal_background: Color,

    // Shadow colors
    pub shadow_default: Color,
    pub shadow_blue: Color,
    pub shadow_yellow: Color,
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

            // Semantic colors
            blue_bright: Color::from_rgba(0.3, 0.6, 1.0, 0.8),
            blue_glow: Color::from_rgba(0.3, 0.6, 1.0, 0.6),
            green_success: Color::from_rgb(0.2, 0.9, 0.2),
            red_error: Color::from_rgb(0.9, 0.2, 0.2),
            yellow_warning: Color::from_rgb(1.0, 0.8, 0.0),
            orange_warning: Color::from_rgba(0.8, 0.6, 0.0, 0.15),

            // Overlay/Modal colors
            overlay_background: Color::from_rgba(0.0, 0.0, 0.0, 0.7),
            modal_background: Color::from_rgba(0.1, 0.1, 0.15, 0.95),

            // Shadow colors
            shadow_default: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            shadow_blue: Color::from_rgba(0.3, 0.6, 1.0, 0.3),
            shadow_yellow: Color::from_rgba(1.0, 0.8, 0.0, 0.3),
        }
    }
}

impl ColorPalette {
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
