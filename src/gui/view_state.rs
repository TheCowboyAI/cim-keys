// Copyright (c) 2025 - Cowboy AI, LLC.

//! GUI View State (Presentation Layer)
//!
//! This module defines **view state structs** for the GUI presentation layer.
//! These are NOT domain Value Objects (those live in `src/value_objects/`).
//!
//! ## Distinction from Domain Value Objects
//!
//! | View State (this module) | Domain Value Objects |
//! |--------------------------|----------------------|
//! | Transient UI state | Persistent domain concepts |
//! | Changes with keystrokes | Immutable after creation |
//! | Not persisted | Stored in event store |
//! | No CID identity needed | CID-based identity |
//! | Presentation layer | Domain layer |
//!
//! ## Functional Updates
//!
//! View state follows FRP principles with `with_*` methods for immutable updates:
//! ```rust,ignore
//! // Replace whole struct, never mutate fields
//! let new_form = old_form.with_name("New Name".to_string());
//! ```
//!
//! ## Why Serializable?
//!
//! Structs derive Serialize/Deserialize for:
//! - Debug snapshots
//! - State persistence (optional)
//! - NOT for CID identity (use domain Value Objects for that)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use uuid::Uuid;

use crate::domain::{
    KeyOwnerRole, LocationType, OrganizationUnit, OrganizationUnitType,
    ServiceAccount, InvariantKeyPurpose, KeyPermission,
    ids::UnitId,
};
use crate::ports::gpg::{GpgKeyId, GpgKeyInfo, GpgKeyType};
use crate::ports::yubikey::PivSlot;
use crate::projections::{CertificateEntry, KeyEntry, LocationEntry, PersonEntry};

// ============================================================================
// ORGANIZATION FORM VALUE OBJECT
// ============================================================================

/// Organization creation/editing form state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationForm {
    pub name: String,
    pub domain: String,
    pub admin_email: String,
}


impl OrganizationForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn with_domain(self, domain: String) -> Self {
        Self { domain, ..self }
    }

    pub fn with_admin_email(self, admin_email: String) -> Self {
        Self { admin_email, ..self }
    }

    pub fn clear(self) -> Self {
        Self::default()
    }
}

// ============================================================================
// PASSPHRASE STATE VALUE OBJECT
// ============================================================================

/// Passphrase entry state (master and root)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PassphraseState {
    pub master: String,
    pub master_confirm: String,
    pub root: String,
    pub root_confirm: String,
    pub show_passphrase: bool,
}


impl PassphraseState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_master(self, master: String) -> Self {
        Self { master, ..self }
    }

    pub fn with_master_confirm(self, master_confirm: String) -> Self {
        Self { master_confirm, ..self }
    }

    pub fn with_root(self, root: String) -> Self {
        Self { root, ..self }
    }

    pub fn with_root_confirm(self, root_confirm: String) -> Self {
        Self { root_confirm, ..self }
    }

    pub fn with_visibility(self, show_passphrase: bool) -> Self {
        Self { show_passphrase, ..self }
    }

    pub fn toggle_visibility(self) -> Self {
        Self { show_passphrase: !self.show_passphrase, ..self }
    }

    pub fn master_matches(&self) -> bool {
        !self.master.is_empty() && self.master == self.master_confirm
    }

    pub fn root_matches(&self) -> bool {
        !self.root.is_empty() && self.root == self.root_confirm
    }
}

// ============================================================================
// NEW PERSON FORM VALUE OBJECT
// ============================================================================

/// Form state for adding a new person
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewPersonForm {
    pub name: String,
    pub email: String,
    pub role: Option<KeyOwnerRole>,
}


impl NewPersonForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn with_email(self, email: String) -> Self {
        Self { email, ..self }
    }

    pub fn with_role(self, role: KeyOwnerRole) -> Self {
        Self { role: Some(role), ..self }
    }

    pub fn clear(self) -> Self {
        Self::default()
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.email.is_empty()
    }
}

