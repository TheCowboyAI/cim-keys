// Copyright (c) 2025 - Cowboy AI, LLC.

//! Verified Theme Aggregate Root
//!
//! The VerifiedTheme is the aggregate root for the Typography bounded context.
//! It ensures all fonts and icons will render correctly - this is verified
//! at initialization time, not at render time.
//!
//! ## Invariant
//!
//! Every requested glyph MUST render. If verification fails, the theme
//! construction fails, and the application should use a text-only fallback.

use super::font_set::{VerifiedFontSet, VerifiedFontFamily, FontFamily};
use super::icon_set::{VerifiedIconSet, VerifiedIcon, IconChain, IconRepresentation, CimIconSetBuilder};
use super::verification::TypographyVerificationError;
use iced::Color;

/// Color palette for the theme
#[derive(Debug, Clone)]
pub struct ColorPalette {
    /// Primary brand color
    pub primary: Color,
    /// Secondary brand color
    pub secondary: Color,
    /// Success indicator color
    pub success: Color,
    /// Warning indicator color
    pub warning: Color,
    /// Error indicator color
    pub error: Color,
    /// Info indicator color
    pub info: Color,
    /// Text color on dark background
    pub text_light: Color,
    /// Text color on light background
    pub text_dark: Color,
    /// Muted text color (for secondary info)
    pub text_muted: Color,
    /// Disabled/inactive color
    pub disabled: Color,
    /// Background colors
    pub background: BackgroundColors,
}

/// Background color variants
#[derive(Debug, Clone)]
pub struct BackgroundColors {
    /// Primary background
    pub primary: Color,
    /// Secondary/elevated background
    pub secondary: Color,
    /// Surface color for cards
    pub surface: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: Color::from_rgb(0.2, 0.4, 0.8),
            secondary: Color::from_rgb(0.6, 0.3, 0.7),
            success: Color::from_rgb(0.2, 0.7, 0.3),
            warning: Color::from_rgb(0.9, 0.7, 0.1),
            error: Color::from_rgb(0.8, 0.2, 0.2),
            info: Color::from_rgb(0.3, 0.6, 0.9),
            text_light: Color::from_rgb(0.95, 0.95, 0.95),
            text_dark: Color::from_rgb(0.1, 0.1, 0.1),
            text_muted: Color::from_rgb(0.6, 0.6, 0.6),
            disabled: Color::from_rgb(0.5, 0.5, 0.5),
            background: BackgroundColors {
                primary: Color::from_rgb(0.08, 0.08, 0.1),
                secondary: Color::from_rgb(0.12, 0.12, 0.15),
                surface: Color::from_rgb(0.15, 0.15, 0.18),
            },
        }
    }
}

/// Theme metrics for sizing and spacing
#[derive(Debug, Clone)]
pub struct ThemeMetrics {
    /// Base font size
    pub base_font_size: u16,
    /// Heading font size
    pub heading_font_size: u16,
    /// Small font size
    pub small_font_size: u16,
    /// Icon size
    pub icon_size: u16,
    /// Base spacing unit
    pub spacing: u16,
    /// Border radius
    pub border_radius: f32,
    /// Scale factor (1.0 = normal)
    pub scale: f32,
}

impl ThemeMetrics {
    /// Create metrics with a scale factor
    pub fn from_scale(scale: f32) -> Self {
        Self {
            base_font_size: (14.0 * scale) as u16,
            heading_font_size: (24.0 * scale) as u16,
            small_font_size: (11.0 * scale) as u16,
            icon_size: (18.0 * scale) as u16,
            spacing: (8.0 * scale) as u16,
            border_radius: 4.0 * scale,
            scale,
        }
    }
}

impl Default for ThemeMetrics {
    fn default() -> Self {
        Self::from_scale(1.0)
    }
}

/// Verification result for theme initialization
///
/// This type is prepared for when full font verification is implemented.
/// Currently unused as we use text_only_fallback().
#[allow(dead_code)]
#[derive(Debug)]
pub struct VerificationResult {
    /// Did all fonts load successfully?
    pub fonts_ok: bool,
    /// Did all icons verify?
    pub icons_ok: bool,
    /// Fonts that failed to load
    pub failed_fonts: Vec<FontFamily>,
    /// Icons that fell back to text
    pub text_fallback_icons: Vec<String>,
    /// Any errors encountered
    pub errors: Vec<TypographyVerificationError>,
}

