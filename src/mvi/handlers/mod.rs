// Copyright (c) 2025 - Cowboy AI, LLC.

//! Handler Modules for Compositional Intent Routing
//!
//! This module organizes handlers by Intent category for FRP Axiom A6 compliance.
//! Instead of a massive match statement, handlers are composed via subject routing.
//!
//! ## Handler Categories
//!
//! - `ui` - UI-originated intents (user interactions)
//! - `domain` - Domain event intents (business logic results)
//! - `port` - Port/adapter intents (external system responses)
//! - `system` - System-level intents (OS, clipboard, etc.)
//! - `error` - Error handling intents
//! - `graph` - Graph manipulation intents
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cim_keys::mvi::handlers::{ui, domain, port};
//! use cim_keys::routing::{HierarchicalRouter, IntentCategory};
//!
//! let router = HierarchicalRouter::new()
//!     .category(IntentCategory::Ui, ui::router())
//!     .category(IntentCategory::Domain, domain::router())
//!     .category(IntentCategory::Port, port::router());
//! ```

pub mod ui;
pub mod domain;
pub mod port;
pub mod system;
pub mod error;
pub mod graph;
pub mod router;

// Re-export handler result type
pub use super::{Intent, Model};

// Re-export router function
pub use router::route_intent;

use iced::Task;
use std::sync::Arc;
use crate::ports::{StoragePort, X509Port, SshKeyPort, YubiKeyPort};

/// Ports context for handlers that need to create async commands
#[derive(Clone)]
pub struct Ports {
    pub storage: Arc<dyn StoragePort>,
    pub x509: Arc<dyn X509Port>,
    pub ssh: Arc<dyn SshKeyPort>,
    pub yubikey: Arc<dyn YubiKeyPort>,
}

/// Handler result type: (updated_model, optional_command)
pub type HandlerResult = (Model, Task<Intent>);

/// A pure handler function signature
pub type Handler = fn(Model, Intent, &Ports) -> HandlerResult;
