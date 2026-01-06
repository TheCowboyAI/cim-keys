// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Domain Module
//!
//! This module handles the Organization bounded context, including:
//! - Domain creation and loading
//! - Organization configuration
//! - People management (add/remove/select)
//! - Location management
//! - Organization unit hierarchy
//! - Service account management
//!
//! ## Message Flow
//!
//! ```text
//! User Action → OrganizationMessage → update() → Task<Message>
//!                                        ↓
//!                               OrganizationState mutated
//! ```
//!
//! ## Domain Aggregate
//!
//! The organization domain corresponds to the Organization aggregate root,
//! with People, Locations, and OrganizationUnits as child entities.

use iced::Task;
use uuid::Uuid;

use crate::domain::{
    KeyOwnerRole, LocationType, OrganizationUnit, OrganizationUnitType,
    ServiceAccount,
};
use crate::secrets_loader::BootstrapData;
use crate::projections::{
    CertificateEntry, KeyEntry, LocationEntry, OrganizationInfo, PersonEntry,
};

use super::super::{BootstrapConfig, Message};

/// Organization domain messages
///
/// These messages handle all organization-related UI interactions and
/// async operation results. They are delegated from the root Message enum.
#[derive(Debug, Clone)]
pub enum OrganizationMessage {
    // ============================================================================
    // Domain Operations
    // ============================================================================
    /// Create a new domain (organization + PKI hierarchy)
    CreateNewDomain,
    /// Load an existing domain from the projection
    LoadExistingDomain,
    /// Import organization data from secrets file
    ImportFromSecrets,
    /// Result of domain creation
    DomainCreated(Result<String, String>),
    /// Result of domain loading
    DomainLoaded(Result<BootstrapConfig, String>),
    /// Result of secrets import
    SecretsImported(Result<BootstrapData, String>),
    /// Result of loading manifest data
    ManifestDataLoaded(
        Result<
            (
                OrganizationInfo,
                Vec<LocationEntry>,
                Vec<PersonEntry>,
                Vec<CertificateEntry>,
                Vec<KeyEntry>,
            ),
            String,
        >,
    ),

    // ============================================================================
    // Organization Form Inputs
    // ============================================================================
    /// Organization name changed
    NameChanged(String),
    /// Organization domain changed
    DomainChanged(String),
    /// Master passphrase changed
    MasterPassphraseChanged(String),
    /// Master passphrase confirmation changed
    MasterPassphraseConfirmChanged(String),

    // ============================================================================
    // People Operations
    // ============================================================================
    /// New person name changed
    NewPersonNameChanged(String),
    /// New person email changed
    NewPersonEmailChanged(String),
    /// New person role selected
    NewPersonRoleSelected(KeyOwnerRole),
    /// Add a new person to the organization
    AddPerson,
    /// Remove a person from the organization
    RemovePerson(Uuid),
    /// Select a person (for detail view)
    SelectPerson(Uuid),

    // ============================================================================
    // Inline Editing
    // ============================================================================
    /// Context-aware node type selection from dropdown
    NodeTypeSelected(String),
    /// Inline edit name changed
    InlineEditNameChanged(String),
    /// Submit inline edit
    InlineEditSubmit,
    /// Cancel inline edit
    InlineEditCancel,

    // ============================================================================
    // Location Operations
    // ============================================================================
    /// New location name changed
    NewLocationNameChanged(String),
    /// New location type selected
    NewLocationTypeSelected(LocationType),
    /// New location street changed
    NewLocationStreetChanged(String),
    /// New location city changed
    NewLocationCityChanged(String),
    /// New location region changed
    NewLocationRegionChanged(String),
    /// New location country changed
    NewLocationCountryChanged(String),
    /// New location postal code changed
    NewLocationPostalChanged(String),
    /// New location URL changed
    NewLocationUrlChanged(String),
    /// Add a new location
    AddLocation,
    /// Remove a location
    RemoveLocation(Uuid),

    // ============================================================================
    // Organization Unit Operations
    // ============================================================================
    /// Toggle organization unit section visibility
    ToggleOrgUnitSection,
    /// New unit name changed
    NewUnitNameChanged(String),
    /// New unit type selected
    NewUnitTypeSelected(OrganizationUnitType),
    /// New unit parent selected
    NewUnitParentSelected(String),
    /// New unit NATS account changed
    NewUnitNatsAccountChanged(String),
    /// New unit responsible person selected
    NewUnitResponsiblePersonSelected(Uuid),
    /// Create organization unit
    CreateOrganizationUnit,
    /// Result of organization unit creation
    OrganizationUnitCreated(Result<OrganizationUnit, String>),
    /// Remove organization unit
    RemoveOrganizationUnit(Uuid),

