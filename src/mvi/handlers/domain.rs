// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Intent Handlers
//!
//! Pure handlers for domain event intents. These handlers process the results
//! of domain operations and update model state accordingly.
//!
//! ## Subject Patterns
//!
//! - `domain.created` - Domain creation completed
//! - `domain.person.*` - Person-related domain events
//! - `domain.key.*` - Key generation domain events
//! - `domain.organization.*` - Organization domain events
//! - `domain.node.*` - Graph node domain events
//! - `domain.edge.*` - Graph edge domain events

use super::{Model, HandlerResult};
use crate::mvi::model::{DomainStatus, Tab};
use iced::Task;

/// Handle domain created event
pub fn handle_domain_created(
    model: Model,
    organization_id: String,
    organization_name: String,
) -> HandlerResult {
    let updated = model
        .with_domain_status(DomainStatus::Created {
            organization_id: organization_id.clone(),
            organization_name: organization_name.clone(),
        })
        .with_status_message(format!("Domain '{}' created successfully", organization_name))
        .with_tab(Tab::Organization);

    (updated, Task::none())
}

/// Handle person added event
pub fn handle_person_added(
    model: Model,
    person_id: String,
    name: String,
    email: String,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Added person: {} ({}) with ID: {}",
        name, email, person_id
    ));
    (updated, Task::none())
}

/// Handle root CA generated event
pub fn handle_root_ca_generated(
    model: Model,
    certificate_id: String,
    subject: String,
) -> HandlerResult {
    let updated = model
        .with_root_ca_generated()
        .with_status_message(format!(
            "Root CA generated: {} (Certificate ID: {})",
            subject, certificate_id
        ))
        .with_key_progress(0.5);

    (updated, Task::none())
}

/// Handle SSH key generated event
pub fn handle_ssh_key_generated(
    model: Model,
    person_id: String,
    key_type: String,
    fingerprint: String,
) -> HandlerResult {
    let updated = model
        .with_ssh_key_generated(person_id.clone())
        .with_status_message(format!(
            "SSH key ({}) generated for {}: {}",
            key_type, person_id, fingerprint
        ));

    (updated, Task::none())
}

/// Handle YubiKey provisioned event
pub fn handle_yubikey_provisioned(
    model: Model,
    person_id: String,
    yubikey_serial: String,
    slot: String,
) -> HandlerResult {
    let updated = model
        .with_yubikey_provisioned(person_id.clone())
        .with_status_message(format!(
            "YubiKey {} slot {} provisioned for {}",
            yubikey_serial, slot, person_id
        ));

    (updated, Task::none())
}

/// Handle master seed derived event
pub fn handle_master_seed_derived(
    model: Model,
    organization_id: String,
    entropy_bits: f64,
    seed: crate::crypto::seed_derivation::MasterSeed,
) -> HandlerResult {
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

/// Handle master seed derivation failed event
pub fn handle_master_seed_derivation_failed(model: Model, error: String) -> HandlerResult {
    let updated = model
        .with_master_seed_derived(false)
        .with_error(Some(error.clone()))
        .with_status_message(format!("Failed to derive master seed: {}", error));

    (updated, Task::none())
}

/// Handle node created event
pub fn handle_node_created(model: Model, node_id: String, node_type: String) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "{} node created: {}",
        node_type, node_id
    ));
    (updated, Task::none())
}

/// Handle edge created event
pub fn handle_edge_created(
    model: Model,
    from: String,
    to: String,
    edge_type: String,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Edge created: {} -> {} ({})",
        from, to, edge_type
    ));
    (updated, Task::none())
}

/// Handle node deleted event
pub fn handle_node_deleted(model: Model, node_id: String) -> HandlerResult {
    let updated = model.with_status_message(format!("Node deleted: {}", node_id));
    (updated, Task::none())
}

/// Handle node updated event
pub fn handle_node_updated(
    model: Model,
    node_id: String,
    properties: std::collections::HashMap<String, String>,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Node updated: {} ({} properties changed)",
        node_id,
        properties.len()
    ));
    (updated, Task::none())
}

/// Handle organization created event
pub fn handle_organization_created(model: Model, org_id: String, name: String) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Organization created: {} (ID: {})",
        name, org_id
    ));
    (updated, Task::none())
}

/// Handle org unit created event
pub fn handle_org_unit_created(
    model: Model,
    unit_id: String,
    name: String,
    parent_id: Option<String>,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Org unit created: {} (ID: {}, Parent: {:?})",
        name, unit_id, parent_id
    ));
    (updated, Task::none())
}

/// Handle location created event
pub fn handle_location_created(
    model: Model,
    location_id: String,
    name: String,
    location_type: String,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Location created: {} ({}, ID: {})",
        name, location_type, location_id
    ));
    (updated, Task::none())
}

/// Handle role created event
pub fn handle_role_created(
    model: Model,
    role_id: String,
    name: String,
    organization_id: String,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Role created: {} (ID: {}, Org: {})",
        name, role_id, organization_id
    ));
    (updated, Task::none())
}

/// Handle policy created event
pub fn handle_policy_created(
    model: Model,
    policy_id: String,
    name: String,
    claims: Vec<String>,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Policy created: {} ({} claims, ID: {})",
        name,
        claims.len(),
        policy_id
    ));
    (updated, Task::none())
}

/// Handle policy bound event
pub fn handle_policy_bound(
    model: Model,
    policy_id: String,
    entity_id: String,
    entity_type: String,
) -> HandlerResult {
    let updated = model.with_status_message(format!(
        "Policy bound: {} -> {} ({})",
        policy_id, entity_id, entity_type
    ));
    (updated, Task::none())
}
