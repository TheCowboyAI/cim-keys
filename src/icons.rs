//! Material Icons - Helper Module
//!
//! Provides icon rendering functions for the GUI using Unicode characters

use iced::{widget::text, Color, Element, Font};

// Font with emoji support - uses system emoji fonts
pub const EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji"); // Windows
// Fallback chain: Linux uses "Noto Color Emoji", macOS uses "Apple Color Emoji"

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

// Material Icons font
pub const MATERIAL_ICONS: Font = Font::DEFAULT;

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
