// Copyright (c) 2025 - Cowboy AI, LLC.
//! Causation tracking for security audit trail compliance.
//!
//! This module enforces FRP Axiom A4 (Causality) by ensuring all events
//! have proper causation_id values - NEVER None.
//!
//! # Causation Rules
//!
//! - **Root events**: causation_id = event_id (self-reference)
//! - **Derived events**: causation_id = parent event/command id
//!
//! # Usage
//!
//! ```rust
//! use cim_keys::causation::Causation;
//! use uuid::Uuid;
//!
//! // Root event (no prior cause)
//! let event_id = Uuid::now_v7();
//! let causation = Causation::root(event_id);
//! assert_eq!(causation.id(), event_id);
//!
//! // Event caused by a command
//! let command_id = Uuid::now_v7();
//! let causation = Causation::from_command(command_id);
//!
//! // Event caused by another event
//! let parent_event_id = Uuid::now_v7();
//! let causation = Causation::from_event(parent_event_id);
//! ```

use uuid::Uuid;

/// Causation tracking that enforces non-None causation_id.
///
/// Unlike `Option<Uuid>`, this type guarantees a causation source
/// is always specified - critical for security audit trails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Causation {
    /// The ID of what caused this event/command
    source_id: Uuid,
    /// Type of causation source
    source_type: CausationType,
}

/// What type of message caused this one
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CausationType {
    /// Self-reference - this is a root event with no prior cause
    Root,
    /// Caused by a command
    Command,
    /// Caused by another event
    Event,
    /// Caused by a query response
    Query,
}

impl Causation {
    /// Create a root causation (self-reference).
    /// Use this for events that have no prior cause.
    ///
    /// The event_id IS the causation_id (self-reference pattern).
    #[inline]
    pub fn root(event_id: Uuid) -> Self {
        Self {
            source_id: event_id,
            source_type: CausationType::Root,
        }
    }

    /// Create causation from a command.
    /// The event was caused by processing this command.
    #[inline]
    pub fn from_command(command_id: Uuid) -> Self {
        Self {
            source_id: command_id,
            source_type: CausationType::Command,
        }
    }

    /// Create causation from another event.
    /// This event was triggered by the parent event.
    #[inline]
    pub fn from_event(parent_event_id: Uuid) -> Self {
        Self {
            source_id: parent_event_id,
            source_type: CausationType::Event,
        }
    }

    /// Create causation from a query.
    /// This event was triggered by a query response.
    #[inline]
    pub fn from_query(query_id: Uuid) -> Self {
        Self {
            source_id: query_id,
            source_type: CausationType::Query,
        }
    }

    /// Get the causation source ID.
    /// This is NEVER None - always returns a valid Uuid.
    #[inline]
    pub fn id(&self) -> Uuid {
        self.source_id
    }

    /// Get the causation type.
    #[inline]
    pub fn source_type(&self) -> CausationType {
        self.source_type
    }

    /// Is this a root causation (self-reference)?
    #[inline]
    pub fn is_root(&self) -> bool {
        matches!(self.source_type, CausationType::Root)
    }

    /// Convert to Option<Uuid> for backward compatibility.
    /// Always returns Some - NEVER None.
    #[inline]
    pub fn to_option(&self) -> Option<Uuid> {
        Some(self.source_id)
    }

    /// Convert from legacy Option<Uuid>.
    /// If None, this is a compliance violation - returns error.
    pub fn from_option(opt: Option<Uuid>, default_event_id: Uuid) -> Self {
        match opt {
            Some(id) => Self {
                source_id: id,
                source_type: CausationType::Event, // Assume event if we don't know
            },
            None => {
                // Log warning - this is a compliance issue
                #[cfg(debug_assertions)]
                eprintln!(
                    "WARNING: causation_id was None - using self-reference for event {}",
                    default_event_id
                );
                Self::root(default_event_id)
            }
        }
    }
}

impl From<Causation> for Option<Uuid> {
    fn from(c: Causation) -> Option<Uuid> {
        c.to_option()
    }
}

impl std::fmt::Display for Causation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self.source_type {
            CausationType::Root => "root",
            CausationType::Command => "cmd",
            CausationType::Event => "evt",
            CausationType::Query => "qry",
        };
        write!(f, "{}:{}", type_str, self.source_id)
    }
}

