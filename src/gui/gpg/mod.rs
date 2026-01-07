// Copyright (c) 2025 - Cowboy AI, LLC.

//! GPG Keys Domain Module
//!
//! This module handles the GPG Keys bounded context:
//! - GPG/PGP key pair generation
//! - Key type selection (EdDSA, ECDSA, RSA, DSA)
//! - Key length and expiration configuration
//! - Key listing and management
//!
//! ## Message Flow
//!
//! ```text
//! User Action → GpgMessage → update() → Task<Message>
//!                                      ↓
//!                              GpgState mutated
//! ```

pub mod generation;

// Re-export primary types
pub use generation::{GpgMessage, GpgState};
