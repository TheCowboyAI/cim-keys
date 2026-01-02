// Copyright (c) 2025 - Cowboy AI, LLC.

//! Edit Field Extraction Fold
//!
//! Implements the categorical fold pattern for extracting editable
//! field data from domain nodes, replacing pattern matching with
//! the universal property of the coproduct.
//!
//! NOTE: Uses deprecated `FoldDomainNode` and `DomainNode` types. Migration pending.
//!
//! ## Categorical Foundation
//!
//! This fold is a natural transformation:
//! ```text
//! η: DomainNode ⟹ EditFieldData
//! ```
//!
//! For each injection type A, we have a morphism f_A: A → EditFieldData,
//! and the fold produces the unique morphism DomainNode → EditFieldData
//! satisfying the universal property.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let edit_data = domain_node.fold(&FoldEditFields);
//! property_card.set_from_edit_data(edit_data);
//! ```

use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    Person, KeyOwnerRole, Organization, OrganizationUnit, Location, Role, Policy,
    LocationType, PolicyClaim, RoleType,
};
use crate::domain::pki::{KeyAlgorithm, KeyPurpose};
use crate::domain::yubikey::PIVSlot;
use crate::domain_projections::NatsIdentityProjection;
use crate::policy::SeparationClass;
use crate::gui::domain_node::FoldDomainNode;

/// Data extracted from a domain node for editing in property card
#[derive(Debug, Clone, Default)]
pub struct EditFieldData {
    // Core fields (all entity types)
    pub name: String,
    pub description: String,
    pub email: String,
    pub enabled: bool,

    // Roles (for Person) - string representation for display
    pub roles: Vec<String>,
    // Actual RoleType values for checkbox binding
    pub role_types: Vec<RoleType>,

    // Location-specific fields
    pub location_type: Option<LocationType>,
    pub address: Option<String>,
    pub coordinates: Option<String>,
    pub virtual_location: Option<String>,

    // Policy-specific fields (claim descriptions: "resource:action")
    pub claims: Option<Vec<String>>,
    // Actual PolicyClaim values for checkbox binding
    pub policy_claims: Vec<PolicyClaim>,

    // Read-only indicator (some types can't be edited)
    pub read_only: bool,

    // Entity type for UI display
    pub entity_type: EntityType,
}

/// Categorization of entity types for UI purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityType {
    #[default]
    Organization,
    OrganizationUnit,
    Person,
    Location,
    Role,
    Policy,
    NatsOperator,
    NatsAccount,
    NatsUser,
    NatsServiceAccount,
    Certificate,
    YubiKey,
    PivSlot,
    Key,
    Manifest,
    PolicyRole,
    PolicyClaim,
    PolicyCategory,
    PolicyGroup,
    Aggregate,
    Unknown,
}

/// Fold that extracts edit field data from domain nodes
///
/// This implements the universal property of the DomainNode coproduct,
/// providing morphisms from each injection type to EditFieldData.
pub struct FoldEditFields;

impl FoldDomainNode for FoldEditFields {
    type Output = EditFieldData;

    fn fold_person(&self, person: &Person, _role: &KeyOwnerRole) -> Self::Output {
        EditFieldData {
            name: person.name.clone(),
            email: person.email.clone(),
            enabled: person.active,
            roles: person.roles.iter().map(|r| format!("{:?}", r.role_type)).collect(),
            role_types: person.roles.iter().map(|r| r.role_type.clone()).collect(),
            entity_type: EntityType::Person,
            ..Default::default()
        }
    }

    fn fold_organization(&self, org: &Organization) -> Self::Output {
        EditFieldData {
            name: org.name.clone(),
            description: org.description.clone().unwrap_or_default(),
            enabled: true,
            entity_type: EntityType::Organization,
            ..Default::default()
        }
    }

    fn fold_organization_unit(&self, unit: &OrganizationUnit) -> Self::Output {
        EditFieldData {
            name: unit.name.clone(),
            enabled: true,
            entity_type: EntityType::OrganizationUnit,
            ..Default::default()
        }
    }

