// Copyright (c) 2025 - Cowboy AI, LLC.

//! Event Log Replay Bounded Context
//!
//! This module implements the Event Log domain with:
//! - Message enum for all event log operations
//! - State struct for event log fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Loading**: Load events from store
//! 3. **Selection**: Select events for replay
//! 4. **Replay**: Replay selected events

use iced::Task;
use std::collections::HashSet;

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

/// Event Log State
///
/// Contains all state related to event log management.
#[derive(Debug, Clone, Default)]
pub struct EventLogState {
    // === UI State ===
    /// Whether the event log section is collapsed
    pub section_collapsed: bool,

    // === Loaded Data ===
    /// Events loaded from the event store
    pub loaded_events: Vec<StoredEventRecord>,

    // === Selection ===
    /// CIDs of events selected for replay
    pub selected_cids: HashSet<String>,

    // === Status ===
    /// Status message for user feedback
    pub status: Option<String>,
}

impl EventLogState {
    /// Create a new EventLogState with sensible defaults
    pub fn new() -> Self {
        Self {
            section_collapsed: true,
            loaded_events: Vec::new(),
            selected_cids: HashSet::new(),
            status: None,
        }
    }

    /// Check if any events are loaded
    pub fn has_events(&self) -> bool {
        !self.loaded_events.is_empty()
    }

    /// Get count of loaded events
    pub fn event_count(&self) -> usize {
        self.loaded_events.len()
    }

    /// Check if any events are selected
    pub fn has_selection(&self) -> bool {
        !self.selected_cids.is_empty()
    }

    /// Get count of selected events
    pub fn selection_count(&self) -> usize {
        self.selected_cids.len()
    }

    /// Check if a specific event is selected
    pub fn is_selected(&self, cid: &str) -> bool {
        self.selected_cids.contains(cid)
    }

    /// Check if ready to replay
    pub fn is_ready_to_replay(&self) -> bool {
        self.has_events() && self.has_selection()
    }

    /// Get validation error if not ready
    pub fn validation_error(&self) -> Option<String> {
        if !self.has_events() {
            return Some("No events loaded. Click 'Load Events' first.".to_string());
        }
        if !self.has_selection() {
            return Some("No events selected for replay".to_string());
        }
        None
    }

    /// Get selected events
    pub fn selected_events(&self) -> Vec<&StoredEventRecord> {
        self.loaded_events
            .iter()
            .filter(|e| self.selected_cids.contains(&e.cid))
            .collect()
    }

    /// Select all events
    pub fn select_all(&mut self) {
        self.selected_cids = self.loaded_events.iter().map(|e| e.cid.clone()).collect();
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_cids.clear();
    }

    /// Clear all state
    pub fn reset(&mut self) {
        self.loaded_events.clear();
        self.selected_cids.clear();
        self.status = None;
    }

    /// Set status for loading
    pub fn set_status_loading(&mut self) {
        self.status = Some("📂 Loading events...".to_string());
    }

    /// Set status for loaded
    pub fn set_status_loaded(&mut self, count: usize) {
        self.status = Some(format!("✅ Loaded {} events", count));
    }

    /// Set status for replaying
    pub fn set_status_replaying(&mut self, count: usize) {
        self.status = Some(format!("⏪ Replaying {} events...", count));
    }

    /// Set status for replayed
    pub fn set_status_replayed(&mut self, count: usize) {
        self.status = Some(format!("✅ Successfully replayed {} events", count));
    }

    /// Set status for failure
    pub fn set_status_failure(&mut self, error: &str) {
        self.status = Some(format!("❌ {}", error));
    }

    /// Get events by NATS subject (filter by envelope.nats_subject field)
    pub fn events_by_subject(&self, subject: &str) -> Vec<&StoredEventRecord> {
        self.loaded_events
            .iter()
            .filter(|e| e.envelope.nats_subject.contains(subject))
            .collect()
    }

