// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Unit Domain Module
//!
//! This module handles the Organization Unit bounded context:
//! - Create and manage organizational units (divisions, departments, teams)
//! - Unit hierarchy (parent-child relationships)
//! - NATS account mapping for units
//! - Responsible person assignment
//!
//! ## Message Flow
//!
//! ```text
//! User Action → OrgUnitMessage → update() → Task<Message>
//!                                          ↓
//!                                  OrgUnitState mutated
//! ```

pub mod management;

// Re-export primary types
pub use management::{OrgUnitMessage, OrgUnitState};
