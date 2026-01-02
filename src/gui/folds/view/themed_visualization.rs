// Copyright (c) 2025 - Cowboy AI, LLC.

//! Themed Visualization Fold - Typography-Aware View Transformation
//!
//! This fold transforms domain nodes into visualization data using the
//! Typography bounded context's VerifiedTheme. Unlike the raw visualization
//! fold, this version guarantees no tofu boxes by using verified icons.
//!
//! NOTE: Uses deprecated `FoldDomainNode` trait. Migration pending.
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
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domains::typography::{
    VerifiedTheme, VerifiedIcon, LabelledElement, LabelCategory,
};

// Domain types for FoldDomainNode implementation
use crate::domain::{
    Person, KeyOwnerRole, Organization, OrganizationUnit, Location, Role, Policy,
};
use crate::domain::pki::{KeyAlgorithm, KeyPurpose};
use crate::domain::yubikey::PIVSlot;
use crate::domain_projections::NatsIdentityProjection;
use crate::policy::SeparationClass;
use crate::gui::domain_node::FoldDomainNode;

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

// ============================================================================
// FoldDomainNode Implementation
// ============================================================================
//
// This implements the categorical fold pattern for DomainNode, allowing
// themed visualization to work through the proper fold mechanism instead
// of directly accessing private fields.

impl<'a> FoldDomainNode for ThemedVisualizationFold<'a> {
    type Output = ThemedVisualizationData;

