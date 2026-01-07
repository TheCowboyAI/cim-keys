// Copyright (c) 2025 - Cowboy AI, LLC.

//! YubiKey Domain Module
//!
//! This module defines the YubiKey bounded context messages.
//! Handlers are implemented in gui.rs.

pub mod management;

// Re-export primary types
pub use management::YubiKeyMessage;
