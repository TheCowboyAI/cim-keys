// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Message Definitions
//!
//! This module defines the message types for the Organization bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **Domain Operations**: Create, load, import
//! 2. **Organization Form**: Name, domain, passphrase
//! 3. **People**: Add, remove, select persons
//! 4. **Locations**: Physical/virtual location management
//! 5. **Organization Units**: Department/team hierarchy
//! 6. **Service Accounts**: Non-human identity management

use uuid::Uuid;

use crate::domain::{KeyOwnerRole, LocationType, OrganizationUnit, OrganizationUnitType, ServiceAccount};
use crate::projections::{CertificateEntry, KeyEntry, LocationEntry, OrganizationInfo, PersonEntry};
use crate::secrets_loader::BootstrapData;

use super::super::BootstrapConfig;

/// Organization domain messages
#[derive(Debug, Clone)]
pub enum OrganizationMessage {
    // === Domain Operations ===
    CreateNewDomain,
    LoadExistingDomain,
    ImportFromSecrets,
    DomainCreated(Result<String, String>),
    DomainLoaded(Result<BootstrapConfig, String>),
    SecretsImported(Result<BootstrapData, String>),
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

    // === Organization Form Inputs ===
    NameChanged(String),
    DomainChanged(String),
    MasterPassphraseChanged(String),
    MasterPassphraseConfirmChanged(String),

    // === People Operations ===
    NewPersonNameChanged(String),
    NewPersonEmailChanged(String),
    NewPersonRoleSelected(KeyOwnerRole),
    AddPerson,
    RemovePerson(Uuid),
    SelectPerson(Uuid),

    // === Inline Editing ===
    NodeTypeSelected(String),
    InlineEditNameChanged(String),
    InlineEditSubmit,
    InlineEditCancel,

    // === Location Operations ===
    NewLocationNameChanged(String),
    NewLocationTypeSelected(LocationType),
    NewLocationStreetChanged(String),
    NewLocationCityChanged(String),
    NewLocationRegionChanged(String),
    NewLocationCountryChanged(String),
    NewLocationPostalChanged(String),
    NewLocationUrlChanged(String),
    AddLocation,
    RemoveLocation(Uuid),

    // === Organization Unit Operations ===
    ToggleOrgUnitSection,
    NewUnitNameChanged(String),
    NewUnitTypeSelected(OrganizationUnitType),
    NewUnitParentSelected(String),
    NewUnitNatsAccountChanged(String),
    NewUnitResponsiblePersonSelected(Uuid),
    CreateOrganizationUnit,
    OrganizationUnitCreated(Result<OrganizationUnit, String>),
    RemoveOrganizationUnit(Uuid),

    // === Service Account Operations ===
    ToggleServiceAccountSection,
    NewServiceAccountNameChanged(String),
    NewServiceAccountPurposeChanged(String),
    ServiceAccountOwningUnitSelected(Uuid),
    ServiceAccountResponsiblePersonSelected(Uuid),
    CreateServiceAccount,
    ServiceAccountCreated(Result<ServiceAccount, String>),
    DeactivateServiceAccount(Uuid),
    RemoveServiceAccount(Uuid),
    GenerateServiceAccountKey { service_account_id: Uuid },
    ServiceAccountKeyGenerated(Result<(Uuid, KeyOwnerRole), String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_message_variants() {
        let _ = OrganizationMessage::CreateNewDomain;
        let _ = OrganizationMessage::LoadExistingDomain;
        let _ = OrganizationMessage::ImportFromSecrets;
        let _ = OrganizationMessage::NameChanged("Org".to_string());
        let _ = OrganizationMessage::DomainChanged("org.com".to_string());
        let _ = OrganizationMessage::MasterPassphraseChanged("pass".to_string());
        let _ = OrganizationMessage::NewPersonNameChanged("John".to_string());
        let _ = OrganizationMessage::NewPersonEmailChanged("john@org.com".to_string());
        let _ = OrganizationMessage::AddPerson;
        let _ = OrganizationMessage::RemovePerson(Uuid::nil());
        let _ = OrganizationMessage::SelectPerson(Uuid::nil());
        let _ = OrganizationMessage::NodeTypeSelected("Person".to_string());
        let _ = OrganizationMessage::InlineEditNameChanged("New Name".to_string());
        let _ = OrganizationMessage::InlineEditSubmit;
        let _ = OrganizationMessage::InlineEditCancel;
        let _ = OrganizationMessage::NewLocationNameChanged("HQ".to_string());
        let _ = OrganizationMessage::AddLocation;
        let _ = OrganizationMessage::RemoveLocation(Uuid::nil());
        let _ = OrganizationMessage::ToggleOrgUnitSection;
        let _ = OrganizationMessage::NewUnitNameChanged("Engineering".to_string());
        let _ = OrganizationMessage::CreateOrganizationUnit;
        let _ = OrganizationMessage::RemoveOrganizationUnit(Uuid::nil());
        let _ = OrganizationMessage::ToggleServiceAccountSection;
        let _ = OrganizationMessage::NewServiceAccountNameChanged("svc-api".to_string());
        let _ = OrganizationMessage::CreateServiceAccount;
        let _ = OrganizationMessage::DeactivateServiceAccount(Uuid::nil());
        let _ = OrganizationMessage::RemoveServiceAccount(Uuid::nil());
        let _ = OrganizationMessage::GenerateServiceAccountKey { service_account_id: Uuid::nil() };
    }
}
