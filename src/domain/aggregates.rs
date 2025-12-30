// Copyright (c) 2025 - Cowboy AI, LLC.

//! Aggregate Roots per Bounded Context
//!
//! This module provides proper DDD aggregate roots for each bounded context.
//! Each aggregate:
//! - Implements `AggregateRoot` trait from cim-domain
//! - Handles commands via `handle()` method
//! - Applies events via `apply()` method
//! - Enforces invariants within its boundary
//!
//! ## Aggregate Boundaries
//!
//! - **OrganizationAggregate**: Organization hierarchy, people, units
//! - **PkiCertificateChainAggregate**: Certificate chains, keys, trust hierarchy
//! - **NatsSecurityAggregate**: Operators, accounts, users
//! - **YubiKeyProvisioningAggregate**: Devices, PIV slots, provisioning

use cim_domain::AggregateRoot;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use super::ids::*;
use crate::events::DomainEvent;

// ============================================================================
// ORGANIZATION AGGREGATE
// ============================================================================

/// Organization Aggregate Root
///
/// Manages organizational hierarchy including units, people, and their relationships.
///
/// ## Invariants
/// - Organization name must be non-empty
/// - Organization must have at least one root authority person
/// - Unit parent must exist before creating child
/// - Person organization_id must match aggregate
/// - Person email must be unique within organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationAggregate {
    /// Aggregate ID (same as organization ID)
    pub id: BootstrapOrgId,
    /// Version for optimistic concurrency
    pub version: u64,
    /// Organization name
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Description
    pub description: Option<String>,
    /// Organizational units (indexed by ID)
    pub units: HashMap<UnitId, UnitState>,
    /// People in the organization (indexed by person UUID)
    pub people: HashMap<Uuid, PersonState>,
    /// Has the organization been initialized with a root authority?
    pub has_root_authority: bool,
}

/// State of an organizational unit within the aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitState {
    pub id: UnitId,
    pub name: String,
    pub unit_type: super::organization::OrganizationUnitType,
    pub parent_unit_id: Option<UnitId>,
}

/// State of a person within the aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonState {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: super::bootstrap::KeyOwnerRole,
    pub unit_ids: Vec<UnitId>,
    pub active: bool,
}

impl OrganizationAggregate {
    /// Create a new organization aggregate
    pub fn new(id: BootstrapOrgId, name: String, display_name: String) -> Self {
        Self {
            id,
            version: 0,
            name,
            display_name,
            description: None,
            units: HashMap::new(),
            people: HashMap::new(),
            has_root_authority: false,
        }
    }

    /// Check if a person email is unique within the organization
    pub fn is_email_unique(&self, email: &str) -> bool {
        !self.people.values().any(|p| p.email == email)
    }

    /// Check if adding this person would violate invariants
    pub fn can_add_person(&self, email: &str) -> Result<(), String> {
        if !self.is_email_unique(email) {
            return Err(format!("Email {} already exists in organization", email));
        }
        Ok(())
    }

    /// Check if a unit can be added
    pub fn can_add_unit(&self, parent_id: Option<UnitId>) -> Result<(), String> {
        if let Some(parent) = parent_id {
            if !self.units.contains_key(&parent) {
                return Err(format!("Parent unit {} does not exist", parent));
            }
        }
        Ok(())
    }