// ============================================================================
// NEW LOCATION FORM VALUE OBJECT
// ============================================================================

/// Form state for adding a new location
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewLocationForm {
    pub name: String,
    pub location_type: Option<LocationType>,
    pub street: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub postal: String,
    pub url: String,
}


impl NewLocationForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn with_location_type(self, location_type: LocationType) -> Self {
        Self { location_type: Some(location_type), ..self }
    }

    pub fn with_street(self, street: String) -> Self {
        Self { street, ..self }
    }

    pub fn with_city(self, city: String) -> Self {
        Self { city, ..self }
    }

    pub fn with_region(self, region: String) -> Self {
        Self { region, ..self }
    }

    pub fn with_country(self, country: String) -> Self {
        Self { country, ..self }
    }

    pub fn with_postal(self, postal: String) -> Self {
        Self { postal, ..self }
    }

    pub fn with_url(self, url: String) -> Self {
        Self { url, ..self }
    }

    pub fn clear(self) -> Self {
        Self::default()
    }
}

// ============================================================================
// NEW ORGANIZATION UNIT FORM VALUE OBJECT
// ============================================================================

/// Form state for creating organization units
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewOrgUnitForm {
    pub name: String,
    pub unit_type: Option<OrganizationUnitType>,
    pub parent: Option<String>,
    pub nats_account: String,
    pub responsible_person: Option<Uuid>,
}


impl NewOrgUnitForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn with_unit_type(self, unit_type: OrganizationUnitType) -> Self {
        Self { unit_type: Some(unit_type), ..self }
    }

    pub fn with_parent(self, parent: String) -> Self {
        Self { parent: Some(parent), ..self }
    }

    pub fn with_nats_account(self, nats_account: String) -> Self {
        Self { nats_account, ..self }
    }

    pub fn with_responsible_person(self, responsible_person: Uuid) -> Self {
        Self { responsible_person: Some(responsible_person), ..self }
    }

    pub fn clear(self) -> Self {
        Self::default()
    }
}

// ============================================================================
// NEW SERVICE ACCOUNT FORM VALUE OBJECT
// ============================================================================

/// Form state for creating service accounts
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewServiceAccountForm {
    pub name: String,
    pub purpose: String,
    pub owning_unit: Option<Uuid>,
    pub responsible_person: Option<Uuid>,
}


impl NewServiceAccountForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn with_purpose(self, purpose: String) -> Self {
        Self { purpose, ..self }
    }

    pub fn with_owning_unit(self, owning_unit: Uuid) -> Self {
        Self { owning_unit: Some(owning_unit), ..self }
    }

    pub fn with_responsible_person(self, responsible_person: Uuid) -> Self {
        Self { responsible_person: Some(responsible_person), ..self }
    }

    pub fn clear(self) -> Self {
        Self::default()
    }
}

// ============================================================================
// SECTION VISIBILITY VALUE OBJECT
// ============================================================================

/// Collapsible section visibility state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionVisibility {
    pub org_unit: bool,
    pub service_account: bool,
    pub root_ca: bool,
    pub intermediate_ca: bool,
    pub server_cert: bool,
    pub yubikey: bool,
    pub nats: bool,
    pub certificates: bool,
    pub keys: bool,
    pub gpg: bool,
    pub recovery: bool,
    pub yubikey_slot: bool,
    pub trust_chain: bool,
    pub delegation: bool,
    pub multi_purpose_key: bool,
    pub event_log: bool,
    pub nats_viz: bool,
}


impl Default for SectionVisibility {
    fn default() -> Self {
        Self {
            // Most sections start collapsed
            org_unit: true,
            service_account: true,
            root_ca: false,  // Root CA visible by default
            intermediate_ca: true,
            server_cert: true,
            yubikey: true,
            nats: true,
            certificates: true,
            keys: true,
            gpg: true,
            recovery: true,
            yubikey_slot: true,
            trust_chain: true,
            delegation: true,
            multi_purpose_key: true,
            event_log: true,
            nats_viz: true,
        }
    }
}

