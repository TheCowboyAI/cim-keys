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
use crate::lifting::{
    Injection, LiftedNode,
    // Domain types
    NatsOperatorSimple, NatsAccountSimple, NatsUserSimple,
    AggregateOrganization, AggregatePkiChain, AggregateNatsSecurity, AggregateYubiKeyProvisioning,
};
use crate::gui::folds::view::{ThemedVisualizationData, ThemedVisualizationFold, CertificateType as VisCertType};
use crate::domains::typography::VerifiedTheme;
use crate::domain::{Organization, OrganizationUnit, Person, Location, Role, Policy};
use crate::domain::pki::{Certificate, CertificateType, CryptographicKey};
use crate::domain::yubikey::{YubiKeyDevice, PivSlotView, YubiKeyStatus};
use crate::domain::visualization::{PolicyGroup, PolicyCategory, PolicyRole, PolicyClaimView};

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
        // Clone theme for each closure that needs it
        let theme_person = theme.clone();
        let theme_org = theme.clone();
        let theme_unit = theme.clone();
        let theme_loc = theme.clone();
        let theme_role = theme.clone();
        let theme_policy = theme.clone();
        let theme_cert = theme.clone();
        let theme_key = theme.clone();
        let theme_yubikey = theme.clone();
        let theme_piv = theme.clone();
        let theme_yk_status = theme.clone();
        let theme_nats_op = theme.clone();
        let theme_nats_acc = theme.clone();
        let theme_nats_user = theme.clone();
        let theme_pol_grp = theme.clone();
        let theme_pol_cat = theme.clone();
        let theme_pol_role = theme.clone();
        let theme_pol_claim = theme.clone();
        let theme_agg_org = theme.clone();
        let theme_agg_pki = theme.clone();
        let theme_agg_nats = theme.clone();
        let theme_agg_yk = theme.clone();

        let registry = MorphismRegistry::<ThemedVisualizationData>::new()
            // ================================================================
            // ORGANIZATION BOUNDED CONTEXT
            // ================================================================

            // Person morphism
            .with::<Person, _>(move |person| {
                use crate::domain::RoleType;
                use crate::domain::KeyOwnerRole;
                use iced::Color;

                let fold = ThemedVisualizationFold::new(&theme_person);

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
                let fold = ThemedVisualizationFold::new(&theme_org);
                fold.fold_organization(&org.name, &org.display_name, org.description.as_deref())
            })
            // OrganizationUnit morphism
            .with::<OrganizationUnit, _>(move |unit| {
                let fold = ThemedVisualizationFold::new(&theme_unit);
                fold.fold_organization_unit(&unit.name)
            })
            // Location morphism
            .with::<Location, _>(move |loc| {
                let fold = ThemedVisualizationFold::new(&theme_loc);
                fold.fold_location(&loc.name, &format!("{:?}", loc.location_type))
            })
            // Role morphism
            .with::<Role, _>(move |role| {
                let fold = ThemedVisualizationFold::new(&theme_role);
                fold.fold_role(&role.name, &role.description)
            })
            // Policy morphism
            .with::<Policy, _>(move |policy| {
                let fold = ThemedVisualizationFold::new(&theme_policy);
                fold.fold_policy(&policy.name, &policy.description)
            })

            // ================================================================
            // PKI BOUNDED CONTEXT
            // ================================================================

            // Certificate morphism
            .with::<Certificate, _>(move |cert| {
                let fold = ThemedVisualizationFold::new(&theme_cert);
                let expires = cert.not_after.format("%Y-%m-%d").to_string();
                let vis_cert_type = match cert.cert_type {
                    CertificateType::Root => VisCertType::Root,
                    CertificateType::Intermediate => VisCertType::Intermediate,
                    CertificateType::Leaf | CertificateType::Policy => VisCertType::Leaf,
                };
                fold.fold_certificate(&cert.subject, &expires, vis_cert_type)
            })
            // CryptographicKey morphism
            .with::<CryptographicKey, _>(move |key| {
                let fold = ThemedVisualizationFold::new(&theme_key);
                let purpose = format!("{:?}", key.purpose);
                let algorithm = format!("{:?}", key.algorithm);
                let expires = key.expires_at.map(|dt| dt.format("%Y-%m-%d").to_string());
                fold.fold_key(&purpose, &algorithm, expires.as_deref())
            })

            // ================================================================
            // YUBIKEY BOUNDED CONTEXT
            // ================================================================

            // YubiKeyDevice morphism
            .with::<YubiKeyDevice, _>(move |yk| {
                let fold = ThemedVisualizationFold::new(&theme_yubikey);
                fold.fold_yubikey(&yk.serial, &yk.version, yk.slots_used.len())
            })
            // PivSlotView morphism
            .with::<PivSlotView, _>(move |slot| {
                let fold = ThemedVisualizationFold::new(&theme_piv);
                let status = if slot.has_key { "Key present" } else { "Empty" };
                fold.fold_key(&slot.slot_name, status, slot.certificate_subject.as_deref())
            })
            // YubiKeyStatus morphism
            .with::<YubiKeyStatus, _>(move |status| {
                let fold = ThemedVisualizationFold::new(&theme_yk_status);
                let serial = status.yubikey_serial.clone().unwrap_or_else(|| "Not assigned".to_string());
                let version = format!("{}/{} slots", status.slots_provisioned.len(), status.slots_needed.len());
                fold.fold_yubikey(&serial, &version, status.slots_provisioned.len())
            })

            // ================================================================
            // NATS BOUNDED CONTEXT
            // ================================================================

            // NatsOperatorSimple morphism
            .with::<NatsOperatorSimple, _>(move |op| {
                let fold = ThemedVisualizationFold::new(&theme_nats_op);
                fold.fold_nats_operator(&op.name)
            })
            // NatsAccountSimple morphism
            .with::<NatsAccountSimple, _>(move |acc| {
                let fold = ThemedVisualizationFold::new(&theme_nats_acc);
                fold.fold_nats_account(&acc.name, acc.is_system)
            })
            // NatsUserSimple morphism
            .with::<NatsUserSimple, _>(move |user| {
                let fold = ThemedVisualizationFold::new(&theme_nats_user);
                fold.fold_nats_user(&user.name, &user.account_name)
            })

            // ================================================================
            // POLICY VISUALIZATION TYPES
            // ================================================================

            // PolicyGroup (SeparationClass) morphism
            .with::<PolicyGroup, _>(move |group| {
                let fold = ThemedVisualizationFold::new(&theme_pol_grp);
                fold.fold_policy(&group.name, &format!("Separation: {:?} - {} roles", group.separation_class, group.role_count))
            })
            // PolicyCategory morphism
            .with::<PolicyCategory, _>(move |cat| {
                let fold = ThemedVisualizationFold::new(&theme_pol_cat);
                fold.fold_policy(&cat.name, &format!("{} claims", cat.claim_count))
            })
            // PolicyRole morphism
            .with::<PolicyRole, _>(move |role| {
                let fold = ThemedVisualizationFold::new(&theme_pol_role);
                fold.fold_role(&role.name, &format!("{} - {} claims", role.purpose, role.claim_count))
            })
            // PolicyClaimView morphism
            .with::<PolicyClaimView, _>(move |claim| {
                let fold = ThemedVisualizationFold::new(&theme_pol_claim);
                fold.fold_policy(&claim.name, &format!("Category: {}", claim.category))
            })

            // ================================================================
            // AGGREGATE VISUALIZATION TYPES
            // ================================================================

            // AggregateOrganization morphism
            .with::<AggregateOrganization, _>(move |agg| {
                let fold = ThemedVisualizationFold::new(&theme_agg_org);
                let desc = format!("v{} | {} people | {} units", agg.version, agg.people_count, agg.units_count);
                fold.fold_organization(&agg.name, "Aggregate", Some(&desc))
            })
            // AggregatePkiChain morphism
            .with::<AggregatePkiChain, _>(move |agg| {
                let fold = ThemedVisualizationFold::new(&theme_agg_pki);
                fold.fold_policy(&agg.name, &format!("v{} | {} certs | {} keys", agg.version, agg.certificates_count, agg.keys_count))
            })
            // AggregateNatsSecurity morphism
            .with::<AggregateNatsSecurity, _>(move |agg| {
                let fold = ThemedVisualizationFold::new(&theme_agg_nats);
                fold.fold_nats_operator(&format!("{} (Aggregate)", agg.name))
            })
            // AggregateYubiKeyProvisioning morphism
            .with::<AggregateYubiKeyProvisioning, _>(move |agg| {
                let fold = ThemedVisualizationFold::new(&theme_agg_yk);
                fold.fold_yubikey(&agg.name, &format!("v{}", agg.version), agg.devices_count)
            });

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
    fn test_visualization_registry_key() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        // Key type is now registered
        assert!(registry.has_visualization(Injection::Key));
    }

    #[test]
    fn test_visualization_registry_certificate() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        // Certificate type is registered (uses LeafCertificate as default injection)
        // Note: The actual visualization for Root/Intermediate certs works via
        // the Certificate.fold_certificate() which checks cert_type internally
        assert!(registry.has_visualization(Injection::LeafCertificate));
    }

    #[test]
    fn test_visualization_registry_yubikey() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        assert!(registry.has_visualization(Injection::YubiKey));
        assert!(registry.has_visualization(Injection::PivSlot));
        assert!(registry.has_visualization(Injection::YubiKeyStatus));
    }

    #[test]
    fn test_visualization_registry_nats() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        assert!(registry.has_visualization(Injection::NatsOperatorSimple));
        assert!(registry.has_visualization(Injection::NatsAccountSimple));
        assert!(registry.has_visualization(Injection::NatsUserSimple));
    }

    #[test]
    fn test_visualization_registry_policy() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        assert!(registry.has_visualization(Injection::Policy));
        assert!(registry.has_visualization(Injection::PolicyGroup));
        assert!(registry.has_visualization(Injection::PolicyCategory));
        assert!(registry.has_visualization(Injection::PolicyRole));
        assert!(registry.has_visualization(Injection::PolicyClaim));
    }

    #[test]
    fn test_visualization_registry_aggregates() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        assert!(registry.has_visualization(Injection::AggregateOrganization));
        assert!(registry.has_visualization(Injection::AggregatePkiChain));
        assert!(registry.has_visualization(Injection::AggregateNatsSecurity));
        assert!(registry.has_visualization(Injection::AggregateYubiKeyProvisioning));
    }

    #[test]
    fn test_visualization_registry_all_types_count() {
        let theme = test_theme();
        let registry = VisualizationRegistry::new(&theme);

        // We registered 22 domain types:
        // Organization BC: Person, Organization, OrganizationUnit, Location, Role, Policy (6)
        // PKI BC: Certificate, CryptographicKey (2)
        // YubiKey BC: YubiKeyDevice, PivSlotView, YubiKeyStatus (3)
        // NATS BC: NatsOperatorSimple, NatsAccountSimple, NatsUserSimple (3)
        // Policy Viz: PolicyGroup, PolicyCategory, PolicyRole, PolicyClaimView (4)
        // Aggregates: AggregateOrganization, AggregatePkiChain, AggregateNatsSecurity, AggregateYubiKeyProvisioning (4)
        // Total: 22 types
        assert_eq!(registry.len(), 22, "Expected 22 domain types registered");
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