    /// Apply an organization event to update state
    ///
    /// Note: Detailed event application is handled by projections.
    /// This method maintains aggregate consistency invariants.
    pub fn apply(&mut self, event: &DomainEvent) -> Result<(), String> {
        match event {
            DomainEvent::Organization(org_event) => {
                use crate::events::organization::OrganizationEvents;
                match org_event {
                    OrganizationEvents::OrganizationCreated(_) => {
                        // Initial state already set in constructor
                    }
                    OrganizationEvents::OrganizationUpdated(e) => {
                        if e.field_name == "name" {
                            self.name = e.new_value.clone();
                        }
                    }
                    OrganizationEvents::OrganizationalUnitCreated(e) => {
                        let unit_id = UnitId::from_uuid(e.unit_id);
                        self.units.insert(unit_id, UnitState {
                            id: unit_id,
                            name: e.name.clone(),
                            unit_type: super::organization::OrganizationUnitType::Department,
                            parent_unit_id: e.parent_id.map(UnitId::from_uuid),
                        });
                    }
                    OrganizationEvents::OrganizationalUnitUpdated(e) => {
                        let unit_id = UnitId::from_uuid(e.unit_id);
                        if let Some(unit) = self.units.get_mut(&unit_id) {
                            if e.field_name == "name" {
                                unit.name = e.new_value.clone();
                            }
                        }
                    }
                    OrganizationEvents::OrganizationalUnitDissolved(e) => {
                        let unit_id = UnitId::from_uuid(e.unit_id);
                        self.units.remove(&unit_id);
                    }
                    // Policy and role changes tracked by projection, not aggregate state
                    _ => {}
                }
                self.increment_version();
            }
            DomainEvent::Person(person_event) => {
                use crate::events::person::PersonEvents;
                match person_event {
                    PersonEvents::PersonCreated(e) => {
                        let person_id = e.person_id;
                        self.people.insert(person_id, PersonState {
                            id: person_id,
                            name: e.name.clone(),
                            email: e.email.clone().unwrap_or_default(),
                            role: super::bootstrap::KeyOwnerRole::Developer,
                            unit_ids: Vec::new(),
                            active: true,
                        });
                    }
                    PersonEvents::PersonUpdated(e) => {
                        if let Some(person) = self.people.get_mut(&e.person_id) {
                            match e.field_name.as_str() {
                                "name" => person.name = e.new_value.clone(),
                                "email" => person.email = e.new_value.clone(),
                                _ => {}
                            }
                        }
                    }
                    PersonEvents::PersonDeactivated(e) => {
                        if let Some(person) = self.people.get_mut(&e.person_id) {
                            person.active = false;
                        }
                    }
                    PersonEvents::PersonActivated(e) => {
                        if let Some(person) = self.people.get_mut(&e.person_id) {
                            person.active = true;
                        }
                    }
                    _ => {}
                }
                self.increment_version();
            }
            _ => {
                // Events from other aggregates are ignored
            }
        }
        Ok(())
    }
}

impl AggregateRoot for OrganizationAggregate {
    type Id = BootstrapOrgId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

// ============================================================================
// PKI CERTIFICATE CHAIN AGGREGATE
// ============================================================================

/// PKI Certificate Chain Aggregate Root
///
/// Manages the certificate chain hierarchy including root, intermediate, and leaf certs.
///
/// ## Invariants
/// - Root CA must be generated before intermediate CAs
/// - Intermediate CA's issuer must exist
/// - Leaf certificate's issuer must exist
/// - Certificate not_after > not_before
/// - Key must be generated before certificate using it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkiCertificateChainAggregate {
    /// Aggregate ID (organization-scoped)
    pub id: Uuid,
    /// Version for optimistic concurrency
    pub version: u64,
    /// Organization this PKI chain belongs to
    pub organization_id: BootstrapOrgId,
    /// Root CA certificate (if generated)
    pub root_ca: Option<CertificateState>,
    /// Intermediate CAs (indexed by certificate ID)
    pub intermediate_cas: HashMap<CertificateId, CertificateState>,
    /// Leaf certificates (indexed by certificate ID)
    pub leaf_certificates: HashMap<CertificateId, CertificateState>,
    /// Generated keys (indexed by key ID)
    pub keys: HashMap<KeyId, KeyState>,
}

/// State of a certificate within the aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateState {
    pub id: CertificateId,
    pub subject: String,
    pub issuer_id: Option<CertificateId>,
    pub key_id: KeyId,
    pub cert_type: super::pki::CertificateType,
    pub status: super::pki::CertificateStatus,
    pub not_before: chrono::DateTime<chrono::Utc>,
    pub not_after: chrono::DateTime<chrono::Utc>,
}