impl SectionVisibility {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_org_unit(self) -> Self {
        Self { org_unit: !self.org_unit, ..self }
    }

    pub fn toggle_service_account(self) -> Self {
        Self { service_account: !self.service_account, ..self }
    }

    pub fn toggle_root_ca(self) -> Self {
        Self { root_ca: !self.root_ca, ..self }
    }

    pub fn toggle_intermediate_ca(self) -> Self {
        Self { intermediate_ca: !self.intermediate_ca, ..self }
    }

    pub fn toggle_server_cert(self) -> Self {
        Self { server_cert: !self.server_cert, ..self }
    }

    pub fn toggle_yubikey(self) -> Self {
        Self { yubikey: !self.yubikey, ..self }
    }

    pub fn toggle_nats(self) -> Self {
        Self { nats: !self.nats, ..self }
    }

    pub fn toggle_certificates(self) -> Self {
        Self { certificates: !self.certificates, ..self }
    }

    pub fn toggle_keys(self) -> Self {
        Self { keys: !self.keys, ..self }
    }

    pub fn toggle_gpg(self) -> Self {
        Self { gpg: !self.gpg, ..self }
    }

    pub fn toggle_recovery(self) -> Self {
        Self { recovery: !self.recovery, ..self }
    }

    pub fn toggle_yubikey_slot(self) -> Self {
        Self { yubikey_slot: !self.yubikey_slot, ..self }
    }

    pub fn toggle_trust_chain(self) -> Self {
        Self { trust_chain: !self.trust_chain, ..self }
    }

    pub fn toggle_delegation(self) -> Self {
        Self { delegation: !self.delegation, ..self }
    }

    pub fn toggle_multi_purpose_key(self) -> Self {
        Self { multi_purpose_key: !self.multi_purpose_key, ..self }
    }

    pub fn toggle_event_log(self) -> Self {
        Self { event_log: !self.event_log, ..self }
    }

    pub fn toggle_nats_viz(self) -> Self {
        Self { nats_viz: !self.nats_viz, ..self }
    }
}

// ============================================================================
// GPG STATE VALUE OBJECT
// ============================================================================

/// GPG key generation state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpgState {
    pub user_id: String,
    pub key_type: Option<GpgKeyType>,
    pub key_length: String,
    pub expires_days: String,
    pub generated_keys: Vec<GpgKeyInfo>,
    pub generation_status: Option<String>,
}


impl GpgState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user_id(self, user_id: String) -> Self {
        Self { user_id, ..self }
    }

    pub fn with_key_type(self, key_type: GpgKeyType) -> Self {
        Self { key_type: Some(key_type), ..self }
    }

    pub fn with_key_length(self, key_length: String) -> Self {
        Self { key_length, ..self }
    }

    pub fn with_expires_days(self, expires_days: String) -> Self {
        Self { expires_days, ..self }
    }

    pub fn with_generated_key(self, key: GpgKeyInfo) -> Self {
        let mut generated_keys = self.generated_keys;
        generated_keys.push(key);
        Self { generated_keys, ..self }
    }

    pub fn with_generation_status(self, status: String) -> Self {
        Self { generation_status: Some(status), ..self }
    }

    pub fn clear_form(self) -> Self {
        Self {
            user_id: String::new(),
            key_type: None,
            key_length: String::new(),
            expires_days: String::new(),
            generation_status: None,
            ..self
        }
    }
}

// ============================================================================
// RECOVERY STATE VALUE OBJECT
// ============================================================================

/// Key recovery from seed state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryState {
    pub passphrase: String,
    pub passphrase_confirm: String,
    pub organization_id: String,
    pub status: Option<String>,
    pub seed_verified: bool,
}


