// Copyright (c) 2025 - Cowboy AI, LLC.

//! PKI Domain Module
//!
//! This module handles the PKI (Public Key Infrastructure) bounded context:
//! - Root CA generation
//! - Intermediate CA management
//! - Server certificate generation
//! - SSH key generation
//! - GPG key management
//! - Key recovery from seed
//! - Client certificate (mTLS)
//! - Multi-purpose key bundles
//!
//! ## Message Flow
//!
//! ```text
//! User Action → PkiMessage → update() → Task<Message>
//!                                 ↓
//!                         PkiState mutated
//! ```

pub mod key_generation;

// Re-export primary types
pub use key_generation::{PkiMessage, PkiState};
