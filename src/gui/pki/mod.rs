// Copyright (c) 2025 - Cowboy AI, LLC.

//! PKI Domain Module
//!
//! This module defines the PKI bounded context messages.
//! Handlers are implemented in gui.rs.

pub mod key_generation;

// Re-export primary types
pub use key_generation::PkiMessage;