impl RecoveryState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_passphrase(self, passphrase: String) -> Self {
        Self { passphrase, ..self }
    }

    pub fn with_passphrase_confirm(self, passphrase_confirm: String) -> Self {
        Self { passphrase_confirm, ..self }
    }

    pub fn with_organization_id(self, organization_id: String) -> Self {
        Self { organization_id, ..self }
    }

    pub fn with_status(self, status: String) -> Self {
        Self { status: Some(status), ..self }
    }

    pub fn with_seed_verified(self, seed_verified: bool) -> Self {
        Self { seed_verified, ..self }
    }

    pub fn clear(self) -> Self {
        Self::default()
    }

    pub fn passphrase_matches(&self) -> bool {
        !self.passphrase.is_empty() && self.passphrase == self.passphrase_confirm
    }
}

// ============================================================================
// CERTIFICATE FORM VALUE OBJECT
// ============================================================================

/// Certificate generation form state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CertificateForm {
    pub intermediate_ca_name: String,
    pub server_cert_cn: String,
    pub server_cert_sans: String,
    pub selected_intermediate_ca: Option<String>,
    pub selected_location: Option<String>,
    pub selected_unit_for_ca: Option<String>,
    // Metadata
    pub organization: String,
    pub organizational_unit: String,
    pub locality: String,
    pub state_province: String,
    pub country: String,
    pub validity_days: String,
}


impl CertificateForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_intermediate_ca_name(self, name: String) -> Self {
        Self { intermediate_ca_name: name, ..self }
    }

    pub fn with_server_cert_cn(self, cn: String) -> Self {
        Self { server_cert_cn: cn, ..self }
    }

    pub fn with_server_cert_sans(self, sans: String) -> Self {
        Self { server_cert_sans: sans, ..self }
    }

    pub fn with_selected_intermediate_ca(self, ca: String) -> Self {
        Self { selected_intermediate_ca: Some(ca), ..self }
    }

    pub fn with_selected_location(self, location: String) -> Self {
        Self { selected_location: Some(location), ..self }
    }

    pub fn with_organization(self, organization: String) -> Self {
        Self { organization, ..self }
    }

    pub fn with_organizational_unit(self, ou: String) -> Self {
        Self { organizational_unit: ou, ..self }
    }

    pub fn with_locality(self, locality: String) -> Self {
        Self { locality, ..self }
    }

    pub fn with_state_province(self, state_province: String) -> Self {
        Self { state_province, ..self }
    }

    pub fn with_country(self, country: String) -> Self {
        Self { country, ..self }
    }

    pub fn with_validity_days(self, validity_days: String) -> Self {
        Self { validity_days, ..self }
    }

    pub fn clear_intermediate(self) -> Self {
        Self {
            intermediate_ca_name: String::new(),
            selected_unit_for_ca: None,
            ..self
        }
    }

    pub fn clear_server(self) -> Self {
        Self {
            server_cert_cn: String::new(),
            server_cert_sans: String::new(),
            selected_intermediate_ca: None,
            selected_location: None,
            ..self
        }
    }
}

// ============================================================================
// DELEGATION STATE VALUE OBJECT
// ============================================================================

/// Delegation management state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DelegationState {
    pub from_person: Option<Uuid>,
    pub to_person: Option<Uuid>,
    pub permissions: HashSet<KeyPermission>,
    pub expires_days: String,
    pub active_delegations: Vec<DelegationEntry>,
}


/// A single active delegation entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationEntry {
    pub id: Uuid,
    pub from_person_id: Uuid,
    pub from_person_name: String,
    pub to_person_id: Uuid,
    pub to_person_name: String,
    pub permissions: HashSet<KeyPermission>,
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
}


