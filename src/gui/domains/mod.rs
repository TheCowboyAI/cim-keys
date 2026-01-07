// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain-bounded GUI modules
//!
//! This module organizes the GUI into bounded contexts following DDD principles.
//! Each domain module contains only message definitions - handlers are in gui.rs.

pub mod organization;

// Re-export primary types
pub use organization::OrganizationMessage;
