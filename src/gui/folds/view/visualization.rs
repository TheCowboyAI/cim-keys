// Copyright (c) 2025 - Cowboy AI, LLC.

//! Visualization Data Types
//!
//! This module provides data structures for visualization without the
//! deprecated FoldDomainNode trait. For visualization of LiftedNode,
//! use `LiftedNode::themed_visualization()` instead.
//!
//! ## Color Palette
//!
//! The palette module provides centralized color constants for consistent
//! rendering across the application.

use iced::Color;

// ============================================================================
// OUTPUT TYPE
// ============================================================================

/// Visualization data extracted from a domain node.
///
/// Contains all visual properties needed for rendering without any
/// domain-specific business logic. This separation ensures the view
/// layer remains pure.
#[derive(Debug, Clone)]
pub struct VisualizationData {
    /// Display label for the node
    pub label: String,
    /// Subtitle or secondary information
    pub subtitle: String,
    /// Primary color for rendering
    pub color: Color,
    /// Icon identifier (maps to icon assets)
    pub icon: String,
    /// Tooltip text for hover states
    pub tooltip: String,
    /// Whether this node is expandable (has children)
    pub expandable: bool,
}

impl VisualizationData {
    /// Create empty visualization data
    pub fn empty() -> Self {
        Self {
            label: String::new(),
            subtitle: String::new(),
            color: Color::from_rgb(0.5, 0.5, 0.5),
            icon: "unknown".to_string(),
            tooltip: String::new(),
            expandable: false,
        }
    }
}

// ============================================================================
// COLOR PALETTE (Centralized for consistency)
// ============================================================================

pub mod palette {
    use iced::Color;

    // Organization context
    pub const ORGANIZATION: Color = Color::from_rgb(0.2, 0.4, 0.8);
    pub const ORGANIZATION_UNIT: Color = Color::from_rgb(0.3, 0.5, 0.7);
    pub const LOCATION: Color = Color::from_rgb(0.4, 0.6, 0.4);

    // People context - by KeyOwnerRole
    pub const PERSON_ROOT: Color = Color::from_rgb(0.8, 0.2, 0.2);       // Red - highest authority
    pub const PERSON_SECURITY: Color = Color::from_rgb(0.8, 0.5, 0.2);   // Orange - security admin
    pub const PERSON_DEVELOPER: Color = Color::from_rgb(0.3, 0.6, 0.3);  // Green - developer
    pub const PERSON_SERVICE: Color = Color::from_rgb(0.4, 0.4, 0.6);    // Blue-gray - service
    pub const PERSON_BACKUP: Color = Color::from_rgb(0.5, 0.5, 0.5);     // Gray - backup
    pub const PERSON_AUDITOR: Color = Color::from_rgb(0.6, 0.3, 0.6);    // Purple - auditor

    // NATS context
    pub const NATS_OPERATOR: Color = Color::from_rgb(0.6, 0.2, 0.8);
    pub const NATS_ACCOUNT: Color = Color::from_rgb(0.5, 0.3, 0.7);
    pub const NATS_USER: Color = Color::from_rgb(0.4, 0.4, 0.6);

    // PKI context
    pub const CERT_ROOT: Color = Color::from_rgb(0.8, 0.6, 0.0);
    pub const CERT_INTERMEDIATE: Color = Color::from_rgb(0.7, 0.5, 0.2);
    pub const CERT_LEAF: Color = Color::from_rgb(0.5, 0.4, 0.3);
    pub const KEY: Color = Color::from_rgb(0.6, 0.6, 0.2);

    // YubiKey context
    pub const YUBIKEY: Color = Color::from_rgb(0.0, 0.6, 0.4);
    pub const PIV_SLOT: Color = Color::from_rgb(0.2, 0.5, 0.5);
    pub const YUBIKEY_STATUS: Color = Color::from_rgb(0.3, 0.7, 0.5);

    // Policy context
    pub const POLICY: Color = Color::from_rgb(0.5, 0.3, 0.6);
    pub const POLICY_ROLE: Color = Color::from_rgb(0.6, 0.4, 0.7);
    pub const POLICY_CLAIM: Color = Color::from_rgb(0.4, 0.4, 0.5);
    pub const POLICY_CATEGORY: Color = Color::from_rgb(0.5, 0.5, 0.6);
    pub const POLICY_GROUP: Color = Color::from_rgb(0.6, 0.3, 0.5);

    // Role context
    pub const ROLE: Color = Color::from_rgb(0.4, 0.5, 0.6);

    // Aggregate context
    pub const AGGREGATE: Color = Color::from_rgb(0.3, 0.3, 0.5);

    // Export context
    pub const MANIFEST: Color = Color::from_rgb(0.4, 0.6, 0.8);
}

// NOTE: FoldVisualization struct and impl FoldDomainNode have been removed.
// Use LiftedNode::themed_visualization() for node visualization.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_colors_are_distinct() {
        // Verify key colors are visually distinct
        assert_ne!(palette::ORGANIZATION, palette::PERSON_ROOT);
        assert_ne!(palette::NATS_OPERATOR, palette::CERT_ROOT);
        assert_ne!(palette::YUBIKEY, palette::KEY);
    }

    #[test]
    fn test_visualization_data_empty() {
        let data = VisualizationData::empty();
        assert!(data.label.is_empty());
        assert!(!data.expandable);
    }
}
