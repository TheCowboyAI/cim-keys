// Copyright (c) 2025 - Cowboy AI, LLC.

//! Certificate Domain Module
//!
//! This module handles the Certificate bounded context:
//! - X.509 certificate metadata (organization, locality, validity)
//! - Intermediate CA generation and selection
//! - Server certificate generation with SANs
//! - mTLS client certificate generation
//! - Certificate chain viewing
//!
//! ## Message Flow
//!
//! ```text
//! User Action → CertificateMessage → update() → Task<Message>
//!                                               ↓
//!                                       CertificateState mutated
//! ```

pub mod management;

// Re-export primary types
pub use management::{CertificateMessage, CertificateState};
