// Copyright (c) 2025 - Cowboy AI, LLC.

//! Label Specification
//!
//! Defines specifications for how labels should be rendered,
//! including icon, text, and styling options.

use super::category::LabelCategory;
use super::element::LabelledElement;
use crate::domains::typography::icon_set::SemanticIcon;
use crate::domains::typography::font_set::FontFamily;
use crate::domains::typography::verified_theme::VerifiedTheme;
use iced::Color;

/// Specification for creating a labelled element
#[derive(Debug, Clone)]
pub struct LabelSpec {
    /// Category of label
    category: LabelCategory,
    /// Semantic icon (if any)
    icon: Option<SemanticIcon>,
    /// Text content (if any)
    text: Option<String>,
    /// Font family override
    font_family: Option<FontFamily>,
    /// Color override
    color: Option<Color>,
    /// Size override (in pixels)
    size: Option<u16>,
    /// Whether to show icon
    show_icon: bool,
    /// Whether to show text
    show_text: bool,
}

impl LabelSpec {
    /// Create a new label spec
    pub fn new(category: LabelCategory) -> Self {
        Self {
            category,
            icon: None,
            text: None,
            font_family: None,
            color: None,
            size: None,
            show_icon: category.has_icon(),
            show_text: category.has_text(),
        }
    }

    /// Create a status label
    pub fn status(icon: SemanticIcon) -> Self {
        Self::new(LabelCategory::Status).with_icon(icon)
    }

    /// Create an entity label
    pub fn entity(icon: SemanticIcon, text: impl Into<String>) -> Self {
        Self::new(LabelCategory::Entity)
            .with_icon(icon)
            .with_text(text)
    }

    /// Create an action label
    pub fn action(icon: SemanticIcon, text: impl Into<String>) -> Self {
        Self::new(LabelCategory::Action)
            .with_icon(icon)
            .with_text(text)
    }

    /// Create a text-only label
    pub fn text_only(category: LabelCategory, text: impl Into<String>) -> Self {
        Self::new(category)
            .with_text(text)
            .without_icon()
    }

    /// Create an icon-only label
    pub fn icon_only(category: LabelCategory, icon: SemanticIcon) -> Self {
        Self::new(category)
            .with_icon(icon)
            .without_text()
    }

    // Builder methods

    /// Set the semantic icon
    pub fn with_icon(mut self, icon: SemanticIcon) -> Self {
        self.icon = Some(icon);
        self.show_icon = true;
        self
    }

    /// Set the text content
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self.show_text = true;
        self
    }

    /// Set font family override
    pub fn with_font(mut self, family: FontFamily) -> Self {
        self.font_family = Some(family);
        self
    }

    /// Set color override
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set size override
    pub fn with_size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Hide the icon
    pub fn without_icon(mut self) -> Self {
        self.show_icon = false;
        self
    }

    /// Hide the text
    pub fn without_text(mut self) -> Self {
        self.show_text = false;
        self
    }

    // Accessors

    /// Get the category
    pub fn category(&self) -> LabelCategory {
        self.category
    }

    /// Get the semantic icon
    pub fn icon(&self) -> Option<SemanticIcon> {
        self.icon
    }

    /// Get the text
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Check if icon should be shown
    pub fn should_show_icon(&self) -> bool {
        self.show_icon && self.icon.is_some()
    }

    /// Check if text should be shown
    pub fn should_show_text(&self) -> bool {
        self.show_text && self.text.is_some()
    }

    /// Resolve this spec against a theme to create a LabelledElement
    pub fn resolve(&self, theme: &VerifiedTheme) -> LabelledElement {
        // Get the verified icon
        let verified_icon = if self.should_show_icon() {
            self.icon.and_then(|semantic| {
                theme.icon(semantic.name()).cloned()
            })
        } else {
            None
        };

        // Get the text
        let text = if self.should_show_text() {
            self.text.clone()
        } else {
            None
        };

        // Resolve font family
        let font_family = self.font_family.unwrap_or(match self.category {
            LabelCategory::Heading => FontFamily::Heading,
            LabelCategory::Body | LabelCategory::Tooltip => FontFamily::Body,
            _ => FontFamily::Body,
        });

        // Resolve color
        let color = self.color.unwrap_or(theme.colors().text_light);

        // Resolve size
        let size = self.size.unwrap_or(match self.category {
            LabelCategory::Heading => theme.metrics().heading_font_size,
            LabelCategory::Tooltip => theme.metrics().small_font_size,
            _ => theme.metrics().base_font_size,
        });

        LabelledElement::new(
            self.category,
            verified_icon,
            text,
            font_family,
            color,
            size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::typography::icon_set::StatusIcon;

    #[test]
    fn test_status_spec() {
        let spec = LabelSpec::status(SemanticIcon::Status(StatusIcon::Success));

        assert_eq!(spec.category(), LabelCategory::Status);
        assert!(spec.should_show_icon());
        assert!(!spec.should_show_text());
    }

    #[test]
    fn test_entity_spec() {
        let spec = LabelSpec::entity(
            SemanticIcon::Entity(crate::domains::typography::icon_set::EntityIcon::Person),
            "John Doe",
        );

        assert_eq!(spec.category(), LabelCategory::Entity);
        assert!(spec.should_show_icon());
        assert!(spec.should_show_text());
        assert_eq!(spec.text(), Some("John Doe"));
    }

    #[test]
    fn test_builder_pattern() {
        let spec = LabelSpec::new(LabelCategory::Action)
            .with_icon(SemanticIcon::Action(crate::domains::typography::icon_set::ActionIcon::Save))
            .with_text("Save")
            .with_color(Color::from_rgb(0.0, 0.5, 1.0))
            .with_size(16);

        assert!(spec.should_show_icon());
        assert!(spec.should_show_text());
        assert_eq!(spec.text(), Some("Save"));
    }
}
