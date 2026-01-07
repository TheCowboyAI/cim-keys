// Copyright (c) 2025 - Cowboy AI, LLC.

//! Event Log Message Definitions
//!
//! This module defines the message types for the Event Log bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Loading**: Load events from store
//! 3. **Selection**: Select events for replay
//! 4. **Replay**: Replay selected events

use crate::event_store::StoredEventRecord;

/// Event Log Message
///
/// Organized by sub-domain:
/// - UI State (1 message)
/// - Loading (2 messages)
/// - Selection (2 messages)
/// - Replay (2 messages)
#[derive(Debug, Clone)]
pub enum EventLogMessage {
    // === UI State ===
    /// Toggle event log section visibility
    ToggleSection,

    // === Loading ===
    /// Load events from event store
    Load,
    /// Events loaded result
    Loaded(Result<Vec<StoredEventRecord>, String>),

    // === Selection ===
    /// Toggle selection of an event by CID
    ToggleSelection(String),
    /// Clear all selected events
    ClearSelection,

    // === Replay ===
    /// Replay selected events for state reconstruction
    Replay,
    /// Replay completed result
    Replayed(Result<usize, String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_log_message_variants() {
        // Just verify the enum variants compile correctly
        let _ = EventLogMessage::ToggleSection;
        let _ = EventLogMessage::Load;
        let _ = EventLogMessage::Loaded(Ok(vec![]));
        let _ = EventLogMessage::ToggleSelection("cid123".to_string());
        let _ = EventLogMessage::ClearSelection;
        let _ = EventLogMessage::Replay;
        let _ = EventLogMessage::Replayed(Ok(0));
    }
}
