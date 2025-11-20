//! Material Icons constants
//!
//! Using Material Icons instead of Unicode emojis for better rendering.
//! Font file: assets/fonts/MaterialIcons-Regular.ttf

/// Material Icons font family name
pub const MATERIAL_ICONS: iced::Font = iced::Font {
    family: iced::font::Family::Name("Material Icons"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

// Common icons used in CIM Keys
pub const ICON_LOCK: &str = "\u{e897}";           // lock (🔐 replacement)
pub const ICON_FOLDER: &str = "\u{e2c7}";         // folder (📁 replacement)
pub const ICON_WARNING: &str = "\u{e002}";        // warning (⚠️ replacement)
pub const ICON_CHECK: &str = "\u{e5ca}";          // check (✓ replacement)
pub const ICON_CHECK_CIRCLE: &str = "\u{e86c}";   // check_circle
pub const ICON_ROCKET: &str = "\u{e320}";         // rocket_launch (🚀 replacement)
pub const ICON_KEY: &str = "\u{e73c}";            // vpn_key
pub const ICON_SECURITY: &str = "\u{e32a}";       // security
pub const ICON_VERIFIED: &str = "\u{ef76}";       // verified_user
pub const ICON_ERROR: &str = "\u{e000}";          // error
pub const ICON_INFO: &str = "\u{e88e}";           // info
pub const ICON_PERSON: &str = "\u{e7fd}";         // person
pub const ICON_GROUP: &str = "\u{e7ef}";          // group
pub const ICON_BUSINESS: &str = "\u{e0af}";       // business
pub const ICON_LOCATION: &str = "\u{e55f}";       // location_on
pub const ICON_SAVE: &str = "\u{e161}";           // save
pub const ICON_DOWNLOAD: &str = "\u{f090}";       // download
pub const ICON_UPLOAD: &str = "\u{f09b}";         // upload
pub const ICON_SETTINGS: &str = "\u{e8b8}";       // settings
pub const ICON_VISIBILITY: &str = "\u{e8f4}";     // visibility
pub const ICON_VISIBILITY_OFF: &str = "\u{e8f5}"; // visibility_off
pub const ICON_DELETE: &str = "\u{e872}";         // delete
pub const ICON_EDIT: &str = "\u{e3c9}";           // edit
pub const ICON_ADD: &str = "\u{e145}";            // add
pub const ICON_REMOVE: &str = "\u{e15b}";         // remove
pub const ICON_CLOSE: &str = "\u{e5cd}";          // close
pub const ICON_DONE: &str = "\u{e876}";           // done
pub const ICON_MENU: &str = "\u{e5d2}";           // menu
pub const ICON_MORE: &str = "\u{e5d3}";           // more_vert
pub const ICON_REFRESH: &str = "\u{e5d5}";        // refresh
pub const ICON_SEARCH: &str = "\u{e8b6}";         // search
pub const ICON_HELP: &str = "\u{e887}";           // help
pub const ICON_HOME: &str = "\u{e88a}";           // home
pub const ICON_ARROW_BACK: &str = "\u{e5c4}";     // arrow_back
pub const ICON_ARROW_FORWARD: &str = "\u{e5c8}";  // arrow_forward
pub const ICON_EXPAND_MORE: &str = "\u{e5cf}";    // expand_more
pub const ICON_EXPAND_LESS: &str = "\u{e5ce}";    // expand_less

/// Helper function to create icon text with Material Icons font
pub fn icon(code: &str) -> iced::widget::Text<'static> {
    iced::widget::text(code.to_string())
        .font(MATERIAL_ICONS)
}

/// Helper function to create icon text with size
pub fn icon_sized(code: &str, size: u16) -> iced::widget::Text<'static> {
    iced::widget::text(code.to_string())
        .font(MATERIAL_ICONS)
        .size(size)
}

/// Helper function to create colored icon
pub fn icon_colored(code: &str, size: u16, color: iced::Color) -> iced::widget::Text<'static> {
    iced::widget::text(code.to_string())
        .font(MATERIAL_ICONS)
        .size(size)
        .color(color)
}