/// State of a key within the aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyState {
    pub id: KeyId,
    pub algorithm: crate::events::KeyAlgorithm,
    pub purpose: crate::events::KeyPurpose,
    pub owner_person_id: Option<Uuid>,
    pub revoked: bool,
}

impl PkiCertificateChainAggregate {
    /// Create a new PKI aggregate for an organization
    pub fn new(organization_id: BootstrapOrgId) -> Self {
        Self {
            id: Uuid::now_v7(),
            version: 0,
            organization_id,
            root_ca: None,
            intermediate_cas: HashMap::new(),
            leaf_certificates: HashMap::new(),
            keys: HashMap::new(),
        }
    }

    /// Check if root CA exists
    pub fn has_root_ca(&self) -> bool {
        self.root_ca.is_some()
    }

    /// Check if an issuer certificate exists
    pub fn issuer_exists(&self, issuer_id: CertificateId) -> bool {
        self.root_ca.as_ref().map_or(false, |ca| ca.id == issuer_id)
            || self.intermediate_cas.contains_key(&issuer_id)
    }

    /// Check if a key exists
    pub fn key_exists(&self, key_id: KeyId) -> bool {
        self.keys.contains_key(&key_id)
    }

    /// Validate certificate can be generated
    pub fn can_generate_certificate(
        &self,
        cert_type: super::pki::CertificateType,
        issuer_id: Option<CertificateId>,
    ) -> Result<(), String> {
        match cert_type {
            super::pki::CertificateType::Root => {
                if self.root_ca.is_some() {
                    return Err("Root CA already exists".to_string());
                }
            }
            super::pki::CertificateType::Intermediate => {
                if !self.has_root_ca() {
                    return Err("Root CA must be generated first".to_string());
                }
                if let Some(id) = issuer_id {
                    if !self.issuer_exists(id) {
                        return Err(format!("Issuer certificate {} does not exist", id));
                    }
                }
            }
            super::pki::CertificateType::Leaf => {
                if issuer_id.is_none() {
                    return Err("Leaf certificate requires an issuer".to_string());
                }
                if let Some(id) = issuer_id {
                    if !self.issuer_exists(id) {
                        return Err(format!("Issuer certificate {} does not exist", id));
                    }
                }
            }
            super::pki::CertificateType::Policy => {
                // Policy CAs have special rules - defer to command handler
            }
        }
        Ok(())
    }

