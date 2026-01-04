// Copyright (c) 2025 - Cowboy AI, LLC.

//! Edit Field Extraction - FRP A5/A6 Compliant
//!
//! Extracts editable field data from lifted domain nodes for use
//! in property cards and edit forms.
//!
//! ## FRP Axiom Compliance
//!
//! - **A5 (Totality)**: All fold cases provided at construction time via `Foldable` trait
//! - **A6 (Explicit Routing)**: Fold capability captured at lift time, no pattern matching
//!
//! ## Categorical Foundation
//!
//! This module implements extraction via the **universal property of coproducts**:
//!
//! For a coproduct `A + B + C + ...` and morphisms `f_A: A → EditFieldData`,
//! `f_B: B → EditFieldData`, etc., the universal property guarantees a unique
//! morphism `[f_A, f_B, ...]: A + B + ... → EditFieldData`.
//!
//! The `FoldCapability<EditFieldData>` stored in `LiftedNode` IS this unique morphism,
//! captured at lift time. Fold execution requires NO pattern matching or downcasting.
//!
//! ## Usage
//!
//! ```rust,ignore
//! // At lift time - fold captured via new_with_fold()
//! let node = person.lift(); // FoldCapability stored internally
//!
//! // At fold time - NO pattern matching, just closure execution
//! let edit_data = extract_edit_fields_from_lifted(&node);
//! property_card.set_from_edit_data(edit_data);
//! ```

use crate::domain::{
    Person, Organization, OrganizationUnit, Location, Role, Policy,
    LocationType, PolicyClaim, RoleType,
};
use crate::lifting::LiftedNode;

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
    WorkflowGap,
    Unknown,
}

/// Fold struct for edit field extraction (kept for backward compatibility)
///
/// Note: Use `extract_edit_fields_from_lifted` function directly instead.
pub struct FoldEditFields;

// ============================================================================
// LIFTED NODE EXTRACTION - FRP A5/A6 Compliant Categorical Fold
// ============================================================================

/// Extract edit field data from a LiftedNode.
///
/// ## FRP A5/A6 Compliance
///
/// For domain types with `Foldable<EditFieldData>` implementations (Person, Organization,
/// OrganizationUnit, Location, Role, Policy), the fold is captured at lift time via
/// `LiftedNode::new_with_fold()`. This function first checks for a fold capability and
/// executes it directly - **NO pattern matching or downcasting required**.
///
/// For legacy types that don't have fold capabilities, this falls back to downcasting.
///
/// ## Categorical Foundation
///
/// The fold capability is the **coproduct eliminator**: given morphisms f_A: A → X,
/// f_B: B → X, ..., the unique morphism [f_A, f_B, ...]: A+B+... → X is captured
/// at lift time. Fold execution is simply closure invocation.
pub fn extract_edit_fields_from_lifted(node: &LiftedNode) -> EditFieldData {
    // FRP A5/A6: Try categorical fold first (no pattern matching)
    if let Some(edit_data) = node.fold_edit_fields() {
        return edit_data;
    }

    // Legacy fallback: downcast for types without fold capability
    // This branch is deprecated - new types should implement Foldable<EditFieldData>
    legacy_extract_via_downcast(node)
}

/// Legacy extraction via downcasting.
///
/// **DEPRECATED**: New domain types should implement `Foldable<EditFieldData>` and
/// use `LiftedNode::new_with_fold()` in their `lift()` implementation instead.
fn legacy_extract_via_downcast(node: &LiftedNode) -> EditFieldData {
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
fn is_read_only_injection(injection: crate::lifting::Injection) -> bool {
    use crate::lifting::Injection;
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
fn entity_type_from_injection(injection: crate::lifting::Injection) -> EntityType {
    use crate::lifting::Injection;
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
        Injection::WorkflowGap => EntityType::WorkflowGap,
        // State machine visualization nodes - treat similarly to WorkflowGap
        Injection::StateMachineState | Injection::StateMachineTransition => EntityType::WorkflowGap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifting::{Injection, LiftableDomain};
    use crate::domain::ids::BootstrapOrgId;

    #[test]
    fn test_extract_organization_from_lifted() {
        let org = Organization {
            id: BootstrapOrgId::new(),
            name: "Test Org".to_string(),
            display_name: "Test Organization".to_string(),
            description: Some("A test organization".to_string()),
            parent_id: None,
            units: vec![],
            metadata: std::collections::HashMap::new(),
        };

        // Use the LiftableDomain::lift() functor - the correct categorical pattern
        let node = org.lift();

        let edit_data = extract_edit_fields_from_lifted(&node);

        assert_eq!(edit_data.name, "Test Org");
        assert_eq!(edit_data.description, "A test organization");
        assert!(edit_data.enabled);
        assert!(!edit_data.read_only);
        assert_eq!(edit_data.entity_type, EntityType::Organization);
    }

    #[test]
    fn test_nats_injection_is_read_only() {
        assert!(is_read_only_injection(Injection::NatsOperator));
        assert!(is_read_only_injection(Injection::NatsOperatorSimple));
        assert!(is_read_only_injection(Injection::NatsAccount));
        assert!(is_read_only_injection(Injection::YubiKey));
        assert!(is_read_only_injection(Injection::RootCertificate));
    }

    #[test]
    fn test_editable_injection_not_read_only() {
        assert!(!is_read_only_injection(Injection::Organization));
        assert!(!is_read_only_injection(Injection::Person));
        assert!(!is_read_only_injection(Injection::Location));
        assert!(!is_read_only_injection(Injection::Role));
        assert!(!is_read_only_injection(Injection::Policy));
    }

    #[test]
    fn test_entity_type_from_injection() {
        assert_eq!(entity_type_from_injection(Injection::Organization), EntityType::Organization);
        assert_eq!(entity_type_from_injection(Injection::Person), EntityType::Person);
        assert_eq!(entity_type_from_injection(Injection::NatsOperator), EntityType::NatsOperator);
        assert_eq!(entity_type_from_injection(Injection::NatsOperatorSimple), EntityType::NatsOperator);
        assert_eq!(entity_type_from_injection(Injection::RootCertificate), EntityType::Certificate);
        assert_eq!(entity_type_from_injection(Injection::LeafCertificate), EntityType::Certificate);
    }
}
