// Copyright (c) 2025 - Cowboy AI, LLC.

//! Typography Injection Tags
//!
//! Defines the injection morphisms for the Typography coproduct.
//! Each variant represents a category of typographic element.

use std::fmt;

/// Injection tag for Typography bounded context
///
/// Each variant represents a morphism from a specific type into
/// the TypographyEntity coproduct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypographyInjection {
    // ========================================================================
    // Font Families
    // ========================================================================

    /// Body text font (monospace, data-friendly)
    BodyFont,
    /// Heading/display font (emphasis)
    HeadingFont,
    /// Icon font (Material Icons, etc.)
    IconFont,
    /// Emoji font (color emoji rendering)
    EmojiFont,
    /// Monospace font (code, technical data)
    MonospaceFont,

    // ========================================================================
    // Icon Sets
    // ========================================================================

    /// Material Design icon
    MaterialIcon,
    /// Unicode emoji icon
    EmojiIcon,
    /// Unicode symbol (basic symbol fonts)
    UnicodeSymbol,
    /// Text fallback (always works)
    TextFallback,

    // ========================================================================
    // Labelled Elements
    // ========================================================================

    /// Status indicator (success, warning, error, info)
    StatusIndicator,
    /// Navigation icon (menu, back, forward)
    NavigationIcon,
    /// Action button icon (save, delete, edit)
    ActionIcon,
    /// Entity label (person, organization, key, etc.)
    EntityLabel,
    /// Tooltip content
    TooltipLabel,
}

impl TypographyInjection {
    /// Display name for this typography element type
    pub fn display_name(&self) -> &'static str {
        match self {
            // Fonts
            Self::BodyFont => "Body Font",
            Self::HeadingFont => "Heading Font",
            Self::IconFont => "Icon Font",
            Self::EmojiFont => "Emoji Font",
            Self::MonospaceFont => "Monospace Font",

            // Icons
            Self::MaterialIcon => "Material Icon",
            Self::EmojiIcon => "Emoji Icon",
            Self::UnicodeSymbol => "Unicode Symbol",
            Self::TextFallback => "Text Fallback",

            // Labels
            Self::StatusIndicator => "Status Indicator",
            Self::NavigationIcon => "Navigation Icon",
            Self::ActionIcon => "Action Icon",
            Self::EntityLabel => "Entity Label",
            Self::TooltipLabel => "Tooltip",
        }
    }

    /// Category of this injection (for grouping)
    pub fn category(&self) -> TypographyCategory {
        match self {
            Self::BodyFont | Self::HeadingFont | Self::IconFont |
            Self::EmojiFont | Self::MonospaceFont => TypographyCategory::Font,

            Self::MaterialIcon | Self::EmojiIcon |
            Self::UnicodeSymbol | Self::TextFallback => TypographyCategory::Icon,

            Self::StatusIndicator | Self::NavigationIcon |
            Self::ActionIcon | Self::EntityLabel | Self::TooltipLabel => TypographyCategory::Label,
        }
    }

    /// All injection variants
    pub fn all() -> Vec<Self> {
        vec![
            Self::BodyFont,
            Self::HeadingFont,
            Self::IconFont,
            Self::EmojiFont,
            Self::MonospaceFont,
            Self::MaterialIcon,
            Self::EmojiIcon,
            Self::UnicodeSymbol,
            Self::TextFallback,
            Self::StatusIndicator,
            Self::NavigationIcon,
            Self::ActionIcon,
            Self::EntityLabel,
            Self::TooltipLabel,
        ]
    }
}

impl fmt::Display for TypographyInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Category of typography elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypographyCategory {
    /// Font family definitions
    Font,
    /// Icon representations
    Icon,
    /// Labelled elements (icon + text)
    Label,
}

impl TypographyCategory {
    /// Display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Font => "Fonts",
            Self::Icon => "Icons",
            Self::Label => "Labels",
        }
    }
}

impl fmt::Display for TypographyCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
