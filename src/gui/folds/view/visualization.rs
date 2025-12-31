// Copyright (c) 2025 - Cowboy AI, LLC.

//! Visualization Fold - View Layer Natural Transformation
//!
//! This fold transforms domain nodes into visualization data for rendering.
//! It executes in the view() function, producing pure visual output with no side effects.
//!
//! ## FRP Pipeline Role
//!
//! ```text
//! Model → view() → [FoldVisualization] → VisualizationData → Element
//! ```
//!
//! ## Categorical Structure
//!
//! FoldVisualization is a natural transformation:
//! ```text
//! η: DomainNode ⟹ VisualizationData
//! ```
//!
//! The naturality condition ensures consistent rendering regardless of how
//! we traverse the domain structure.

use chrono::{DateTime, Utc};
use iced::Color;
use uuid::Uuid;

use crate::domain::{KeyOwnerRole, Person, Organization, OrganizationUnit, Location, Policy, Role};
use crate::domain::pki::{KeyAlgorithm, KeyPurpose};
use crate::domain::yubikey::PIVSlot;
use crate::domain::visualization::SeparationClass;
use crate::domain_projections::NatsIdentityProjection;
use crate::gui::domain_node::FoldDomainNode;

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

mod palette {
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

// ============================================================================
// FOLD IMPLEMENTATION
// ============================================================================

/// Folder that transforms domain nodes into visualization data.
///
/// This is a VIEW fold - it executes in the view() function
/// and produces pure visual output with no side effects.
pub struct FoldVisualization;

impl FoldDomainNode for FoldVisualization {
    type Output = VisualizationData;

    fn fold_person(&self, person: &Person, role: &KeyOwnerRole) -> Self::Output {
        let color = match role {
            KeyOwnerRole::RootAuthority => palette::PERSON_ROOT,
            KeyOwnerRole::SecurityAdmin => palette::PERSON_SECURITY,
            KeyOwnerRole::Developer => palette::PERSON_DEVELOPER,
            KeyOwnerRole::ServiceAccount => palette::PERSON_SERVICE,
            KeyOwnerRole::BackupHolder => palette::PERSON_BACKUP,
            KeyOwnerRole::Auditor => palette::PERSON_AUDITOR,
        };

        VisualizationData {
            label: person.name.clone(),
            subtitle: person.email.clone(),
            color,
            icon: "person".to_string(),
            tooltip: format!("{} ({:?})", person.name, role),
            expandable: true,
        }
    }

    fn fold_organization(&self, org: &Organization) -> Self::Output {
        VisualizationData {
            label: org.display_name.clone(),
            subtitle: org.name.clone(),
            color: palette::ORGANIZATION,
            icon: "organization".to_string(),
            tooltip: org.description.clone().unwrap_or_default(),
            expandable: true,
        }
    }

    fn fold_organization_unit(&self, unit: &OrganizationUnit) -> Self::Output {
        VisualizationData {
            label: unit.name.clone(),
            subtitle: "Organization Unit".to_string(),
            color: palette::ORGANIZATION_UNIT,
            icon: "unit".to_string(),
            tooltip: format!("Unit: {}", unit.name),
            expandable: true,
        }
    }

    fn fold_location(&self, loc: &Location) -> Self::Output {
        VisualizationData {
            label: loc.name.clone(),
            subtitle: format!("{:?}", loc.location_type),
            color: palette::LOCATION,
            icon: "location".to_string(),
            tooltip: format!("{} ({:?})", loc.name, loc.location_type),
            expandable: false,
        }
    }

    fn fold_role(&self, role: &Role) -> Self::Output {
        VisualizationData {
            label: role.name.clone(),
            subtitle: role.description.clone(),
            color: palette::ROLE,
            icon: "role".to_string(),
            tooltip: format!("Role: {}", role.name),
            expandable: false,
        }
    }

    fn fold_policy(&self, policy: &Policy) -> Self::Output {
        VisualizationData {
            label: policy.name.clone(),
            subtitle: policy.description.clone(),
            color: palette::POLICY,
            icon: "policy".to_string(),
            tooltip: format!("Policy: {}", policy.name),
            expandable: true,
        }
    }

    fn fold_nats_operator(&self, proj: &NatsIdentityProjection) -> Self::Output {
        VisualizationData {
            label: proj.nkey.name.clone().unwrap_or_else(|| "Operator".to_string()),
            subtitle: "NATS Operator".to_string(),
            color: palette::NATS_OPERATOR,
            icon: "operator".to_string(),
            tooltip: "NATS Operator - Root of trust".to_string(),
            expandable: true,
        }
    }

