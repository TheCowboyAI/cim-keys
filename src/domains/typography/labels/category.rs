// Copyright (c) 2025 - Cowboy AI, LLC.

//! Label Categories
//!
//! Defines the categories of labels used in the UI.

use std::fmt;

/// Categories of labelled elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelCategory {
    /// Status indicators (success, error, warning, etc.)
    Status,
    /// Entity labels (person, organization, key, etc.)
    Entity,
    /// Action buttons/icons
    Action,
    /// Navigation elements
    Navigation,
    /// Informational tooltips
    Tooltip,
    /// Section headings
    Heading,
    /// Body text
    Body,
}

impl LabelCategory {
    /// Display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Entity => "Entity",
            Self::Action => "Action",
            Self::Navigation => "Navigation",
            Self::Tooltip => "Tooltip",
            Self::Heading => "Heading",
            Self::Body => "Body",
        }
    }

    /// Whether this category typically has an icon
    pub fn has_icon(&self) -> bool {
        matches!(self,
            Self::Status | Self::Entity | Self::Action | Self::Navigation
        )
    }

    /// Whether this category typically has text
    pub fn has_text(&self) -> bool {
        !matches!(self, Self::Status) // Status might be icon-only
    }

    /// All categories
    pub fn all() -> Vec<Self> {
        vec![
            Self::Status,
            Self::Entity,
            Self::Action,
            Self::Navigation,
            Self::Tooltip,
            Self::Heading,
            Self::Body,
        ]
    }
}

impl fmt::Display for LabelCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_properties() {
        assert!(LabelCategory::Status.has_icon());
        assert!(LabelCategory::Entity.has_icon());
        assert!(!LabelCategory::Body.has_icon());

        assert!(LabelCategory::Entity.has_text());
        assert!(LabelCategory::Body.has_text());
    }
}