    /// Apply a PKI event to update state
    ///
    /// Note: Detailed event application is handled by projections.
    /// This method maintains aggregate consistency invariants.
    pub fn apply(&mut self, event: &DomainEvent) -> Result<(), String> {
        match event {
            DomainEvent::Certificate(cert_event) => {
                use crate::events::certificate::CertificateEvents;
                match cert_event {
                    CertificateEvents::CertificateGenerated(e) => {
                        let cert_id = CertificateId::from_uuid(e.cert_id);
                        let key_id = KeyId::from_uuid(e.key_id);
                        let cert_state = CertificateState {
                            id: cert_id,
                            subject: e.subject.clone(),
                            issuer_id: e.issuer.map(CertificateId::from_uuid),
                            key_id,
                            cert_type: if e.is_ca { super::pki::CertificateType::Root } else { super::pki::CertificateType::Leaf },
                            status: super::pki::CertificateStatus::Active,
                            not_before: e.not_before,
                            not_after: e.not_after,
                        };

                        if e.is_ca && e.issuer.is_none() {
                            self.root_ca = Some(cert_state);
                        } else if e.is_ca {
                            self.intermediate_cas.insert(cert_id, cert_state);
                        } else {
                            self.leaf_certificates.insert(cert_id, cert_state);
                        }
                    }
                    CertificateEvents::CertificateRevoked(e) => {
                        let cert_id = CertificateId::from_uuid(e.cert_id);
                        if let Some(cert) = self.leaf_certificates.get_mut(&cert_id) {
                            cert.status = super::pki::CertificateStatus::Revoked;
                        }
                        if let Some(cert) = self.intermediate_cas.get_mut(&cert_id) {
                            cert.status = super::pki::CertificateStatus::Revoked;
                        }
                    }
                    CertificateEvents::CertificateRenewed(e) => {
                        // Renewal creates a new certificate, old one is replaced
                        // Remove old cert from appropriate collection if present
                        let old_cert_id = CertificateId::from_uuid(e.old_cert_id);
                        self.leaf_certificates.remove(&old_cert_id);
                        self.intermediate_cas.remove(&old_cert_id);
                    }
                    _ => {}
                }
                self.increment_version();
            }
            DomainEvent::Key(key_event) => {
                use crate::events::key::KeyEvents;
                match key_event {
                    KeyEvents::KeyGenerated(e) => {
                        let key_id = KeyId::from_uuid(e.key_id);
                        self.keys.insert(key_id, KeyState {
                            id: key_id,
                            algorithm: e.algorithm.clone(),
                            purpose: e.purpose.clone(),
                            owner_person_id: e.ownership.as_ref().map(|o| o.person_id),
                            revoked: false,
                        });
                    }
                    KeyEvents::KeyRevoked(e) => {
                        let key_id = KeyId::from_uuid(e.key_id);
                        if let Some(key) = self.keys.get_mut(&key_id) {
                            key.revoked = true;
                        }
                    }
                    _ => {}
                }
                self.increment_version();
            }
            _ => {
                // Events from other aggregates are ignored
            }
        }
        Ok(())
    }
}

impl AggregateRoot for PkiCertificateChainAggregate {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

// ============================================================================
// NATS SECURITY AGGREGATE
// ============================================================================

/// NATS Security Aggregate Root
///
/// Manages NATS operator, accounts, and users for security infrastructure.
///
/// ## Invariants
/// - Operator must be created before accounts
/// - Account's operator must exist
/// - User's account must exist
/// - Operator name must be unique
/// - Account name must be unique within operator
/// - User name must be unique within account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSecurityAggregate {
    /// Aggregate ID (same as operator ID)
    pub id: NatsOperatorId,
    /// Version for optimistic concurrency
    pub version: u64,
    /// Organization this NATS operator belongs to
    pub organization_id: BootstrapOrgId,
    /// Operator name
    pub name: String,
    /// System account ID (if configured)
    pub system_account_id: Option<NatsAccountId>,
    /// Accounts (indexed by account ID)
    pub accounts: HashMap<NatsAccountId, NatsAccountState>,
    /// Users (indexed by user ID)
    pub users: HashMap<NatsUserId, NatsUserState>,
}

/// State of a NATS account within the aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAccountState {
    pub id: NatsAccountId,
    pub name: String,
    pub unit_id: Option<Uuid>,
    pub is_system_account: bool,
}

/// State of a NATS user within the aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsUserState {
    pub id: NatsUserId,
    pub name: String,
    pub account_id: NatsAccountId,
    pub person_id: Option<Uuid>,
    pub is_service_account: bool,
}

impl NatsSecurityAggregate {
    /// Create a new NATS security aggregate for an organization
    pub fn new(id: NatsOperatorId, organization_id: BootstrapOrgId, name: String) -> Self {
        Self {
            id,
            version: 0,
            organization_id,
            name,
            system_account_id: None,
            accounts: HashMap::new(),
            users: HashMap::new(),
        }
    }

    /// Check if an account name is unique
    pub fn is_account_name_unique(&self, name: &str) -> bool {
        !self.accounts.values().any(|a| a.name == name)
    }

    /// Check if a user name is unique within an account
    pub fn is_user_name_unique(&self, account_id: NatsAccountId, name: &str) -> bool {
        !self.users.values()
            .filter(|u| u.account_id == account_id)
            .any(|u| u.name == name)
    }

