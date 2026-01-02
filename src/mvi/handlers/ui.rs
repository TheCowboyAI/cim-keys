// Copyright (c) 2025 - Cowboy AI, LLC.

//! UI Intent Handlers
//!
//! Pure handlers for UI-originated intents. These handlers process user
//! interactions and produce model updates + commands.
//!
//! ## Subject Patterns
//!
//! - `ui.tab.selected` - Tab selection
//! - `ui.domain.create` - Domain creation
//! - `ui.domain.load` - Domain loading
//! - `ui.organization.*` - Organization field changes
//! - `ui.person.*` - Person CRUD operations
//! - `ui.passphrase.*` - Passphrase input
//! - `ui.key.generate.*` - Key generation triggers
//! - `ui.export.*` - Export operations
//! - `ui.yubikey.*` - YubiKey operations

use super::{Intent, Model, HandlerResult, Ports};
use crate::mvi::model::{DomainStatus, ExportStatus, PersonInput, Tab};
use iced::Task;

/// Handle tab selection
pub fn handle_tab_selected(model: Model, tab: Tab) -> HandlerResult {
    let updated = model.with_tab(tab);
    (updated, Task::none())
}

/// Handle create domain button click
pub fn handle_create_domain(model: Model) -> HandlerResult {
    let org_name = model.organization_name.clone();
    let org_id = model.organization_id.clone();

    let updated = model
        .with_domain_status(DomainStatus::Creating)
        .with_status_message("Creating domain...".to_string());

    let command = Task::perform(
        async move {
            Intent::DomainCreated {
                organization_id: org_id,
                organization_name: org_name,
            }
        },
        |intent| intent,
    );

    (updated, command)
}

/// Handle load domain button click
pub fn handle_load_domain(model: Model, path: std::path::PathBuf, ports: &Ports) -> HandlerResult {
    let updated = model
        .with_domain_status(DomainStatus::Creating)
        .with_status_message(format!("Loading domain from {:?}...", path));

    let storage_clone = ports.storage.clone();
    let command = Task::perform(
        async move {
            match storage_clone.read("domain.json").await {
                Ok(_data) => {
                    Intent::DomainCreated {
                        organization_id: "loaded".to_string(),
                        organization_name: "Loaded Org".to_string(),
                    }
                }
                Err(e) => Intent::ErrorOccurred {
                    context: "Domain load".to_string(),
                    message: e.to_string(),
                },
            }
        },
        |intent| intent,
    );

    (updated, command)
}

/// Handle organization name change
pub fn handle_organization_name_changed(model: Model, name: String) -> HandlerResult {
    let updated = model.with_organization_name(name);
    (updated, Task::none())
}

/// Handle organization ID change
pub fn handle_organization_id_changed(model: Model, id: String) -> HandlerResult {
    let updated = model.with_organization_id(id);
    (updated, Task::none())
}

/// Handle add person click
pub fn handle_add_person(model: Model) -> HandlerResult {
    let person = PersonInput::new();
    let updated = model.with_person_added(person);
    (updated, Task::none())
}

/// Handle person name change
pub fn handle_person_name_changed(model: Model, index: usize, name: String) -> HandlerResult {
    let updated = model.with_person_name_updated(index, name);
    (updated, Task::none())
}

/// Handle person email change
pub fn handle_person_email_changed(model: Model, index: usize, email: String) -> HandlerResult {
    let updated = model.with_person_email_updated(index, email);
    (updated, Task::none())
}

/// Handle remove person click
pub fn handle_remove_person(model: Model, index: usize) -> HandlerResult {
    let updated = model.with_person_removed(index);
    (updated, Task::none())
}

/// Handle passphrase change
pub fn handle_passphrase_changed(model: Model, passphrase: String) -> HandlerResult {
    use crate::crypto::passphrase::validate_passphrase;

    let validation = validate_passphrase(&passphrase);
    let updated = model
        .with_passphrase(passphrase)
        .with_passphrase_strength(Some(validation.strength));

    (updated, Task::none())
}

