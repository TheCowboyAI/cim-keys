// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain-bounded GUI modules
//!
//! This module organizes the GUI into bounded contexts following DDD principles.
//! Each domain module encapsulates:
//! - Domain-specific message types
//! - Domain state management
//! - Update handlers for domain messages
//! - View functions for domain UI
//!
//! ## Architecture
//!
//! The GUI follows a delegated message pattern:
//! ```text
//! Message::Organization(OrganizationMessage)
//!     → domains::organization::update()
//!     → Task<Message>
//! ```
//!
//! Each domain module implements a consistent API:
//! - `pub enum DomainMessage` - Domain-specific messages
//! - `pub struct DomainState` - Domain-specific state
//! - `pub fn update(state, message) -> Task<Message>` - Pure update function
//! - `pub fn view(state) -> Element<Message>` - View function
//!
//! ## Bounded Contexts
//!
//! - **organization**: Organization/Person/Unit management
//! - **key_generation**: PKI and key generation (future)
//! - **yubikey**: YubiKey lifecycle management (future)
//! - **nats**: NATS infrastructure (future)
//! - **export**: Projection & export operations (future)

pub mod organization;

// Re-export primary types
pub use organization::{OrganizationMessage, OrganizationState};
