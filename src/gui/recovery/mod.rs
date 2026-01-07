// Copyright (c) 2025 - Cowboy AI, LLC.

//! Key Recovery Domain Module
//!
//! This module handles the Key Recovery bounded context:
//! - BIP-39 seed phrase verification
//! - Passphrase-based key recovery
//! - Deterministic key regeneration
//! - Organization-scoped recovery
//!
//! ## Message Flow
//!
//! ```text
//! User Action → RecoveryMessage → update() → Task<Message>
//!                                           ↓
//!                                   RecoveryState mutated
//! ```

pub mod seed;

// Re-export primary types
pub use seed::{RecoveryMessage, RecoveryState};
