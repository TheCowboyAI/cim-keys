// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Infrastructure Domain Module
//!
//! This module defines the NATS bounded context messages.
//! Handlers are implemented in gui.rs.

pub mod infrastructure;

// Re-export primary types
pub use infrastructure::NatsMessage;
