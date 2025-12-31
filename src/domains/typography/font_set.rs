// Copyright (c) 2025 - Cowboy AI, LLC.

//! Font Set Types
//!
//! Defines verified font families and font sets that guarantee
//! glyph coverage for required Unicode blocks.

use super::verification::GlyphCoverage;
use std::collections::HashMap;

/// Font family categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// Body text (readable, monospace for data)
    Body,
    /// Headings and display text
    Heading,
    /// Icon fonts (Material Icons, etc.)
    Icon,
    /// Emoji fonts (Noto Color Emoji, etc.)
    Emoji,
    /// Monospace for code and technical data
    Monospace,
}

impl FontFamily {
    /// Display name for this font family
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Body => "Body",
            Self::Heading => "Heading",
            Self::Icon => "Icon",
            Self::Emoji => "Emoji",
            Self::Monospace => "Monospace",
        }
    }

    /// All font families
    pub fn all() -> Vec<Self> {
        vec![
            Self::Body,
            Self::Heading,
            Self::Icon,
            Self::Emoji,
            Self::Monospace,
        ]
    }
}

/// A verified font family that has been checked for glyph coverage
#[derive(Debug, Clone)]
pub struct VerifiedFontFamily {
    /// Font family name (e.g., "Rec Mono Linear")
    name: String,
    /// Font family category
    family: FontFamily,
    /// Verified glyph coverage
    coverage: GlyphCoverage,
    /// Whether this font successfully loaded
    loaded: bool,
}

impl VerifiedFontFamily {
    /// Create a new verified font family
    pub fn new(name: impl Into<String>, family: FontFamily, coverage: GlyphCoverage) -> Self {
        Self {
            name: name.into(),
            family,
            coverage,
            loaded: true,
        }
    }

    /// Create a fallback (unverified) font
    pub fn fallback(family: FontFamily) -> Self {
        let name = match family {
            FontFamily::Body => "System Default",
            FontFamily::Heading => "System Default",
            FontFamily::Icon => "System Default",
            FontFamily::Emoji => "System Default",
            FontFamily::Monospace => "Monospace",
        };
        Self {
            name: name.to_string(),
            family,
            coverage: GlyphCoverage::basic_latin(),
            loaded: false,
        }
    }

    /// Get the font name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the font family
    pub fn family(&self) -> FontFamily {
        self.family
    }

    /// Get the glyph coverage
    pub fn coverage(&self) -> &GlyphCoverage {
        &self.coverage
    }

    /// Check if the font was successfully loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Check if this font can render a character
    pub fn can_render(&self, c: char) -> bool {
        self.coverage.contains(c)
    }
}

/// A complete set of verified fonts for a theme
#[derive(Debug, Clone)]
pub struct VerifiedFontSet {
    /// Fonts indexed by family
    fonts: HashMap<FontFamily, VerifiedFontFamily>,
}

impl VerifiedFontSet {
    /// Create a new empty font set
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    /// Create a font set with system defaults (ASCII only)
    pub fn system_defaults() -> Self {
        let mut fonts = HashMap::new();
        for family in FontFamily::all() {
            fonts.insert(family, VerifiedFontFamily::fallback(family));
        }
        Self { fonts }
    }

    /// Add a verified font to the set
    pub fn add(&mut self, font: VerifiedFontFamily) {
        self.fonts.insert(font.family(), font);
    }

    /// Get a font by family
    pub fn get(&self, family: FontFamily) -> Option<&VerifiedFontFamily> {
        self.fonts.get(&family)
    }

    /// Get body font
    pub fn body(&self) -> Option<&VerifiedFontFamily> {
        self.get(FontFamily::Body)
    }

    /// Get heading font
    pub fn heading(&self) -> Option<&VerifiedFontFamily> {
        self.get(FontFamily::Heading)
    }

    /// Get icon font
    pub fn icon(&self) -> Option<&VerifiedFontFamily> {
        self.get(FontFamily::Icon)
    }

    /// Get emoji font
    pub fn emoji(&self) -> Option<&VerifiedFontFamily> {
        self.get(FontFamily::Emoji)
    }

    /// Get monospace font
    pub fn monospace(&self) -> Option<&VerifiedFontFamily> {
        self.get(FontFamily::Monospace)
    }

    /// Check if all required fonts are loaded
    pub fn all_loaded(&self) -> bool {
        FontFamily::all().iter().all(|f| {
            self.fonts.get(f).map(|font| font.is_loaded()).unwrap_or(false)
        })
    }

    /// Get list of missing or failed fonts
    pub fn missing_fonts(&self) -> Vec<FontFamily> {
        FontFamily::all()
            .into_iter()
            .filter(|f| {
                self.fonts.get(f).map(|font| !font.is_loaded()).unwrap_or(true)
            })
            .collect()
    }
}

impl Default for VerifiedFontSet {
    fn default() -> Self {
        Self::system_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verified_font_family() {
        let font = VerifiedFontFamily::new(
            "Rec Mono Linear",
            FontFamily::Body,
            GlyphCoverage::basic_latin(),
        );

        assert_eq!(font.name(), "Rec Mono Linear");
        assert_eq!(font.family(), FontFamily::Body);
        assert!(font.is_loaded());
        assert!(font.can_render('A'));
        assert!(!font.can_render('😀'));
    }

    #[test]
    fn test_font_set_defaults() {
        let set = VerifiedFontSet::system_defaults();

        assert!(set.body().is_some());
        assert!(set.heading().is_some());
        assert!(set.icon().is_some());
        assert!(set.emoji().is_some());
        assert!(set.monospace().is_some());

        // System defaults are not fully loaded
        assert!(!set.all_loaded());
    }

    #[test]
    fn test_font_set_custom() {
        let mut set = VerifiedFontSet::new();
        set.add(VerifiedFontFamily::new(
            "Custom Body",
            FontFamily::Body,
            GlyphCoverage::basic_latin(),
        ));

        assert!(set.body().is_some());
        assert_eq!(set.body().unwrap().name(), "Custom Body");
        assert!(set.heading().is_none());
    }
}