impl DelegationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_from_person(self, from_person: Uuid) -> Self {
        Self { from_person: Some(from_person), ..self }
    }

    pub fn with_to_person(self, to_person: Uuid) -> Self {
        Self { to_person: Some(to_person), ..self }
    }

    pub fn with_permission_added(self, permission: KeyPermission) -> Self {
        let mut permissions = self.permissions;
        permissions.insert(permission);
        Self { permissions, ..self }
    }

    pub fn with_permission_removed(self, permission: &KeyPermission) -> Self {
        let mut permissions = self.permissions;
        permissions.remove(permission);
        Self { permissions, ..self }
    }

    pub fn with_expires_days(self, expires_days: String) -> Self {
        Self { expires_days, ..self }
    }

    pub fn with_delegation_added(self, entry: DelegationEntry) -> Self {
        let mut active_delegations = self.active_delegations;
        active_delegations.push(entry);
        Self { active_delegations, ..self }
    }

    pub fn with_delegation_removed(self, id: Uuid) -> Self {
        let active_delegations: Vec<_> = self.active_delegations
            .into_iter()
            .filter(|d| d.id != id)
            .collect();
        Self { active_delegations, ..self }
    }

    pub fn clear_form(self) -> Self {
        Self {
            from_person: None,
            to_person: None,
            permissions: HashSet::new(),
            expires_days: String::new(),
            ..self
        }
    }
}

// ============================================================================
// YUBIKEY STATE VALUE OBJECT
// ============================================================================

/// YubiKey management state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct YubiKeyState {
    pub serial: String,
    pub detected_devices: Vec<crate::ports::yubikey::YubiKeyDevice>,
    pub detection_status: String,
    pub configs: Vec<crate::domain::YubiKeyConfig>,
    pub assignments: HashMap<String, Uuid>,  // serial -> person_id
    pub provisioning_status: HashMap<String, String>,  // serial -> status
    pub selected_for_assignment: Option<String>,
    pub selected_for_management: Option<String>,
    pub pin_input: String,
    pub new_pin: String,
    pub pin_confirm: String,
    pub management_key: String,
    pub new_management_key: String,
    #[serde(skip)]  // SlotInfo may not be serializable
    pub slot_info: HashMap<String, Vec<super::SlotInfo>>,
    pub slot_operation_status: Option<String>,
    pub attestation_result: Option<String>,
    pub selected_piv_slot: Option<PivSlot>,
    pub registration_name: String,
    pub registered: HashMap<String, Uuid>,  // serial -> domain entity ID
}


impl YubiKeyState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_serial(self, serial: String) -> Self {
        Self { serial, ..self }
    }

    pub fn with_detected_devices(self, devices: Vec<crate::ports::yubikey::YubiKeyDevice>) -> Self {
        Self { detected_devices: devices, ..self }
    }

    pub fn with_detection_status(self, status: String) -> Self {
        Self { detection_status: status, ..self }
    }

    pub fn with_selected_for_assignment(self, serial: String) -> Self {
        Self { selected_for_assignment: Some(serial), ..self }
    }

    pub fn with_selected_for_management(self, serial: String) -> Self {
        Self { selected_for_management: Some(serial), ..self }
    }

    pub fn with_pin_input(self, pin: String) -> Self {
        Self { pin_input: pin, ..self }
    }

    pub fn with_new_pin(self, pin: String) -> Self {
        Self { new_pin: pin, ..self }
    }

    pub fn with_pin_confirm(self, pin: String) -> Self {
        Self { pin_confirm: pin, ..self }
    }

    pub fn with_management_key(self, key: String) -> Self {
        Self { management_key: key, ..self }
    }

    pub fn with_new_management_key(self, key: String) -> Self {
        Self { new_management_key: key, ..self }
    }

    pub fn with_selected_piv_slot(self, slot: PivSlot) -> Self {
        Self { selected_piv_slot: Some(slot), ..self }
    }

    pub fn with_registration_name(self, name: String) -> Self {
        Self { registration_name: name, ..self }
    }

    pub fn with_slot_operation_status(self, status: String) -> Self {
        Self { slot_operation_status: Some(status), ..self }
    }

    pub fn with_attestation_result(self, result: String) -> Self {
        Self { attestation_result: Some(result), ..self }
    }

    pub fn clear_pin_form(self) -> Self {
        Self {
            pin_input: String::new(),
            new_pin: String::new(),
            pin_confirm: String::new(),
            ..self
        }
    }
}

