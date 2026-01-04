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
use crate::lifting::{
    DetailPanelData, Injection, LiftedNode,
    // Domain types
    NatsOperatorSimple, NatsAccountSimple, NatsUserSimple,
    AggregateOrganization, AggregatePkiChain, AggregateNatsSecurity, AggregateYubiKeyProvisioning,
};
use crate::domain::{Organization, OrganizationUnit, Person, Location, Role, Policy};
use crate::domain::pki::{Certificate, CryptographicKey};
use crate::domain::yubikey::{YubiKeyDevice, PivSlotView, YubiKeyStatus};
use crate::domain::visualization::{PolicyGroup, PolicyCategory, PolicyRole, PolicyClaimView};

/// Registry of detail panel morphisms for all domain types.
pub struct DetailPanelRegistry {
    inner: MorphismRegistry<DetailPanelData>,
}

impl DetailPanelRegistry {
    /// Create a new detail panel registry with all core domain morphisms registered.
    pub fn new() -> Self {
        let registry = MorphismRegistry::<DetailPanelData>::new()
            // ================================================================
            // ORGANIZATION BOUNDED CONTEXT
            // ================================================================

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
            })
            // Role morphism
            .with::<Role, _>(|role| {
                DetailPanelData::new("Selected Role:")
                    .with_field("Name", &role.name)
                    .with_field("Description", &role.description)
                    .with_field("Active", if role.active { "✓" } else { "✗" })
                    .with_field("Responsibilities", role.responsibilities.len().to_string())
            })
            // Policy morphism
            .with::<Policy, _>(|policy| {
                DetailPanelData::new("Selected Policy:")
                    .with_field("Name", &policy.name)
                    .with_field("Description", &policy.description)
                    .with_field("Priority", policy.priority.to_string())
                    .with_field("Enabled", if policy.enabled { "✓" } else { "✗" })
                    .with_field("Claims", policy.claims.len().to_string())
            })

            // ================================================================
            // PKI BOUNDED CONTEXT
            // ================================================================

            // Certificate morphism
            .with::<Certificate, _>(|cert| {
                DetailPanelData::new("Selected Certificate:")
                    .with_field("Subject", &cert.subject)
                    .with_field("Issuer", &cert.issuer)
                    .with_field("Type", format!("{:?}", cert.cert_type))
                    .with_field("Not Before", cert.not_before.format("%Y-%m-%d").to_string())
                    .with_field("Not After", cert.not_after.format("%Y-%m-%d").to_string())
                    .with_field("Key Usage", cert.key_usage.join(", "))
            })
            // CryptographicKey morphism
            .with::<CryptographicKey, _>(|key| {
                let expires = key.expires_at
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Never".to_string());
                DetailPanelData::new("Selected Key:")
                    .with_field("Purpose", format!("{:?}", key.purpose))
                    .with_field("Algorithm", format!("{:?}", key.algorithm))
                    .with_field("Expires", expires)
            })

            // ================================================================
            // YUBIKEY BOUNDED CONTEXT
            // ================================================================

            // YubiKeyDevice morphism
            .with::<YubiKeyDevice, _>(|yk| {
                let provisioned = yk.provisioned_at
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Not provisioned".to_string());
                DetailPanelData::new("Selected YubiKey:")
                    .with_field("Serial", &yk.serial)
                    .with_field("Version", &yk.version)
                    .with_field("Provisioned", provisioned)
                    .with_field("Slots Used", yk.slots_used.len().to_string())
            })
            // PivSlotView morphism
            .with::<PivSlotView, _>(|slot| {
                let cert_info = slot.certificate_subject.clone()
                    .unwrap_or_else(|| "No certificate".to_string());
                DetailPanelData::new("Selected PIV Slot:")
                    .with_field("Slot", &slot.slot_name)
                    .with_field("YubiKey", &slot.yubikey_serial)
                    .with_field("Has Key", if slot.has_key { "✓" } else { "✗" })
                    .with_field("Certificate", cert_info)
            })
            // YubiKeyStatus morphism
            .with::<YubiKeyStatus, _>(|status| {
                let serial = status.yubikey_serial.clone()
                    .unwrap_or_else(|| "Not assigned".to_string());
                DetailPanelData::new("YubiKey Status:")
                    .with_field("Serial", serial)
                    .with_field("Slots Needed", status.slots_needed.len().to_string())
                    .with_field("Slots Provisioned", status.slots_provisioned.len().to_string())
            })

            // ================================================================
            // NATS BOUNDED CONTEXT
            // ================================================================

            // NatsOperatorSimple morphism
            .with::<NatsOperatorSimple, _>(|op| {
                DetailPanelData::new("NATS Operator:")
                    .with_field("Name", &op.name)
                    .with_field("ID", op.id.to_string())
            })
            // NatsAccountSimple morphism
            .with::<NatsAccountSimple, _>(|acc| {
                DetailPanelData::new("NATS Account:")
                    .with_field("Name", &acc.name)
                    .with_field("ID", acc.id.to_string())
                    .with_field("System", if acc.is_system { "✓" } else { "✗" })
            })
            // NatsUserSimple morphism
            .with::<NatsUserSimple, _>(|user| {
                DetailPanelData::new("NATS User:")
                    .with_field("Name", &user.name)
                    .with_field("Account", &user.account_name)
                    .with_field("ID", user.id.to_string())
            })

            // ================================================================
            // POLICY VISUALIZATION TYPES
            // ================================================================

            // PolicyGroup (SeparationClass) morphism
            .with::<PolicyGroup, _>(|group| {
                DetailPanelData::new("Policy Group:")
                    .with_field("Name", &group.name)
                    .with_field("Separation", format!("{:?}", group.separation_class))
                    .with_field("Roles", group.role_count.to_string())
            })
            // PolicyCategory morphism
            .with::<PolicyCategory, _>(|cat| {
                DetailPanelData::new("Policy Category:")
                    .with_field("Name", &cat.name)
                    .with_field("Claims", cat.claim_count.to_string())
            })
            // PolicyRole morphism
            .with::<PolicyRole, _>(|role| {
                DetailPanelData::new("Policy Role:")
                    .with_field("Name", &role.name)
                    .with_field("Purpose", &role.purpose)
                    .with_field("Level", role.level.to_string())
                    .with_field("Separation", format!("{:?}", role.separation_class))
                    .with_field("Claims", role.claim_count.to_string())
            })
            // PolicyClaimView morphism
            .with::<PolicyClaimView, _>(|claim| {
                DetailPanelData::new("Policy Claim:")
                    .with_field("Name", &claim.name)
                    .with_field("Category", &claim.category)
            })

            // ================================================================
            // AGGREGATE VISUALIZATION TYPES
            // ================================================================

            // AggregateOrganization morphism
            .with::<AggregateOrganization, _>(|agg| {
                DetailPanelData::new("Organization Aggregate:")
                    .with_field("Name", &agg.name)
                    .with_field("Version", agg.version.to_string())
                    .with_field("People", agg.people_count.to_string())
                    .with_field("Units", agg.units_count.to_string())
            })
            // AggregatePkiChain morphism
            .with::<AggregatePkiChain, _>(|agg| {
                DetailPanelData::new("PKI Chain Aggregate:")
                    .with_field("Name", &agg.name)
                    .with_field("Version", agg.version.to_string())
                    .with_field("Certificates", agg.certificates_count.to_string())
                    .with_field("Keys", agg.keys_count.to_string())
            })
            // AggregateNatsSecurity morphism
            .with::<AggregateNatsSecurity, _>(|agg| {
                DetailPanelData::new("NATS Security Aggregate:")
                    .with_field("Name", &agg.name)
                    .with_field("Version", agg.version.to_string())
                    .with_field("Operators", agg.operators_count.to_string())
                    .with_field("Accounts", agg.accounts_count.to_string())
                    .with_field("Users", agg.users_count.to_string())
            })
            // AggregateYubiKeyProvisioning morphism
            .with::<AggregateYubiKeyProvisioning, _>(|agg| {
                DetailPanelData::new("YubiKey Provisioning Aggregate:")
                    .with_field("Name", &agg.name)
                    .with_field("Version", agg.version.to_string())
                    .with_field("Devices", agg.devices_count.to_string())
                    .with_field("Slots", agg.slots_provisioned.to_string())
            });

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
    use crate::lifting::LiftableDomain;

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
    fn test_detail_panel_registry_key() {
        let registry = DetailPanelRegistry::new();

        // Key is now registered
        assert!(registry.has_morphism(Injection::Key));
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

    #[test]
    fn test_detail_panel_registry_yubikey() {
        let registry = DetailPanelRegistry::new();

        assert!(registry.has_morphism(Injection::YubiKey));
        assert!(registry.has_morphism(Injection::PivSlot));
        assert!(registry.has_morphism(Injection::YubiKeyStatus));
    }

    #[test]
    fn test_detail_panel_registry_nats() {
        let registry = DetailPanelRegistry::new();

        assert!(registry.has_morphism(Injection::NatsOperatorSimple));
        assert!(registry.has_morphism(Injection::NatsAccountSimple));
        assert!(registry.has_morphism(Injection::NatsUserSimple));
    }

    #[test]
    fn test_detail_panel_registry_policy() {
        let registry = DetailPanelRegistry::new();

        assert!(registry.has_morphism(Injection::Policy));
        assert!(registry.has_morphism(Injection::PolicyGroup));
        assert!(registry.has_morphism(Injection::PolicyCategory));
        assert!(registry.has_morphism(Injection::PolicyRole));
        assert!(registry.has_morphism(Injection::PolicyClaim));
    }

    #[test]
    fn test_detail_panel_registry_aggregates() {
        let registry = DetailPanelRegistry::new();

        assert!(registry.has_morphism(Injection::AggregateOrganization));
        assert!(registry.has_morphism(Injection::AggregatePkiChain));
        assert!(registry.has_morphism(Injection::AggregateNatsSecurity));
        assert!(registry.has_morphism(Injection::AggregateYubiKeyProvisioning));
    }

    #[test]
    fn test_detail_panel_registry_all_types_count() {
        let registry = DetailPanelRegistry::new();

        // We registered 22 domain types (same as VisualizationRegistry)
        assert_eq!(registry.len(), 22, "Expected 22 domain types registered");
    }
}