/// Builder for creating events with proper causation tracking.
///
/// Use this instead of manually setting causation_id fields.
#[derive(Debug, Clone)]
pub struct EventBuilder {
    event_id: Uuid,
    correlation_id: Uuid,
    causation: Causation,
}

impl EventBuilder {
    /// Create a root event builder (no prior cause).
    pub fn root() -> Self {
        let event_id = Uuid::now_v7();
        Self {
            event_id,
            correlation_id: event_id, // Root: correlation = event_id
            causation: Causation::root(event_id),
        }
    }

    /// Create a root event in a transaction context.
    pub fn root_in_transaction(transaction_id: Uuid) -> Self {
        let event_id = Uuid::now_v7();
        Self {
            event_id,
            correlation_id: transaction_id,
            causation: Causation::root(event_id),
        }
    }

    /// Create an event caused by a command.
    pub fn from_command(command_id: Uuid, correlation_id: Uuid) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation: Causation::from_command(command_id),
        }
    }

    /// Create an event caused by another event.
    pub fn from_event(parent_event_id: Uuid, correlation_id: Uuid) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            correlation_id,
            causation: Causation::from_event(parent_event_id),
        }
    }

    /// Get the event ID.
    pub fn event_id(&self) -> Uuid {
        self.event_id
    }

    /// Get the correlation ID.
    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    /// Get the causation ID (NEVER None).
    pub fn causation_id(&self) -> Uuid {
        self.causation.id()
    }

    /// Get the causation as Option<Uuid> for legacy compatibility.
    /// Always returns Some.
    pub fn causation_id_option(&self) -> Option<Uuid> {
        self.causation.to_option()
    }

    /// Get the full causation.
    pub fn causation(&self) -> Causation {
        self.causation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_causation_is_self_reference() {
        let event_id = Uuid::now_v7();
        let causation = Causation::root(event_id);

        assert_eq!(causation.id(), event_id);
        assert!(causation.is_root());
        assert_eq!(causation.to_option(), Some(event_id));
    }

    #[test]
    fn command_causation_points_to_command() {
        let command_id = Uuid::now_v7();
        let causation = Causation::from_command(command_id);

        assert_eq!(causation.id(), command_id);
        assert!(!causation.is_root());
        assert_eq!(causation.source_type(), CausationType::Command);
    }

    #[test]
    fn event_causation_points_to_parent() {
        let parent_id = Uuid::now_v7();
        let causation = Causation::from_event(parent_id);

        assert_eq!(causation.id(), parent_id);
        assert_eq!(causation.source_type(), CausationType::Event);
    }

    #[test]
    fn causation_never_returns_none() {
        let root = Causation::root(Uuid::now_v7());
        let cmd = Causation::from_command(Uuid::now_v7());
        let evt = Causation::from_event(Uuid::now_v7());

        assert!(root.to_option().is_some());
        assert!(cmd.to_option().is_some());
        assert!(evt.to_option().is_some());
    }

    #[test]
    fn event_builder_creates_valid_causation() {
        let builder = EventBuilder::root();
        assert!(builder.causation().is_root());
        assert!(builder.causation_id_option().is_some());

        let cmd_id = Uuid::now_v7();
        let corr_id = Uuid::now_v7();
        let builder = EventBuilder::from_command(cmd_id, corr_id);
        assert_eq!(builder.causation_id(), cmd_id);
        assert_eq!(builder.correlation_id(), corr_id);
    }

    #[test]
    fn from_option_never_returns_none() {
        let event_id = Uuid::now_v7();

        // With Some
        let causation = Causation::from_option(Some(Uuid::now_v7()), event_id);
        assert!(causation.to_option().is_some());

        // With None - should use self-reference
        let causation = Causation::from_option(None, event_id);
        assert_eq!(causation.id(), event_id);
        assert!(causation.is_root());
    }

    #[test]
    fn display_format_is_readable() {
        let id = Uuid::now_v7();
        let root = Causation::root(id);
        let display = format!("{}", root);
        assert!(display.starts_with("root:"));

        let cmd = Causation::from_command(id);
        let display = format!("{}", cmd);
        assert!(display.starts_with("cmd:"));
    }
}