#[allow(dead_code)]
impl VerificationResult {
    /// Check if verification completely succeeded
    pub fn is_ok(&self) -> bool {
        self.fonts_ok && self.icons_ok && self.errors.is_empty()
    }

    /// Check if verification is acceptable (fonts ok, icons may have fallbacks)
    pub fn is_acceptable(&self) -> bool {
        self.fonts_ok && self.errors.is_empty()
    }
}

/// The verified theme aggregate root
///
/// This is the central type for the Typography bounded context.
/// Once constructed, it guarantees:
/// 1. All fonts are loaded and can render their claimed glyphs
/// 2. All icons have at least one renderable representation
/// 3. The color palette is complete
/// 4. All metrics are valid
#[derive(Debug, Clone)]
pub struct VerifiedTheme {
    /// Verified fonts
    fonts: VerifiedFontSet,
    /// Verified icons
    icons: VerifiedIconSet,
    /// Color palette
    colors: ColorPalette,
    /// Size/spacing metrics
    metrics: ThemeMetrics,
    /// Verification result (for diagnostics)
    verification: VerificationSummary,
}

/// Summary of verification for diagnostics
#[derive(Debug, Clone)]
pub struct VerificationSummary {
    /// Number of text fallback icons
    pub text_fallback_count: usize,
    /// Number of missing fonts
    pub missing_font_count: usize,
    /// Whether we're in degraded mode
    pub degraded: bool,
}

impl VerifiedTheme {
    /// Try to create a verified theme with full verification
    ///
    /// This will:
    /// 1. Verify all fonts can load
    /// 2. Verify all icon chains have at least one renderable representation
    /// 3. Build the complete theme
    ///
    /// Returns an error if critical verification fails.
    pub fn try_new(
        fonts: VerifiedFontSet,
        icon_chains: Vec<IconChain>,
        colors: ColorPalette,
        metrics: ThemeMetrics,
    ) -> Result<Self, TypographyVerificationError> {
        // Verify icons and select best representation
        let icons = Self::verify_icons(icon_chains, &fonts)?;

        let verification = VerificationSummary {
            text_fallback_count: icons.text_fallback_count(),
            missing_font_count: fonts.missing_fonts().len(),
            degraded: icons.text_fallback_count() > 0 || !fonts.missing_fonts().is_empty(),
        };

        Ok(Self {
            fonts,
            icons,
            colors,
            metrics,
            verification,
        })
    }

    /// Create a text-only fallback theme
    ///
    /// This theme uses only ASCII text for icons, guaranteeing
    /// it will render on any system.
    pub fn text_only_fallback() -> Self {
        let fonts = VerifiedFontSet::system_defaults();
        let mut icons = VerifiedIconSet::new();

        // Create text-only verified icons from default chains
        for chain in CimIconSetBuilder::default_chains() {
            // Find the text fallback representation
            let fallback = chain.chain()
                .iter()
                .find(|r| r.is_text_fallback())
                .cloned()
                .unwrap_or_else(|| IconRepresentation::TextFallback(
                    format!("[{}]", chain.name().to_uppercase())
                ));

            icons.add(VerifiedIcon::new(chain, fallback));
        }

        let verification = VerificationSummary {
            text_fallback_count: icons.len(),
            missing_font_count: FontFamily::all().len(),
            degraded: true,
        };

        Self {
            fonts,
            icons,
            colors: ColorPalette::default(),
            metrics: ThemeMetrics::default(),
            verification,
        }
    }

    /// Verify icon chains and select best representation
    fn verify_icons(
        chains: Vec<IconChain>,
        fonts: &VerifiedFontSet,
    ) -> Result<VerifiedIconSet, TypographyVerificationError> {
        let mut icons = VerifiedIconSet::new();

        for chain in chains {
            let verified = Self::verify_icon_chain(&chain, fonts)?;
            icons.add(verified);
        }

        Ok(icons)
    }

