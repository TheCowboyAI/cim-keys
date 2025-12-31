// Copyright (c) 2025 - Cowboy AI, LLC.

//! Aggregate State Coproduct
//!
//! This module defines the coproduct for aggregate state visualization.
//! Aggregates are DDD consistency boundaries - state machines that manage
//! entity collections. This coproduct captures their summary state for display.
//!
//! ## Why Separate from Entities?
//!
//! 1. **Different Abstraction Level**: Entities are domain objects, aggregates
//!    are state machines that manage collections of entities
//! 2. **Different Lifecycle**: Entities change via events, aggregates track
//!    version and consistency state
//! 3. **DDD Principle**: Aggregates define boundaries, entities live within them
//!
//! ## Aggregate Types
//!
//! - OrganizationAggregate: Manages people, units, locations
//! - PkiChainAggregate: Manages certificate chain and keys
//! - NatsSecurityAggregate: Manages operators, accounts, users
//! - YubiKeyProvisioningAggregate: Manages devices and slot provisioning

use std::fmt;
use uuid::Uuid;

/// Injection tag for aggregate state coproduct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregateInjection {
    Organization,
    PkiChain,
    NatsSecurity,
    YubiKeyProvisioning,
}

impl AggregateInjection {
    /// Display name for this aggregate type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Organization => "Organization Aggregate",
            Self::PkiChain => "PKI Chain Aggregate",
            Self::NatsSecurity => "NATS Security Aggregate",
            Self::YubiKeyProvisioning => "YubiKey Provisioning Aggregate",
        }
    }

    /// Get the bounded context this aggregate belongs to
    pub fn context(&self) -> &'static str {
        match self {
            Self::Organization => "organization",
            Self::PkiChain => "pki",
            Self::NatsSecurity => "nats",
            Self::YubiKeyProvisioning => "yubikey",
        }
    }

    /// All aggregate types
    pub fn all() -> Vec<Self> {
        vec![
            Self::Organization,
            Self::PkiChain,
            Self::NatsSecurity,
            Self::YubiKeyProvisioning,
        ]
    }
}

impl fmt::Display for AggregateInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Organization aggregate state summary
#[derive(Debug, Clone)]
pub struct OrganizationAggregateState {
    /// Aggregate identifier
    pub id: Uuid,
    /// Display name
    pub name: String,
    /// Current version (event sequence number)
    pub version: u64,
    /// Number of people in the organization
    pub people_count: usize,
    /// Number of organizational units
    pub units_count: usize,
    /// Number of locations
    pub locations_count: usize,
    /// Number of roles defined
    pub roles_count: usize,
    /// Number of policies defined
    pub policies_count: usize,
}

/// PKI chain aggregate state summary
#[derive(Debug, Clone)]
pub struct PkiChainAggregateState {
    /// Aggregate identifier
    pub id: Uuid,
    /// Display name
    pub name: String,
    /// Current version
    pub version: u64,
    /// Total certificates in chain
    pub certificates_count: usize,
    /// Root CA count (usually 1)
    pub root_cas: usize,
    /// Intermediate CA count
    pub intermediate_cas: usize,
    /// Leaf certificate count
    pub leaf_certs: usize,
    /// Keys managed
    pub keys_count: usize,
}

/// NATS security aggregate state summary
#[derive(Debug, Clone)]
pub struct NatsSecurityAggregateState {
    /// Aggregate identifier
    pub id: Uuid,
    /// Display name
    pub name: String,
    /// Current version
    pub version: u64,
    /// Number of operators
    pub operators_count: usize,
    /// Number of accounts
    pub accounts_count: usize,
    /// Number of user identities
    pub users_count: usize,
    /// Number of service accounts
    pub service_accounts_count: usize,
}

/// YubiKey provisioning aggregate state summary
#[derive(Debug, Clone)]
pub struct YubiKeyProvisioningAggregateState {
    /// Aggregate identifier
    pub id: Uuid,
    /// Display name
    pub name: String,
    /// Current version
    pub version: u64,
    /// Number of YubiKey devices
    pub devices_count: usize,
    /// Total slots provisioned
    pub slots_provisioned: usize,
    /// Total slots available
    pub total_slots: usize,
}

