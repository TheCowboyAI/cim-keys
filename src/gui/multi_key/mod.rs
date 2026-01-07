// Copyright (c) 2025 - Cowboy AI, LLC.

//! Multi-Purpose Key Domain Module
//!
//! This module handles the Multi-Purpose Key bounded context:
//! - Generate multiple keys for a single person
//! - Select key purposes (Authentication, Signing, Encryption, KeyAgreement)
//! - Batch key generation workflow
//!
//! ## Message Flow
//!
//! ```text
//! User Action → MultiKeyMessage → update() → Task<Message>
//!                                            ↓
//!                                    MultiKeyState mutated
//! ```

pub mod generation;

// Re-export primary types
pub use generation::{MultiKeyMessage, MultiKeyState};
