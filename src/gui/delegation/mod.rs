// Copyright (c) 2025 - Cowboy AI, LLC.

//! Delegation Domain Module
//!
//! This module defines the Delegation bounded context messages.
//! Handlers are implemented in gui.rs.

pub mod authorization;

// Re-export primary types
pub use authorization::{DelegationEntry, DelegationMessage};
