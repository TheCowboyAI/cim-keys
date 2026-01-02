// Copyright (c) 2025 - Cowboy AI, LLC.

//! Typography and Icons - Font Definitions
//!
//! Defines all fonts used in the CIM Keys application with explicit purposes.
//!
//! ## Verified Icon System (Recommended)
//!
//! For new code, prefer using the verified icon system which guarantees no tofu:
//!
//! ```rust,ignore
//! use cim_keys::icons::verified;
//!
//! // Get a verified icon by semantic name
//! let lock_icon = verified::icon("locked", 16);
//!
//! // Or use the typed API
//! let person_icon = verified::entity_icon(EntityIcon::Person, 16);
//! ```
//!
//! ## Legacy Icon Constants
//!
//! The `ICON_*` constants are maintained for backward compatibility.
//! They map to the verified icon system internally.

use iced::{widget::text, Color, Element, Font};

use crate::domains::typography::{
    VerifiedTheme, VerifiedIcon, IconRepresentation,
};

// ============================================================================
// FONT DEFINITIONS - Custom fonts referenced by family name
//
// Fonts are loaded in gui.rs run() function via .font(include_bytes!())
// and then referenced here by their internal family names
// ============================================================================

/// Rec Mono Linear - Standard body text font (monospace)
/// Use for: Code, data, general UI text
pub const FONT_BODY: Font = Font {
    family: iced::font::Family::Name("Rec Mono Linear"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

/// Poller One - Heading font (display)
/// Use for: Page titles, section headings, emphasis
pub const FONT_HEADING: Font = Font {
    family: iced::font::Family::Name("Poller One"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

/// Material Icons - UI icon glyphs
/// Use for: Interface icons, buttons, navigation
pub const MATERIAL_ICONS: Font = Font {
    family: iced::font::Family::Name("Material Icons"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

/// Noto Color Emoji - Emoji rendering
/// Use for: Emoji characters, status indicators
pub const EMOJI_FONT: Font = Font {
    family: iced::font::Family::Name("Noto Color Emoji"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

// ============================================================================
// LEGACY ICON CONSTANTS
//
// These constants are maintained for backward compatibility.
// For new code, prefer using the verified icon system in the `verified` module.
// ============================================================================

/// Lock icon - use `verified::icon("locked", size)` for fallback support
pub const ICON_LOCK: char = '🔒';
/// Warning icon - use `verified::icon("warning", size)` for fallback support
pub const ICON_WARNING: char = '⚠';
/// Check icon - use `verified::icon("check", size)` for fallback support
pub const ICON_CHECK: char = '✓';
/// Close icon - use `verified::icon("close", size)` for fallback support
pub const ICON_CLOSE: char = '✕';
/// Business/Organization icon - use `verified::icon("organization", size)` for fallback support
pub const ICON_BUSINESS: char = '🏢';
/// Group icon - use `verified::icon("group", size)` for fallback support
pub const ICON_GROUP: char = '👥';
/// Person icon - use `verified::icon("person", size)` for fallback support
pub const ICON_PERSON: char = '👤';
/// Location icon - use `verified::icon("location", size)` for fallback support
pub const ICON_LOCATION: char = '📍';
/// Security/YubiKey icon - use `verified::icon("yubikey", size)` for fallback support
pub const ICON_SECURITY: char = '🔐';
/// Verified/Certificate icon - use `verified::icon("certificate", size)` for fallback support
pub const ICON_VERIFIED: char = '✅';
/// Visibility icon - use `verified::icon("visibility", size)` for fallback support
pub const ICON_VISIBILITY: char = '👁';
/// Visibility off icon - use `verified::icon("visibility_off", size)` for fallback support
pub const ICON_VISIBILITY_OFF: char = '🙈';
/// Cloud icon - use `verified::icon("cloud", size)` for fallback support
pub const ICON_CLOUD: char = '☁';
/// Account circle icon - use `verified::icon("person", size)` for fallback support
pub const ICON_ACCOUNT_CIRCLE: char = '👤';
/// Settings icon - use `verified::icon("operator", size)` for fallback support
pub const ICON_SETTINGS: char = '⚙';

// Progressive disclosure icons (folder metaphor)
/// Folder icon - use `verified::icon("folder", size)` for fallback support
pub const ICON_FOLDER: char = '📁';
/// Folder open icon - use `verified::icon("folder_open", size)` for fallback support
pub const ICON_FOLDER_OPEN: char = '📂';

// Additional icons for DomainNode visualization
/// Key icon - use `verified::icon("key", size)` for fallback support
pub const ICON_KEY: char = '🔑';
/// USB icon - use `verified::icon("usb", size)` for fallback support
pub const ICON_USB: char = '🔌';
/// Memory/Save icon - use `verified::icon("save", size)` for fallback support
pub const ICON_MEMORY: char = '💾';
/// Download icon - use `verified::icon("download", size)` for fallback support
pub const ICON_DOWNLOAD: char = '📥';
/// Help icon - use `verified::icon("help", size)` for fallback support
pub const ICON_HELP: char = '❓';
/// Check circle icon - use `verified::icon("check_circle", size)` for fallback support
pub const ICON_CHECK_CIRCLE: char = '✅';

// ============================================================================
// LEGACY ICON FUNCTIONS
//
// These functions render icons using the emoji font directly.
// For new code, prefer using the verified icon system.
// ============================================================================

/// Render an icon with default size and emoji font
pub fn icon<'a, Message: 'a>(icon_char: char) -> Element<'a, Message> {
    text(icon_char)
        .font(EMOJI_FONT)
        .size(16)
        .into()
}

/// Render an icon with specific size and emoji font
pub fn icon_sized<'a, Message: 'a>(icon_char: char, size: u16) -> Element<'a, Message> {
    text(icon_char)
        .font(EMOJI_FONT)
        .size(size)
        .into()
}

/// Render an icon with specific size, color, and emoji font
pub fn icon_colored<'a, Message: 'a>(icon_char: char, size: u16, color: Color) -> Element<'a, Message> {
    text(icon_char)
        .font(EMOJI_FONT)
        .size(size)
        .color(color)
        .into()
}

// ============================================================================
// VERIFIED ICON SYSTEM
//
// Uses the Typography bounded context to provide icons with fallback chains.
// Guarantees that something will render (no tofu boxes).
// ============================================================================

/// Verified icon module - provides icons with fallback chains
pub mod verified {
    use super::*;
    use std::sync::OnceLock;

    // Re-export icon types for convenience
    pub use crate::domains::typography::{
        VerifiedIconSet, StatusIcon, NavigationIcon, ActionIcon, EntityIcon, SemanticIcon,
    };

    /// Get the default verified theme (cached)
    ///
    /// Uses `cim_default()` which assumes fonts are loaded via Iced.
    /// Falls back to emoji icons first, then Material Icons, then text.
    pub fn default_theme() -> &'static VerifiedTheme {
        static THEME: OnceLock<VerifiedTheme> = OnceLock::new();
        THEME.get_or_init(|| VerifiedTheme::cim_default())
    }

    /// Get the theme's color palette
    pub fn colors() -> &'static crate::domains::typography::ColorPalette {
        default_theme().colors()
    }

    /// Get the theme's metrics
    pub fn metrics() -> &'static crate::domains::typography::ThemeMetrics {
        default_theme().metrics()
    }

    /// Render a verified icon by semantic name
    ///
    /// Returns the icon with the first available representation from the fallback chain.
    /// Guaranteed to render something (falls back to text if needed).
    ///
    /// # Arguments
    /// * `name` - Semantic name like "locked", "warning", "person", etc.
    /// * `size` - Font size in pixels
    ///
    /// # Example
    /// ```rust,ignore
    /// use cim_keys::icons::verified;
    /// let lock_icon = verified::icon("locked", 16);
    /// ```
    pub fn icon<'a, Message: 'a>(name: &str, size: u16) -> Element<'a, Message> {
        let theme = default_theme();
        if let Some(verified_icon) = theme.icons().get(name) {
            render_verified_icon(verified_icon, size)
        } else {
            // Fallback: use text
            text(format!("[{}]", name.to_uppercase()))
                .size(size)
                .into()
        }
    }

    /// Render a verified icon with color
    pub fn icon_colored<'a, Message: 'a>(name: &str, size: u16, color: Color) -> Element<'a, Message> {
        let theme = default_theme();
        if let Some(verified_icon) = theme.icons().get(name) {
            render_verified_icon_colored(verified_icon, size, color)
        } else {
            text(format!("[{}]", name.to_uppercase()))
                .size(size)
                .color(color)
                .into()
        }
    }

    /// Render a status icon
    pub fn status_icon<'a, Message: 'a>(icon: StatusIcon, size: u16) -> Element<'a, Message> {
        let name = SemanticIcon::Status(icon).name();
        self::icon(name, size)
    }

    /// Render a status icon with color
    pub fn status_icon_colored<'a, Message: 'a>(icon: StatusIcon, size: u16, color: Color) -> Element<'a, Message> {
        let name = SemanticIcon::Status(icon).name();
        self::icon_colored(name, size, color)
    }

    /// Render a navigation icon
    pub fn navigation_icon<'a, Message: 'a>(icon: NavigationIcon, size: u16) -> Element<'a, Message> {
        let name = SemanticIcon::Navigation(icon).name();
        self::icon(name, size)
    }

    /// Render a navigation icon with color
    pub fn navigation_icon_colored<'a, Message: 'a>(icon: NavigationIcon, size: u16, color: Color) -> Element<'a, Message> {
        let name = SemanticIcon::Navigation(icon).name();
        self::icon_colored(name, size, color)
    }

    /// Render an action icon
    pub fn action_icon<'a, Message: 'a>(icon: ActionIcon, size: u16) -> Element<'a, Message> {
        let name = SemanticIcon::Action(icon).name();
        self::icon(name, size)
    }

    /// Render an action icon with color
    pub fn action_icon_colored<'a, Message: 'a>(icon: ActionIcon, size: u16, color: Color) -> Element<'a, Message> {
        let name = SemanticIcon::Action(icon).name();
        self::icon_colored(name, size, color)
    }

    /// Render an entity icon
    pub fn entity_icon<'a, Message: 'a>(icon: EntityIcon, size: u16) -> Element<'a, Message> {
        let name = SemanticIcon::Entity(icon).name();
        self::icon(name, size)
    }

    /// Render an entity icon with color
    pub fn entity_icon_colored<'a, Message: 'a>(icon: EntityIcon, size: u16, color: Color) -> Element<'a, Message> {
        let name = SemanticIcon::Entity(icon).name();
        self::icon_colored(name, size, color)
    }

    /// Internal: render a verified icon
    fn render_verified_icon<'a, Message: 'a>(icon: &VerifiedIcon, size: u16) -> Element<'a, Message> {
        let display = icon.display();
        match icon.representation() {
            IconRepresentation::Emoji(_) => {
                text(display)
                    .font(super::EMOJI_FONT)
                    .size(size)
                    .into()
            }
            IconRepresentation::MaterialIcon(_) => {
                text(display)
                    .font(super::MATERIAL_ICONS)
                    .size(size)
                    .into()
            }
            IconRepresentation::UnicodeSymbol(_) | IconRepresentation::TextFallback(_) => {
                text(display)
                    .size(size)
                    .into()
            }
        }
    }

    /// Internal: render a verified icon with color
    fn render_verified_icon_colored<'a, Message: 'a>(icon: &VerifiedIcon, size: u16, color: Color) -> Element<'a, Message> {
        let display = icon.display();
        match icon.representation() {
            IconRepresentation::Emoji(_) => {
                text(display)
                    .font(super::EMOJI_FONT)
                    .size(size)
                    .color(color)
                    .into()
            }
            IconRepresentation::MaterialIcon(_) => {
                text(display)
                    .font(super::MATERIAL_ICONS)
                    .size(size)
                    .color(color)
                    .into()
            }
            IconRepresentation::UnicodeSymbol(_) | IconRepresentation::TextFallback(_) => {
                text(display)
                    .size(size)
                    .color(color)
                    .into()
            }
        }
    }
}

// Re-export semantic icon types for convenience through the verified module
// Use: icons::verified::StatusIcon, icons::verified::EntityIcon, etc.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verified_icon_lookup() {
        // Verify that the theme is created successfully
        let theme = verified::default_theme();
        assert!(!theme.icons().is_empty());
    }

    #[test]
    fn test_icon_names_available() {
        let theme = verified::default_theme();

        // Check core icons exist
        assert!(theme.icons().get("locked").is_some(), "locked icon missing");
        assert!(theme.icons().get("warning").is_some(), "warning icon missing");
        assert!(theme.icons().get("person").is_some(), "person icon missing");
        assert!(theme.icons().get("close").is_some(), "close icon missing");
    }
}