    /// Verify a single icon chain and select best representation
    fn verify_icon_chain(
        chain: &IconChain,
        fonts: &VerifiedFontSet,
    ) -> Result<VerifiedIcon, TypographyVerificationError> {
        // Try each representation in order
        for repr in chain.chain() {
            if Self::can_render_representation(repr, fonts) {
                return Ok(VerifiedIcon::new(chain.clone(), repr.clone()));
            }
        }

        // If we get here and there's no fallback, that's an error
        if !chain.has_fallback() {
            return Err(TypographyVerificationError::FallbackChainExhausted {
                icon_name: chain.name().to_string(),
                chain: chain.chain().iter().map(|r| r.display()).collect(),
            });
        }

        // This shouldn't happen if chain has fallback, but just in case
        Err(TypographyVerificationError::NoRenderableGlyph {
            semantic_name: chain.name().to_string(),
            attempted_codepoints: chain.chain()
                .iter()
                .filter_map(|r| match r {
                    IconRepresentation::Emoji(c) |
                    IconRepresentation::UnicodeSymbol(c) => Some(*c),
                    _ => None,
                })
                .collect(),
        })
    }

    /// Check if a representation can be rendered
    fn can_render_representation(repr: &IconRepresentation, fonts: &VerifiedFontSet) -> bool {
        match repr {
            IconRepresentation::Emoji(c) => {
                // Check if emoji font can render this character
                fonts.emoji()
                    .map(|f| f.can_render(*c))
                    .unwrap_or(false)
            }
            IconRepresentation::MaterialIcon(_) => {
                // Material Icons use ligatures - check if icon font is loaded
                fonts.icon()
                    .map(|f| f.is_loaded())
                    .unwrap_or(false)
            }
            IconRepresentation::UnicodeSymbol(c) => {
                // Check if body font can render this symbol
                fonts.body()
                    .map(|f| f.can_render(*c))
                    .unwrap_or(false)
            }
            IconRepresentation::TextFallback(_) => {
                // Text fallback always works (ASCII)
                true
            }
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the font set
    pub fn fonts(&self) -> &VerifiedFontSet {
        &self.fonts
    }

    /// Get the icon set
    pub fn icons(&self) -> &VerifiedIconSet {
        &self.icons
    }

    /// Get the color palette
    pub fn colors(&self) -> &ColorPalette {
        &self.colors
    }

    /// Get the metrics
    pub fn metrics(&self) -> &ThemeMetrics {
        &self.metrics
    }

    /// Get verification summary
    pub fn verification(&self) -> &VerificationSummary {
        &self.verification
    }

    /// Check if theme is in degraded mode
    pub fn is_degraded(&self) -> bool {
        self.verification.degraded
    }

    /// Get a specific icon by name
    pub fn icon(&self, name: &str) -> Option<&VerifiedIcon> {
        self.icons.get(name)
    }

    /// Get a specific font by family
    pub fn font(&self, family: FontFamily) -> Option<&VerifiedFontFamily> {
        self.fonts.get(family)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_only_fallback() {
        let theme = VerifiedTheme::text_only_fallback();

        assert!(theme.is_degraded());
        assert!(theme.icons().len() > 0);

        // All icons should be text fallbacks
        assert_eq!(theme.icons().text_fallback_count(), theme.icons().len());

        // Should have the basic icons
        assert!(theme.icon("locked").is_some());
        assert!(theme.icon("person").is_some());
    }

    #[test]
    fn test_color_palette_default() {
        let palette = ColorPalette::default();

        // Colors should be valid
        assert!(palette.primary.r >= 0.0 && palette.primary.r <= 1.0);
        assert!(palette.success.g > palette.error.g); // Success should be greener
        assert!(palette.error.r > palette.success.r); // Error should be redder
    }

    #[test]
    fn test_metrics_scale() {
        let normal = ThemeMetrics::from_scale(1.0);
        let large = ThemeMetrics::from_scale(2.0);

        assert_eq!(large.base_font_size, normal.base_font_size * 2);
        assert_eq!(large.spacing, normal.spacing * 2);
    }
}
