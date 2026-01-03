// Copyright (c) 2025 - Cowboy AI, LLC.

//! Detail Panel Morphism Registry
//!
//! This module provides a `DetailPanelRegistry` that registers morphisms
//! for extracting detail panel data from domain entities, replacing the
//! 29-arm match in `LiftedNode::detail_panel()`.
//!
//! # Migration Path
//!
//! Before (29-arm pattern matching):
//! ```rust,ignore
//! match self.injection {
//!     Injection::Person => { DetailPanelData::new("Person:").with_field(...) }
//!     Injection::Organization => { DetailPanelData::new("Org:").with_field(...) }
//!     // ... 27 more arms
//! }
//! ```
//!
//! After (morphisms as DATA):
//! ```rust,ignore
//! let registry = DetailPanelRegistry::new();
//! registry.fold(&node)
//! ```

use crate::graph::morphism::MorphismRegistry;
use crate::lifting::{DetailPanelData, Injection, LiftedNode, LiftableDomain};
use crate::domain::{Organization, OrganizationUnit, Person, Location};

/// Registry of detail panel morphisms for all domain types.
pub struct DetailPanelRegistry {
    inner: MorphismRegistry<DetailPanelData>,
}

impl DetailPanelRegistry {
    /// Create a new detail panel registry with all core domain morphisms registered.
    pub fn new() -> Self {
        let registry = MorphismRegistry::<DetailPanelData>::new()
            // Person morphism
            .with::<Person, _>(|person| {
                use crate::domain::{RoleType, KeyOwnerRole};

                // Derive role from first role
                let role = person.roles.first()
                    .map(|r| match r.role_type {
                        RoleType::Executive => KeyOwnerRole::RootAuthority,
                        RoleType::Administrator => KeyOwnerRole::SecurityAdmin,
                        RoleType::Developer => KeyOwnerRole::Developer,
                        RoleType::Operator => KeyOwnerRole::ServiceAccount,
                        RoleType::Auditor => KeyOwnerRole::Auditor,
                        RoleType::Service => KeyOwnerRole::ServiceAccount,
                    })
                    .unwrap_or(KeyOwnerRole::Developer);

                DetailPanelData::new("Selected Person:")
                    .with_field("Name", &person.name)
                    .with_field("Email", &person.email)
                    .with_field("Active", if person.active { "✓" } else { "✗" })
                    .with_field("Key Role", format!("{:?}", role))
            })
            // Organization morphism
            .with::<Organization, _>(|org| {
                DetailPanelData::new("Selected Organization:")
                    .with_field("Name", &org.name)
                    .with_field("Display Name", &org.display_name)
                    .with_field("Units", org.units.len().to_string())
            })
            // OrganizationUnit morphism
            .with::<OrganizationUnit, _>(|unit| {
                DetailPanelData::new("Selected Unit:")
                    .with_field("Name", &unit.name)
                    .with_field("Type", format!("{:?}", unit.unit_type))
            })
            // Location morphism
            .with::<Location, _>(|loc| {
                DetailPanelData::new("Selected Location:")
                    .with_field("Name", &loc.name)
                    .with_field("Type", format!("{:?}", loc.location_type))
            });
            // Additional morphisms can be registered for:
            // - Key, Certificate, YubiKey, NATS entities, etc.

        DetailPanelRegistry { inner: registry }
    }

    /// Fold a lifted node to detail panel data.
    ///
    /// Returns `None` if no morphism is registered for the node's injection type.
    #[inline]
    pub fn fold(&self, node: &LiftedNode) -> Option<DetailPanelData> {
        self.inner.fold(node)
    }

    /// Check if a detail panel morphism exists for an injection type.
    pub fn has_morphism(&self, injection: Injection) -> bool {
        self.inner.has_morphism(injection)
    }

    /// Get the number of registered morphisms.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for DetailPanelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::BootstrapOrgId;

    #[test]
    fn test_detail_panel_registry_person() {
        let registry = DetailPanelRegistry::new();

        let org_id = BootstrapOrgId::new();
        let person = Person::new("Alice", "alice@example.com", org_id);
        let node = LiftableDomain::lift(&person);

        let detail = registry.fold(&node);
        assert!(detail.is_some());

        let data = detail.unwrap();
        assert!(data.title.contains("Person"));
        assert!(data.fields.iter().any(|(label, _)| label == "Name"));
    }

    #[test]
    fn test_detail_panel_registry_organization() {
        let registry = DetailPanelRegistry::new();

        let org = Organization::new("TestOrg", "Test Organization");
        let node = LiftableDomain::lift(&org);

        let detail = registry.fold(&node);
        assert!(detail.is_some());

        let data = detail.unwrap();
        assert!(data.title.contains("Organization"));
    }

    #[test]
    fn test_detail_panel_registry_unregistered() {
        let registry = DetailPanelRegistry::new();

        // Key is not registered
        assert!(!registry.has_morphism(Injection::Key));
    }

    #[test]
    fn test_detail_panel_registry_fields() {
        let registry = DetailPanelRegistry::new();

        let org_id = BootstrapOrgId::new();
        let person = Person::new("Bob", "bob@example.com", org_id);
        let node = LiftableDomain::lift(&person);

        let data = registry.fold(&node).unwrap();

        // Verify all expected fields are present
        let field_labels: Vec<&str> = data.fields.iter().map(|(l, _)| l.as_str()).collect();
        assert!(field_labels.contains(&"Name"));
        assert!(field_labels.contains(&"Email"));
        assert!(field_labels.contains(&"Active"));
        assert!(field_labels.contains(&"Key Role"));
    }
}