    fn fold_nats_account(&self, proj: &NatsIdentityProjection) -> Self::Output {
        VisualizationData {
            label: proj.nkey.name.clone().unwrap_or_else(|| "Account".to_string()),
            subtitle: "NATS Account".to_string(),
            color: palette::NATS_ACCOUNT,
            icon: "account".to_string(),
            tooltip: "NATS Account - Tenant isolation".to_string(),
            expandable: true,
        }
    }

    fn fold_nats_user(&self, proj: &NatsIdentityProjection) -> Self::Output {
        VisualizationData {
            label: proj.nkey.name.clone().unwrap_or_else(|| "User".to_string()),
            subtitle: "NATS User".to_string(),
            color: palette::NATS_USER,
            icon: "user".to_string(),
            tooltip: "NATS User - Identity mapping".to_string(),
            expandable: false,
        }
    }

    fn fold_nats_service_account(&self, proj: &NatsIdentityProjection) -> Self::Output {
        VisualizationData {
            label: proj.nkey.name.clone().unwrap_or_else(|| "Service".to_string()),
            subtitle: "Service Account".to_string(),
            color: palette::NATS_USER,
            icon: "service".to_string(),
            tooltip: "NATS Service Account".to_string(),
            expandable: false,
        }
    }

    fn fold_nats_operator_simple(&self, name: &str, _organization_id: Option<Uuid>) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: "NATS Operator".to_string(),
            color: palette::NATS_OPERATOR,
            icon: "operator".to_string(),
            tooltip: "NATS Operator".to_string(),
            expandable: true,
        }
    }

    fn fold_nats_account_simple(&self, name: &str, _unit_id: Option<Uuid>, is_system: bool) -> Self::Output {
        let subtitle = if is_system { "System Account" } else { "NATS Account" };
        VisualizationData {
            label: name.to_string(),
            subtitle: subtitle.to_string(),
            color: palette::NATS_ACCOUNT,
            icon: "account".to_string(),
            tooltip: format!("NATS Account: {}", name),
            expandable: true,
        }
    }

    fn fold_nats_user_simple(&self, name: &str, _person_id: Option<Uuid>, account_name: &str) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: format!("in {}", account_name),
            color: palette::NATS_USER,
            icon: "user".to_string(),
            tooltip: format!("NATS User in account {}", account_name),
            expandable: false,
        }
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
        VisualizationData {
            label: subject.to_string(),
            subtitle: format!("Expires: {}", not_after.format("%Y-%m-%d")),
            color: palette::CERT_ROOT,
            icon: "certificate_root".to_string(),
            tooltip: "Root CA Certificate".to_string(),
            expandable: true,
        }
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
        VisualizationData {
            label: subject.to_string(),
            subtitle: format!("Expires: {}", not_after.format("%Y-%m-%d")),
            color: palette::CERT_INTERMEDIATE,
            icon: "certificate_intermediate".to_string(),
            tooltip: "Intermediate CA Certificate".to_string(),
            expandable: true,
        }
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
        VisualizationData {
            label: subject.to_string(),
            subtitle: format!("Expires: {}", not_after.format("%Y-%m-%d")),
            color: palette::CERT_LEAF,
            icon: "certificate_leaf".to_string(),
            tooltip: "Leaf Certificate".to_string(),
            expandable: false,
        }
    }

    fn fold_key(
        &self,
        _key_id: Uuid,
        algorithm: &KeyAlgorithm,
        purpose: &KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output {
        let subtitle = match expires_at {
            Some(exp) => format!("Expires: {}", exp.format("%Y-%m-%d")),
            None => "No expiration".to_string(),
        };

        VisualizationData {
            label: format!("{:?}", purpose),
            subtitle,
            color: palette::KEY,
            icon: "key".to_string(),
            tooltip: format!("{:?} key ({:?})", purpose, algorithm),
            expandable: false,
        }
    }

    fn fold_yubikey(
        &self,
        _device_id: Uuid,
        serial: &str,
        version: &str,
        _provisioned_at: Option<DateTime<Utc>>,
        slots_used: &[String],
    ) -> Self::Output {
        VisualizationData {
            label: format!("YubiKey {}", serial),
            subtitle: format!("v{} - {} slots", version, slots_used.len()),
            color: palette::YUBIKEY,
            icon: "yubikey".to_string(),
            tooltip: format!("YubiKey {} (firmware {})", serial, version),
            expandable: true,
        }
    }

    fn fold_piv_slot(
        &self,
        _slot_id: Uuid,
        slot_name: &str,
        _yubikey_serial: &str,
        has_key: bool,
        certificate_subject: Option<&String>,
    ) -> Self::Output {
        let subtitle = if has_key {
            certificate_subject.map(|s| s.clone()).unwrap_or_else(|| "Key present".to_string())
        } else {
            "Empty".to_string()
        };

        VisualizationData {
            label: slot_name.to_string(),
            subtitle,
            color: palette::PIV_SLOT,
            icon: "slot".to_string(),
            tooltip: format!("PIV Slot: {}", slot_name),
            expandable: false,
        }
    }

    fn fold_yubikey_status(
        &self,
        _person_id: Uuid,
        yubikey_serial: Option<&String>,
        slots_provisioned: &[PIVSlot],
        slots_needed: &[PIVSlot],
    ) -> Self::Output {
        let label = yubikey_serial.map(|s| s.clone()).unwrap_or_else(|| "No YubiKey".to_string());
        let total = slots_provisioned.len() + slots_needed.len();

        VisualizationData {
            label,
            subtitle: format!("{}/{} slots provisioned", slots_provisioned.len(), total),
            color: palette::YUBIKEY_STATUS,
            icon: "status".to_string(),
            tooltip: format!("{} provisioned, {} needed", slots_provisioned.len(), slots_needed.len()),
            expandable: false,
        }
    }

    fn fold_manifest(
        &self,
        _manifest_id: Uuid,
        name: &str,
        destination: Option<&std::path::PathBuf>,
        _checksum: Option<&String>,
    ) -> Self::Output {
        let subtitle = destination
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "No destination".to_string());

        VisualizationData {
            label: name.to_string(),
            subtitle,
            color: palette::MANIFEST,
            icon: "manifest".to_string(),
            tooltip: format!("Export Manifest: {}", name),
            expandable: true,
        }
    }

    fn fold_policy_role(
        &self,
        _role_id: Uuid,
        name: &str,
        purpose: &str,
        level: u8,
        _separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: format!("Level {} - {} claims", level, claim_count),
            color: palette::POLICY_ROLE,
            icon: "policy_role".to_string(),
            tooltip: purpose.to_string(),
            expandable: claim_count > 0,
        }
    }

    fn fold_policy_claim(
        &self,
        _claim_id: Uuid,
        name: &str,
        category: &str,
    ) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: category.to_string(),
            color: palette::POLICY_CLAIM,
            icon: "claim".to_string(),
            tooltip: format!("Claim: {} ({})", name, category),
            expandable: false,
        }
    }

    fn fold_policy_category(
        &self,
        _category_id: Uuid,
        name: &str,
        claim_count: usize,
        _expanded: bool,
    ) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: format!("{} claims", claim_count),
            color: palette::POLICY_CATEGORY,
            icon: "category".to_string(),
            tooltip: format!("Category: {} ({} claims)", name, claim_count),
            expandable: claim_count > 0,
        }
    }

    fn fold_policy_group(
        &self,
        _class_id: Uuid,
        name: &str,
        separation_class: SeparationClass,
        role_count: usize,
        _expanded: bool,
    ) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: format!("{:?} - {} roles", separation_class, role_count),
            color: palette::POLICY_GROUP,
            icon: "group".to_string(),
            tooltip: format!("Separation Group: {} ({:?})", name, separation_class),
            expandable: role_count > 0,
        }
    }

    fn fold_aggregate_organization(
        &self,
        name: &str,
        version: u64,
        people_count: usize,
        units_count: usize,
    ) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: format!("v{} - {} people, {} units", version, people_count, units_count),
            color: palette::AGGREGATE,
            icon: "aggregate".to_string(),
            tooltip: format!("Organization Aggregate (v{})", version),
            expandable: true,
        }
    }

    fn fold_aggregate_pki_chain(
        &self,
        name: &str,
        version: u64,
        certificates_count: usize,
        keys_count: usize,
    ) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: format!("v{} - {} certs, {} keys", version, certificates_count, keys_count),
            color: palette::AGGREGATE,
            icon: "aggregate".to_string(),
            tooltip: format!("PKI Chain Aggregate (v{})", version),
            expandable: true,
        }
    }

    fn fold_aggregate_nats_security(
        &self,
        name: &str,
        version: u64,
        operators_count: usize,
        accounts_count: usize,
        users_count: usize,
    ) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: format!("v{} - {}op/{}acc/{}usr", version, operators_count, accounts_count, users_count),
            color: palette::AGGREGATE,
            icon: "aggregate".to_string(),
            tooltip: format!("NATS Security Aggregate (v{})", version),
            expandable: true,
        }
    }

    fn fold_aggregate_yubikey_provisioning(
        &self,
        name: &str,
        version: u64,
        devices_count: usize,
        slots_provisioned: usize,
    ) -> Self::Output {
        VisualizationData {
            label: name.to_string(),
            subtitle: format!("v{} - {} devices, {} slots", version, devices_count, slots_provisioned),
            color: palette::AGGREGATE,
            icon: "aggregate".to_string(),
            tooltip: format!("YubiKey Provisioning Aggregate (v{})", version),
            expandable: true,
        }
    }
}

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
