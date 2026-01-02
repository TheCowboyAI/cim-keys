// Copyright (c) 2025 - Cowboy AI, LLC.

//! Update - Pure State Transition Function
//!
//! **Signature**: `(Model, Intent) → (Model, Command<Intent>)`
//!
//! This module provides a thin delegation to the compositional router.
//! All intent handling logic is now in the `handlers` module, organized
//! by category (ui, domain, port, system, error, graph).
//!
//! ## FRP Axiom A6: Explicit Routing
//!
//! Instead of a 1000+ line match statement, we use compositional routing:
//! - Intents are categorized by `intent.category()`
//! - Each category has its own handler module
//! - The router dispatches to the appropriate handler

use super::{Intent, Model};
use super::handlers::{route_intent, Ports};
use iced::Task;
use std::sync::Arc;

// Import ports for Ports struct construction
use crate::ports::{StoragePort, X509Port, SshKeyPort, YubiKeyPort};

/// Pure update function: (Model, Intent) → (Model, Command<Intent>)
///
/// **Design Principle**: This function is completely pure.
/// - NO async operations
/// - NO side effects
/// - NO port calls
/// All effects are described in the returned Command.
///
/// ## Compositional Routing
///
/// This function delegates to `route_intent()` which provides:
/// - Category-based dispatch (Ui, Domain, Port, System, Error)
/// - Handler-per-intent isolation
/// - Subject-based routing for NATS integration
pub fn update(
    model: Model,
    intent: Intent,
    // Ports passed in for command construction (not called directly!)
    storage: Arc<dyn StoragePort>,
    x509: Arc<dyn X509Port>,
    ssh: Arc<dyn SshKeyPort>,
    yubikey: Arc<dyn YubiKeyPort>,
) -> (Model, Task<Intent>) {
    // Construct Ports context for handlers
    let ports = Ports {
        storage,
        x509,
        ssh,
        yubikey,
    };

    // Delegate to compositional router
    route_intent(model, intent, &ports)
}
