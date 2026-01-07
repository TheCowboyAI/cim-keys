// Copyright (c) 2025 - Cowboy AI, LLC.

//! Location Domain Module
//!
//! This module defines the Location bounded context messages.
//! Handlers are implemented in gui.rs.

pub mod management;

// Re-export primary types
pub use management::LocationMessage;
