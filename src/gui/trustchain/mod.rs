// Copyright (c) 2025 - Cowboy AI, LLC.

//! TrustChain Domain Module
//!
//! This module defines the TrustChain bounded context messages.
//! Handlers are implemented in gui.rs.

pub mod verification;

// Re-export primary types
pub use verification::{TrustChainMessage, TrustChainStatus};