    // ============================================================================
    // Service Account Operations
    // ============================================================================
    /// Toggle service account section visibility
    ToggleServiceAccountSection,
    /// New service account name changed
    NewServiceAccountNameChanged(String),
    /// New service account purpose changed
    NewServiceAccountPurposeChanged(String),
    /// Service account owning unit selected
    ServiceAccountOwningUnitSelected(Uuid),
    /// Service account responsible person selected
    ServiceAccountResponsiblePersonSelected(Uuid),
    /// Create service account
    CreateServiceAccount,
    /// Result of service account creation
    ServiceAccountCreated(Result<ServiceAccount, String>),
    /// Deactivate service account
    DeactivateServiceAccount(Uuid),
    /// Remove service account
    RemoveServiceAccount(Uuid),
    /// Generate key for service account
    GenerateServiceAccountKey { service_account_id: Uuid },
    /// Result of service account key generation
    ServiceAccountKeyGenerated(Result<(Uuid, KeyOwnerRole), String>),
}

/// Organization domain state
///
/// This struct holds all state relevant to the organization bounded context.
/// Field names match CimKeysApp for easy state synchronization.
#[derive(Debug, Clone, Default)]
pub struct OrganizationState {
    // Domain status
    pub domain_loaded: bool,

    // Organization form fields
    pub organization_name: String,
    pub organization_domain: String,
    pub organization_id: Option<Uuid>,
    pub admin_email: String,

    // Master passphrase
    pub master_passphrase: String,
    pub master_passphrase_confirm: String,

    // People management
    pub new_person_name: String,
    pub new_person_email: String,
    pub new_person_role: Option<KeyOwnerRole>,
    pub selected_person: Option<Uuid>,

    // Inline editing (matches CimKeysApp field names)
    pub inline_edit_name: String,
    pub editing_new_node: Option<Uuid>,

    // Location management
    pub new_location_name: String,
    pub new_location_type: Option<LocationType>,
    pub new_location_street: String,
    pub new_location_city: String,
    pub new_location_region: String,
    pub new_location_country: String,
    pub new_location_postal: String,
    pub new_location_url: String,

    // Organization unit management (collapsed = !show)
    pub org_unit_section_collapsed: bool,
    pub new_unit_name: String,
    pub new_unit_type: Option<OrganizationUnitType>,
    pub new_unit_parent: Option<String>,
    pub new_unit_nats_account: String,
    pub new_unit_responsible_person: Option<Uuid>,

    // Service account management (collapsed = !show)
    pub service_account_section_collapsed: bool,
    pub new_service_account_name: String,
    pub new_service_account_purpose: String,
    pub new_service_account_owning_unit: Option<Uuid>,
    pub new_service_account_responsible_person: Option<Uuid>,
}

impl OrganizationState {
    /// Create new organization state with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method: set organization name
    pub fn with_organization_name(mut self, name: String) -> Self {
        self.organization_name = name;
        self
    }

    /// Builder method: set organization domain
    pub fn with_organization_domain(mut self, domain: String) -> Self {
        self.organization_domain = domain;
        self
    }

    /// Builder method: mark domain as loaded
    pub fn with_domain_loaded(mut self, loaded: bool) -> Self {
        self.domain_loaded = loaded;
        self
    }
}

