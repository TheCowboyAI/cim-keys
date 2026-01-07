// Copyright (c) 2025 - Cowboy AI, LLC.

//! Service Account Domain Module
//!
//! This module handles the Service Account bounded context:
//! - Automated system account creation and management
//! - Service account lifecycle (create, deactivate, remove)
//! - Key generation for service accounts
//! - Ownership and responsibility tracking
//!
//! ## Message Flow
//!
//! ```text
//! User Action → ServiceAccountMessage → update() → Task<Message>
//!                                                 ↓
//!                                         ServiceAccountState mutated
//! ```

pub mod management;

// Re-export primary types
pub use management::{ServiceAccountMessage, ServiceAccountState};
