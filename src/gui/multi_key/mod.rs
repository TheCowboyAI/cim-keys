// Copyright (c) 2025 - Cowboy AI, LLC.

//! Multi-Purpose Key Domain Module
//!
//! This module defines the Multi-Purpose Key bounded context messages.
//! Handlers are implemented in gui.rs.
//!
//! ## Message Flow
//!
//! ```text
//! User Action → MultiKeyMessage → update() in gui.rs
//!                                         ↓
//!                                 CimKeysApp fields mutated
//! ```

pub mod generation;

// Re-export primary types
pub use generation::{MultiKeyMessage, GenerationResult, available_purposes, purpose_display_name};