// ============================================================================
// EXPORT STATE VALUE OBJECT
// ============================================================================

/// Export/projection configuration state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportState {
    pub path: PathBuf,
    pub include_public_keys: bool,
    pub include_certificates: bool,
    pub include_nats_config: bool,
    pub include_private_keys: bool,
    pub password: String,
}


impl Default for ExportState {
    fn default() -> Self {
        Self {
            path: PathBuf::from("/tmp/cim-keys-export"),
            include_public_keys: true,
            include_certificates: true,
            include_nats_config: false,
            include_private_keys: false,
            password: String::new(),
        }
    }
}

impl ExportState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(self, path: PathBuf) -> Self {
        Self { path, ..self }
    }

    pub fn with_include_public_keys(self, include: bool) -> Self {
        Self { include_public_keys: include, ..self }
    }

    pub fn with_include_certificates(self, include: bool) -> Self {
        Self { include_certificates: include, ..self }
    }

    pub fn with_include_nats_config(self, include: bool) -> Self {
        Self { include_nats_config: include, ..self }
    }

    pub fn with_include_private_keys(self, include: bool) -> Self {
        Self { include_private_keys: include, ..self }
    }

    pub fn with_password(self, password: String) -> Self {
        Self { password, ..self }
    }
}

// ============================================================================
// MULTI-PURPOSE KEY STATE VALUE OBJECT
// ============================================================================

/// Multi-purpose key generation state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiKeyState {
    pub selected_person: Option<Uuid>,
    pub selected_purposes: HashSet<InvariantKeyPurpose>,
}


impl MultiKeyState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_selected_person(self, person: Uuid) -> Self {
        Self { selected_person: Some(person), ..self }
    }

    pub fn with_purpose_added(self, purpose: InvariantKeyPurpose) -> Self {
        let mut selected_purposes = self.selected_purposes;
        selected_purposes.insert(purpose);
        Self { selected_purposes, ..self }
    }

    pub fn with_purpose_removed(self, purpose: &InvariantKeyPurpose) -> Self {
        let mut selected_purposes = self.selected_purposes;
        selected_purposes.remove(purpose);
        Self { selected_purposes, ..self }
    }

    pub fn clear(self) -> Self {
        Self::default()
    }
}

// ============================================================================
// CLIENT CERTIFICATE STATE VALUE OBJECT
// ============================================================================

/// mTLS client certificate state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientCertState {
    pub cn: String,
    pub email: String,
}


impl ClientCertState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cn(self, cn: String) -> Self {
        Self { cn, ..self }
    }

    pub fn with_email(self, email: String) -> Self {
        Self { email, ..self }
    }

    pub fn clear(self) -> Self {
        Self::default()
    }
}

// ============================================================================
// LOADED DATA VALUE OBJECT
// ============================================================================

/// Data loaded from manifest/secrets
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadedData {
    pub locations: Vec<LocationEntry>,
    pub people: Vec<PersonEntry>,
    pub certificates: Vec<CertificateEntry>,
    pub keys: Vec<KeyEntry>,
    pub units: Vec<OrganizationUnit>,
    pub service_accounts: Vec<ServiceAccount>,
}


