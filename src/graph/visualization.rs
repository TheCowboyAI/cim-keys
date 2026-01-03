// Copyright (c) 2025 - Cowboy AI, LLC.

//! Visualization Morphism Registry
//!
//! This module provides a `VisualizationRegistry` that registers morphisms
//! for converting domain entities to visualization data, replacing the
//! 29-arm match in `LiftedNode::themed_visualization()`.
//!
//! # Migration Path
//!
//! Before (29-arm pattern matching):
//! ```rust,ignore
//! match self.injection {
//!     Injection::Person => { ... fold_person(...) }
//!     Injection::Organization => { ... fold_organization(...) }
//!     // ... 27 more arms
//! }
//! ```
//!
//! After (morphisms as DATA):
//! ```rust,ignore
//! let registry = VisualizationRegistry::new(theme);
//! registry.fold(&node)
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use cim_keys::graph::visualization::VisualizationRegistry;
//!
//! let registry = VisualizationRegistry::new(&theme);
//! let vis_data = registry.fold(&lifted_node)?;
//! ```

use crate::graph::morphism::MorphismRegistry;
use crate::lifting::{Injection, LiftedNode};
use crate::gui::folds::view::{ThemedVisualizationData, ThemedVisualizationFold};
use crate::domains::typography::VerifiedTheme;
use crate::domain::{Organization, OrganizationUnit, Person, Location};

/// Registry of visualization morphisms for all domain types.
///
/// This wraps a `MorphismRegistry<ThemedVisualizationData>` with the
/// theme context needed for visualization.
pub struct VisualizationRegistry {
    inner: MorphismRegistry<ThemedVisualizationData>,
}

impl VisualizationRegistry {
    /// Create a new visualization registry for the given theme.
    ///
    /// This registers morphisms for all core domain types.
    /// Each morphism transforms a domain type to `ThemedVisualizationData`.
    pub fn new(theme: &VerifiedTheme) -> Self {
        let theme_clone = theme.clone();
        let theme_for_org = theme.clone();
        let theme_for_unit = theme.clone();
        let theme_for_loc = theme.clone();

        let registry = MorphismRegistry::<ThemedVisualizationData>::new()
            // Person morphism
            .with::<Person, _>(move |person| {
                use crate::domain::RoleType;
                use crate::domain::KeyOwnerRole;
                use iced::Color;

                let fold = ThemedVisualizationFold::new(&theme_clone);

                // Derive KeyOwnerRole from primary role
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

                let role_name = format!("{:?}", role);
                let role_color = match role {
                    KeyOwnerRole::RootAuthority => Color::from_rgb(0.8, 0.1, 0.1),
                    KeyOwnerRole::SecurityAdmin => Color::from_rgb(0.6, 0.2, 0.8),
                    KeyOwnerRole::Developer => Color::from_rgb(0.2, 0.7, 0.3),
                    KeyOwnerRole::ServiceAccount => Color::from_rgb(0.4, 0.4, 0.6),
                    KeyOwnerRole::Auditor => Color::from_rgb(0.5, 0.5, 0.5),
                    KeyOwnerRole::BackupHolder => Color::from_rgb(0.7, 0.5, 0.2),
                };

                fold.fold_person(&person.name, &person.email, &role_name, role_color)
            })
            // Organization morphism
            .with::<Organization, _>(move |org| {
                let fold = ThemedVisualizationFold::new(&theme_for_org);
                fold.fold_organization(&org.name, &org.display_name, org.description.as_deref())
            })
            // OrganizationUnit morphism
            .with::<OrganizationUnit, _>(move |unit| {
                let fold = ThemedVisualizationFold::new(&theme_for_unit);
                fold.fold_organization_unit(&unit.name)
            })
            // Location morphism
            .with::<Location, _>(move |loc| {
                let fold = ThemedVisualizationFold::new(&theme_for_loc);
                fold.fold_location(&loc.name, &format!("{:?}", loc.location_type))
            });
            // Additional morphisms would be registered here for:
            // - Key, Certificate, YubiKey, NATS entities, etc.
            // Each follows the same pattern: .with::<Type, _>(|entity| fold.fold_xxx(...))

        VisualizationRegistry { inner: registry }
    }

    /// Fold a lifted node to visualization data.
    ///
    /// Returns `None` if no morphism is registered for the node's injection type.
    /// This indicates a domain type that doesn't have visualization support.
    #[inline]
    pub fn fold(&self, node: &LiftedNode) -> Option<ThemedVisualizationData> {
        self.inner.fold(node)
    }

    /// Check if a visualization morphism exists for an injection type.
    pub fn has_visualization(&self, injection: Injection) -> bool {
        self.inner.has_morphism(injection)
    }

    /// Get the number of registered visualization morphisms.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// ============================================================================
// BUILDER FOR COMPLETE VISUALIZATION REGISTRY
// ============================================================================

/// Builder for incrementally constructing a complete visualization registry.
///
/// This pattern allows gradual migration from the 29-arm match to
/// individual morphism registrations.
pub struct VisualizationRegistryBuilder {
    inner: MorphismRegistry<ThemedVisualizationData>,
}

impl VisualizationRegistryBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        VisualizationRegistryBuilder {
            inner: MorphismRegistry::new(),
        }
    }

    /// Register a custom morphism for a domain type.
    pub fn with<A, F>(mut self, f: F) -> Self
    where
        A: crate::lifting::LiftableDomain + Clone + Send + Sync + 'static,
        F: Fn(&A) -> ThemedVisualizationData + Send + Sync + 'static,
    {
        self.inner = self.inner.with::<A, F>(f);
        self
    }

    /// Register a morphism for a specific injection tag.
    pub fn with_injection<A, F>(mut self, injection: Injection, f: F) -> Self
    where
        A: Clone + Send + Sync + 'static,
        F: Fn(&A) -> ThemedVisualizationData + Send + Sync + 'static,
    {
        self.inner = self.inner.with_injection::<A, F>(injection, f);
        self
    }

    /// Build the final registry.
    pub fn build(self) -> VisualizationRegistry {
        VisualizationRegistry { inner: self.inner }
    }
}

impl Default for VisualizationRegistryBuilder {
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
    use crate::domains::typography::VerifiedTheme;
    use crate::lifting::LiftableDomain;

    fn test_theme() -> VerifiedTheme {
        VerifiedTheme::cim_default()
    }

    #[test]
    fn test_visualization_registry_person() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        let org_id = BootstrapOrgId::new();
        let person = Person::new("Alice", "alice@example.com", org_id);
        let node = LiftableDomain::lift(&person);

        let vis = registry.fold(&node);
        assert!(vis.is_some());

        // ThemedVisualizationData has `primary` field with LabelledElement
        // Just verify we got a result
        let _vis_data = vis.unwrap();
    }

    #[test]
    fn test_visualization_registry_organization() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        let org = Organization::new("TestOrg", "Test Organization");
        let node = LiftableDomain::lift(&org);

        let vis = registry.fold(&node);
        assert!(vis.is_some());

        // Just verify we got a result
        let _vis_data = vis.unwrap();
    }

    #[test]
    fn test_visualization_registry_unregistered_type() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        // Key type is not registered in the basic registry
        assert!(!registry.has_visualization(Injection::Key));
    }

    #[test]
    fn test_builder_pattern() {
        let theme = test_theme();
        let registry = VisualizationRegistryBuilder::new()
            .with::<Person, _>(move |_p| ThemedVisualizationData::empty(&theme))
            .build();

        // Registry should have exactly 1 morphism
        assert_eq!(registry.len(), 1);
    }
}