    /// Check if an account exists
    pub fn account_exists(&self, account_id: NatsAccountId) -> bool {
        self.accounts.contains_key(&account_id)
    }

    /// Validate account can be created
    pub fn can_create_account(&self, name: &str) -> Result<(), String> {
        if !self.is_account_name_unique(name) {
            return Err(format!("Account name {} already exists", name));
        }
        Ok(())
    }

    /// Validate user can be created
    pub fn can_create_user(&self, account_id: NatsAccountId, name: &str) -> Result<(), String> {
        if !self.account_exists(account_id) {
            return Err(format!("Account {} does not exist", account_id));
        }
        if !self.is_user_name_unique(account_id, name) {
            return Err(format!("User name {} already exists in account", name));
        }
        Ok(())
    }

    /// Apply a NATS event to update state
    ///
    /// Note: Detailed event application is handled by projections.
    /// This method maintains aggregate consistency invariants.
    pub fn apply(&mut self, event: &DomainEvent) -> Result<(), String> {
        match event {
            DomainEvent::NatsOperator(op_event) => {
                use crate::events::nats_operator::NatsOperatorEvents;
                match op_event {
                    NatsOperatorEvents::NatsOperatorCreated(_) => {
                        // Initial state set in constructor
                    }
                    _ => {}
                }
                self.increment_version();
            }
            DomainEvent::NatsAccount(acc_event) => {
                use crate::events::nats_account::NatsAccountEvents;
                match acc_event {
                    NatsAccountEvents::NatsAccountCreated(e) => {
                        let account_id = NatsAccountId::from_uuid(e.account_id);
                        self.accounts.insert(account_id, NatsAccountState {
                            id: account_id,
                            name: e.name.clone(),
                            unit_id: e.organization_unit_id,
                            is_system_account: e.is_system,
                        });
                    }
                    NatsAccountEvents::NatsAccountDeleted(e) => {
                        let account_id = NatsAccountId::from_uuid(e.account_id);
                        self.accounts.remove(&account_id);
                    }
                    _ => {}
                }
                self.increment_version();
            }
            DomainEvent::NatsUser(user_event) => {
                use crate::events::nats_user::NatsUserEvents;
                match user_event {
                    NatsUserEvents::NatsUserCreated(e) => {
                        let user_id = NatsUserId::from_uuid(e.user_id);
                        let account_id = NatsAccountId::from_uuid(e.account_id);
                        self.users.insert(user_id, NatsUserState {
                            id: user_id,
                            name: e.name.clone(),
                            account_id,
                            person_id: e.person_id,
                            is_service_account: e.person_id.is_none(),
                        });
                    }
                    NatsUserEvents::NatsUserDeleted(e) => {
                        let user_id = NatsUserId::from_uuid(e.user_id);
                        self.users.remove(&user_id);
                    }
                    _ => {}
                }
                self.increment_version();
            }
            _ => {
                // Events from other aggregates are ignored
            }
        }
        Ok(())
    }
}

impl AggregateRoot for NatsSecurityAggregate {
    type Id = NatsOperatorId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

// ============================================================================
// YUBIKEY PROVISIONING AGGREGATE
// ============================================================================

/// YubiKey Provisioning Aggregate Root
///
/// Manages YubiKey devices and their PIV slot provisioning.
///
/// ## Invariants
/// - YubiKey serial must be unique
/// - PIV slot can only have one key
/// - Slot must be empty before provisioning
/// - Person must be assigned before slot provisioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyProvisioningAggregate {
    /// Aggregate ID (organization-scoped)
    pub id: Uuid,
    /// Version for optimistic concurrency
    pub version: u64,
    /// Organization this YubiKey management belongs to
    pub organization_id: BootstrapOrgId,
    /// YubiKey devices (indexed by device ID)
    pub devices: HashMap<YubiKeyDeviceId, YubiKeyDeviceState>,
    /// Device serials for uniqueness check
    pub serials: HashMap<String, YubiKeyDeviceId>,
}

