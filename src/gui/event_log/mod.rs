// Copyright (c) 2025 - Cowboy AI, LLC.

//! Event Log Domain Module
//!
//! This module defines the Event Log bounded context messages.
//! Handlers are implemented in gui.rs.
//!
//! ## Message Flow
//!
//! ```text
//! User Action → EventLogMessage → update() in gui.rs
//!                                         ↓
//!                                 CimKeysApp fields mutated
//! ```

pub mod replay;

// Re-export primary types
pub use replay::EventLogMessage;
