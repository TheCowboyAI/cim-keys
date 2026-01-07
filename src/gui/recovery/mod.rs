// Copyright (c) 2025 - Cowboy AI, LLC.

//! Key Recovery Domain Module
//!
//! This module defines the Key Recovery bounded context messages.
//! Handlers are implemented in gui.rs.

pub mod seed;

// Re-export primary types
pub use seed::RecoveryMessage;