    /// Get unique NATS subjects
    pub fn nats_subjects(&self) -> Vec<String> {
        let mut subjects: Vec<_> = self
            .loaded_events
            .iter()
            .map(|e| e.envelope.nats_subject.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        subjects.sort();
        subjects
    }

    /// Find event by CID
    pub fn find_event(&self, cid: &str) -> Option<&StoredEventRecord> {
        self.loaded_events.iter().find(|e| e.cid == cid)
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update event log state based on message
///
/// This function handles event log domain messages. Note that Load and Replay
/// require event store access and will be delegated to the main update function.
pub fn update(state: &mut EventLogState, message: EventLogMessage) -> Task<Message> {
    use EventLogMessage::*;

    match message {
        // === UI State ===
        ToggleSection => {
            state.section_collapsed = !state.section_collapsed;
            Task::none()
        }

        // === Loading (delegated to main for event store access) ===
        Load => {
            // Actual loading requires event store access
            // Delegated to main update function
            Task::none()
        }

        Loaded(result) => {
            match result {
                Ok(events) => {
                    let count = events.len();
                    state.loaded_events = events;
                    state.set_status_loaded(count);
                }
                Err(error) => {
                    state.set_status_failure(&error);
                }
            }
            Task::none()
        }

        // === Selection ===
        ToggleSelection(cid) => {
            if state.selected_cids.contains(&cid) {
                state.selected_cids.remove(&cid);
            } else {
                state.selected_cids.insert(cid);
            }
            Task::none()
        }

        ClearSelection => {
            state.clear_selection();
            Task::none()
        }

        // === Replay (delegated to main for state reconstruction) ===
        Replay => {
            // Actual replay requires state reconstruction
            // Delegated to main update function
            Task::none()
        }

        Replayed(result) => {
            match result {
                Ok(count) => {
                    state.set_status_replayed(count);
                    state.clear_selection();
                }
                Err(error) => {
                    state.set_status_failure(&error);
                }
            }
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use crate::events::{EventEnvelope, DomainEvent, OrganizationEvents};
    use crate::events::organization::OrganizationCreatedEvent;

    fn create_test_event(cid: &str, subject: &str) -> StoredEventRecord {
        // Create a minimal domain event for testing
        let org_created = OrganizationCreatedEvent {
            organization_id: Uuid::now_v7(),
            name: "Test Org".to_string(),
            domain: Some("test.example.com".to_string()),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        };
        let domain_event = DomainEvent::Organization(OrganizationEvents::OrganizationCreated(org_created));

        let envelope = EventEnvelope {
            event_id: Uuid::now_v7(),
            correlation_id: Uuid::now_v7(),
            causation_id: None,
            nats_subject: subject.to_string(),
            timestamp: Utc::now(),
            cid: Some(cid.to_string()),
            event: domain_event,
        };

        StoredEventRecord {
            cid: cid.to_string(),
            envelope,
            stored_at: Utc::now(),
            causation_cid: None,
        }
    }

    #[test]
    fn test_event_log_state_default() {
        let state = EventLogState::default();
        assert!(!state.section_collapsed);
        assert!(state.loaded_events.is_empty());
        assert!(state.selected_cids.is_empty());
        assert!(state.status.is_none());
    }

    #[test]
    fn test_event_log_state_new() {
        let state = EventLogState::new();
        assert!(state.section_collapsed);
        assert!(state.loaded_events.is_empty());
        assert!(state.selected_cids.is_empty());
    }

    #[test]
    fn test_toggle_section() {
        let mut state = EventLogState::new();
        assert!(state.section_collapsed);

        let _ = update(&mut state, EventLogMessage::ToggleSection);
        assert!(!state.section_collapsed);

        let _ = update(&mut state, EventLogMessage::ToggleSection);
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_has_events() {
        let mut state = EventLogState::new();
        assert!(!state.has_events());

        state.loaded_events.push(create_test_event("cid1", "TestEvent"));
        assert!(state.has_events());
    }

    #[test]
    fn test_event_count() {
        let mut state = EventLogState::new();
        assert_eq!(state.event_count(), 0);

        state.loaded_events.push(create_test_event("cid1", "TestEvent"));
        state.loaded_events.push(create_test_event("cid2", "TestEvent"));
        assert_eq!(state.event_count(), 2);
    }

    #[test]
    fn test_toggle_selection() {
        let mut state = EventLogState::new();
        let cid = "test-cid".to_string();

        let _ = update(&mut state, EventLogMessage::ToggleSelection(cid.clone()));
        assert!(state.is_selected(&cid));

        let _ = update(&mut state, EventLogMessage::ToggleSelection(cid.clone()));
        assert!(!state.is_selected(&cid));
    }

    #[test]
    fn test_clear_selection() {
        let mut state = EventLogState::new();
        state.selected_cids.insert("cid1".to_string());
        state.selected_cids.insert("cid2".to_string());

        let _ = update(&mut state, EventLogMessage::ClearSelection);
        assert!(state.selected_cids.is_empty());
    }

    #[test]
    fn test_has_selection() {
        let mut state = EventLogState::new();
        assert!(!state.has_selection());

        state.selected_cids.insert("cid1".to_string());
        assert!(state.has_selection());
    }

    #[test]
    fn test_selection_count() {
        let mut state = EventLogState::new();
        assert_eq!(state.selection_count(), 0);

        state.selected_cids.insert("cid1".to_string());
        state.selected_cids.insert("cid2".to_string());
        assert_eq!(state.selection_count(), 2);
    }

    #[test]
    fn test_is_ready_to_replay() {
        let mut state = EventLogState::new();
        assert!(!state.is_ready_to_replay());

        state.loaded_events.push(create_test_event("cid1", "TestEvent"));
        assert!(!state.is_ready_to_replay());

        state.selected_cids.insert("cid1".to_string());
        assert!(state.is_ready_to_replay());
    }

    #[test]
    fn test_validation_error_no_events() {
        let state = EventLogState::new();
        assert_eq!(
            state.validation_error(),
            Some("No events loaded. Click 'Load Events' first.".to_string())
        );
    }

    #[test]
    fn test_validation_error_no_selection() {
        let mut state = EventLogState::new();
        state.loaded_events.push(create_test_event("cid1", "TestEvent"));

        assert_eq!(
            state.validation_error(),
            Some("No events selected for replay".to_string())
        );
    }

    #[test]
    fn test_validation_no_error() {
        let mut state = EventLogState::new();
        state.loaded_events.push(create_test_event("cid1", "TestEvent"));
        state.selected_cids.insert("cid1".to_string());

        assert!(state.validation_error().is_none());
    }

    #[test]
    fn test_selected_events() {
        let mut state = EventLogState::new();
        state.loaded_events.push(create_test_event("cid1", "TestEvent"));
        state.loaded_events.push(create_test_event("cid2", "TestEvent"));
        state.loaded_events.push(create_test_event("cid3", "TestEvent"));

        state.selected_cids.insert("cid1".to_string());
        state.selected_cids.insert("cid3".to_string());

        let selected = state.selected_events();
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn test_select_all() {
        let mut state = EventLogState::new();
        state.loaded_events.push(create_test_event("cid1", "TestEvent"));
        state.loaded_events.push(create_test_event("cid2", "TestEvent"));

        state.select_all();
        assert_eq!(state.selection_count(), 2);
    }

    #[test]
    fn test_reset() {
        let mut state = EventLogState::new();
        state.loaded_events.push(create_test_event("cid1", "TestEvent"));
        state.selected_cids.insert("cid1".to_string());
        state.status = Some("Test status".to_string());

        state.reset();

        assert!(state.loaded_events.is_empty());
        assert!(state.selected_cids.is_empty());
        assert!(state.status.is_none());
    }

    #[test]
    fn test_loaded_result_success() {
        let mut state = EventLogState::new();
        let events = vec![
            create_test_event("cid1", "TestEvent"),
            create_test_event("cid2", "TestEvent"),
        ];

        let _ = update(&mut state, EventLogMessage::Loaded(Ok(events)));

        assert_eq!(state.event_count(), 2);
        assert!(state.status.as_ref().unwrap().contains("2 events"));
    }

    #[test]
    fn test_loaded_result_error() {
        let mut state = EventLogState::new();

        let _ = update(
            &mut state,
            EventLogMessage::Loaded(Err("Test error".to_string())),
        );

        assert!(state.loaded_events.is_empty());
        assert!(state.status.as_ref().unwrap().contains("Test error"));
    }

    #[test]
    fn test_replayed_result_success() {
        let mut state = EventLogState::new();
        state.selected_cids.insert("cid1".to_string());

        let _ = update(&mut state, EventLogMessage::Replayed(Ok(5)));

        assert!(state.selected_cids.is_empty());
        assert!(state.status.as_ref().unwrap().contains("5 events"));
    }

    #[test]
    fn test_replayed_result_error() {
        let mut state = EventLogState::new();
        state.selected_cids.insert("cid1".to_string());

        let _ = update(
            &mut state,
            EventLogMessage::Replayed(Err("Replay failed".to_string())),
        );

        // Selection preserved on error
        assert!(!state.selected_cids.is_empty());
        assert!(state.status.as_ref().unwrap().contains("Replay failed"));
    }

    #[test]
    fn test_events_by_subject() {
        let mut state = EventLogState::new();
        state.loaded_events.push(create_test_event("cid1", "org.person.created"));
        state.loaded_events.push(create_test_event("cid2", "org.key.generated"));
        state.loaded_events.push(create_test_event("cid3", "org.person.created"));

        let person_events = state.events_by_subject("person");
        assert_eq!(person_events.len(), 2);

        let key_events = state.events_by_subject("key");
        assert_eq!(key_events.len(), 1);
    }

    #[test]
    fn test_nats_subjects() {
        let mut state = EventLogState::new();
        state.loaded_events.push(create_test_event("cid1", "org.person.created"));
        state.loaded_events.push(create_test_event("cid2", "org.key.generated"));
        state.loaded_events.push(create_test_event("cid3", "org.person.created"));

        let subjects = state.nats_subjects();
        assert_eq!(subjects.len(), 2);
        assert!(subjects.contains(&"org.key.generated".to_string()));
        assert!(subjects.contains(&"org.person.created".to_string()));
    }

    #[test]
    fn test_find_event() {
        let mut state = EventLogState::new();
        state.loaded_events.push(create_test_event("cid1", "TestEvent"));
        state.loaded_events.push(create_test_event("cid2", "TestEvent"));

        assert!(state.find_event("cid1").is_some());
        assert!(state.find_event("cid2").is_some());
        assert!(state.find_event("cid3").is_none());
    }

    #[test]
    fn test_status_helpers() {
        let mut state = EventLogState::new();

        state.set_status_loading();
        assert!(state.status.as_ref().unwrap().contains("Loading"));

        state.set_status_loaded(10);
        assert!(state.status.as_ref().unwrap().contains("10 events"));

        state.set_status_replaying(5);
        assert!(state.status.as_ref().unwrap().contains("5 events"));

        state.set_status_replayed(5);
        assert!(state.status.as_ref().unwrap().contains("5 events"));

        state.set_status_failure("Test error");
        assert!(state.status.as_ref().unwrap().contains("Test error"));
    }
}
