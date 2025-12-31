// Copyright (c) 2025 - Cowboy AI, LLC.

//! Themed Visualization Fold - Typography-Aware View Transformation
//!
//! This fold transforms domain nodes into visualization data using the
//! Typography bounded context's VerifiedTheme. Unlike the raw visualization
//! fold, this version guarantees no tofu boxes by using verified icons.
//!
//! ## FRP Pipeline Role
//!
//! ```text
//! (Model, VerifiedTheme) → view() → [ThemedVisualizationFold] → ThemedVisualizationData → Element
//! ```
//!
//! ## Typography Integration
//!
//! All icons are resolved through the VerifiedIconSet, which ensures:
//! 1. Every icon has a verified representation that will render
//! 2. Fallback chains are pre-verified at theme initialization
//! 3. No hardcoded emoji or icon strings leak into the view layer

use iced::Color;

use crate::domains::typography::{
    VerifiedTheme, VerifiedIcon, LabelledElement, LabelCategory,
};

/// Themed visualization data extracted from a domain node.
///
/// Unlike raw VisualizationData, this version uses verified typography
/// elements that are guaranteed to render correctly.
#[derive(Debug, Clone)]
pub struct ThemedVisualizationData {
    /// Primary label (icon + text)
    pub primary: LabelledElement,
    /// Secondary/subtitle text
    pub secondary: Option<String>,
    /// Tooltip content
    pub tooltip: String,
    /// Whether this node is expandable (has children)
    pub expandable: bool,
}

impl ThemedVisualizationData {
    /// Create themed visualization data
    pub fn new(
        primary: LabelledElement,
        secondary: Option<String>,
        tooltip: String,
        expandable: bool,
    ) -> Self {
        Self {
            primary,
            secondary,
            tooltip,
            expandable,
        }
    }

    /// Create empty visualization data with a fallback theme
    pub fn empty(theme: &VerifiedTheme) -> Self {
        Self {
            primary: LabelledElement::text_label("Unknown")
                .with_color(theme.colors().text_light),
            secondary: None,
            tooltip: String::new(),
            expandable: false,
        }
    }

    /// Get the display label
    pub fn label(&self) -> String {
        self.primary.text().map(|s| s.to_string()).unwrap_or_default()
    }

    /// Get the icon display string
    pub fn icon_display(&self) -> String {
        self.primary.icon_display().unwrap_or_default()
    }

    /// Get the color
    pub fn color(&self) -> Color {
        self.primary.color()
    }
}

/// Folder that creates themed visualization data from domain concepts.
///
/// This fold uses a VerifiedTheme to ensure all visual elements will render
/// correctly without tofu boxes.
pub struct ThemedVisualizationFold<'a> {
    theme: &'a VerifiedTheme,
}

impl<'a> ThemedVisualizationFold<'a> {
    /// Create a new themed visualization fold with the given theme
    pub fn new(theme: &'a VerifiedTheme) -> Self {
        Self { theme }
    }

    /// Get a verified icon by name, or text fallback if not found
    fn get_icon(&self, name: &str) -> Option<VerifiedIcon> {
        self.theme.icons().get(name).cloned()
    }