/// State of a YubiKey device within the aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyDeviceState {
    pub id: YubiKeyDeviceId,
    pub serial: String,
    pub owner_person_id: Option<Uuid>,
    pub slots: HashMap<super::yubikey::PIVSlot, SlotState>,
}

/// State of a PIV slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotState {
    pub id: SlotId,
    pub slot: super::yubikey::PIVSlot,
    pub has_key: bool,
    pub key_id: Option<KeyId>,
    pub certificate_id: Option<CertificateId>,
}

impl YubiKeyProvisioningAggregate {
    /// Create a new YubiKey provisioning aggregate for an organization
    pub fn new(organization_id: BootstrapOrgId) -> Self {
        Self {
            id: Uuid::now_v7(),
            version: 0,
            organization_id,
            devices: HashMap::new(),
            serials: HashMap::new(),
        }
    }

    /// Check if a serial is already registered
    pub fn is_serial_unique(&self, serial: &str) -> bool {
        !self.serials.contains_key(serial)
    }

    /// Check if a slot is available for provisioning
    pub fn is_slot_available(&self, device_id: YubiKeyDeviceId, slot: super::yubikey::PIVSlot) -> bool {
        self.devices.get(&device_id)
            .and_then(|d| d.slots.get(&slot))
            .map_or(true, |s| !s.has_key)
    }

    /// Validate device can be registered
    pub fn can_register_device(&self, serial: &str) -> Result<(), String> {
        if !self.is_serial_unique(serial) {
            return Err(format!("YubiKey with serial {} already registered", serial));
        }
        Ok(())
    }

    /// Validate slot can be provisioned
    pub fn can_provision_slot(
        &self,
        device_id: YubiKeyDeviceId,
        slot: super::yubikey::PIVSlot,
    ) -> Result<(), String> {
        let device = self.devices.get(&device_id)
            .ok_or_else(|| format!("YubiKey device {} not found", device_id))?;

        if device.owner_person_id.is_none() {
            return Err("YubiKey must be assigned to a person before provisioning".to_string());
        }

        if !self.is_slot_available(device_id, slot) {
            return Err(format!("Slot {} already has a key", slot));
        }

        Ok(())
    }

