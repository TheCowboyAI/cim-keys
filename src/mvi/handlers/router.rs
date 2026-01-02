// Copyright (c) 2025 - Cowboy AI, LLC.

//! Router Builder for Compositional Intent Dispatch
//!
//! This module provides the `route_intent()` function that dispatches intents
//! to the appropriate handler based on subject patterns.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cim_keys::mvi::handlers::{route_intent, Ports};
//!
//! let ports = Ports { storage, x509, ssh, yubikey };
//! let (new_model, task) = route_intent(model, intent, &ports);
//! ```

use super::{Model, Intent, HandlerResult, Ports};
use super::{ui, domain, port, system, error, graph};
use crate::routing::IntentCategory;
use iced::Task;

/// Route an intent to its handler based on category and subject
///
/// This provides compositional routing without the giant match statement.
/// Each category has its own routing logic, and within each category,
/// subjects determine the specific handler.
pub fn route_intent(model: Model, intent: Intent, ports: &Ports) -> HandlerResult {
    match intent.category() {
        IntentCategory::Ui => route_ui_intent(model, intent, ports),
        IntentCategory::Domain => route_domain_intent(model, intent),
        IntentCategory::Port => route_port_intent(model, intent),
        IntentCategory::System => route_system_intent(model, intent),
        IntentCategory::Error => route_error_intent(model, intent),
    }
}

/// Route UI-category intents
fn route_ui_intent(model: Model, intent: Intent, ports: &Ports) -> HandlerResult {
    match intent {
        // Tab selection
        Intent::UiTabSelected(tab) => ui::handle_tab_selected(model, tab),

        // Domain creation/loading
        Intent::UiCreateDomainClicked => ui::handle_create_domain(model),
        Intent::UiLoadDomainClicked { path } => ui::handle_load_domain(model, path, ports),

        // Organization fields
        Intent::UiOrganizationNameChanged(name) => ui::handle_organization_name_changed(model, name),
        Intent::UiOrganizationIdChanged(id) => ui::handle_organization_id_changed(model, id),

        // Person management
        Intent::UiAddPersonClicked => ui::handle_add_person(model),
        Intent::UiPersonNameChanged { index, name } => ui::handle_person_name_changed(model, index, name),
        Intent::UiPersonEmailChanged { index, email } => ui::handle_person_email_changed(model, index, email),
        Intent::UiRemovePersonClicked { index } => ui::handle_remove_person(model, index),

        // Passphrase handling
        Intent::UiPassphraseChanged(passphrase) => ui::handle_passphrase_changed(model, passphrase),
        Intent::UiPassphraseConfirmChanged(confirmed) => ui::handle_passphrase_confirm_changed(model, confirmed),
        Intent::UiDeriveMasterSeedClicked => ui::handle_derive_master_seed(model),

        // Key generation
        Intent::UiGenerateRootCAClicked => ui::handle_generate_root_ca(model),
        Intent::UiGenerateIntermediateCAClicked { name } => ui::handle_generate_intermediate_ca(model, name),
        Intent::UiGenerateServerCertClicked { common_name, san_entries, intermediate_ca_name } => {
            ui::handle_generate_server_cert(model, common_name, san_entries, intermediate_ca_name)
        }
        Intent::UiGenerateSSHKeysClicked => ui::handle_generate_ssh_keys(model, ports),
        Intent::UiGenerateAllKeysClicked => ui::handle_generate_all_keys(model),

        // Export
        Intent::UiExportClicked { output_path } => ui::handle_export(model, output_path, ports),

        // YubiKey
        Intent::UiProvisionYubiKeyClicked { person_index } => ui::handle_provision_yubikey(model, person_index, ports),

        // Graph operations
        Intent::UiGraphCreateNode { node_type, position } => {
            graph::handle_create_node(model, format!("{:?}", node_type), position)
        }
        Intent::UiGraphCreateEdgeStarted { from_node } => graph::handle_create_edge_started(model, from_node),
        Intent::UiGraphCreateEdgeCompleted { from, to, edge_type } => {
            graph::handle_create_edge_completed(model, from, to, edge_type)
        }
        Intent::UiGraphCreateEdgeCancelled => graph::handle_create_edge_cancelled(model),
        Intent::UiConceptEntityClicked { node_id } => graph::handle_entity_clicked(model, node_id),
        Intent::UiGraphDeleteNode { node_id } => graph::handle_delete_node(model, node_id),
        Intent::UiGraphDeleteEdge { from, to } => graph::handle_delete_edge(model, from, to),
        Intent::UiGraphEditNodeProperties { node_id } => graph::handle_edit_node_properties(model, node_id),
        Intent::UiGraphPropertyChanged { node_id, property, value } => {
            graph::handle_property_changed(model, node_id, property, value)
        }
        Intent::UiGraphPropertiesSaved { node_id } => graph::handle_properties_saved(model, node_id),
        Intent::UiGraphPropertiesCancelled => graph::handle_properties_cancelled(model),
        Intent::UiGraphAutoLayout => graph::handle_auto_layout(model),

        // Fallback - should not reach here for UI intents
        _ => (model, Task::none()),
    }
}

