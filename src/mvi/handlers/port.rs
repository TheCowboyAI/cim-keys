// Copyright (c) 2025 - Cowboy AI, LLC.

//! Port Intent Handlers
//!
//! Pure handlers for port/adapter response intents. These handlers process
//! responses from external systems (storage, X509, SSH, YubiKey, NATS).
//!
//! ## Subject Patterns
//!
//! - `port.storage.*` - Storage port responses
//! - `port.x509.*` - X509 certificate port responses
//! - `port.ssh.*` - SSH key port responses
//! - `port.yubikey.*` - YubiKey port responses
//! - `port.nats.*` - NATS port responses
//! - `port.domain.*` - Domain loading responses
//! - `port.policy.*` - Policy loading responses

use super::{Model, HandlerResult};
use crate::mvi::model::{ExportStatus, IntermediateCACert, ServerCert};
use iced::Task;

// ===== Storage Port Handlers =====

/// Handle storage write completed
pub fn handle_storage_write_completed(
    model: Model,
    path: String,
    bytes_written: usize,
) -> HandlerResult {
    let updated = model
        .with_export_status(ExportStatus::Completed {
            path: path.clone().into(),
            bytes_written,
        })
        .with_status_message(format!("Exported {} bytes to {}", bytes_written, path));

    (updated, Task::none())
}

/// Handle storage write failed
pub fn handle_storage_write_failed(model: Model, path: String, error: String) -> HandlerResult {
    let updated = model
        .with_export_status(ExportStatus::Failed { error: error.clone() })
        .with_error(Some(format!("Export failed to {}: {}", path, error)));

    (updated, Task::none())
}

// ===== X509 Port Handlers =====

/// Handle X509 root CA generated
pub fn handle_x509_root_ca_generated(
    model: Model,
    certificate_pem: String,
    private_key_pem: String,
    fingerprint: String,
) -> HandlerResult {
    let updated = model
        .with_root_ca_certificate(certificate_pem, private_key_pem, fingerprint.clone())
        .with_status_message(format!("Root CA generated successfully\nFingerprint: {}", fingerprint))
        .with_key_progress(1.0);

    (updated, Task::none())
}

/// Handle X509 intermediate CA generated
pub fn handle_x509_intermediate_ca_generated(
    model: Model,
    name: String,
    certificate_pem: String,
    private_key_pem: String,
    fingerprint: String,
) -> HandlerResult {
    let intermediate = IntermediateCACert {
        name: name.clone(),
        certificate_pem,
        private_key_pem,
        fingerprint: fingerprint.clone(),
    };

    let updated = model
        .with_intermediate_ca(intermediate)
        .with_status_message(format!(
            "Intermediate CA '{}' generated successfully\nFingerprint: {}",
            name, fingerprint
        ));

    (updated, Task::none())
}

/// Handle X509 server certificate generated
pub fn handle_x509_server_cert_generated(
    model: Model,
    common_name: String,
    certificate_pem: String,
    private_key_pem: String,
    fingerprint: String,
    signed_by: String,
) -> HandlerResult {
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

    (updated, Task::none())
}

/// Handle X509 generation failed
pub fn handle_x509_generation_failed(model: Model, error: String) -> HandlerResult {
    let updated = model
        .with_error(Some(format!("Certificate generation failed: {}", error)))
        .with_key_progress(0.0);

    (updated, Task::none())
}

// ===== SSH Port Handlers =====

/// Handle SSH keypair generated
pub fn handle_ssh_keypair_generated(
    model: Model,
    person_id: String,
    public_key: String,
    fingerprint: String,
) -> HandlerResult {
    let updated = model
        .with_ssh_key_generated(person_id)
        .with_status_message(format!(
            "SSH key generated: {} (Public key: {}...)",
            fingerprint,
            &public_key[..public_key.len().min(16)]
        ))
        .with_key_progress(0.7);

    (updated, Task::none())
}

