// Copyright (c) 2025 - Cowboy AI, LLC.

//! Certificate Domain Module
//!
//! This module defines the Certificate bounded context messages.
//! Handlers are implemented in gui.rs.
//!
//! ## Message Flow
//!
//! ```text
//! User Action → CertificateMessage → update() in gui.rs
//!                                            ↓
//!                                    CimKeysApp fields mutated
//! ```

pub mod management;

// Re-export primary types
pub use management::CertificateMessage;
