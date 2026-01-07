// Copyright (c) 2025 - Cowboy AI, LLC.

//! YubiKey Domain Module
//!
//! This module handles the YubiKey bounded context:
//! - Device detection and enumeration
//! - Assignment to persons
//! - PIV slot management
//! - PIN and management key operations
//! - Domain registration and lifecycle
//! - Attestation and key generation
//!
//! ## Message Flow
//!
//! ```text
//! User Action → YubiKeyMessage → update() → Task<Message>
//!                                      ↓
//!                              YubiKeyState mutated
//! ```

pub mod management;

// Re-export primary types
pub use management::{YubiKeyMessage, YubiKeyState};
