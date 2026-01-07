// Copyright (c) 2025 - Cowboy AI, LLC.

//! Location Domain Module
//!
//! This module handles the Location bounded context:
//! - Physical, virtual, and hybrid location management
//! - Location form validation and submission
//! - Address field management
//! - Location lifecycle (add, remove)
//!
//! ## Message Flow
//!
//! ```text
//! User Action → LocationMessage → update() → Task<Message>
//!                                          ↓
//!                                  LocationState mutated
//! ```

pub mod management;

// Re-export primary types
pub use management::{LocationMessage, LocationState};