    /// Create a labelled element for a person entity
    pub fn fold_person(
        &self,
        name: &str,
        email: &str,
        role_name: &str,
        role_color: Color,
    ) -> ThemedVisualizationData {
        let icon = self.get_icon("person");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            role_color,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(email.to_string()),
            format!("{} ({})", name, role_name),
            true,
        )
    }

    /// Create a labelled element for an organization entity
    pub fn fold_organization(
        &self,
        display_name: &str,
        name: &str,
        description: Option<&str>,
    ) -> ThemedVisualizationData {
        let icon = self.get_icon("organization");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(display_name.to_string()),
            crate::domains::typography::FontFamily::Heading,
            self.theme.colors().primary,
            self.theme.metrics().heading_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(name.to_string()),
            description.unwrap_or("").to_string(),
            true,
        )
    }

    /// Create a labelled element for an organization unit
    pub fn fold_organization_unit(&self, name: &str) -> ThemedVisualizationData {
        // Use organization icon for units too
        let icon = self.get_icon("organization");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            adjust_color(self.theme.colors().primary, 0.8),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some("Organization Unit".to_string()),
            format!("Unit: {}", name),
            true,
        )
    }

    /// Create a labelled element for a location
    pub fn fold_location(
        &self,
        name: &str,
        location_type: &str,
    ) -> ThemedVisualizationData {
        let icon = self.get_icon("location");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            self.theme.colors().info,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(location_type.to_string()),
            format!("{} ({})", name, location_type),
            false,
        )
    }

    /// Create a labelled element for a key
    pub fn fold_key(
        &self,
        purpose: &str,
        algorithm: &str,
        expires: Option<&str>,
    ) -> ThemedVisualizationData {
        let icon = self.get_icon("key");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(purpose.to_string()),
            crate::domains::typography::FontFamily::Monospace,
            Color::from_rgb(0.6, 0.6, 0.2), // Key color
            self.theme.metrics().base_font_size,
        );

        let subtitle = expires
            .map(|e| format!("Expires: {}", e))
            .unwrap_or_else(|| "No expiration".to_string());

        ThemedVisualizationData::new(
            primary,
            Some(subtitle),
            format!("{} key ({})", purpose, algorithm),
            false,
        )
    }

    /// Create a labelled element for a certificate
    pub fn fold_certificate(
        &self,
        subject: &str,
        expires: &str,
        cert_type: CertificateType,
    ) -> ThemedVisualizationData {
        let icon = self.get_icon("certificate");

        let color = match cert_type {
            CertificateType::Root => Color::from_rgb(0.8, 0.6, 0.0),
            CertificateType::Intermediate => Color::from_rgb(0.7, 0.5, 0.2),
            CertificateType::Leaf => Color::from_rgb(0.5, 0.4, 0.3),
        };

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(subject.to_string()),
            crate::domains::typography::FontFamily::Body,
            color,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("Expires: {}", expires)),
            format!("{:?} Certificate", cert_type),
            matches!(cert_type, CertificateType::Root | CertificateType::Intermediate),
        )
    }

    /// Create a labelled element for a YubiKey
    pub fn fold_yubikey(
        &self,
        serial: &str,
        version: &str,
        slots_used: usize,
    ) -> ThemedVisualizationData {
        let icon = self.get_icon("yubikey");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(format!("YubiKey {}", serial)),
            crate::domains::typography::FontFamily::Monospace,
            Color::from_rgb(0.0, 0.6, 0.4),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("v{} - {} slots", version, slots_used)),
            format!("YubiKey {} (firmware {})", serial, version),
            true,
        )
    }

    /// Create a labelled element for a NATS operator
    pub fn fold_nats_operator(&self, name: &str) -> ThemedVisualizationData {
        let icon = self.get_icon("operator");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.6, 0.2, 0.8),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some("NATS Operator".to_string()),
            "NATS Operator - Root of trust".to_string(),
            true,
        )
    }

    /// Create a labelled element for a NATS account
    pub fn fold_nats_account(&self, name: &str, is_system: bool) -> ThemedVisualizationData {
        let icon = self.get_icon("account");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.5, 0.3, 0.7),
            self.theme.metrics().base_font_size,
        );

        let subtitle = if is_system { "System Account" } else { "NATS Account" };

        ThemedVisualizationData::new(
            primary,
            Some(subtitle.to_string()),
            format!("NATS Account: {}", name),
            true,
        )
    }

    /// Create a labelled element for a NATS user
    pub fn fold_nats_user(&self, name: &str, account_name: &str) -> ThemedVisualizationData {
        let icon = self.get_icon("person"); // Users are like people

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.4, 0.4, 0.6),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("in {}", account_name)),
            format!("NATS User in account {}", account_name),
            false,
        )
    }

    /// Create a labelled element for a role
    pub fn fold_role(&self, name: &str, description: &str) -> ThemedVisualizationData {
        let icon = self.get_icon("role");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.4, 0.5, 0.6),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(description.to_string()),
            format!("Role: {}", name),
            false,
        )
    }

    /// Create a labelled element for a policy
    pub fn fold_policy(&self, name: &str, description: &str) -> ThemedVisualizationData {
        let icon = self.get_icon("policy");

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.5, 0.3, 0.6),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(description.to_string()),
            format!("Policy: {}", name),
            true,
        )
    }

    /// Create a status indicator
    pub fn fold_status(&self, status: StatusIndicator) -> ThemedVisualizationData {
        let (icon_name, color, text) = match status {
            StatusIndicator::Success => ("success", self.theme.colors().success, "Success"),
            StatusIndicator::Warning => ("warning", self.theme.colors().warning, "Warning"),
            StatusIndicator::Error => ("error", self.theme.colors().error, "Error"),
            StatusIndicator::Info => ("info", self.theme.colors().info, "Info"),
            StatusIndicator::Pending => ("pending", Color::from_rgb(0.5, 0.5, 0.5), "Pending"),
        };

        let icon = self.get_icon(icon_name);

        let primary = LabelledElement::new(
            LabelCategory::Status,
            icon,
            Some(text.to_string()),
            crate::domains::typography::FontFamily::Body,
            color,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            None,
            text.to_string(),
            false,
        )
    }
}