/// Handle passphrase confirmation change
pub fn handle_passphrase_confirm_changed(model: Model, confirmed: String) -> HandlerResult {
    let updated = model.with_passphrase_confirmed(confirmed);
    (updated, Task::none())
}

/// Handle derive master seed click
pub fn handle_derive_master_seed(model: Model) -> HandlerResult {
    use crate::crypto::passphrase::validate_passphrase;
    use crate::crypto::seed_derivation::derive_master_seed;

    let passphrase = model.passphrase.clone();
    let passphrase_confirmed = model.passphrase_confirmed.clone();
    let org_id = model.organization_id.clone();

    // Validate passphrases match
    if passphrase != passphrase_confirmed {
        let updated = model
            .with_error(Some("Passphrases do not match".to_string()))
            .with_status_message("Error: Passphrases do not match".to_string());
        return (updated, Task::none());
    }

    // Validate passphrase is not empty
    if passphrase.is_empty() {
        let updated = model
            .with_error(Some("Passphrase cannot be empty".to_string()))
            .with_status_message("Error: Passphrase required".to_string());
        return (updated, Task::none());
    }

    // Check strength is acceptable
    let validation = validate_passphrase(&passphrase);
    if !validation.strength.is_acceptable() {
        let updated = model
            .with_error(Some(format!("Passphrase too weak: {}", validation.strength.description())))
            .with_status_message("Error: Passphrase too weak".to_string());
        return (updated, Task::none());
    }

    let updated = model
        .with_status_message("Deriving master seed...".to_string())
        .with_error(None);

    let command = Task::perform(
        async move {
            match derive_master_seed(&passphrase, &org_id) {
                Ok(seed) => Intent::MasterSeedDerived {
                    organization_id: org_id,
                    entropy_bits: validation.entropy_bits,
                    seed,
                },
                Err(e) => Intent::MasterSeedDerivationFailed { error: e },
            }
        },
        |intent| intent,
    );

    (updated, command)
}

/// Handle generate root CA click
pub fn handle_generate_root_ca(model: Model) -> HandlerResult {
    use crate::crypto::{generate_root_ca, RootCAParams};

    let org_name = model.organization_name.clone();

    let master_seed = match &model.master_seed {
        Some(seed) => seed.clone(),
        None => {
            let updated = model
                .with_error(Some("Please derive master seed first".to_string()))
                .with_status_message("Error: Master seed not available".to_string());
            return (updated, Task::none());
        }
    };

    let updated = model
        .with_status_message("Generating Root CA from master seed...".to_string())
        .with_key_progress(0.1)
        .with_error(None);

    let command = Task::perform(
        async move {
            let root_ca_seed = master_seed.derive_child("root-ca");

            let params = RootCAParams {
                organization: org_name.clone(),
                common_name: format!("{} Root CA", org_name),
                country: Some("US".to_string()),
                state: None,
                locality: None,
                validity_years: 20,
                pathlen: 1,
            };

            let correlation_id = uuid::Uuid::now_v7();
            match generate_root_ca(&root_ca_seed, params, correlation_id, None) {
                Ok((cert, _event)) => Intent::PortX509RootCAGenerated {
                    certificate_pem: cert.certificate_pem,
                    private_key_pem: cert.private_key_pem,
                    fingerprint: cert.fingerprint,
                },
                Err(e) => Intent::PortX509GenerationFailed {
                    error: e.to_string(),
                },
            }
        },
        |intent| intent,
    );

    (updated, command)
}

