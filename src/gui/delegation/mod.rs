// Copyright (c) 2025 - Cowboy AI, LLC.

//! Delegation Domain Module
//!
//! This module handles the Delegation bounded context:
//! - Permission delegation between people
//! - Grantor (from) and grantee (to) selection
//! - Permission set management
//! - Delegation expiration
//! - Active delegation tracking
//!
//! ## Message Flow
//!
//! ```text
//! User Action → DelegationMessage → update() → Task<Message>
//!                                            ↓
//!                                    DelegationState mutated
//! ```

pub mod authorization;

// Re-export primary types
pub use authorization::{DelegationMessage, DelegationState, DelegationEntry};
