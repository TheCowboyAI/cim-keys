// Copyright (c) 2025 - Cowboy AI, LLC.

//! Export Domain Module
//!
//! This module handles the Export bounded context:
//! - SD Card export (encrypted air-gapped storage)
//! - Cypher/Neo4j export
//! - NSC (NATS credentials) export
//! - Graph export/import
//! - Projection configuration and status
//!
//! ## Message Flow
//!
//! ```text
//! User Action → ExportMessage → update() → Task<Message>
//!                                       ↓
//!                               ExportState mutated
//! ```

pub mod projection;

// Re-export primary types
pub use projection::{ExportMessage, ExportState};
