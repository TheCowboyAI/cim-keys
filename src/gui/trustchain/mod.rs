// Copyright (c) 2025 - Cowboy AI, LLC.

//! TrustChain Domain Module
//!
//! This module handles the TrustChain bounded context:
//! - Certificate trust chain verification
//! - Chain traversal from leaf to root
//! - Verification status tracking
//! - Expired/self-signed/missing issuer detection
//!
//! ## Message Flow
//!
//! ```text
//! User Action → TrustChainMessage → update() → Task<Message>
//!                                            ↓
//!                                    TrustChainState mutated
//! ```

pub mod verification;

// Re-export primary types
pub use verification::{TrustChainMessage, TrustChainState, TrustChainStatus};