    fn fold_location(&self, loc: &Location) -> Self::Output {
        let address = loc.address.as_ref().map(|addr| {
            let mut parts = vec![addr.street1.clone()];
            if let Some(street2) = &addr.street2 {
                parts.push(street2.clone());
            }
            parts.push(addr.locality.clone());
            parts.push(addr.region.clone());
            parts.push(addr.country.clone());
            parts.push(addr.postal_code.clone());
            parts.join(", ")
        });

        let coordinates = loc.coordinates.as_ref()
            .map(|coords| format!("{}, {}", coords.latitude, coords.longitude));

        let virtual_location = loc.virtual_location.as_ref().map(|vl| {
            if !vl.urls.is_empty() {
                vl.urls[0].url.clone()
            } else {
                vl.primary_identifier.clone()
            }
        });

        EditFieldData {
            name: loc.name.clone(),
            enabled: true,
            location_type: Some(loc.location_type.clone()),
            address,
            coordinates,
            virtual_location,
            entity_type: EntityType::Location,
            ..Default::default()
        }
    }

    fn fold_role(&self, role: &Role) -> Self::Output {
        EditFieldData {
            name: role.name.clone(),
            description: role.description.clone(),
            enabled: role.active,
            entity_type: EntityType::Role,
            ..Default::default()
        }
    }

    fn fold_policy(&self, policy: &Policy) -> Self::Output {
        EditFieldData {
            name: policy.name.clone(),
            description: policy.description.clone(),
            enabled: policy.enabled,
            claims: Some(policy.claims.iter().map(|c| format!("{:?}", c)).collect()),
            policy_claims: policy.claims.clone(),
            entity_type: EntityType::Policy,
            ..Default::default()
        }
    }

    // NATS Infrastructure - read-only
    fn fold_nats_operator(&self, proj: &NatsIdentityProjection) -> Self::Output {
        EditFieldData {
            name: "NATS Operator".to_string(),
            description: format!("NKey: {}", proj.nkey.public_key.public_key()),
            read_only: true,
            entity_type: EntityType::NatsOperator,
            ..Default::default()
        }
    }

    fn fold_nats_account(&self, proj: &NatsIdentityProjection) -> Self::Output {
        EditFieldData {
            name: "NATS Account".to_string(),
            description: format!("NKey: {}", proj.nkey.public_key.public_key()),
            read_only: true,
            entity_type: EntityType::NatsAccount,
            ..Default::default()
        }
    }

    fn fold_nats_user(&self, proj: &NatsIdentityProjection) -> Self::Output {
        EditFieldData {
            name: "NATS User".to_string(),
            description: format!("NKey: {}", proj.nkey.public_key.public_key()),
            read_only: true,
            entity_type: EntityType::NatsUser,
            ..Default::default()
        }
    }

    fn fold_nats_service_account(&self, proj: &NatsIdentityProjection) -> Self::Output {
        EditFieldData {
            name: "Service Account".to_string(),
            description: format!("NKey: {}", proj.nkey.public_key.public_key()),
            read_only: true,
            entity_type: EntityType::NatsServiceAccount,
            ..Default::default()
        }
    }

    fn fold_nats_operator_simple(&self, name: &str, _org_id: Option<Uuid>) -> Self::Output {
        EditFieldData {
            name: format!("Operator: {}", name),
            read_only: true,
            entity_type: EntityType::NatsOperator,
            ..Default::default()
        }
    }

    fn fold_nats_account_simple(&self, name: &str, _unit_id: Option<Uuid>, is_system: bool) -> Self::Output {
        EditFieldData {
            name: format!("Account: {}", name),
            description: if is_system { "System Account".to_string() } else { String::new() },
            read_only: true,
            entity_type: EntityType::NatsAccount,
            ..Default::default()
        }
    }

    fn fold_nats_user_simple(&self, name: &str, _person_id: Option<Uuid>, account_name: &str) -> Self::Output {
        EditFieldData {
            name: format!("User: {}", name),
            description: format!("Account: {}", account_name),
            read_only: true,
            entity_type: EntityType::NatsUser,
            ..Default::default()
        }
    }