impl LoadedData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_locations(self, locations: Vec<LocationEntry>) -> Self {
        Self { locations, ..self }
    }

    pub fn with_people(self, people: Vec<PersonEntry>) -> Self {
        Self { people, ..self }
    }

    pub fn with_certificates(self, certificates: Vec<CertificateEntry>) -> Self {
        Self { certificates, ..self }
    }

    pub fn with_keys(self, keys: Vec<KeyEntry>) -> Self {
        Self { keys, ..self }
    }

    pub fn with_units(self, units: Vec<OrganizationUnit>) -> Self {
        Self { units, ..self }
    }

    pub fn with_service_accounts(self, service_accounts: Vec<ServiceAccount>) -> Self {
        Self { service_accounts, ..self }
    }

    pub fn with_unit_added(self, unit: OrganizationUnit) -> Self {
        let mut units = self.units;
        units.push(unit);
        Self { units, ..self }
    }

    pub fn with_unit_removed(self, id: UnitId) -> Self {
        let units: Vec<_> = self.units.into_iter().filter(|u| u.id != id).collect();
        Self { units, ..self }
    }

    pub fn with_service_account_added(self, sa: ServiceAccount) -> Self {
        let mut service_accounts = self.service_accounts;
        service_accounts.push(sa);
        Self { service_accounts, ..self }
    }

    pub fn with_service_account_removed(self, id: Uuid) -> Self {
        let service_accounts: Vec<_> = self.service_accounts
            .into_iter()
            .filter(|sa| sa.id != id)
            .collect();
        Self { service_accounts, ..self }
    }
}

// ============================================================================
// STATUS STATE VALUE OBJECT
// ============================================================================

/// Application status/error message state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusState {
    pub message: String,
    pub error: Option<String>,
    pub overwrite_warning: Option<String>,
}


impl StatusState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_message(self, message: String) -> Self {
        Self { message, error: None, overwrite_warning: None }
    }

    pub fn with_error(self, error: String) -> Self {
        Self { error: Some(error), ..self }
    }

    pub fn with_overwrite_warning(self, warning: String) -> Self {
        Self { overwrite_warning: Some(warning), ..self }
    }

    pub fn clear_error(self) -> Self {
        Self { error: None, ..self }
    }

    pub fn clear_warning(self) -> Self {
        Self { overwrite_warning: None, ..self }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_form_immutability() {
        let form = OrganizationForm::new();
        let form2 = form.clone().with_name("Test Org".to_string());

        // Original unchanged
        assert!(form.name.is_empty());
        // New has value
        assert_eq!(form2.name, "Test Org");
    }

    #[test]
    fn test_section_visibility_toggle() {
        let vis = SectionVisibility::default();
        let vis2 = vis.clone().toggle_root_ca();

        // Root CA starts non-collapsed
        assert!(!vis.root_ca);
        // After toggle it's collapsed
        assert!(vis2.root_ca);
    }

    #[test]
    fn test_passphrase_matching() {
        let state = PassphraseState::new()
            .with_master("test123".to_string())
            .with_master_confirm("test123".to_string());

        assert!(state.master_matches());

        let state2 = state.with_master_confirm("different".to_string());
        assert!(!state2.master_matches());
    }

    #[test]
    fn test_new_person_form_validation() {
        let form = NewPersonForm::new();
        assert!(!form.is_valid());

        let form2 = form
            .with_name("John Doe".to_string())
            .with_email("john@example.com".to_string());
        assert!(form2.is_valid());
    }

    #[test]
    fn test_delegation_state_permissions() {
        let state = DelegationState::new()
            .with_permission_added(KeyPermission::Sign)
            .with_permission_added(KeyPermission::Encrypt);

        assert_eq!(state.permissions.len(), 2);

        let state2 = state.with_permission_removed(&KeyPermission::Sign);
        assert_eq!(state2.permissions.len(), 1);
        assert!(state2.permissions.contains(&KeyPermission::Encrypt));
    }

    #[test]
    fn test_gpg_state_with_generated_key() {
        let key = GpgKeyInfo {
            key_id: GpgKeyId("ABC123".to_string()),
            fingerprint: "FINGERPRINT".to_string(),
            user_ids: vec!["Test <test@example.com>".to_string()],
            creation_time: 0,
            expiration_time: None,
            is_revoked: false,
            is_expired: false,
        };

        let state = GpgState::new().with_generated_key(key);
        assert_eq!(state.generated_keys.len(), 1);
    }

    // Note: CID tests removed - view state is presentation layer, not domain.
    // Domain Value Objects with CID identity live in src/value_objects/
}