/// Certificate type for visualization
#[derive(Debug, Clone, Copy)]
pub enum CertificateType {
    Root,
    Intermediate,
    Leaf,
}

/// Status indicator types
#[derive(Debug, Clone, Copy)]
pub enum StatusIndicator {
    Success,
    Warning,
    Error,
    Info,
    Pending,
}

/// Adjust a color's brightness
fn adjust_color(color: Color, factor: f32) -> Color {
    Color::from_rgb(
        (color.r * factor).min(1.0),
        (color.g * factor).min(1.0),
        (color.b * factor).min(1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themed_visualization_fold() {
        let theme = VerifiedTheme::text_only_fallback();
        let fold = ThemedVisualizationFold::new(&theme);

        let data = fold.fold_organization("CowboyAI", "cowboyai", Some("A company"));

        assert_eq!(data.label(), "CowboyAI");
        assert_eq!(data.secondary, Some("cowboyai".to_string()));
        assert!(data.expandable);
    }

    #[test]
    fn test_themed_visualization_person() {
        let theme = VerifiedTheme::text_only_fallback();
        let fold = ThemedVisualizationFold::new(&theme);

        let data = fold.fold_person(
            "Alice",
            "alice@example.com",
            "Developer",
            Color::from_rgb(0.3, 0.6, 0.3),
        );

        assert_eq!(data.label(), "Alice");
        assert_eq!(data.secondary, Some("alice@example.com".to_string()));
        assert!(data.tooltip.contains("Developer"));
    }

    #[test]
    fn test_themed_visualization_uses_verified_icons() {
        let theme = VerifiedTheme::text_only_fallback();
        let fold = ThemedVisualizationFold::new(&theme);

        let data = fold.fold_organization("Test", "test", None);

        // With text_only_fallback, icon should be a text fallback
        let icon_display = data.icon_display();
        assert!(
            icon_display.starts_with('[') || icon_display.contains("ORG"),
            "Expected text fallback icon, got: {}",
            icon_display
        );
    }

    #[test]
    fn test_status_indicators() {
        let theme = VerifiedTheme::text_only_fallback();
        let fold = ThemedVisualizationFold::new(&theme);

        let success = fold.fold_status(StatusIndicator::Success);
        assert_eq!(success.color(), theme.colors().success);

        let error = fold.fold_status(StatusIndicator::Error);
        assert_eq!(error.color(), theme.colors().error);
    }
}