/// Route Domain-category intents
fn route_domain_intent(model: Model, intent: Intent) -> HandlerResult {
    match intent {
        Intent::DomainCreated { organization_id, organization_name } => {
            domain::handle_domain_created(model, organization_id, organization_name)
        }
        Intent::PersonAdded { person_id, name, email } => {
            domain::handle_person_added(model, person_id, name, email)
        }
        Intent::RootCAGenerated { certificate_id, subject } => {
            domain::handle_root_ca_generated(model, certificate_id, subject)
        }
        Intent::SSHKeyGenerated { person_id, key_type, fingerprint } => {
            domain::handle_ssh_key_generated(model, person_id, key_type, fingerprint)
        }
        Intent::YubiKeyProvisioned { person_id, yubikey_serial, slot } => {
            domain::handle_yubikey_provisioned(model, person_id, yubikey_serial, slot)
        }
        Intent::MasterSeedDerived { organization_id, entropy_bits, seed } => {
            domain::handle_master_seed_derived(model, organization_id, entropy_bits, seed)
        }
        Intent::MasterSeedDerivationFailed { error } => {
            domain::handle_master_seed_derivation_failed(model, error)
        }
        Intent::DomainNodeCreated { node_id, node_type } => {
            domain::handle_node_created(model, node_id, node_type)
        }
        Intent::DomainEdgeCreated { from, to, edge_type } => {
            domain::handle_edge_created(model, from, to, edge_type)
        }
        Intent::DomainNodeDeleted { node_id } => domain::handle_node_deleted(model, node_id),
        Intent::DomainNodeUpdated { node_id, properties } => {
            domain::handle_node_updated(model, node_id, properties.into_iter().collect())
        }
        Intent::DomainOrganizationCreated { org_id, name } => {
            domain::handle_organization_created(model, org_id, name)
        }
        Intent::DomainOrgUnitCreated { unit_id, name, parent_id } => {
            domain::handle_org_unit_created(model, unit_id, name, parent_id)
        }
        Intent::DomainLocationCreated { location_id, name, location_type } => {
            domain::handle_location_created(model, location_id, name, location_type)
        }
        Intent::DomainRoleCreated { role_id, name, organization_id } => {
            domain::handle_role_created(model, role_id, name, organization_id)
        }
        Intent::DomainPolicyCreated { policy_id, name, claims } => {
            domain::handle_policy_created(model, policy_id, name, claims)
        }
        Intent::DomainPolicyBound { policy_id, entity_id, entity_type } => {
            domain::handle_policy_bound(model, policy_id, entity_id, entity_type)
        }

        // Fallback
        _ => (model, Task::none()),
    }
}

