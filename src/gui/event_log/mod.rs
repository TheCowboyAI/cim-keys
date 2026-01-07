// Copyright (c) 2025 - Cowboy AI, LLC.

//! Event Log Domain Module
//!
//! This module handles the Event Log bounded context:
//! - Load events from CID-based event store
//! - Select events for replay
//! - Replay selected events for state reconstruction
//!
//! ## Message Flow
//!
//! ```text
//! User Action → EventLogMessage → update() → Task<Message>
//!                                            ↓
//!                                    EventLogState mutated
//! ```

pub mod replay;

// Re-export primary types
pub use replay::{EventLogMessage, EventLogState};