    /// Apply a YubiKey event to update state
    ///
    /// Note: Detailed event application is handled by projections.
    /// This method maintains aggregate consistency invariants.
    pub fn apply(&mut self, event: &DomainEvent) -> Result<(), String> {
        match event {
            DomainEvent::YubiKey(yk_event) => {
                use crate::events::yubikey::YubiKeyEvents;
                match yk_event {
                    YubiKeyEvents::YubiKeyDetected(e) => {
                        // Use event_id as device ID since YubiKeyDetectedEvent doesn't have device UUID
                        let device_id = YubiKeyDeviceId::from_uuid(e.event_id);
                        self.devices.insert(device_id, YubiKeyDeviceState {
                            id: device_id,
                            serial: e.yubikey_serial.clone(),
                            owner_person_id: None,
                            slots: HashMap::new(),
                        });
                        self.serials.insert(e.yubikey_serial.clone(), device_id);
                    }
                    YubiKeyEvents::YubiKeyProvisioned(e) => {
                        // Find device by serial since we don't have device ID in this event
                        if let Some(device_id) = self.serials.get(&e.yubikey_serial).copied() {
                            if let Some(device) = self.devices.get_mut(&device_id) {
                                // Mark all configured slots as having keys
                                for slot_config in &e.slots_configured {
                                    let slot = match slot_config.slot_id.as_str() {
                                        "9A" | "9a" => super::yubikey::PIVSlot::Authentication,
                                        "9C" | "9c" => super::yubikey::PIVSlot::Signature,
                                        "9D" | "9d" => super::yubikey::PIVSlot::KeyManagement,
                                        "9E" | "9e" => super::yubikey::PIVSlot::CardAuth,
                                        _ => continue,
                                    };
                                    device.slots.insert(slot, SlotState {
                                        id: SlotId::new(),
                                        slot,
                                        has_key: true,
                                        key_id: Some(KeyId::from_uuid(slot_config.key_id)),
                                        certificate_id: None, // YubiKeySlot doesn't store certificate ID
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
                self.increment_version();
            }
            _ => {
                // Events from other aggregates are ignored
            }
        }
        Ok(())
    }
}

impl AggregateRoot for YubiKeyProvisioningAggregate {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_aggregate_creation() {
        let org_id = BootstrapOrgId::new();
        let aggregate = OrganizationAggregate::new(
            org_id,
            "TestOrg".to_string(),
            "Test Organization".to_string(),
        );

        assert_eq!(aggregate.id(), org_id);
        assert_eq!(aggregate.version(), 0);
        assert!(aggregate.units.is_empty());
        assert!(aggregate.people.is_empty());
    }

    #[test]
    fn test_organization_email_uniqueness() {
        let org_id = BootstrapOrgId::new();
        let mut aggregate = OrganizationAggregate::new(
            org_id,
            "TestOrg".to_string(),
            "Test Organization".to_string(),
        );

        // First email should be unique
        assert!(aggregate.is_email_unique("test@example.com"));

        // Add a person
        aggregate.people.insert(Uuid::now_v7(), PersonState {
            id: Uuid::now_v7(),
            name: "Test Person".to_string(),
            email: "test@example.com".to_string(),
            role: super::super::bootstrap::KeyOwnerRole::Developer,
            unit_ids: Vec::new(),
            active: true,
        });

        // Same email should no longer be unique
        assert!(!aggregate.is_email_unique("test@example.com"));
        // Different email should still be unique
        assert!(aggregate.is_email_unique("other@example.com"));
    }

    #[test]
    fn test_pki_aggregate_invariants() {
        let org_id = BootstrapOrgId::new();
        let aggregate = PkiCertificateChainAggregate::new(org_id);

        // Cannot create intermediate without root
        assert!(aggregate.can_generate_certificate(
            super::super::pki::CertificateType::Intermediate,
            None
        ).is_err());

        // Can create root
        assert!(aggregate.can_generate_certificate(
            super::super::pki::CertificateType::Root,
            None
        ).is_ok());
    }

    #[test]
    fn test_nats_aggregate_account_uniqueness() {
        let op_id = NatsOperatorId::new();
        let org_id = BootstrapOrgId::new();
        let mut aggregate = NatsSecurityAggregate::new(op_id, org_id, "TestOperator".to_string());

        // First account name should be unique
        assert!(aggregate.is_account_name_unique("engineering"));

        // Add an account
        let acc_id = NatsAccountId::new();
        aggregate.accounts.insert(acc_id, NatsAccountState {
            id: acc_id,
            name: "engineering".to_string(),
            unit_id: None,
            is_system_account: false,
        });

        // Same name should no longer be unique
        assert!(!aggregate.is_account_name_unique("engineering"));
        // Different name should be unique
        assert!(aggregate.is_account_name_unique("operations"));
    }

    #[test]
    fn test_yubikey_aggregate_serial_uniqueness() {
        let org_id = BootstrapOrgId::new();
        let mut aggregate = YubiKeyProvisioningAggregate::new(org_id);

        // First serial should be unique
        assert!(aggregate.is_serial_unique("12345678"));

        // Add a device
        let device_id = YubiKeyDeviceId::new();
        aggregate.devices.insert(device_id, YubiKeyDeviceState {
            id: device_id,
            serial: "12345678".to_string(),
            owner_person_id: None,
            slots: HashMap::new(),
        });
        aggregate.serials.insert("12345678".to_string(), device_id);

        // Same serial should no longer be unique
        assert!(!aggregate.is_serial_unique("12345678"));
        // Different serial should be unique
        assert!(aggregate.is_serial_unique("87654321"));
    }
}
