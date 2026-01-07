// Copyright (c) 2025 - Cowboy AI, LLC.

//! Service Account Domain Module
//!
//! This module defines the Service Account bounded context messages.
//! Handlers are implemented in gui.rs.

pub mod management;

// Re-export primary types
pub use management::ServiceAccountMessage;
