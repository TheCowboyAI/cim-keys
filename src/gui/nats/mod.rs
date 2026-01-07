// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Infrastructure Domain Module
//!
//! This module handles the NATS bounded context:
//! - Hierarchy generation and bootstrap
//! - Operator, account, and user management
//! - Visualization state (expanded nodes, selections)
//! - NATS configuration export
//!
//! ## Message Flow
//!
//! ```text
//! User Action → NatsMessage → update() → Task<Message>
//!                                     ↓
//!                             NatsState mutated
//! ```

pub mod infrastructure;

// Re-export primary types
pub use infrastructure::{NatsMessage, NatsState};