    // Certificates - read-only
    fn fold_root_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        EditFieldData {
            name: format!("Root CA: {}", subject),
            description: format!("Issuer: {}, Valid: {} to {}", issuer, not_before.format("%Y-%m-%d"), not_after.format("%Y-%m-%d")),
            read_only: true,
            entity_type: EntityType::Certificate,
            ..Default::default()
        }
    }

    fn fold_intermediate_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
    ) -> Self::Output {
        EditFieldData {
            name: format!("Intermediate CA: {}", subject),
            description: format!("Issuer: {}, Valid: {} to {}", issuer, not_before.format("%Y-%m-%d"), not_after.format("%Y-%m-%d")),
            read_only: true,
            entity_type: EntityType::Certificate,
            ..Default::default()
        }
    }

    fn fold_leaf_certificate(
        &self,
        _cert_id: Uuid,
        subject: &str,
        issuer: &str,
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        _key_usage: &[String],
        _san: &[String],
    ) -> Self::Output {
        EditFieldData {
            name: format!("Certificate: {}", subject),
            description: format!("Issuer: {}, Valid: {} to {}", issuer, not_before.format("%Y-%m-%d"), not_after.format("%Y-%m-%d")),
            read_only: true,
            entity_type: EntityType::Certificate,
            ..Default::default()
        }
    }

    // Key - read-only
    fn fold_key(
        &self,
        _key_id: Uuid,
        algorithm: &KeyAlgorithm,
        purpose: &KeyPurpose,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self::Output {
        let expiry = expires_at.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or("Never".to_string());
        EditFieldData {
            name: format!("{:?} Key", algorithm),
            description: format!("Purpose: {:?}, Expires: {}", purpose, expiry),
            read_only: true,
            entity_type: EntityType::Key,
            ..Default::default()
        }
    }

    // YubiKey - read-only
    fn fold_yubikey(
        &self,
        _device_id: Uuid,
        serial: &str,
        version: &str,
        _provisioned_at: Option<DateTime<Utc>>,
        slots_used: &[String],
    ) -> Self::Output {
        EditFieldData {
            name: format!("YubiKey {}", serial),
            description: format!("Version: {}, Slots: {}", version, slots_used.len()),
            read_only: true,
            entity_type: EntityType::YubiKey,
            ..Default::default()
        }
    }

    fn fold_piv_slot(
        &self,
        _slot_id: Uuid,
        slot_name: &str,
        yubikey_serial: &str,
        has_key: bool,
        _certificate_subject: Option<&String>,
    ) -> Self::Output {
        EditFieldData {
            name: format!("PIV Slot: {}", slot_name),
            description: format!("YubiKey: {}, Has Key: {}", yubikey_serial, has_key),
            read_only: true,
            entity_type: EntityType::PivSlot,
            ..Default::default()
        }
    }

    fn fold_yubikey_status(
        &self,
        _person_id: Uuid,
        yubikey_serial: Option<&String>,
        slots_provisioned: &[PIVSlot],
        slots_needed: &[PIVSlot],
    ) -> Self::Output {
        EditFieldData {
            name: format!("YubiKey Status: {}", yubikey_serial.map(|s| s.as_str()).unwrap_or("None")),
            description: format!("Provisioned: {}, Needed: {}", slots_provisioned.len(), slots_needed.len()),
            read_only: true,
            entity_type: EntityType::YubiKey,
            ..Default::default()
        }
    }

    // Manifest - read-only
    fn fold_manifest(
        &self,
        _manifest_id: Uuid,
        name: &str,
        destination: Option<&PathBuf>,
        _checksum: Option<&String>,
    ) -> Self::Output {
        EditFieldData {
            name: name.to_string(),
            description: format!("Destination: {}", destination.map(|p| p.display().to_string()).unwrap_or("Not set".to_string())),
            read_only: true,
            entity_type: EntityType::Manifest,
            ..Default::default()
        }
    }

    // Policy hierarchy - read-only
    fn fold_policy_role(
        &self,
        _role_id: Uuid,
        name: &str,
        purpose: &str,
        level: u8,
        _separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self::Output {
        EditFieldData {
            name: name.to_string(),
            description: format!("Purpose: {}, Level: {}, Claims: {}", purpose, level, claim_count),
            read_only: true,
            entity_type: EntityType::PolicyRole,
            ..Default::default()
        }
    }

    fn fold_policy_claim(
        &self,
        _claim_id: Uuid,
        name: &str,
        category: &str,
    ) -> Self::Output {
        EditFieldData {
            name: name.to_string(),
            description: format!("Category: {}", category),
            read_only: true,
            entity_type: EntityType::PolicyClaim,
            ..Default::default()
        }
    }

    fn fold_policy_category(
        &self,
        _category_id: Uuid,
        name: &str,
        claim_count: usize,
        _expanded: bool,
    ) -> Self::Output {
        EditFieldData {
            name: name.to_string(),
            description: format!("Claims: {}", claim_count),
            read_only: true,
            entity_type: EntityType::PolicyCategory,
            ..Default::default()
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
        // Note: SeparationClass should be renamed to DutyBoundary in domain language
        EditFieldData {
            name: name.to_string(),
            description: format!("DutyBoundary: {:?}, Roles: {}", separation_class, role_count),
            read_only: true,
            entity_type: EntityType::PolicyGroup,
            ..Default::default()
        }
    }

    // Aggregates - read-only
    fn fold_aggregate_organization(
        &self,
        name: &str,
        version: u64,
        people_count: usize,
        units_count: usize,
    ) -> Self::Output {
        EditFieldData {
            name: name.to_string(),
            description: format!("v{}: {} people, {} units", version, people_count, units_count),
            read_only: true,
            entity_type: EntityType::Aggregate,
            ..Default::default()
        }
    }

    fn fold_aggregate_pki_chain(
        &self,
        name: &str,
        version: u64,
        certificates_count: usize,
        keys_count: usize,
    ) -> Self::Output {
        EditFieldData {
            name: name.to_string(),
            description: format!("v{}: {} certs, {} keys", version, certificates_count, keys_count),
            read_only: true,
            entity_type: EntityType::Aggregate,
            ..Default::default()
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
        EditFieldData {
            name: name.to_string(),
            description: format!("v{}: {} ops, {} accts, {} users", version, operators_count, accounts_count, users_count),
            read_only: true,
            entity_type: EntityType::Aggregate,
            ..Default::default()
        }
    }

    fn fold_aggregate_yubikey_provisioning(
        &self,
        name: &str,
        version: u64,
        devices_count: usize,
        slots_provisioned: usize,
    ) -> Self::Output {
        EditFieldData {
            name: name.to_string(),
            description: format!("v{}: {} devices, {} slots", version, devices_count, slots_provisioned),
            read_only: true,
            entity_type: EntityType::Aggregate,
            ..Default::default()
        }
    }
}

// ============================================================================
// LIFTED NODE EXTRACTION - Direct extraction without deprecated fold pattern
// ============================================================================

use crate::lifting::LiftedNode;

/// Extract edit field data from a LiftedNode using downcast.
///
/// This function replaces the deprecated `domain_node.fold(&FoldEditFields)` pattern
/// by using LiftedNode's downcast capability to extract domain data directly.
pub fn extract_edit_fields_from_lifted(node: &LiftedNode) -> EditFieldData {
    // Try to downcast to each known type and extract data directly
    if let Some(person) = node.downcast::<Person>() {
        return EditFieldData {
            name: person.name.clone(),
            email: person.email.clone(),
            enabled: person.active,
            roles: person.roles.iter().map(|r| format!("{:?}", r.role_type)).collect(),
            role_types: person.roles.iter().map(|r| r.role_type.clone()).collect(),
            entity_type: EntityType::Person,
            ..Default::default()
        };
    }

    if let Some(org) = node.downcast::<Organization>() {
        return EditFieldData {
            name: org.name.clone(),
            description: org.description.clone().unwrap_or_default(),
            enabled: true,
            entity_type: EntityType::Organization,
            ..Default::default()
        };
    }

    if let Some(unit) = node.downcast::<OrganizationUnit>() {
        return EditFieldData {
            name: unit.name.clone(),
            enabled: true,
            entity_type: EntityType::OrganizationUnit,
            ..Default::default()
        };
    }

    if let Some(location) = node.downcast::<Location>() {
        let address = location.address.as_ref().map(|addr| {
            let mut parts = vec![addr.street1.clone()];
            if let Some(street2) = &addr.street2 {
                parts.push(street2.clone());
            }
            parts.push(addr.locality.clone());
            parts.push(addr.region.clone());
            parts.push(addr.country.clone());
            parts.push(addr.postal_code.clone());
            parts.join(", ")
        });
        let coordinates = location.coordinates.as_ref()
            .map(|coords| format!("{}, {}", coords.latitude, coords.longitude));
        let virtual_location = location.virtual_location.as_ref().map(|vl| {
            if !vl.urls.is_empty() {
                vl.urls[0].url.clone()
            } else {
                vl.primary_identifier.clone()
            }
        });
        return EditFieldData {
            name: location.name.clone(),
            enabled: true,
            location_type: Some(location.location_type.clone()),
            address,
            coordinates,
            virtual_location,
            entity_type: EntityType::Location,
            ..Default::default()
        };
    }

    if let Some(role) = node.downcast::<Role>() {
        return EditFieldData {
            name: role.name.clone(),
            description: role.description.clone(),
            enabled: role.active,
            entity_type: EntityType::Role,
            ..Default::default()
        };
    }

    if let Some(policy) = node.downcast::<Policy>() {
        return EditFieldData {
            name: policy.name.clone(),
            description: policy.description.clone(),
            enabled: policy.enabled,
            claims: Some(policy.claims.iter().map(|c| format!("{:?}", c)).collect()),
            policy_claims: policy.claims.clone(),
            entity_type: EntityType::Policy,
            ..Default::default()
        };
    }

    // For types that don't have dedicated downcast support yet,
    // use label from the lifted node
    let read_only = is_read_only_injection(node.injection);
    EditFieldData {
        name: node.label.clone(),
        description: node.secondary.clone().unwrap_or_default(),
        read_only,
        entity_type: entity_type_from_injection(node.injection),
        ..Default::default()
    }
}

/// Check if an Injection type represents a read-only entity
fn is_read_only_injection(injection: crate::gui::domain_node::Injection) -> bool {
    use crate::gui::domain_node::Injection;
    matches!(injection,
        Injection::NatsOperator | Injection::NatsOperatorSimple |
        Injection::NatsAccount | Injection::NatsAccountSimple |
        Injection::NatsUser | Injection::NatsUserSimple | Injection::NatsServiceAccount |
        Injection::RootCertificate | Injection::IntermediateCertificate | Injection::LeafCertificate |
        Injection::Key | Injection::YubiKey | Injection::YubiKeyStatus | Injection::PivSlot |
        Injection::Manifest | Injection::PolicyRole | Injection::PolicyClaim |
        Injection::PolicyCategory | Injection::PolicyGroup |
        Injection::AggregateOrganization | Injection::AggregatePkiChain |
        Injection::AggregateNatsSecurity | Injection::AggregateYubiKeyProvisioning
    )
}

/// Convert Injection to EntityType
fn entity_type_from_injection(injection: crate::gui::domain_node::Injection) -> EntityType {
    use crate::gui::domain_node::Injection;
    match injection {
        Injection::Organization => EntityType::Organization,
        Injection::OrganizationUnit => EntityType::OrganizationUnit,
        Injection::Person => EntityType::Person,
        Injection::Location => EntityType::Location,
        Injection::Role => EntityType::Role,
        Injection::Policy => EntityType::Policy,
        Injection::NatsOperator | Injection::NatsOperatorSimple => EntityType::NatsOperator,
        Injection::NatsAccount | Injection::NatsAccountSimple => EntityType::NatsAccount,
        Injection::NatsUser | Injection::NatsUserSimple | Injection::NatsServiceAccount => EntityType::NatsUser,
        Injection::RootCertificate | Injection::IntermediateCertificate | Injection::LeafCertificate => EntityType::Certificate,
        Injection::Key => EntityType::Key,
        Injection::YubiKey | Injection::YubiKeyStatus => EntityType::YubiKey,
        Injection::PivSlot => EntityType::PivSlot,
        Injection::Manifest => EntityType::Manifest,
        Injection::PolicyRole => EntityType::PolicyRole,
        Injection::PolicyClaim => EntityType::PolicyClaim,
        Injection::PolicyCategory => EntityType::PolicyCategory,
        Injection::PolicyGroup => EntityType::PolicyGroup,
        Injection::AggregateOrganization | Injection::AggregatePkiChain |
        Injection::AggregateNatsSecurity | Injection::AggregateYubiKeyProvisioning => EntityType::Aggregate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::domain_node::DomainNode;

    #[test]
    fn test_fold_organization_extracts_edit_fields() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("A test organization".to_string()),
            parent_id: None,
            units: vec![],
            metadata: std::collections::HashMap::new(),
        };

        let node = DomainNode::inject_organization(org);
        let edit_data = node.fold(&FoldEditFields);

        assert_eq!(edit_data.name, "Test Org");
        assert_eq!(edit_data.description, "A test organization");
        assert!(edit_data.enabled);
        assert!(!edit_data.read_only);
        assert_eq!(edit_data.entity_type, EntityType::Organization);
    }

    #[test]
    fn test_fold_nats_is_read_only() {
        let node = DomainNode::inject_nats_operator_simple("test-operator".to_string(), None);
        let edit_data = node.fold(&FoldEditFields);

        assert!(edit_data.read_only);
        assert_eq!(edit_data.entity_type, EntityType::NatsOperator);
    }
}