/// Handle SSH generation failed
pub fn handle_ssh_generation_failed(model: Model, person_id: String, error: String) -> HandlerResult {
    let updated = model
        .with_error(Some(format!("SSH key generation failed for {}: {}", person_id, error)));

    (updated, Task::none())
}

// ===== YubiKey Port Handlers =====

/// Handle YubiKey devices listed
pub fn handle_yubikey_devices_listed(
    model: Model,
    devices: Vec<crate::ports::yubikey::YubiKeyDevice>,
) -> HandlerResult {
    let updated = model
        .with_status_message(format!("Found {} YubiKey device(s)", devices.len()));

    (updated, Task::none())
}

/// Handle YubiKey key generated
pub fn handle_yubikey_key_generated(
    model: Model,
    yubikey_serial: String,
    slot: String,
    public_key: Vec<u8>,
) -> HandlerResult {
    let updated = model
        .with_status_message(format!(
            "YubiKey {} slot {} provisioned ({} bytes public key)",
            yubikey_serial, slot, public_key.len()
        ))
        .with_key_progress(1.0);

    (updated, Task::none())
}

/// Handle YubiKey operation failed
pub fn handle_yubikey_operation_failed(model: Model, error: String) -> HandlerResult {
    let updated = model
        .with_error(Some(format!("YubiKey operation failed: {}", error)));

    (updated, Task::none())
}

// ===== Domain Loading Port Handlers =====

/// Handle domain loaded
pub fn handle_domain_loaded(
    model: Model,
    organization_name: String,
    organization_id: String,
    people_count: usize,
    locations_count: usize,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Domain loaded: {} (ID: {}, {} people, {} locations)",
        organization_name, organization_id, people_count, locations_count
    ));
    (updated, Task::none())
}

/// Handle secrets loaded
pub fn handle_secrets_loaded(
    model: Model,
    organization_name: String,
    people_count: usize,
    yubikey_count: usize,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Secrets loaded: {} ({} people, {} YubiKeys)",
        organization_name, people_count, yubikey_count
    ));
    (updated, Task::none())
}

/// Handle domain exported
pub fn handle_domain_exported(model: Model, path: String, bytes_written: usize) -> HandlerResult {
    let updated = model
        .with_export_status(ExportStatus::Completed {
            path: path.clone().into(),
            bytes_written,
        })
        .with_status_message(format!(
            "Domain exported to {} ({} bytes)",
            path, bytes_written
        ));
    (updated, Task::none())
}

/// Handle domain export failed
pub fn handle_domain_export_failed(model: Model, path: String, error: String) -> HandlerResult {
    let updated = model
        .with_export_status(ExportStatus::Failed { error: error.clone() })
        .with_error(Some(format!(
            "Domain export failed to {}: {}",
            path, error
        )));
    (updated, Task::none())
}

// ===== NATS Port Handlers =====

/// Handle NATS hierarchy generated
pub fn handle_nats_hierarchy_generated(
    model: Model,
    operator_name: String,
    account_count: usize,
    user_count: usize,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "NATS hierarchy generated: {} ({} accounts, {} users)",
        operator_name, account_count, user_count
    ));
    (updated, Task::none())
}

/// Handle NATS hierarchy failed
pub fn handle_nats_hierarchy_failed(model: Model, error: String) -> HandlerResult {
    let updated = model.with_error(Some(format!(
        "NATS hierarchy generation failed: {}",
        error
    )));
    (updated, Task::none())
}

// ===== Policy Port Handlers =====

/// Handle policy loaded
pub fn handle_policy_loaded(model: Model, role_count: usize, assignment_count: usize) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Policy loaded: {} roles, {} assignments",
        role_count, assignment_count
    ));
    (updated, Task::none())
}

/// Handle policy load failed
pub fn handle_policy_load_failed(model: Model, error: String) -> HandlerResult {
    let updated = model.with_error(Some(format!(
        "Policy load failed: {}",
        error
    )));
    (updated, Task::none())
}
