//! Update - Pure State Transition Function
//!
//! **Signature**: `(Model, Intent) → (Model, Command<Intent>)`
//!
//! This is a PURE function with no side effects. All async operations
//! and port interactions are described as Commands, not executed directly.

use super::{Intent, Model};
use super::model::{DomainStatus, ExportStatus, PersonInput, Tab};
use iced::Task;
use std::sync::Arc;

// Import ports for commands
use crate::ports::{StoragePort, X509Port, SshKeyPort, YubiKeyPort};

/// Pure update function: (Model, Intent) → (Model, Command<Intent>)
///
/// **Design Principle**: This function is completely pure.
/// - NO async operations
/// - NO side effects
/// - NO port calls
/// All effects are described in the returned Command.
pub fn update(
    model: Model,
    intent: Intent,
    // Ports passed in for command construction (not called directly!)
    storage: Arc<dyn StoragePort>,
    _x509: Arc<dyn X509Port>,
    ssh: Arc<dyn SshKeyPort>,
    yubikey: Arc<dyn YubiKeyPort>,
) -> (Model, Task<Intent>) {
    match intent {
        // ===== UI Intents =====
        Intent::UiTabSelected(tab) => {
            let updated = model.with_tab(tab);
            (updated, Task::none())
        }

        Intent::UiCreateDomainClicked => {
            // Clone values BEFORE moving model
            let org_name = model.organization_name.clone();
            let org_id = model.organization_id.clone();

            let updated = model
                .with_domain_status(DomainStatus::Creating)
                .with_status_message("Creating domain...".to_string());

            let command = Task::perform(
                async move {
                    // This will be implemented: initialize domain aggregate
                    Intent::DomainCreated {
                        organization_id: org_id,
                        organization_name: org_name,
                    }
                },
                |intent| intent,
            );

            (updated, command)
        }

        Intent::UiLoadDomainClicked { path } => {
            let updated = model
                .with_domain_status(DomainStatus::Creating)
                .with_status_message(format!("Loading domain from {:?}...", path));

            let storage_clone = storage.clone();
            let command = Task::perform(
                async move {
                    match storage_clone.read("domain.json").await {
                        Ok(_data) => {
                            // TODO: Parse domain data from _data
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

        Intent::UiOrganizationNameChanged(name) => {
            let updated = model.with_organization_name(name);
            (updated, Task::none())
        }

        Intent::UiOrganizationIdChanged(id) => {
            let updated = model.with_organization_id(id);
            (updated, Task::none())
        }

        Intent::UiAddPersonClicked => {
            let person = PersonInput::new();
            let updated = model.with_person_added(person);
            (updated, Task::none())
        }

        Intent::UiPersonNameChanged { index, name } => {
            let updated = model.with_person_name_updated(index, name);
            (updated, Task::none())
        }

        Intent::UiPersonEmailChanged { index, email } => {
            let updated = model.with_person_email_updated(index, email);
            (updated, Task::none())
        }

        Intent::UiRemovePersonClicked { index } => {
            let updated = model.with_person_removed(index);
            (updated, Task::none())
        }

        Intent::UiPassphraseChanged(passphrase) => {
            use crate::crypto::passphrase::validate_passphrase;

            // Validate passphrase strength as user types
            let validation = validate_passphrase(&passphrase);
            let updated = model
                .with_passphrase(passphrase)
                .with_passphrase_strength(Some(validation.strength));

            (updated, Task::none())
        }

        Intent::UiPassphraseConfirmChanged(confirmed) => {
            let updated = model.with_passphrase_confirmed(confirmed);
            (updated, Task::none())
        }

        Intent::UiDeriveMasterSeedClicked => {
            use crate::crypto::seed_derivation::derive_master_seed;

            // Clone values BEFORE moving model
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

            // Validate passphrase is strong enough
            if passphrase.is_empty() {
                let updated = model
                    .with_error(Some("Passphrase cannot be empty".to_string()))
                    .with_status_message("Error: Passphrase required".to_string());
                return (updated, Task::none());
            }

            // Check strength is acceptable
            use crate::crypto::passphrase::validate_passphrase;
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
                        Ok(seed) => {
                            Intent::MasterSeedDerived {
                                organization_id: org_id,
                                entropy_bits: validation.entropy_bits,
                                seed,
                            }
                        }
                        Err(e) => Intent::MasterSeedDerivationFailed { error: e },
                    }
                },
                |intent| intent,
            );

            (updated, command)
        }

        Intent::MasterSeedDerived {
            organization_id,
            entropy_bits,
            seed,
        } => {
            let updated = model
                .with_master_seed(seed)
                .with_master_seed_derived(true)
                .with_status_message(format!(
                    "Master seed derived successfully for org {} ({:.1} bits entropy)",
                    organization_id, entropy_bits
                ))
                .with_error(None);

            (updated, Task::none())
        }

        Intent::MasterSeedDerivationFailed { error } => {
            let updated = model
                .with_master_seed_derived(false)
                .with_error(Some(error.clone()))
                .with_status_message(format!("Failed to derive master seed: {}", error));

            (updated, Task::none())
        }

        Intent::UiGenerateRootCAClicked => {
            use crate::crypto::{generate_root_ca, RootCAParams};

            // Clone values BEFORE moving model
            let org_name = model.organization_name.clone();

            // Get the stored master seed
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
                    // Derive root CA seed from stored master seed
                    let root_ca_seed = master_seed.derive_child("root-ca");

                    // Generate root CA certificate
                    let params = RootCAParams {
                        organization: org_name.clone(),
                        common_name: format!("{} Root CA", org_name),
                        country: Some("US".to_string()),
                        state: None,
                        locality: None,
                        validity_years: 20,
                    };

                    match generate_root_ca(&root_ca_seed, params) {
                        Ok(cert) => Intent::PortX509RootCAGenerated {
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

        Intent::UiGenerateIntermediateCAClicked { name } => {
            use crate::crypto::{generate_intermediate_ca, IntermediateCAParams};

            // Get stored master seed
            let master_seed = match &model.master_seed {
                Some(seed) => seed.clone(),
                None => {
                    let updated = model
                        .with_error(Some("Please derive master seed first".to_string()))
                        .with_status_message("Error: Master seed not available".to_string());
                    return (updated, Task::none());
                }
            };

            // Get Root CA certificate and key
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

            // Clone for async
            let name_clone = name.clone();
            let org_name = model.organization_name.clone();

            let updated = model
                .with_status_message(format!("Generating intermediate CA '{}'...", name))
                .with_error(None);

            let command = Task::perform(
                async move {
                    // Derive intermediate seed from master seed
                    let intermediate_seed = master_seed.derive_child(&format!("intermediate-{}", name_clone));

                    // Generate intermediate CA certificate
                    let params = IntermediateCAParams {
                        organization: org_name.clone(),
                        organizational_unit: name_clone.clone(),
                        common_name: format!("{} Intermediate CA", name_clone),
                        country: Some("US".to_string()),
                        validity_years: 10,
                    };

                    match generate_intermediate_ca(&intermediate_seed, params, &root_ca_cert, &root_ca_key) {
                        Ok(cert) => Intent::PortX509IntermediateCAGenerated {
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

        Intent::UiGenerateServerCertClicked {
            common_name,
            san_entries,
            intermediate_ca_name,
        } => {
            use crate::crypto::{generate_server_certificate, ServerCertParams};

            // Get stored master seed
            let master_seed = match &model.master_seed {
                Some(seed) => seed.clone(),
                None => {
                    let updated = model
                        .with_error(Some("Please derive master seed first".to_string()))
                        .with_status_message("Error: Master seed not available".to_string());
                    return (updated, Task::none());
                }
            };

            // Find the specified intermediate CA
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

            // Clone for async
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
                    // Derive server seed from master seed
                    let server_seed = master_seed.derive_child(&format!("server-{}", common_name_clone));

                    // Generate server certificate
                    let params = ServerCertParams {
                        common_name: common_name_clone.clone(),
                        san_entries: san_entries_clone.clone(),
                        organization: org_name.clone(),
                        organizational_unit: Some(intermediate_ca_name_clone.clone()),
                        validity_days: 365, // 1 year
                    };

                    match generate_server_certificate(&server_seed, params, &intermediate_cert_pem, &intermediate_key_pem) {
                        Ok(cert) => Intent::PortX509ServerCertGenerated {
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

        Intent::UiGenerateSSHKeysClicked => {
            use crate::ports::ssh::SshKeyType;

            // Clone values BEFORE moving model
            let people = model.people.clone();
            let ssh_clone = ssh.clone();

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

                    // Return first intent (we'll handle batching later)
                    intents.into_iter().next().unwrap_or(Intent::NoOp)
                },
                |intent| intent,
            );

            (updated, command)
        }

        Intent::UiGenerateAllKeysClicked => {
            let updated = model
                .with_status_message("Generating all cryptographic keys...".to_string())
                .with_key_progress(0.0);

            // This will chain multiple commands
            // For now, just trigger root CA generation
            (updated, Task::perform(async { Intent::UiGenerateRootCAClicked }, |i| i))
        }

        Intent::UiExportClicked { output_path } => {
            let updated = model
                .with_status_message("Exporting to SD card...".to_string())
                .with_export_status(ExportStatus::InProgress);

            let storage_clone = storage.clone();
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

        Intent::UiProvisionYubiKeyClicked { person_index } => {
            use crate::ports::yubikey::{KeyAlgorithm, PivSlot, SecureString};

            // Clone values BEFORE moving model
            let yubikey_clone = yubikey.clone();
            let _person = model.people.get(person_index).cloned();

            let updated = model
                .with_status_message("Provisioning YubiKey...".to_string());

            let command = Task::perform(
                async move {
                    // List devices first
                    match yubikey_clone.list_devices().await {
                        Ok(devices) => {
                            if let Some(device) = devices.first() {
                                let pin = SecureString::new("123456"); // TODO: Prompt user

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

        // ===== Domain Intents =====
        Intent::DomainCreated { organization_id, organization_name } => {
            let updated = model
                .with_domain_status(DomainStatus::Created {
                    organization_id: organization_id.clone(),
                    organization_name: organization_name.clone(),
                })
                .with_status_message(format!("Domain '{}' created successfully", organization_name))
                .with_tab(Tab::Organization);

            (updated, Task::none())
        }

        Intent::PersonAdded { person_id, name, email } => {
            let updated = model.with_status_message(format!(
                "Added person: {} ({}) with ID: {}",
                name, email, person_id
            ));
            (updated, Task::none())
        }

        Intent::RootCAGenerated { certificate_id, subject } => {
            let updated = model
                .with_root_ca_generated()
                .with_status_message(format!(
                    "Root CA generated: {} (Certificate ID: {})",
                    subject, certificate_id
                ))
                .with_key_progress(0.5);

            (updated, Task::none())
        }

        Intent::SSHKeyGenerated { person_id, key_type, fingerprint } => {
            let updated = model
                .with_ssh_key_generated(person_id.clone())
                .with_status_message(format!(
                    "SSH key ({}) generated for {}: {}",
                    key_type, person_id, fingerprint
                ));

            (updated, Task::none())
        }

        Intent::YubiKeyProvisioned { person_id, yubikey_serial, slot } => {
            let updated = model
                .with_yubikey_provisioned(person_id.clone())
                .with_status_message(format!(
                    "YubiKey {} slot {} provisioned for {}",
                    yubikey_serial, slot, person_id
                ));

            (updated, Task::none())
        }

        // ===== Port Intents =====
        Intent::PortStorageWriteCompleted { path, bytes_written } => {
            let updated = model
                .with_export_status(ExportStatus::Completed {
                    path: path.clone().into(),
                    bytes_written,
                })
                .with_status_message(format!("Exported {} bytes to {}", bytes_written, path));

            (updated, Task::none())
        }

        Intent::PortStorageWriteFailed { path, error } => {
            let updated = model
                .with_export_status(ExportStatus::Failed { error: error.clone() })
                .with_error(Some(format!("Export failed to {}: {}", path, error)));

            (updated, Task::none())
        }

        Intent::PortX509RootCAGenerated { certificate_pem, private_key_pem, fingerprint } => {
            let updated = model
                .with_root_ca_certificate(certificate_pem.clone(), private_key_pem.clone(), fingerprint.clone())
                .with_status_message(format!("Root CA generated successfully\nFingerprint: {}", fingerprint))
                .with_key_progress(1.0);

            // TODO: Save certificate and private key via storage port
            (updated, Task::none())
        }

        Intent::PortX509IntermediateCAGenerated {
            name,
            certificate_pem,
            private_key_pem,
            fingerprint,
        } => {
            use crate::mvi::model::IntermediateCACert;

            let intermediate = IntermediateCACert {
                name: name.clone(),
                certificate_pem,
                private_key_pem,
                fingerprint: fingerprint.clone(),
            };

            let updated = model
                .with_intermediate_ca(intermediate)
                .with_status_message(format!("Intermediate CA '{}' generated successfully\nFingerprint: {}", name, fingerprint));

            // TODO: Save certificate and private key via storage port
            (updated, Task::none())
        }

        Intent::PortX509ServerCertGenerated {
            common_name,
            certificate_pem,
            private_key_pem,
            fingerprint,
            signed_by,
        } => {
            use crate::mvi::model::ServerCert;

            let server_cert = ServerCert {
                common_name: common_name.clone(),
                certificate_pem,
                private_key_pem,
                fingerprint: fingerprint.clone(),
                signed_by: signed_by.clone(),
            };

            let updated = model
                .with_server_certificate(server_cert)
                .with_status_message(format!(
                    "Server certificate '{}' generated successfully\nSigned by: {}\nFingerprint: {}",
                    common_name, signed_by, fingerprint
                ));

            // TODO: Save certificate and private key via storage port
            (updated, Task::none())
        }

        Intent::PortX509GenerationFailed { error } => {
            let updated = model
                .with_error(Some(format!("Certificate generation failed: {}", error)))
                .with_key_progress(0.0);

            (updated, Task::none())
        }

        Intent::PortSSHKeypairGenerated { person_id, public_key, fingerprint } => {
            let updated = model
                .with_ssh_key_generated(person_id.clone())
                .with_status_message(format!(
                    "SSH key generated: {} (Public key: {}...)",
                    fingerprint,
                    &public_key[..public_key.len().min(16)]
                ))
                .with_key_progress(0.7);

            (updated, Task::none())
        }

        Intent::PortSSHGenerationFailed { person_id, error } => {
            let updated = model
                .with_error(Some(format!("SSH key generation failed for {}: {}", person_id, error)));

            (updated, Task::none())
        }

        Intent::PortYubiKeyDevicesListed { devices } => {
            let updated = model
                .with_status_message(format!("Found {} YubiKey device(s)", devices.len()));

            (updated, Task::none())
        }

        Intent::PortYubiKeyKeyGenerated { yubikey_serial, slot, public_key } => {
            let updated = model
                .with_status_message(format!(
                    "YubiKey {} slot {} provisioned ({} bytes public key)",
                    yubikey_serial, slot, public_key.len()
                ))
                .with_key_progress(1.0);

            (updated, Task::none())
        }

        Intent::PortYubiKeyOperationFailed { error } => {
            let updated = model
                .with_error(Some(format!("YubiKey operation failed: {}", error)));

            (updated, Task::none())
        }

        // ===== System Intents =====
        Intent::SystemFileSelected(path) => {
            (model, Task::perform(async move { Intent::UiLoadDomainClicked { path } }, |i| i))
        }

        Intent::SystemFilePickerCancelled => {
            let updated = model.with_status_message("File selection cancelled".to_string());
            (updated, Task::none())
        }

        Intent::SystemErrorOccurred { context, error } => {
            let updated = model
                .with_error(Some(format!("{}: {}", context, error)));

            (updated, Task::none())
        }

        Intent::SystemClipboardUpdated(text) => {
            // Log clipboard update for debugging
            let updated = model.with_status_message(format!(
                "Clipboard updated ({} chars)",
                text.len()
            ));
            (updated, Task::none())
        }

        // ===== Error Intents =====
        Intent::ErrorOccurred { context, message } => {
            let updated = model
                .with_error(Some(format!("{}: {}", context, message)));

            (updated, Task::none())
        }

        Intent::ErrorDismissed { error_id } => {
            // Log which error was dismissed
            let updated = model
                .with_error(None)
                .with_status_message(format!("Error dismissed: {}", error_id));
            (updated, Task::none())
        }

        // ===== Graph Intents (Phase 2 - Placeholders for future implementation) =====
        Intent::UiGraphCreateNode { node_type, position } => {
            let updated = model.with_status_message(format!(
                "Creating {:?} node at ({}, {})",
                node_type, position.0, position.1
            ));
            // TODO: Create actual node in graph state
            (updated, Task::none())
        }

        Intent::UiGraphCreateEdgeStarted { from_node } => {
            let updated = model.with_status_message(format!(
                "Edge creation started from node {}",
                from_node
            ));
            // TODO: Start edge creation mode
            (updated, Task::none())
        }

        Intent::UiGraphCreateEdgeCompleted { from, to, edge_type } => {
            let updated = model.with_status_message(format!(
                "Edge created: {} -> {} ({})",
                from, to, edge_type
            ));
            // TODO: Create actual edge in graph state
            (updated, Task::none())
        }

        Intent::UiGraphCreateEdgeCancelled => {
            let updated = model.with_status_message("Edge creation cancelled".to_string());
            // TODO: Cancel edge creation mode
            (updated, Task::none())
        }

        Intent::UiGraphNodeClicked { node_id } => {
            let updated = model.with_status_message(format!("Node selected: {}", node_id));
            // TODO: Select node in graph state
            (updated, Task::none())
        }

        Intent::UiGraphDeleteNode { node_id } => {
            let updated = model.with_status_message(format!("Deleting node: {}", node_id));
            // TODO: Delete node from graph state
            (updated, Task::none())
        }

        Intent::UiGraphDeleteEdge { from, to } => {
            let updated = model.with_status_message(format!(
                "Deleting edge: {} -> {}",
                from, to
            ));
            // TODO: Delete edge from graph state
            (updated, Task::none())
        }

        Intent::UiGraphEditNodeProperties { node_id } => {
            let updated = model.with_status_message(format!(
                "Editing properties for node: {}",
                node_id
            ));
            // TODO: Open property editor
            (updated, Task::none())
        }

        Intent::UiGraphPropertyChanged { node_id, property, value } => {
            let updated = model.with_status_message(format!(
                "Property changed: {}.{} = {}",
                node_id, property, value
            ));
            // TODO: Update property in temporary state (not persisted yet)
            (updated, Task::none())
        }

        Intent::UiGraphPropertiesSaved { node_id } => {
            let updated = model.with_status_message(format!(
                "Properties saved for node: {}",
                node_id
            ));
            // TODO: Persist property changes
            (updated, Task::none())
        }

        Intent::UiGraphPropertiesCancelled => {
            let updated = model.with_status_message("Property editing cancelled".to_string());
            // TODO: Discard property changes
            (updated, Task::none())
        }

        Intent::UiGraphAutoLayout => {
            let updated = model.with_status_message("Auto-layout applied".to_string());
            // TODO: Apply auto-layout algorithm
            (updated, Task::none())
        }

        // Domain events for graph changes
        Intent::DomainNodeCreated { node_id, node_type } => {
            let updated = model.with_status_message(format!(
                "{} node created: {}",
                node_type, node_id
            ));
            (updated, Task::none())
        }

        Intent::DomainEdgeCreated { from, to, edge_type } => {
            let updated = model.with_status_message(format!(
                "Edge created: {} -> {} ({})",
                from, to, edge_type
            ));
            (updated, Task::none())
        }

        Intent::DomainNodeDeleted { node_id } => {
            let updated = model.with_status_message(format!("Node deleted: {}", node_id));
            (updated, Task::none())
        }

        Intent::DomainNodeUpdated { node_id, properties } => {
            let updated = model.with_status_message(format!(
                "Node updated: {} ({} properties changed)",
                node_id,
                properties.len()
            ));
            (updated, Task::none())
        }

        Intent::DomainOrganizationCreated { org_id, name } => {
            let updated = model.with_status_message(format!(
                "Organization created: {} (ID: {})",
                name, org_id
            ));
            (updated, Task::none())
        }

        Intent::DomainOrgUnitCreated { unit_id, name, parent_id } => {
            let updated = model.with_status_message(format!(
                "Org unit created: {} (ID: {}, Parent: {:?})",
                name, unit_id, parent_id
            ));
            (updated, Task::none())
        }

        Intent::DomainLocationCreated { location_id, name, location_type } => {
            let updated = model.with_status_message(format!(
                "Location created: {} ({}, ID: {})",
                name, location_type, location_id
            ));
            (updated, Task::none())
        }

        Intent::DomainRoleCreated { role_id, name, organization_id } => {
            let updated = model.with_status_message(format!(
                "Role created: {} (ID: {}, Org: {})",
                name, role_id, organization_id
            ));
            (updated, Task::none())
        }

        Intent::DomainPolicyCreated { policy_id, name, claims } => {
            let updated = model.with_status_message(format!(
                "Policy created: {} ({} claims, ID: {})",
                name,
                claims.len(),
                policy_id
            ));
            (updated, Task::none())
        }

        Intent::DomainPolicyBound { policy_id, entity_id, entity_type } => {
            let updated = model.with_status_message(format!(
                "Policy bound: {} -> {} ({})",
                policy_id, entity_id, entity_type
            ));
            (updated, Task::none())
        }

        Intent::NoOp => {
            (model, Task::none())
        }
    }
}
