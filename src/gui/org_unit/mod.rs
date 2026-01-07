// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Unit Domain Module
//!
//! This module defines the Organization Unit bounded context messages.
//! Handlers are implemented in gui.rs.
//!
//! ## Message Flow
//!
//! ```text
//! User Action → OrgUnitMessage → update() in gui.rs
//!                                        ↓
//!                                CimKeysApp fields mutated
//! ```

pub mod management;

// Re-export primary types
pub use management::OrgUnitMessage;