/// Handle generate intermediate CA click
pub fn handle_generate_intermediate_ca(model: Model, name: String) -> HandlerResult {
    use crate::crypto::{generate_intermediate_ca, IntermediateCAParams};

    let master_seed = match &model.master_seed {
        Some(seed) => seed.clone(),
        None => {
            let updated = model
                .with_error(Some("Please derive master seed first".to_string()))
                .with_status_message("Error: Master seed not available".to_string());
            return (updated, Task::none());
        }
    };

    let root_ca_cert = match &model.key_generation_status.root_ca_certificate_pem {
        Some(cert) => cert.clone(),
        None => {
            let updated = model
                .with_error(Some("Please generate Root CA first".to_string()))
                .with_status_message("Error: Root CA not available".to_string());
            return (updated, Task::none());
        }
    };

    let root_ca_key = match &model.key_generation_status.root_ca_private_key_pem {
        Some(key) => key.clone(),
        None => {
            let updated = model
                .with_error(Some("Root CA private key not available".to_string()))
                .with_status_message("Error: Root CA private key not available".to_string());
            return (updated, Task::none());
        }
    };

    let name_clone = name.clone();
    let org_name = model.organization_name.clone();

    let updated = model
        .with_status_message(format!("Generating intermediate CA '{}'...", name))
        .with_error(None);

    let command = Task::perform(
        async move {
            let intermediate_seed = master_seed.derive_child(&format!("intermediate-{}", name_clone));

            let params = IntermediateCAParams {
                organization: org_name.clone(),
                organizational_unit: name_clone.clone(),
                common_name: format!("{} Intermediate CA", name_clone),
                country: Some("US".to_string()),
                validity_years: 10,
                pathlen: 0,
            };

            let correlation_id = uuid::Uuid::now_v7();
            let root_ca_id = uuid::Uuid::now_v7();
            match generate_intermediate_ca(&intermediate_seed, params, &root_ca_cert, &root_ca_key, root_ca_id, correlation_id, None) {
                Ok((cert, _gen_event, _sign_event)) => Intent::PortX509IntermediateCAGenerated {
                    name: name_clone,
                    certificate_pem: cert.certificate_pem,
                    private_key_pem: cert.private_key_pem,
                    fingerprint: cert.fingerprint,
                },
                Err(e) => Intent::PortX509GenerationFailed {
                    error: format!("Intermediate CA generation failed: {}", e),
                },
            }
        },
        |intent| intent,
    );

    (updated, command)
}

/// Handle generate server certificate click
pub fn handle_generate_server_cert(
    model: Model,
    common_name: String,
    san_entries: Vec<String>,
    intermediate_ca_name: String,
) -> HandlerResult {
    use crate::crypto::{generate_server_certificate, ServerCertParams};

    let master_seed = match &model.master_seed {
        Some(seed) => seed.clone(),
        None => {
            let updated = model
                .with_error(Some("Please derive master seed first".to_string()))
                .with_status_message("Error: Master seed not available".to_string());
            return (updated, Task::none());
        }
    };

    let intermediate_ca = model.key_generation_status.intermediate_cas
        .iter()
        .find(|ca| ca.name == intermediate_ca_name)
        .cloned();

    let intermediate_ca = match intermediate_ca {
        Some(ca) => ca,
        None => {
            let updated = model
                .with_error(Some(format!("Intermediate CA '{}' not found", intermediate_ca_name)))
                .with_status_message("Error: Intermediate CA not found".to_string());
            return (updated, Task::none());
        }
    };

    let common_name_clone = common_name.clone();
    let san_entries_clone = san_entries.clone();
    let intermediate_ca_name_clone = intermediate_ca_name.clone();
    let org_name = model.organization_name.clone();
    let intermediate_cert_pem = intermediate_ca.certificate_pem.clone();
    let intermediate_key_pem = intermediate_ca.private_key_pem.clone();

    let updated = model
        .with_status_message(format!("Generating server certificate for '{}'...", common_name))
        .with_error(None);

    let command = Task::perform(
        async move {
            let server_seed = master_seed.derive_child(&format!("server-{}", common_name_clone));

            let params = ServerCertParams {
                common_name: common_name_clone.clone(),
                san_entries: san_entries_clone.clone(),
                organization: org_name.clone(),
                organizational_unit: Some(intermediate_ca_name_clone.clone()),
                validity_days: 365,
            };

            let correlation_id = uuid::Uuid::now_v7();
            let intermediate_ca_id = uuid::Uuid::now_v7();
            match generate_server_certificate(&server_seed, params, &intermediate_cert_pem, &intermediate_key_pem, intermediate_ca_id, correlation_id, None) {
                Ok((cert, _gen_event, _sign_event)) => Intent::PortX509ServerCertGenerated {
                    common_name: common_name_clone,
                    certificate_pem: cert.certificate_pem,
                    private_key_pem: cert.private_key_pem,
                    fingerprint: cert.fingerprint,
                    signed_by: intermediate_ca_name_clone,
                },
                Err(e) => Intent::PortX509GenerationFailed {
                    error: format!("Server certificate generation failed: {}", e),
                },
            }
        },
        |intent| intent,
    );

    (updated, command)
}

