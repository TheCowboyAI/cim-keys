// Copyright (c) 2025 - Cowboy AI, LLC.

//! Typography Fold Trait
//!
//! The universal property of the Typography coproduct. For any type X
//! with morphisms from Font, Icon, and Label, there exists a unique
//! morphism from TypographyEntity to X.
//!
//! This is the categorical fold (catamorphism) that enables type-safe
//! dispatch without pattern matching at call sites.

use super::entity::{TypographyEntity, TypographyData};
use super::font_set::VerifiedFontFamily;
use super::icon_set::VerifiedIcon;
use super::labels::LabelledElement;
use super::injection::TypographyInjection;

/// Universal property trait for Typography coproduct
///
/// Implement this trait to define how to consume typography entities.
/// The fold method dispatches to the appropriate handler based on the
/// entity's injection tag.
pub trait FoldTypographyEntity {
    /// The output type of the fold
    type Output;

    /// Fold a font entity
    fn fold_font(&self, font: &VerifiedFontFamily, injection: TypographyInjection) -> Self::Output;

    /// Fold an icon entity
    fn fold_icon(&self, icon: &VerifiedIcon, injection: TypographyInjection) -> Self::Output;

    /// Fold a label entity
    fn fold_label(&self, label: &LabelledElement, injection: TypographyInjection) -> Self::Output;

    /// Execute the fold on a typography entity
    ///
    /// This is the universal morphism - given any entity, dispatch to
    /// the appropriate handler based on its data variant.
    fn fold(&self, entity: &TypographyEntity) -> Self::Output {
        let injection = entity.injection();
        match entity.data() {
            TypographyData::Font(font) => self.fold_font(font, injection),
            TypographyData::Icon(icon) => self.fold_icon(icon, injection),
            TypographyData::Label(label) => self.fold_label(label, injection),
        }
    }
}

/// Identity fold - returns the entity unchanged (for testing fold laws)
pub struct IdentityFold;

impl FoldTypographyEntity for IdentityFold {
    type Output = TypographyData;

    fn fold_font(&self, font: &VerifiedFontFamily, _injection: TypographyInjection) -> Self::Output {
        TypographyData::Font(font.clone())
    }

    fn fold_icon(&self, icon: &VerifiedIcon, _injection: TypographyInjection) -> Self::Output {
        TypographyData::Icon(icon.clone())
    }

    fn fold_label(&self, label: &LabelledElement, _injection: TypographyInjection) -> Self::Output {
        TypographyData::Label(label.clone())
    }
}

/// String representation fold - converts entities to display strings
pub struct ToStringFold;

impl FoldTypographyEntity for ToStringFold {
    type Output = String;

    fn fold_font(&self, font: &VerifiedFontFamily, injection: TypographyInjection) -> Self::Output {
        format!("[{}] Font: {}", injection.display_name(), font.name())
    }

    fn fold_icon(&self, icon: &VerifiedIcon, injection: TypographyInjection) -> Self::Output {
        format!("[{}] Icon: {}", injection.display_name(), icon.display())
    }

    fn fold_label(&self, label: &LabelledElement, injection: TypographyInjection) -> Self::Output {
        format!("[{}] Label: {}", injection.display_name(), label.text().unwrap_or("(no text)"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::typography::icon_set::{IconRepresentation, IconChain, SemanticIcon, StatusIcon};
    use crate::domains::typography::entity::TypographyEntity;

    #[test]
    fn test_to_string_fold() {
        let chain = IconChain::new("lock")
            .try_emoji('🔒')
            .fallback_text("[LOCK]");
        let icon = VerifiedIcon::new(chain, IconRepresentation::TextFallback("[LOCK]".to_string()));
        let entity = TypographyEntity::inject_icon(icon, SemanticIcon::Status(StatusIcon::Locked));

        let fold = ToStringFold;
        let result = fold.fold(&entity);

        assert!(result.contains("Status Indicator"));
        assert!(result.contains("Icon"));
    }

    #[test]
    fn test_identity_fold_preserves_structure() {
        let chain = IconChain::new("test")
            .fallback_text("[TEST]");
        let icon = VerifiedIcon::new(chain, IconRepresentation::TextFallback("[TEST]".to_string()));
        let entity = TypographyEntity::inject_icon(icon, SemanticIcon::Status(StatusIcon::Success));

        let fold = IdentityFold;
        let result = fold.fold(&entity);

        assert!(matches!(result, TypographyData::Icon(_)));
    }
}
