// Copyright (c) 2025 - Cowboy AI, LLC.

//! Typography Entity Coproduct
//!
//! The sum type for typography domain entities. Following the categorical
//! structure: TypographyEntity = Font + Icon + Label
//!
//! Each variant is tagged with its injection morphism.

use super::injection::TypographyInjection;
use super::font_set::{VerifiedFontFamily, FontFamily};
use super::icon_set::{VerifiedIcon, SemanticIcon};
use super::labels::LabelledElement;
use uuid::Uuid;

/// Data carried by each typography entity variant
#[derive(Debug, Clone)]
pub enum TypographyData {
    // ========================================================================
    // Font Variants
    // ========================================================================

    /// A verified font family that can render its claimed glyphs
    Font(VerifiedFontFamily),

    // ========================================================================
    // Icon Variants
    // ========================================================================

    /// A verified icon with fallback chain
    Icon(VerifiedIcon),

    // ========================================================================
    // Label Variants
    // ========================================================================

    /// A labelled element (icon + text with styling)
    Label(LabelledElement),
}

/// Typography entity coproduct
///
/// Combines injection tag with data carrier, following the pattern
/// established by other bounded contexts (Organization, PKI, etc.)
#[derive(Debug, Clone)]
pub struct TypographyEntity {
    /// The injection morphism tag
    injection: TypographyInjection,
    /// The inner data
    data: TypographyData,
    /// Unique identifier for this typography entity
    id: Uuid,
}

impl TypographyEntity {
    // ========================================================================
    // Injection Morphisms (Constructors)
    // ========================================================================

    /// Inject a verified font family
    pub fn inject_font(font: VerifiedFontFamily, family: FontFamily) -> Self {
        let injection = match family {
            FontFamily::Body => TypographyInjection::BodyFont,
            FontFamily::Heading => TypographyInjection::HeadingFont,
            FontFamily::Icon => TypographyInjection::IconFont,
            FontFamily::Emoji => TypographyInjection::EmojiFont,
            FontFamily::Monospace => TypographyInjection::MonospaceFont,
        };
        Self {
            injection,
            data: TypographyData::Font(font),
            id: Uuid::now_v7(),
        }
    }

    /// Inject a verified icon
    pub fn inject_icon(icon: VerifiedIcon, semantic: SemanticIcon) -> Self {
        let injection = match semantic {
            SemanticIcon::Status(_) => TypographyInjection::StatusIndicator,
            SemanticIcon::Navigation(_) => TypographyInjection::NavigationIcon,
            SemanticIcon::Action(_) => TypographyInjection::ActionIcon,
            SemanticIcon::Entity(_) => TypographyInjection::EntityLabel,
        };
        Self {
            injection,
            data: TypographyData::Icon(icon),
            id: Uuid::now_v7(),
        }
    }

    /// Inject a labelled element
    pub fn inject_label(label: LabelledElement) -> Self {
        Self {
            injection: TypographyInjection::EntityLabel,
            data: TypographyData::Label(label),
            id: Uuid::now_v7(),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the injection tag
    pub fn injection(&self) -> TypographyInjection {
        self.injection
    }

    /// Get the inner data
    pub fn data(&self) -> &TypographyData {
        &self.data
    }

    /// Get the entity ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Check if this is a font entity
    pub fn is_font(&self) -> bool {
        matches!(self.data, TypographyData::Font(_))
    }

    /// Check if this is an icon entity
    pub fn is_icon(&self) -> bool {
        matches!(self.data, TypographyData::Icon(_))
    }

    /// Check if this is a label entity
    pub fn is_label(&self) -> bool {
        matches!(self.data, TypographyData::Label(_))
    }

    /// Try to extract font data
    pub fn as_font(&self) -> Option<&VerifiedFontFamily> {
        match &self.data {
            TypographyData::Font(f) => Some(f),
            _ => None,
        }
    }

    /// Try to extract icon data
    pub fn as_icon(&self) -> Option<&VerifiedIcon> {
        match &self.data {
            TypographyData::Icon(i) => Some(i),
            _ => None,
        }
    }

    /// Try to extract label data
    pub fn as_label(&self) -> Option<&LabelledElement> {
        match &self.data {
            TypographyData::Label(l) => Some(l),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::typography::icon_set::{IconRepresentation, IconChain, StatusIcon};

    #[test]
    fn test_inject_icon() {
        let chain = IconChain::new("test")
            .try_emoji('🔒')
            .fallback_text("[LOCK]");
        let icon = VerifiedIcon::new(chain, IconRepresentation::TextFallback("[LOCK]".to_string()));
        let entity = TypographyEntity::inject_icon(icon, SemanticIcon::Status(StatusIcon::Locked));

        assert!(entity.is_icon());
        assert!(!entity.is_font());
        assert!(!entity.is_label());
        assert_eq!(entity.injection(), TypographyInjection::StatusIndicator);
    }
}