/// Handle generate SSH keys click
pub fn handle_generate_ssh_keys(model: Model, ports: &Ports) -> HandlerResult {
    use crate::ports::ssh::SshKeyType;

    let people = model.people.clone();
    let ssh_clone = ports.ssh.clone();

    let updated = model
        .with_status_message("Generating SSH keys for all users...".to_string())
        .with_key_progress(0.3);

    let command = Task::perform(
        async move {
            let mut intents = Vec::new();

            for person in people {
                match ssh_clone.generate_keypair(
                    SshKeyType::Ed25519,
                    None,
                    Some(person.email.clone()),
                ).await {
                    Ok(keypair) => {
                        intents.push(Intent::PortSSHKeypairGenerated {
                            person_id: person.id.clone(),
                            public_key: hex::encode(&keypair.public_key.data),
                            fingerprint: format!("SHA256:{}", hex::encode(&keypair.public_key.data)),
                        });
                    }
                    Err(e) => {
                        intents.push(Intent::PortSSHGenerationFailed {
                            person_id: person.id.clone(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            intents.into_iter().next().unwrap_or(Intent::NoOp)
        },
        |intent| intent,
    );

    (updated, command)
}

/// Handle generate all keys click
pub fn handle_generate_all_keys(model: Model) -> HandlerResult {
    let updated = model
        .with_status_message("Generating all cryptographic keys...".to_string())
        .with_key_progress(0.0);

    (updated, Task::perform(async { Intent::UiGenerateRootCAClicked }, |i| i))
}

/// Handle export click
pub fn handle_export(model: Model, output_path: std::path::PathBuf, ports: &Ports) -> HandlerResult {
    let updated = model
        .with_status_message("Exporting to SD card...".to_string())
        .with_export_status(ExportStatus::InProgress);

    let storage_clone = ports.storage.clone();
    let command = Task::perform(
        async move {
            match storage_clone.write("manifest.json", b"{}").await {
                Ok(_) => Intent::PortStorageWriteCompleted {
                    path: output_path.to_string_lossy().to_string(),
                    bytes_written: 2,
                },
                Err(e) => Intent::PortStorageWriteFailed {
                    path: output_path.to_string_lossy().to_string(),
                    error: e.to_string(),
                },
            }
        },
        |intent| intent,
    );

    (updated, command)
}

/// Handle provision YubiKey click
pub fn handle_provision_yubikey(model: Model, person_index: usize, ports: &Ports) -> HandlerResult {
    use crate::ports::yubikey::{KeyAlgorithm, PivSlot, SecureString};

    let yubikey_clone = ports.yubikey.clone();
    let _person = model.people.get(person_index).cloned();

    let updated = model.with_status_message("Provisioning YubiKey...".to_string());

    let command = Task::perform(
        async move {
            match yubikey_clone.list_devices().await {
                Ok(devices) => {
                    if let Some(device) = devices.first() {
                        let pin = SecureString::new("123456");

                        match yubikey_clone.generate_key_in_slot(
                            &device.serial,
                            PivSlot::Authentication,
                            KeyAlgorithm::EccP256,
                            &pin,
                        ).await {
                            Ok(public_key) => Intent::PortYubiKeyKeyGenerated {
                                yubikey_serial: device.serial.clone(),
                                slot: "9A".to_string(),
                                public_key: public_key.data,
                            },
                            Err(e) => Intent::PortYubiKeyOperationFailed {
                                error: e.to_string(),
                            },
                        }
                    } else {
                        Intent::PortYubiKeyOperationFailed {
                            error: "No YubiKey devices found".to_string(),
                        }
                    }
                }
                Err(e) => Intent::PortYubiKeyOperationFailed {
                    error: e.to_string(),
                },
            }
        },
        |intent| intent,
    );

    (updated, command)
}
