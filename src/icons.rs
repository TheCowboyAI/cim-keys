//! Typography and Icons - Font Definitions
//!
//! Defines all fonts used in the CIM Keys application with explicit purposes

use iced::{widget::text, Color, Element, Font};

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

// Icon constants - emoji for visual clarity
pub const ICON_LOCK: char = '🔒';
pub const ICON_WARNING: char = '⚠';
pub const ICON_CHECK: char = '✓';
pub const ICON_CLOSE: char = '✕';
pub const ICON_BUSINESS: char = '🏢';
pub const ICON_GROUP: char = '👥';
pub const ICON_PERSON: char = '👤';
pub const ICON_LOCATION: char = '📍';
pub const ICON_SECURITY: char = '🔐';
pub const ICON_VERIFIED: char = '✅';
pub const ICON_VISIBILITY: char = '👁';
pub const ICON_VISIBILITY_OFF: char = '🙈';
// NATS Infrastructure icons
pub const ICON_CLOUD: char = '☁';
pub const ICON_ACCOUNT_CIRCLE: char = '👤';
pub const ICON_SETTINGS: char = '⚙';

// Progressive disclosure icons (folder metaphor)
pub const ICON_FOLDER: char = '📁';
pub const ICON_FOLDER_OPEN: char = '📂';

/// Render an icon with default size and emoji font
pub fn icon<'a, Message: 'a>(icon_char: char) -> Element<'a, Message> {
    text(icon_char)
        .font(EMOJI_FONT)
        .size(16)
        .into()
}

/// Render an icon with specific size and emoji font
pub fn icon_sized<'a, Message: 'a>(icon: char, size: u16) -> Element<'a, Message> {
    text(icon)
        .font(EMOJI_FONT)
        .size(size)
        .into()
}

/// Render an icon with specific size, color, and emoji font
pub fn icon_colored<'a, Message: 'a>(icon: char, size: u16, color: Color) -> Element<'a, Message> {
    text(icon)
        .font(EMOJI_FONT)
        .size(size)
        .color(color)
        .into()
}