/// Route Port-category intents
fn route_port_intent(model: Model, intent: Intent) -> HandlerResult {
    match intent {
        // Storage
        Intent::PortStorageWriteCompleted { path, bytes_written } => {
            port::handle_storage_write_completed(model, path, bytes_written)
        }
        Intent::PortStorageWriteFailed { path, error } => {
            port::handle_storage_write_failed(model, path, error)
        }

        // X509
        Intent::PortX509RootCAGenerated { certificate_pem, private_key_pem, fingerprint } => {
            port::handle_x509_root_ca_generated(model, certificate_pem, private_key_pem, fingerprint)
        }
        Intent::PortX509IntermediateCAGenerated { name, certificate_pem, private_key_pem, fingerprint } => {
            port::handle_x509_intermediate_ca_generated(model, name, certificate_pem, private_key_pem, fingerprint)
        }
        Intent::PortX509ServerCertGenerated { common_name, certificate_pem, private_key_pem, fingerprint, signed_by } => {
            port::handle_x509_server_cert_generated(model, common_name, certificate_pem, private_key_pem, fingerprint, signed_by)
        }
        Intent::PortX509GenerationFailed { error } => {
            port::handle_x509_generation_failed(model, error)
        }

        // SSH
        Intent::PortSSHKeypairGenerated { person_id, public_key, fingerprint } => {
            port::handle_ssh_keypair_generated(model, person_id, public_key, fingerprint)
        }
        Intent::PortSSHGenerationFailed { person_id, error } => {
            port::handle_ssh_generation_failed(model, person_id, error)
        }

        // YubiKey
        Intent::PortYubiKeyDevicesListed { devices } => {
            // Convert strings to device structs
            let device_list = devices.into_iter()
                .map(|serial| crate::ports::yubikey::YubiKeyDevice {
                    serial,
                    version: "".to_string(),
                    model: "Unknown".to_string(),
                    piv_enabled: true,
                })
                .collect();
            port::handle_yubikey_devices_listed(model, device_list)
        }
        Intent::PortYubiKeyKeyGenerated { yubikey_serial, slot, public_key } => {
            port::handle_yubikey_key_generated(model, yubikey_serial, slot, public_key)
        }
        Intent::PortYubiKeyOperationFailed { error } => {
            port::handle_yubikey_operation_failed(model, error)
        }

        // Domain loading
        Intent::PortDomainLoaded { organization_name, organization_id, people_count, locations_count } => {
            port::handle_domain_loaded(model, organization_name, organization_id, people_count, locations_count)
        }
        Intent::PortSecretsLoaded { organization_name, people_count, yubikey_count } => {
            port::handle_secrets_loaded(model, organization_name, people_count, yubikey_count)
        }
        Intent::PortDomainExported { path, bytes_written } => {
            port::handle_domain_exported(model, path, bytes_written)
        }
        Intent::PortDomainExportFailed { path, error } => {
            port::handle_domain_export_failed(model, path, error)
        }

        // NATS
        Intent::PortNatsHierarchyGenerated { operator_name, account_count, user_count } => {
            port::handle_nats_hierarchy_generated(model, operator_name, account_count, user_count)
        }
        Intent::PortNatsHierarchyFailed { error } => {
            port::handle_nats_hierarchy_failed(model, error)
        }

        // Policy
        Intent::PortPolicyLoaded { role_count, assignment_count } => {
            port::handle_policy_loaded(model, role_count, assignment_count)
        }
        Intent::PortPolicyLoadFailed { error } => {
            port::handle_policy_load_failed(model, error)
        }

        // Fallback
        _ => (model, Task::none()),
    }
}

/// Route System-category intents
fn route_system_intent(model: Model, intent: Intent) -> HandlerResult {
    match intent {
        Intent::SystemFileSelected(path) => system::handle_file_selected(model, path),
        Intent::SystemFilePickerCancelled => system::handle_file_picker_cancelled(model),
        Intent::SystemErrorOccurred { context, error } => {
            system::handle_system_error(model, context, error)
        }
        Intent::SystemClipboardUpdated(text) => system::handle_clipboard_updated(model, text),
        Intent::NoOp => (model, Task::none()),

        // Fallback
        _ => (model, Task::none()),
    }
}

/// Route Error-category intents
fn route_error_intent(model: Model, intent: Intent) -> HandlerResult {
    match intent {
        Intent::ErrorOccurred { context, message } => {
            error::handle_error_occurred(model, context, message)
        }
        Intent::ErrorDismissed { error_id } => error::handle_error_dismissed(model, error_id),

        // Fallback
        _ => (model, Task::none()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mvi::model::Tab;

    #[test]
    fn test_intent_to_subject() {
        assert_eq!(Intent::UiTabSelected(Tab::Welcome).to_subject(), "ui.tab.selected");
        assert_eq!(Intent::UiCreateDomainClicked.to_subject(), "ui.domain.create");
        assert_eq!(Intent::UiGenerateRootCAClicked.to_subject(), "ui.key.rootca.generate");
        assert_eq!(Intent::NoOp.to_subject(), "system.noop");
    }

    #[test]
    fn test_intent_category() {
        assert_eq!(Intent::UiTabSelected(Tab::Welcome).category(), IntentCategory::Ui);
        assert_eq!(Intent::DomainCreated {
            organization_id: "test".into(),
            organization_name: "Test".into(),
        }.category(), IntentCategory::Domain);
        assert_eq!(Intent::PortX509RootCAGenerated {
            certificate_pem: "".into(),
            private_key_pem: "".into(),
            fingerprint: "".into(),
        }.category(), IntentCategory::Port);
        assert_eq!(Intent::ErrorOccurred {
            context: "".into(),
            message: "".into(),
        }.category(), IntentCategory::Error);
        assert_eq!(Intent::SystemFilePickerCancelled.category(), IntentCategory::System);
    }
}
