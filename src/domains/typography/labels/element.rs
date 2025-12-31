// Copyright (c) 2025 - Cowboy AI, LLC.

//! Labelled Element
//!
//! The rendered output of a label specification resolved against
//! a verified theme. Contains all the information needed to render
//! the label in Iced.

use super::category::LabelCategory;
use crate::domains::typography::icon_set::VerifiedIcon;
use crate::domains::typography::font_set::FontFamily;
use iced::Color;

/// A labelled element ready for rendering
///
/// This is the output of resolving a LabelSpec against a VerifiedTheme.
/// All icons and fonts have been verified to render correctly.
#[derive(Debug, Clone)]
pub struct LabelledElement {
    /// Category of this label
    category: LabelCategory,
    /// Verified icon (if any)
    icon: Option<VerifiedIcon>,
    /// Text content (if any)
    text: Option<String>,
    /// Font family to use
    font_family: FontFamily,
    /// Color to use
    color: Color,
    /// Size in pixels
    size: u16,
}

impl LabelledElement {
    /// Create a new labelled element
    pub fn new(
        category: LabelCategory,
        icon: Option<VerifiedIcon>,
        text: Option<String>,
        font_family: FontFamily,
        color: Color,
        size: u16,
    ) -> Self {
        Self {
            category,
            icon,
            text,
            font_family,
            color,
            size,
        }
    }

    /// Create a simple text label
    pub fn text_label(text: impl Into<String>) -> Self {
        Self {
            category: LabelCategory::Body,
            icon: None,
            text: Some(text.into()),
            font_family: FontFamily::Body,
            color: Color::WHITE,
            size: 14,
        }
    }

    /// Create an icon-only label
    pub fn icon_label(icon: VerifiedIcon, category: LabelCategory) -> Self {
        Self {
            category,
            icon: Some(icon),
            text: None,
            font_family: FontFamily::Icon,
            color: Color::WHITE,
            size: 18,
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the category
    pub fn category(&self) -> LabelCategory {
        self.category
    }

    /// Get the verified icon
    pub fn icon(&self) -> Option<&VerifiedIcon> {
        self.icon.as_ref()
    }

    /// Get the icon display string
    pub fn icon_display(&self) -> Option<String> {
        self.icon.as_ref().map(|i| i.display())
    }

    /// Get the text content
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Get the font family
    pub fn font_family(&self) -> FontFamily {
        self.font_family
    }

    /// Get the color
    pub fn color(&self) -> Color {
        self.color
    }

    /// Get the size
    pub fn size(&self) -> u16 {
        self.size
    }

    /// Check if this has an icon
    pub fn has_icon(&self) -> bool {
        self.icon.is_some()
    }

    /// Check if this has text
    pub fn has_text(&self) -> bool {
        self.text.is_some()
    }

    /// Check if the icon fell back to text
    pub fn icon_is_text_fallback(&self) -> bool {
        self.icon.as_ref().map(|i| i.is_text_fallback()).unwrap_or(false)
    }

    // ========================================================================
    // Builder methods for modifying
    // ========================================================================

    /// Create a copy with different color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Create a copy with different size
    pub fn with_size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Create a copy with additional text
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    // ========================================================================
    // Rendering helpers
    // ========================================================================

    /// Get the full display string (icon + text)
    pub fn full_display(&self) -> String {
        match (&self.icon, &self.text) {
            (Some(icon), Some(text)) => format!("{} {}", icon.display(), text),
            (Some(icon), None) => icon.display(),
            (None, Some(text)) => text.clone(),
            (None, None) => String::new(),
        }
    }

    /// Get just the icon display or empty string
    pub fn icon_or_empty(&self) -> String {
        self.icon.as_ref().map(|i| i.display()).unwrap_or_default()
    }

    /// Get just the text or empty string
    pub fn text_or_empty(&self) -> &str {
        self.text.as_deref().unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::typography::icon_set::{IconChain, IconRepresentation};

    fn make_test_icon() -> VerifiedIcon {
        let chain = IconChain::new("test")
            .try_emoji('✓')
            .fallback_text("[OK]");
        VerifiedIcon::new(chain, IconRepresentation::Emoji('✓'))
    }

    #[test]
    fn test_text_label() {
        let label = LabelledElement::text_label("Hello");

        assert!(!label.has_icon());
        assert!(label.has_text());
        assert_eq!(label.text(), Some("Hello"));
        assert_eq!(label.full_display(), "Hello");
    }

    #[test]
    fn test_icon_label() {
        let icon = make_test_icon();
        let label = LabelledElement::icon_label(icon, LabelCategory::Status);

        assert!(label.has_icon());
        assert!(!label.has_text());
        assert_eq!(label.icon_display(), Some("✓".to_string()));
    }

    #[test]
    fn test_full_label() {
        let icon = make_test_icon();
        let label = LabelledElement::new(
            LabelCategory::Entity,
            Some(icon),
            Some("Success".to_string()),
            FontFamily::Body,
            Color::WHITE,
            14,
        );

        assert!(label.has_icon());
        assert!(label.has_text());
        assert_eq!(label.full_display(), "✓ Success");
    }

    #[test]
    fn test_with_methods() {
        let label = LabelledElement::text_label("Test")
            .with_color(Color::from_rgb(1.0, 0.0, 0.0))
            .with_size(20);

        assert_eq!(label.size(), 20);
        assert_eq!(label.color().r, 1.0);
    }
}