/// Inner data for aggregate state coproduct
#[derive(Debug, Clone)]
pub enum AggregateStateData {
    Organization(OrganizationAggregateState),
    PkiChain(PkiChainAggregateState),
    NatsSecurity(NatsSecurityAggregateState),
    YubiKeyProvisioning(YubiKeyProvisioningAggregateState),
}

/// Aggregate State - Coproduct of aggregate summary states
///
/// This is a higher-level coproduct that represents aggregate root state,
/// separate from the entity coproducts that represent individual domain objects.
#[derive(Debug, Clone)]
pub struct AggregateState {
    injection: AggregateInjection,
    data: AggregateStateData,
}

impl AggregateState {
    // ========================================================================
    // Injection Functions
    // ========================================================================

    /// Inject Organization aggregate state
    pub fn inject_organization(state: OrganizationAggregateState) -> Self {
        Self {
            injection: AggregateInjection::Organization,
            data: AggregateStateData::Organization(state),
        }
    }

    /// Inject PKI Chain aggregate state
    pub fn inject_pki_chain(state: PkiChainAggregateState) -> Self {
        Self {
            injection: AggregateInjection::PkiChain,
            data: AggregateStateData::PkiChain(state),
        }
    }

    /// Inject NATS Security aggregate state
    pub fn inject_nats_security(state: NatsSecurityAggregateState) -> Self {
        Self {
            injection: AggregateInjection::NatsSecurity,
            data: AggregateStateData::NatsSecurity(state),
        }
    }

    /// Inject YubiKey Provisioning aggregate state
    pub fn inject_yubikey_provisioning(state: YubiKeyProvisioningAggregateState) -> Self {
        Self {
            injection: AggregateInjection::YubiKeyProvisioning,
            data: AggregateStateData::YubiKeyProvisioning(state),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the injection tag
    pub fn injection(&self) -> AggregateInjection {
        self.injection
    }

    /// Get reference to inner data
    pub fn data(&self) -> &AggregateStateData {
        &self.data
    }

    /// Get aggregate ID
    pub fn id(&self) -> Uuid {
        match &self.data {
            AggregateStateData::Organization(s) => s.id,
            AggregateStateData::PkiChain(s) => s.id,
            AggregateStateData::NatsSecurity(s) => s.id,
            AggregateStateData::YubiKeyProvisioning(s) => s.id,
        }
    }

    /// Get aggregate name
    pub fn name(&self) -> &str {
        match &self.data {
            AggregateStateData::Organization(s) => &s.name,
            AggregateStateData::PkiChain(s) => &s.name,
            AggregateStateData::NatsSecurity(s) => &s.name,
            AggregateStateData::YubiKeyProvisioning(s) => &s.name,
        }
    }

    /// Get aggregate version
    pub fn version(&self) -> u64 {
        match &self.data {
            AggregateStateData::Organization(s) => s.version,
            AggregateStateData::PkiChain(s) => s.version,
            AggregateStateData::NatsSecurity(s) => s.version,
            AggregateStateData::YubiKeyProvisioning(s) => s.version,
        }
    }

    // ========================================================================
    // Universal Property (Fold)
    // ========================================================================

    /// Apply a fold to this aggregate state (universal property of coproduct)
    pub fn fold<F: FoldAggregateState>(&self, folder: &F) -> F::Output {
        match &self.data {
            AggregateStateData::Organization(s) => folder.fold_organization(s),
            AggregateStateData::PkiChain(s) => folder.fold_pki_chain(s),
            AggregateStateData::NatsSecurity(s) => folder.fold_nats_security(s),
            AggregateStateData::YubiKeyProvisioning(s) => folder.fold_yubikey_provisioning(s),
        }
    }
}

/// Universal property trait for AggregateState coproduct
///
/// For any type X with morphisms from each aggregate state type,
/// this trait captures the unique morphism AggregateState → X.
pub trait FoldAggregateState {
    type Output;

    fn fold_organization(&self, state: &OrganizationAggregateState) -> Self::Output;
    fn fold_pki_chain(&self, state: &PkiChainAggregateState) -> Self::Output;
    fn fold_nats_security(&self, state: &NatsSecurityAggregateState) -> Self::Output;
    fn fold_yubikey_provisioning(&self, state: &YubiKeyProvisioningAggregateState) -> Self::Output;
}

// ============================================================================
// Convenience Builders
// ============================================================================

impl OrganizationAggregateState {
    /// Create new organization aggregate state
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            version: 0,
            people_count: 0,
            units_count: 0,
            locations_count: 0,
            roles_count: 0,
            policies_count: 0,
        }
    }