    fn fold_person(&self, person: &Person, role: &KeyOwnerRole) -> Self::Output {
        let role_name = format!("{:?}", role);
        // Use success color as base for person nodes (green)
        let role_color = Color::from_rgb(0.2, 0.8, 0.3);

        let icon = self.get_icon("person");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(person.name.clone()),
            crate::domains::typography::FontFamily::Body,
            role_color,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(person.email.clone()),
            format!("{} ({})", person.name, role_name),
            true,
        )
    }

    fn fold_organization(&self, org: &Organization) -> Self::Output {
        let icon = self.get_icon("organization");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(org.display_name.clone()),
            crate::domains::typography::FontFamily::Heading,
            self.theme.colors().primary,
            self.theme.metrics().heading_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(org.name.clone()),
            org.description.clone().unwrap_or_default(),
            true,
        )
    }

    fn fold_organization_unit(&self, unit: &OrganizationUnit) -> Self::Output {
        let icon = self.get_icon("organization");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(unit.name.clone()),
            crate::domains::typography::FontFamily::Body,
            adjust_color(self.theme.colors().primary, 0.8),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some("Organization Unit".to_string()),
            format!("Unit: {}", unit.name),
            true,
        )
    }

    fn fold_location(&self, loc: &Location) -> Self::Output {
        let icon = self.get_icon("location");
        let location_type = format!("{:?}", loc.location_type);

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(loc.name.clone()),
            crate::domains::typography::FontFamily::Body,
            self.theme.colors().info,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(location_type.clone()),
            format!("{} ({})", loc.name, location_type),
            false,
        )
    }

    fn fold_role(&self, role: &Role) -> Self::Output {
        let icon = self.get_icon("role");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(role.name.clone()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.4, 0.5, 0.6),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(role.description.clone()),
            format!("Role: {}", role.name),
            false,
        )
    }

    fn fold_policy(&self, policy: &Policy) -> Self::Output {
        let icon = self.get_icon("policy");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(policy.name.clone()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.5, 0.3, 0.6),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(policy.description.clone()),
            format!("Policy: {}", policy.name),
            true,
        )
    }

    fn fold_nats_operator(&self, proj: &NatsIdentityProjection) -> Self::Output {
        let icon = self.get_icon("operator");
        // Extract name from public key prefix
        let display_name = format!("Operator {}", &proj.nkey.public_key.public_key()[..8]);
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(display_name.clone()),
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

    fn fold_nats_account(&self, proj: &NatsIdentityProjection) -> Self::Output {
        let icon = self.get_icon("account");
        let display_name = format!("Account {}", &proj.nkey.public_key.public_key()[..8]);
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(display_name.clone()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.5, 0.3, 0.7),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some("NATS Account".to_string()),
            format!("NATS Account: {}", display_name),
            true,
        )
    }

    fn fold_nats_user(&self, proj: &NatsIdentityProjection) -> Self::Output {
        let icon = self.get_icon("person");
        let display_name = format!("User {}", &proj.nkey.public_key.public_key()[..8]);
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(display_name.clone()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.4, 0.4, 0.6),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some("NATS User".to_string()),
            format!("NATS User: {}", display_name),
            false,
        )
    }

    fn fold_nats_service_account(&self, proj: &NatsIdentityProjection) -> Self::Output {
        let icon = self.get_icon("service");
        let display_name = format!("Service {}", &proj.nkey.public_key.public_key()[..8]);
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(display_name.clone()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.8, 0.2, 0.8),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some("NATS Service Account".to_string()),
            format!("Service Account: {}", display_name),
            false,
        )
    }

    fn fold_nats_operator_simple(&self, name: &str, _organization_id: Option<Uuid>) -> Self::Output {
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
            format!("NATS Operator: {}", name),
            true,
        )
    }

    fn fold_nats_account_simple(&self, name: &str, _unit_id: Option<Uuid>, is_system: bool) -> Self::Output {
        let icon = self.get_icon("account");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.5, 0.3, 0.7),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(if is_system { "System Account" } else { "NATS Account" }.to_string()),
            format!("NATS Account: {}", name),
            true,
        )
    }

    fn fold_nats_user_simple(&self, name: &str, _person_id: Option<Uuid>, account_name: &str) -> Self::Output {
        let icon = self.get_icon("person");
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
            format!("NATS User: {} in {}", name, account_name),
            false,
        )
    }

    fn fold_root_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        let icon = self.get_icon("certificate");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(subject.to_string()),
            crate::domains::typography::FontFamily::Body,
            self.theme.colors().success, // Root CA uses success green
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("Expires: {}", not_after.format("%Y-%m-%d"))),
            "Root Certificate".to_string(),
            true,
        )
    }

    fn fold_intermediate_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        let icon = self.get_icon("certificate");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(subject.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.7, 0.5, 0.2),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("Expires: {}", not_after.format("%Y-%m-%d"))),
            "Intermediate Certificate".to_string(),
            true,
        )
    }

    fn fold_leaf_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        _issuer: &str,
        _not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
        _san: &[String],
    ) -> Self::Output {
        let icon = self.get_icon("certificate");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(subject.to_string()),
            crate::domains::typography::FontFamily::Body,
            self.theme.colors().info, // Leaf cert uses info blue
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("Expires: {}", not_after.format("%Y-%m-%d"))),
            "Leaf Certificate".to_string(),
            false,
        )
    }

    fn fold_key(
        &self,
        _key_id: Uuid,
        algorithm: &KeyAlgorithm,
        purpose: &KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output {
        let icon = self.get_icon("key");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(format!("{:?}", purpose)),
            crate::domains::typography::FontFamily::Monospace,
            Color::from_rgb(0.6, 0.6, 0.2),
            self.theme.metrics().base_font_size,
        );

        let subtitle = expires_at
            .map(|e| format!("Expires: {}", e.format("%Y-%m-%d")))
            .unwrap_or_else(|| "No expiration".to_string());

        ThemedVisualizationData::new(
            primary,
            Some(subtitle),
            format!("{:?} key ({:?})", purpose, algorithm),
            false,
        )
    }

    fn fold_yubikey(
        &self,
        _device_id: Uuid,
        serial: &str,
        version: &str,
        _provisioned_at: Option<DateTime<Utc>>,
        slots_used: &[String],
    ) -> Self::Output {
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
            Some(format!("v{} - {} slots", version, slots_used.len())),
            format!("YubiKey {} (firmware {})", serial, version),
            true,
        )
    }

    fn fold_piv_slot(
        &self,
        _slot_id: Uuid,
        slot_name: &str,
        _yubikey_serial: &str,
        has_key: bool,
        _certificate_subject: Option<&String>,
    ) -> Self::Output {
        let icon = self.get_icon("slot");
        let color = if has_key {
            self.theme.colors().success
        } else {
            Color::from_rgb(0.5, 0.5, 0.5)
        };

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(slot_name.to_string()),
            crate::domains::typography::FontFamily::Monospace,
            color,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(if has_key { "Has key" } else { "Empty" }.to_string()),
            format!("PIV Slot: {}", slot_name),
            false,
        )
    }

    fn fold_yubikey_status(
        &self,
        _person_id: Uuid,
        yubikey_serial: Option<&String>,
        slots_provisioned: &[PIVSlot],
        slots_needed: &[PIVSlot],
    ) -> Self::Output {
        let icon = self.get_icon("yubikey");
        let (color, status) = if yubikey_serial.is_some() {
            (self.theme.colors().success, "Assigned")
        } else {
            (self.theme.colors().warning, "Not assigned")
        };

        let primary = LabelledElement::new(
            LabelCategory::Status,
            icon,
            Some(status.to_string()),
            crate::domains::typography::FontFamily::Body,
            color,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("{}/{} slots", slots_provisioned.len(), slots_needed.len())),
            "YubiKey Status".to_string(),
            false,
        )
    }

    fn fold_manifest(
        &self,
        _manifest_id: Uuid,
        name: &str,
        destination: Option<&std::path::PathBuf>,
        _checksum: Option<&String>,
    ) -> Self::Output {
        let icon = self.get_icon("manifest");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.4, 0.6, 0.8),
            self.theme.metrics().base_font_size,
        );

        let dest_str = destination
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "No destination".to_string());

        ThemedVisualizationData::new(
            primary,
            Some(dest_str),
            format!("Export Manifest: {}", name),
            false,
        )
    }

    fn fold_policy_role(
        &self,
        _role_id: Uuid,
        name: &str,
        purpose: &str,
        level: u8,
        separation_class: SeparationClass,
        _claim_count: usize,
    ) -> Self::Output {
        let icon = self.get_icon("role");
        // Map separation class to colors (DutyBoundary in domain language)
        let color = match separation_class {
            SeparationClass::Operational => Color::from_rgb(0.2, 0.6, 0.8),    // Blue - day-to-day
            SeparationClass::Administrative => Color::from_rgb(0.6, 0.2, 0.6), // Purple - admin
            SeparationClass::Audit => Color::from_rgb(0.8, 0.6, 0.0),          // Gold - compliance
            SeparationClass::Emergency => Color::from_rgb(0.8, 0.2, 0.2),      // Red - break-glass
            SeparationClass::Financial => Color::from_rgb(0.2, 0.8, 0.4),      // Green - financial
            SeparationClass::Personnel => Color::from_rgb(0.4, 0.6, 0.8),      // Light blue - HR
        };

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            color,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("Level {} - {}", level, purpose)),
            format!("Policy Role: {} ({:?})", name, separation_class),
            false,
        )
    }

    fn fold_policy_claim(
        &self,
        _claim_id: Uuid,
        name: &str,
        category: &str,
    ) -> Self::Output {
        let icon = self.get_icon("claim");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.6, 0.5, 0.4),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(category.to_string()),
            format!("Policy Claim: {}", name),
            false,
        )
    }

    fn fold_policy_category(
        &self,
        _category_id: Uuid,
        name: &str,
        claim_count: usize,
        _expanded: bool,
    ) -> Self::Output {
        let icon = self.get_icon("category");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            Color::from_rgb(0.5, 0.6, 0.5),
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("{} claims", claim_count)),
            format!("Policy Category: {}", name),
            claim_count > 0,
        )
    }

    fn fold_policy_group(
        &self,
        _class_id: Uuid,
        name: &str,
        separation_class: SeparationClass,
        role_count: usize,
        _expanded: bool,
    ) -> Self::Output {
        let icon = self.get_icon("group");
        // Map separation class to colors (DutyBoundary in domain language)
        let color = match separation_class {
            SeparationClass::Operational => Color::from_rgb(0.2, 0.6, 0.8),
            SeparationClass::Administrative => Color::from_rgb(0.6, 0.2, 0.6),
            SeparationClass::Audit => Color::from_rgb(0.8, 0.6, 0.0),
            SeparationClass::Emergency => Color::from_rgb(0.8, 0.2, 0.2),
            SeparationClass::Financial => Color::from_rgb(0.2, 0.8, 0.4),
            SeparationClass::Personnel => Color::from_rgb(0.4, 0.6, 0.8),
        };

        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Body,
            color,
            self.theme.metrics().base_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("{} roles", role_count)),
            format!("Duty Boundary: {:?}", separation_class),
            role_count > 0,
        )
    }

    fn fold_aggregate_organization(
        &self,
        name: &str,
        version: u64,
        people_count: usize,
        units_count: usize,
    ) -> Self::Output {
        let icon = self.get_icon("aggregate");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Heading,
            self.theme.colors().primary,
            self.theme.metrics().heading_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("v{} - {} people, {} units", version, people_count, units_count)),
            "Organization Aggregate".to_string(),
            true,
        )
    }

    fn fold_aggregate_pki_chain(
        &self,
        name: &str,
        version: u64,
        certificates_count: usize,
        keys_count: usize,
    ) -> Self::Output {
        let icon = self.get_icon("aggregate");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Heading,
            Color::from_rgb(0.8, 0.6, 0.0),
            self.theme.metrics().heading_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("v{} - {} certs, {} keys", version, certificates_count, keys_count)),
            "PKI Chain Aggregate".to_string(),
            true,
        )
    }

    fn fold_aggregate_nats_security(
        &self,
        name: &str,
        version: u64,
        operators_count: usize,
        accounts_count: usize,
        users_count: usize,
    ) -> Self::Output {
        let icon = self.get_icon("aggregate");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Heading,
            Color::from_rgb(0.6, 0.2, 0.8),
            self.theme.metrics().heading_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("v{} - {} ops, {} accs, {} users", version, operators_count, accounts_count, users_count)),
            "NATS Security Aggregate".to_string(),
            true,
        )
    }

    fn fold_aggregate_yubikey_provisioning(
        &self,
        name: &str,
        version: u64,
        devices_count: usize,
        slots_provisioned: usize,
    ) -> Self::Output {
        let icon = self.get_icon("aggregate");
        let primary = LabelledElement::new(
            LabelCategory::Entity,
            icon,
            Some(name.to_string()),
            crate::domains::typography::FontFamily::Heading,
            Color::from_rgb(0.0, 0.6, 0.4),
            self.theme.metrics().heading_font_size,
        );

        ThemedVisualizationData::new(
            primary,
            Some(format!("v{} - {} devices, {} slots", version, devices_count, slots_provisioned)),
            "YubiKey Provisioning Aggregate".to_string(),
            true,
        )
    }
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