/// Update function for organization messages
///
/// This is a pure function that processes organization messages and returns
/// a Task that may produce further Messages. State mutations are handled
/// by the caller after receiving the returned OrganizationState.
///
/// ## MVI Pattern
///
/// Following the Model-View-Intent pattern:
/// - Intent: OrganizationMessage (user interaction or async result)
/// - Model: OrganizationState (immutable updates)
/// - View: Rendered by view functions in gui.rs
pub fn update(
    state: &mut OrganizationState,
    message: OrganizationMessage,
) -> Task<Message> {
    match message {
        // ========================================================================
        // Domain Operations
        // ========================================================================
        OrganizationMessage::CreateNewDomain => {
            // Handled by main update - returns async task
            Task::none()
        }
        OrganizationMessage::LoadExistingDomain => {
            // Handled by main update - returns async task
            Task::none()
        }
        OrganizationMessage::ImportFromSecrets => {
            // Handled by main update - returns async task
            Task::none()
        }
        OrganizationMessage::DomainCreated(result) => {
            match result {
                Ok(_) => state.domain_loaded = true,
                Err(_) => state.domain_loaded = false,
            }
            Task::none()
        }
        OrganizationMessage::DomainLoaded(result) => {
            if result.is_ok() {
                state.domain_loaded = true;
            }
            Task::none()
        }
        OrganizationMessage::SecretsImported(_) => {
            // Handled by main update for complex state merge
            Task::none()
        }
        OrganizationMessage::ManifestDataLoaded(_) => {
            // Handled by main update for complex state merge
            Task::none()
        }

        // ========================================================================
        // Organization Form Inputs
        // ========================================================================
        OrganizationMessage::NameChanged(name) => {
            state.organization_name = name;
            Task::none()
        }
        OrganizationMessage::DomainChanged(domain) => {
            state.organization_domain = domain;
            Task::none()
        }
        OrganizationMessage::MasterPassphraseChanged(passphrase) => {
            state.master_passphrase = passphrase;
            Task::none()
        }
        OrganizationMessage::MasterPassphraseConfirmChanged(passphrase) => {
            state.master_passphrase_confirm = passphrase;
            Task::none()
        }

        // ========================================================================
        // People Operations
        // ========================================================================
        OrganizationMessage::NewPersonNameChanged(name) => {
            state.new_person_name = name;
            Task::none()
        }
        OrganizationMessage::NewPersonEmailChanged(email) => {
            state.new_person_email = email;
            Task::none()
        }
        OrganizationMessage::NewPersonRoleSelected(role) => {
            state.new_person_role = Some(role);
            Task::none()
        }
        OrganizationMessage::AddPerson => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        OrganizationMessage::RemovePerson(_) => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        OrganizationMessage::SelectPerson(person_id) => {
            state.selected_person = Some(person_id);
            Task::none()
        }

        // ========================================================================
        // Inline Editing
        // ========================================================================
        OrganizationMessage::NodeTypeSelected(_) => {
            // Handled by main update - needs graph context
            Task::none()
        }
        OrganizationMessage::InlineEditNameChanged(name) => {
            state.inline_edit_name = name;
            Task::none()
        }
        OrganizationMessage::InlineEditSubmit => {
            // Handled by main update - needs graph context
            Task::none()
        }
        OrganizationMessage::InlineEditCancel => {
            state.inline_edit_name.clear();
            state.editing_new_node = None;
            Task::none()
        }

        // ========================================================================
        // Location Operations
        // ========================================================================
        OrganizationMessage::NewLocationNameChanged(name) => {
            state.new_location_name = name;
            Task::none()
        }
        OrganizationMessage::NewLocationTypeSelected(loc_type) => {
            state.new_location_type = Some(loc_type);
            Task::none()
        }
        OrganizationMessage::NewLocationStreetChanged(street) => {
            state.new_location_street = street;
            Task::none()
        }
        OrganizationMessage::NewLocationCityChanged(city) => {
            state.new_location_city = city;
            Task::none()
        }
        OrganizationMessage::NewLocationRegionChanged(region) => {
            state.new_location_region = region;
            Task::none()
        }
        OrganizationMessage::NewLocationCountryChanged(country) => {
            state.new_location_country = country;
            Task::none()
        }
        OrganizationMessage::NewLocationPostalChanged(postal) => {
            state.new_location_postal = postal;
            Task::none()
        }
        OrganizationMessage::NewLocationUrlChanged(url) => {
            state.new_location_url = url;
            Task::none()
        }
        OrganizationMessage::AddLocation => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        OrganizationMessage::RemoveLocation(_) => {
            // Handled by main update - needs aggregate access
            Task::none()
        }

        // ========================================================================
        // Organization Unit Operations
        // ========================================================================
        OrganizationMessage::ToggleOrgUnitSection => {
            state.org_unit_section_collapsed = !state.org_unit_section_collapsed;
            Task::none()
        }
        OrganizationMessage::NewUnitNameChanged(name) => {
            state.new_unit_name = name;
            Task::none()
        }
        OrganizationMessage::NewUnitTypeSelected(unit_type) => {
            state.new_unit_type = Some(unit_type);
            Task::none()
        }
        OrganizationMessage::NewUnitParentSelected(parent) => {
            state.new_unit_parent = Some(parent);
            Task::none()
        }
        OrganizationMessage::NewUnitNatsAccountChanged(account) => {
            state.new_unit_nats_account = account;
            Task::none()
        }
        OrganizationMessage::NewUnitResponsiblePersonSelected(person_id) => {
            state.new_unit_responsible_person = Some(person_id);
            Task::none()
        }
        OrganizationMessage::CreateOrganizationUnit => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        OrganizationMessage::OrganizationUnitCreated(_) => {
            // Reset form on success
            state.new_unit_name.clear();
            state.new_unit_type = None;
            state.new_unit_parent = None;
            state.new_unit_nats_account.clear();
            state.new_unit_responsible_person = None;
            Task::none()
        }
        OrganizationMessage::RemoveOrganizationUnit(_) => {
            // Handled by main update - needs aggregate access
            Task::none()
        }

        // ========================================================================
        // Service Account Operations
        // ========================================================================
        OrganizationMessage::ToggleServiceAccountSection => {
            state.service_account_section_collapsed = !state.service_account_section_collapsed;
            Task::none()
        }
        OrganizationMessage::NewServiceAccountNameChanged(name) => {
            state.new_service_account_name = name;
            Task::none()
        }
        OrganizationMessage::NewServiceAccountPurposeChanged(purpose) => {
            state.new_service_account_purpose = purpose;
            Task::none()
        }
        OrganizationMessage::ServiceAccountOwningUnitSelected(unit_id) => {
            state.new_service_account_owning_unit = Some(unit_id);
            Task::none()
        }
        OrganizationMessage::ServiceAccountResponsiblePersonSelected(person_id) => {
            state.new_service_account_responsible_person = Some(person_id);
            Task::none()
        }
        OrganizationMessage::CreateServiceAccount => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        OrganizationMessage::ServiceAccountCreated(_) => {
            // Reset form on success
            state.new_service_account_name.clear();
            state.new_service_account_purpose.clear();
            state.new_service_account_owning_unit = None;
            state.new_service_account_responsible_person = None;
            Task::none()
        }
        OrganizationMessage::DeactivateServiceAccount(_) => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        OrganizationMessage::RemoveServiceAccount(_) => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        OrganizationMessage::GenerateServiceAccountKey { .. } => {
            // Handled by main update - needs aggregate access
            Task::none()
        }
        OrganizationMessage::ServiceAccountKeyGenerated(_) => {
            // Status update handled by main update
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_state_default() {
        let state = OrganizationState::default();
        assert!(!state.domain_loaded);
        assert!(state.organization_name.is_empty());
        assert!(state.organization_domain.is_empty());
        assert!(state.organization_id.is_none());
    }

    #[test]
    fn test_organization_state_builder() {
        let state = OrganizationState::new()
            .with_organization_name("CowboyAI".to_string())
            .with_organization_domain("cowboyai.dev".to_string())
            .with_domain_loaded(true);

        assert_eq!(state.organization_name, "CowboyAI");
        assert_eq!(state.organization_domain, "cowboyai.dev");
        assert!(state.domain_loaded);
    }

    #[test]
    fn test_name_changed_updates_state() {
        let mut state = OrganizationState::default();
        let _ = update(
            &mut state,
            OrganizationMessage::NameChanged("NewOrg".to_string()),
        );
        assert_eq!(state.organization_name, "NewOrg");
    }

    #[test]
    fn test_toggle_org_unit_section() {
        let mut state = OrganizationState::default();
        assert!(!state.org_unit_section_collapsed);

        let _ = update(&mut state, OrganizationMessage::ToggleOrgUnitSection);
        assert!(state.org_unit_section_collapsed);

        let _ = update(&mut state, OrganizationMessage::ToggleOrgUnitSection);
        assert!(!state.org_unit_section_collapsed);
    }

    #[test]
    fn test_inline_edit_cancel_clears_state() {
        let mut state = OrganizationState::default();
        state.inline_edit_name = "Editing...".to_string();
        state.editing_new_node = Some(Uuid::nil());

        let _ = update(&mut state, OrganizationMessage::InlineEditCancel);

        assert!(state.inline_edit_name.is_empty());
        assert!(state.editing_new_node.is_none());
    }

    #[test]
    fn test_toggle_service_account_section() {
        let mut state = OrganizationState::default();
        assert!(!state.service_account_section_collapsed);

        let _ = update(&mut state, OrganizationMessage::ToggleServiceAccountSection);
        assert!(state.service_account_section_collapsed);

        let _ = update(&mut state, OrganizationMessage::ToggleServiceAccountSection);
        assert!(!state.service_account_section_collapsed);
    }
}
