// Copyright (c) 2025 - Cowboy AI, LLC.

//! Foldable Implementations for Domain Types
//!
//! This module implements the `Foldable<EditFieldData>` trait for all domain types,
//! following the categorical fold pattern required by N-ary FRP.
//!
//! ## FRP Axiom Compliance
//!
//! - **A5 (Totality)**: Each fold produces a valid EditFieldData for any valid domain value
//! - **A6 (Explicit Routing)**: No pattern matching at call sites - folds stored at lift time
//!
//! ## Categorical Foundation
//!
//! Each implementation is a morphism `T → EditFieldData` that forms part of the
//! coproduct eliminator `[f_Person, f_Org, ...]: LiftedNode → EditFieldData`.

use crate::fold::Foldable;
use crate::gui::folds::query::edit_fields::{EditFieldData, EntityType};

// Import domain types
use super::{
    Person, Organization, OrganizationUnit, Location, Role, Policy,
};

// ============================================================================
// PERSON FOLD
// ============================================================================

impl Foldable<EditFieldData> for Person {
    /// Fold Person to EditFieldData
    ///
    /// A5 Totality: Always produces valid EditFieldData
    fn fold(&self) -> EditFieldData {
        EditFieldData {
            name: self.name.clone(),
            email: self.email.clone(),
            enabled: self.active,
            roles: self.roles.iter().map(|r| format!("{:?}", r.role_type)).collect(),
            role_types: self.roles.iter().map(|r| r.role_type.clone()).collect(),
            entity_type: EntityType::Person,
            ..Default::default()
        }
    }
}

// ============================================================================
// ORGANIZATION FOLD
// ============================================================================

impl Foldable<EditFieldData> for Organization {
    /// Fold Organization to EditFieldData
    fn fold(&self) -> EditFieldData {
        EditFieldData {
            name: self.name.clone(),
            description: self.description.clone().unwrap_or_default(),
            enabled: true,
            entity_type: EntityType::Organization,
            ..Default::default()
        }
    }
}

// ============================================================================
// ORGANIZATION UNIT FOLD
// ============================================================================

impl Foldable<EditFieldData> for OrganizationUnit {
    /// Fold OrganizationUnit to EditFieldData
    fn fold(&self) -> EditFieldData {
        EditFieldData {
            name: self.name.clone(),
            enabled: true,
            entity_type: EntityType::OrganizationUnit,
            ..Default::default()
        }
    }
}

// ============================================================================
// LOCATION FOLD
// ============================================================================

impl Foldable<EditFieldData> for Location {
    /// Fold Location to EditFieldData
    fn fold(&self) -> EditFieldData {
        let address = self.address.as_ref().map(|addr| {
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

        let coordinates = self.coordinates.as_ref()
            .map(|coords| format!("{}, {}", coords.latitude, coords.longitude));

        let virtual_location = self.virtual_location.as_ref().map(|vl| {
            if !vl.urls.is_empty() {
                vl.urls[0].url.clone()
            } else {
                vl.primary_identifier.clone()
            }
        });

        EditFieldData {
            name: self.name.clone(),
            enabled: true,
            location_type: Some(self.location_type.clone()),
            address,
            coordinates,
            virtual_location,
            entity_type: EntityType::Location,
            ..Default::default()
        }
    }
}

// ============================================================================
// ROLE FOLD
// ============================================================================

impl Foldable<EditFieldData> for Role {
    /// Fold Role to EditFieldData
    fn fold(&self) -> EditFieldData {
        EditFieldData {
            name: self.name.clone(),
            description: self.description.clone(),
            enabled: self.active,
            entity_type: EntityType::Role,
            ..Default::default()
        }
    }
}

// ============================================================================
// POLICY FOLD
// ============================================================================

impl Foldable<EditFieldData> for Policy {
    /// Fold Policy to EditFieldData
    fn fold(&self) -> EditFieldData {
        EditFieldData {
            name: self.name.clone(),
            description: self.description.clone(),
            enabled: self.enabled,
            claims: Some(self.claims.iter().map(|c| format!("{:?}", c)).collect()),
            policy_claims: self.claims.clone(),
            entity_type: EntityType::Policy,
            ..Default::default()
        }
    }
}

// ============================================================================
// TESTS - Verify A5 Totality (all folds produce valid output)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::OrganizationUnitType;
    use crate::domain::ids::{BootstrapOrgId, BootstrapPersonId, UnitId, BootstrapRoleId, BootstrapPolicyId};

    #[test]
    fn test_person_fold_totality() {
        let person = Person {
            id: BootstrapPersonId::new(),
            name: "Alice Smith".to_string(),
            email: "alice@example.com".to_string(),
            roles: vec![],
            organization_id: BootstrapOrgId::new(),
            unit_ids: vec![],
            active: true,
            nats_permissions: None,
            owner_id: None,
        };

        let result: EditFieldData = person.fold();

        assert_eq!(result.name, "Alice Smith");
        assert_eq!(result.email, "alice@example.com");
        assert!(result.enabled);
        assert_eq!(result.entity_type, EntityType::Person);
    }

    #[test]
    fn test_organization_fold_totality() {
        let org = Organization {
            id: BootstrapOrgId::new(),
            name: "Acme Corp".to_string(),
            display_name: "Acme Corporation".to_string(),
            description: Some("A test company".to_string()),
            parent_id: None,
            units: vec![],
            metadata: std::collections::HashMap::new(),
        };

        let result: EditFieldData = org.fold();

        assert_eq!(result.name, "Acme Corp");
        assert_eq!(result.description, "A test company");
        assert_eq!(result.entity_type, EntityType::Organization);
    }

    #[test]
    fn test_organization_unit_fold_totality() {
        let unit = OrganizationUnit {
            id: UnitId::new(),
            name: "Engineering".to_string(),
            unit_type: OrganizationUnitType::Department,
            parent_unit_id: None,
            responsible_person_id: None,
            nats_account_name: None,
        };

        let result: EditFieldData = unit.fold();

        assert_eq!(result.name, "Engineering");
        assert_eq!(result.entity_type, EntityType::OrganizationUnit);
    }

    #[test]
    fn test_role_fold_totality() {
        let role = Role {
            id: BootstrapRoleId::new(),
            name: "Developer".to_string(),
            description: "Software development".to_string(),
            organization_id: BootstrapOrgId::new(),
            unit_id: None,
            required_policies: vec![],
            responsibilities: vec!["Write code".to_string()],
            created_by: BootstrapPersonId::new(),
            active: true,
        };

        let result: EditFieldData = role.fold();

        assert_eq!(result.name, "Developer");
        assert_eq!(result.description, "Software development");
        assert!(result.enabled);
        assert_eq!(result.entity_type, EntityType::Role);
    }

    #[test]
    fn test_policy_fold_totality() {
        let policy = Policy {
            id: BootstrapPolicyId::new(),
            name: "Admin Policy".to_string(),
            description: "Administrative access".to_string(),
            claims: vec![],
            conditions: vec![],
            priority: 100,
            enabled: true,
            created_by: BootstrapPersonId::new(),
            metadata: std::collections::HashMap::new(),
        };

        let result: EditFieldData = policy.fold();

        assert_eq!(result.name, "Admin Policy");
        assert_eq!(result.description, "Administrative access");
        assert!(result.enabled);
        assert_eq!(result.entity_type, EntityType::Policy);
    }
}