    /// Builder: set counts
    pub fn with_counts(
        mut self,
        people: usize,
        units: usize,
        locations: usize,
        roles: usize,
        policies: usize,
    ) -> Self {
        self.people_count = people;
        self.units_count = units;
        self.locations_count = locations;
        self.roles_count = roles;
        self.policies_count = policies;
        self
    }

    /// Builder: set version
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }
}

impl PkiChainAggregateState {
    /// Create new PKI chain aggregate state
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            version: 0,
            certificates_count: 0,
            root_cas: 0,
            intermediate_cas: 0,
            leaf_certs: 0,
            keys_count: 0,
        }
    }

    /// Builder: set certificate counts
    pub fn with_certificates(
        mut self,
        roots: usize,
        intermediates: usize,
        leaves: usize,
    ) -> Self {
        self.root_cas = roots;
        self.intermediate_cas = intermediates;
        self.leaf_certs = leaves;
        self.certificates_count = roots + intermediates + leaves;
        self
    }

    /// Builder: set key count
    pub fn with_keys(mut self, keys: usize) -> Self {
        self.keys_count = keys;
        self
    }

    /// Builder: set version
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }
}

impl NatsSecurityAggregateState {
    /// Create new NATS security aggregate state
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            version: 0,
            operators_count: 0,
            accounts_count: 0,
            users_count: 0,
            service_accounts_count: 0,
        }
    }

    /// Builder: set identity counts
    pub fn with_identities(
        mut self,
        operators: usize,
        accounts: usize,
        users: usize,
        service_accounts: usize,
    ) -> Self {
        self.operators_count = operators;
        self.accounts_count = accounts;
        self.users_count = users;
        self.service_accounts_count = service_accounts;
        self
    }

    /// Builder: set version
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }
}

impl YubiKeyProvisioningAggregateState {
    /// Create new YubiKey provisioning aggregate state
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            version: 0,
            devices_count: 0,
            slots_provisioned: 0,
            total_slots: 0,
        }
    }

    /// Builder: set device/slot counts
    pub fn with_devices(
        mut self,
        devices: usize,
        provisioned: usize,
        total: usize,
    ) -> Self {
        self.devices_count = devices;
        self.slots_provisioned = provisioned;
        self.total_slots = total;
        self
    }

    /// Builder: set version
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test fold that returns injection type
    struct InjectionFolder;

    impl FoldAggregateState for InjectionFolder {
        type Output = AggregateInjection;

        fn fold_organization(&self, _: &OrganizationAggregateState) -> Self::Output {
            AggregateInjection::Organization
        }
        fn fold_pki_chain(&self, _: &PkiChainAggregateState) -> Self::Output {
            AggregateInjection::PkiChain
        }
        fn fold_nats_security(&self, _: &NatsSecurityAggregateState) -> Self::Output {
            AggregateInjection::NatsSecurity
        }
        fn fold_yubikey_provisioning(&self, _: &YubiKeyProvisioningAggregateState) -> Self::Output {
            AggregateInjection::YubiKeyProvisioning
        }
    }

    #[test]
    fn test_organization_aggregate_builder() {
        let state = OrganizationAggregateState::new("Test Org")
            .with_counts(10, 3, 2, 5, 8)
            .with_version(42);

        assert_eq!(state.name, "Test Org");
        assert_eq!(state.people_count, 10);
        assert_eq!(state.units_count, 3);
        assert_eq!(state.version, 42);
    }

    #[test]
    fn test_injection_fold() {
        let state = OrganizationAggregateState::new("Test");
        let aggregate = AggregateState::inject_organization(state);
        let injection = aggregate.fold(&InjectionFolder);

        assert_eq!(injection, AggregateInjection::Organization);
    }

    #[test]
    fn test_pki_certificate_counts() {
        let state = PkiChainAggregateState::new("Root CA Chain")
            .with_certificates(1, 2, 10);

        assert_eq!(state.certificates_count, 13);
        assert_eq!(state.root_cas, 1);
        assert_eq!(state.intermediate_cas, 2);
        assert_eq!(state.leaf_certs, 10);
    }
}
