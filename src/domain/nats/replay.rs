// Copyright (c) 2025 - Cowboy AI, LLC.

//! Event Replay and Projection Rebuilding for JetStream
//!
//! This module provides event sourcing capabilities for replaying events
//! from JetStream streams and rebuilding projections from the event log.
//!
//! ## Architecture
//!
//! ```text
//! JetStream Stream
//!       │
//!       ▼
//! EventReplay trait ──► ReplayOptions
//!       │                    │
//!       ▼                    ▼
//! DomainEvent stream   Filter/Range
//!       │
//!       ▼
//! Projection::apply_event()
//!       │
//!       ▼
//! Rebuilt State
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use cim_keys::domain::nats::replay::{EventReplay, ReplayOptions};
//!
//! // Create replay from JetStream
//! let replay = JetStreamReplay::new(jetstream_port);
//!
//! // Replay all events
//! let events = replay.replay_all(KEYS_EVENTS_STREAM).await?;
//!
//! // Replay with filter
//! let options = ReplayOptions::default()
//!     .with_subject_filter("keys.events.key.>")
//!     .with_start_sequence(100);
//! let events = replay.replay(KEYS_EVENTS_STREAM, options).await?;
//!
//! // Rebuild projection from events
//! let projection = rebuild_projection(&events)?;
//! ```

use crate::events::DomainEvent;
use crate::ports::{JetStreamPort, JetStreamError};
use cim_domain::DomainEvent as DomainEventTrait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Extension trait to get aggregate ID from DomainEvent wrapper
trait DomainEventExt {
    fn aggregate_id(&self) -> Uuid;
}

impl DomainEventExt for DomainEvent {
    fn aggregate_id(&self) -> Uuid {
        match self {
            DomainEvent::Person(e) => e.aggregate_id(),
            DomainEvent::Organization(e) => e.aggregate_id(),
            DomainEvent::Location(e) => e.aggregate_id(),
            DomainEvent::Certificate(e) => e.aggregate_id(),
            DomainEvent::Key(e) => e.aggregate_id(),
            DomainEvent::Delegation(e) => e.aggregate_id(),
            DomainEvent::NatsOperator(e) => e.aggregate_id(),
            DomainEvent::NatsAccount(e) => e.aggregate_id(),
            DomainEvent::NatsUser(e) => e.aggregate_id(),
            DomainEvent::YubiKey(e) => e.aggregate_id(),
            DomainEvent::Relationship(e) => e.aggregate_id(),
            DomainEvent::Manifest(e) => e.aggregate_id(),
            DomainEvent::Saga(e) => e.saga_id(),
        }
    }
}

/// Options for replaying events from a stream
#[derive(Debug, Clone, Default)]
pub struct ReplayOptions {
    /// Filter events by subject pattern
    pub subject_filter: Option<String>,

    /// Start from a specific sequence number
    pub start_sequence: Option<u64>,

    /// Start from a specific timestamp
    pub start_time: Option<DateTime<Utc>>,

    /// Maximum number of events to replay
    pub max_events: Option<u64>,

    /// Filter by correlation ID
    pub correlation_id: Option<Uuid>,

    /// Filter by aggregate ID
    pub aggregate_id: Option<Uuid>,
}

impl ReplayOptions {
    /// Create new replay options
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by subject pattern (e.g., "keys.events.key.>")
    pub fn with_subject_filter(mut self, pattern: impl Into<String>) -> Self {
        self.subject_filter = Some(pattern.into());
        self
    }

    /// Start replay from a specific sequence number
    pub fn with_start_sequence(mut self, seq: u64) -> Self {
        self.start_sequence = Some(seq);
        self
    }

    /// Start replay from a specific timestamp
    pub fn with_start_time(mut self, time: DateTime<Utc>) -> Self {
        self.start_time = Some(time);
        self
    }

    /// Limit the number of events to replay
    pub fn with_max_events(mut self, max: u64) -> Self {
        self.max_events = Some(max);
        self
    }

    /// Filter by correlation ID (saga/workflow tracking)
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Filter by aggregate ID
    pub fn with_aggregate_id(mut self, id: Uuid) -> Self {
        self.aggregate_id = Some(id);
        self
    }
}

/// Stored event with metadata from JetStream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// Stream sequence number
    pub sequence: u64,

    /// Event subject
    pub subject: String,

    /// The domain event
    pub event: DomainEvent,

    /// Event ID from headers
    pub event_id: Option<Uuid>,

    /// Correlation ID from headers
    pub correlation_id: Option<Uuid>,

    /// Causation ID from headers
    pub causation_id: Option<Uuid>,

    /// Event timestamp from headers
    pub timestamp: DateTime<Utc>,

    /// Source service from headers
    pub source: Option<String>,
}

/// Result of a replay operation
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// Events that were replayed
    pub events: Vec<StoredEvent>,

    /// First sequence number in the result
    pub first_sequence: Option<u64>,

    /// Last sequence number in the result
    pub last_sequence: Option<u64>,

    /// Total events in stream (not just replayed)
    pub stream_total: u64,
}

impl ReplayResult {
    /// Check if the replay is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the number of events replayed
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Iterate over events
    pub fn iter(&self) -> impl Iterator<Item = &StoredEvent> {
        self.events.iter()
    }

    /// Extract just the domain events
    pub fn domain_events(&self) -> Vec<&DomainEvent> {
        self.events.iter().map(|e| &e.event).collect()
    }
}

/// Trait for replaying events from an event store
#[async_trait::async_trait]
pub trait EventReplay: Send + Sync {
    /// Replay all events from a stream
    async fn replay_all(&self, stream: &str) -> Result<ReplayResult, EventReplayError>;

    /// Replay events with options
    async fn replay(
        &self,
        stream: &str,
        options: ReplayOptions,
    ) -> Result<ReplayResult, EventReplayError>;

    /// Get the current stream position (last sequence number)
    async fn current_position(&self, stream: &str) -> Result<u64, EventReplayError>;

    /// Get stream statistics
    async fn stream_stats(&self, stream: &str) -> Result<StreamStats, EventReplayError>;
}

/// Stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    /// Stream name
    pub name: String,

    /// Total messages in stream
    pub messages: u64,

    /// Total bytes in stream
    pub bytes: u64,

    /// First sequence number
    pub first_seq: u64,

    /// Last sequence number
    pub last_seq: u64,

    /// Number of consumers
    pub consumer_count: usize,
}

/// Errors during event replay
#[derive(Debug, thiserror::Error)]
pub enum EventReplayError {
    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Failed to read from stream: {0}")]
    ReadError(String),

    #[error("Failed to deserialize event: {0}")]
    DeserializationError(String),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("JetStream error: {0}")]
    JetStreamError(#[from] JetStreamError),
}

/// JetStream-based event replay implementation
pub struct JetStreamReplay<P: JetStreamPort> {
    port: P,
}

impl<P: JetStreamPort> JetStreamReplay<P> {
    /// Create a new JetStream replay
    pub fn new(port: P) -> Self {
        Self { port }
    }
}

#[async_trait::async_trait]
impl<P: JetStreamPort> EventReplay for JetStreamReplay<P> {
    async fn replay_all(&self, stream: &str) -> Result<ReplayResult, EventReplayError> {
        self.replay(stream, ReplayOptions::default()).await
    }

    async fn replay(
        &self,
        stream: &str,
        options: ReplayOptions,
    ) -> Result<ReplayResult, EventReplayError> {
        // Get stream info to determine total messages
        let info = self.port.stream_info(stream).await?;
        let stream_total = info.messages;

        // Create an ephemeral consumer for replay
        let consumer_config = crate::ports::JetStreamConsumerConfig {
            name: format!("replay-{}", Uuid::now_v7()),
            durable_name: None, // Ephemeral
            filter_subject: options.subject_filter.clone(),
            ack_policy: crate::ports::JsAckPolicy::None,
            deliver_policy: match (options.start_sequence, options.start_time) {
                (Some(seq), _) => crate::ports::JsDeliverPolicy::ByStartSequence(seq),
                (_, Some(time)) => crate::ports::JsDeliverPolicy::ByStartTime(
                    time.timestamp_nanos_opt().unwrap_or(0)
                ),
                _ => crate::ports::JsDeliverPolicy::All,
            },
            max_deliver: None,
            ack_wait_ns: None,
            description: Some("Event replay consumer".to_string()),
        };

        let consumer_info = self.port.create_consumer(stream, &consumer_config).await?;

        // Subscribe and collect events
        let mut subscription = self
            .port
            .subscribe(stream, &consumer_info.name, options.subject_filter.as_deref())
            .await?;

        let mut events = Vec::new();
        let max_events = options.max_events.unwrap_or(u64::MAX);

        while events.len() < max_events as usize {
            match subscription.next().await {
                Some(msg) => {
                    // Parse header values
                    let msg_correlation_id = msg.correlation_id()
                        .and_then(|s| Uuid::parse_str(s).ok());
                    let msg_causation_id = msg.causation_id()
                        .and_then(|s| Uuid::parse_str(s).ok());
                    let msg_event_id = msg.message_id()
                        .and_then(|s| Uuid::parse_str(s).ok());
                    let msg_source = msg.headers.get("CIM-Source").map(|s| s.to_string());
                    let msg_timestamp = msg.headers.get("CIM-Timestamp")
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|| {
                            // Fall back to message timestamp
                            DateTime::from_timestamp_nanos(msg.timestamp)
                        });

                    // Check correlation filter
                    if let Some(filter_corr) = options.correlation_id {
                        if msg_correlation_id != Some(filter_corr) {
                            continue;
                        }
                    }

                    // Deserialize the event
                    let event: DomainEvent = serde_json::from_slice(&msg.payload)
                        .map_err(|e| EventReplayError::DeserializationError(e.to_string()))?;

                    // Check aggregate filter
                    if let Some(filter_agg) = options.aggregate_id {
                        if event.aggregate_id() != filter_agg {
                            continue;
                        }
                    }

                    events.push(StoredEvent {
                        sequence: msg.sequence,
                        subject: msg.subject.clone(),
                        event,
                        event_id: msg_event_id,
                        correlation_id: msg_correlation_id,
                        causation_id: msg_causation_id,
                        timestamp: msg_timestamp,
                        source: msg_source,
                    });
                }
                None => break, // No more messages
            }
        }

        let first_sequence = events.first().map(|e| e.sequence);
        let last_sequence = events.last().map(|e| e.sequence);

        Ok(ReplayResult {
            events,
            first_sequence,
            last_sequence,
            stream_total,
        })
    }

    async fn current_position(&self, stream: &str) -> Result<u64, EventReplayError> {
        let info = self.port.stream_info(stream).await?;
        Ok(info.last_seq)
    }

    async fn stream_stats(&self, stream: &str) -> Result<StreamStats, EventReplayError> {
        let info = self.port.stream_info(stream).await?;
        Ok(StreamStats {
            name: info.name,
            messages: info.messages,
            bytes: info.bytes,
            first_seq: info.first_seq,
            last_seq: info.last_seq,
            consumer_count: info.consumer_count,
        })
    }
}

/// Trait for projections that can be rebuilt from events
pub trait Rebuildable: Default {
    /// Apply a single event to update the projection
    fn apply_event(&mut self, event: &StoredEvent) -> Result<(), RebuildError>;

    /// Finalize the projection after all events applied
    fn finalize(&mut self) -> Result<(), RebuildError> {
        Ok(())
    }
}

/// Errors during projection rebuilding
#[derive(Debug, thiserror::Error)]
pub enum RebuildError {
    #[error("Failed to apply event: {0}")]
    ApplyError(String),

    #[error("Projection inconsistency: {0}")]
    InconsistentState(String),

    #[error("Missing required data: {0}")]
    MissingData(String),
}

/// Rebuild a projection from a replay result
pub fn rebuild_projection<P: Rebuildable>(
    replay: &ReplayResult,
) -> Result<P, RebuildError> {
    let mut projection = P::default();

    for event in &replay.events {
        projection.apply_event(event)?;
    }

    projection.finalize()?;

    Ok(projection)
}

/// Snapshot for incremental replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<T> {
    /// The snapshot data
    pub data: T,

    /// Sequence number when snapshot was taken
    pub sequence: u64,

    /// Timestamp when snapshot was taken
    pub timestamp: DateTime<Utc>,

    /// Number of events since last snapshot
    pub events_since_snapshot: u64,
}

impl<T> Snapshot<T> {
    /// Create a new snapshot
    pub fn new(data: T, sequence: u64) -> Self {
        Self {
            data,
            sequence,
            timestamp: Utc::now(),
            events_since_snapshot: 0,
        }
    }

    /// Check if a new snapshot should be taken
    pub fn should_snapshot(&self, events_threshold: u64) -> bool {
        self.events_since_snapshot >= events_threshold
    }
}

// ============================================================================
// Event Store Query Capabilities
// ============================================================================

/// Query builder for event store searches
#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    /// Filter by event type pattern (e.g., "Key.*", "Certificate.*")
    pub event_type_pattern: Option<String>,

    /// Filter by aggregate ID
    pub aggregate_id: Option<Uuid>,

    /// Filter by correlation ID
    pub correlation_id: Option<Uuid>,

    /// Filter by causation ID
    pub causation_id: Option<Uuid>,

    /// Start time (inclusive)
    pub from_time: Option<DateTime<Utc>>,

    /// End time (exclusive)
    pub to_time: Option<DateTime<Utc>>,

    /// Start sequence (inclusive)
    pub from_sequence: Option<u64>,

    /// End sequence (exclusive)
    pub to_sequence: Option<u64>,

    /// Limit number of results
    pub limit: Option<usize>,

    /// Skip first N results
    pub offset: Option<usize>,

    /// Sort order (true = ascending by sequence)
    pub ascending: bool,
}

impl EventQuery {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            ascending: true,
            ..Default::default()
        }
    }

    /// Filter by event type pattern
    pub fn with_event_type(mut self, pattern: impl Into<String>) -> Self {
        self.event_type_pattern = Some(pattern.into());
        self
    }

    /// Filter by aggregate ID
    pub fn with_aggregate(mut self, id: Uuid) -> Self {
        self.aggregate_id = Some(id);
        self
    }

    /// Filter by correlation ID
    pub fn with_correlation(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Filter by causation ID
    pub fn with_causation(mut self, id: Uuid) -> Self {
        self.causation_id = Some(id);
        self
    }

    /// Filter events after this time
    pub fn from_time(mut self, time: DateTime<Utc>) -> Self {
        self.from_time = Some(time);
        self
    }

    /// Filter events before this time
    pub fn to_time(mut self, time: DateTime<Utc>) -> Self {
        self.to_time = Some(time);
        self
    }

    /// Filter events from this sequence number
    pub fn from_sequence(mut self, seq: u64) -> Self {
        self.from_sequence = Some(seq);
        self
    }

    /// Filter events up to this sequence number
    pub fn to_sequence(mut self, seq: u64) -> Self {
        self.to_sequence = Some(seq);
        self
    }

    /// Limit results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Skip first N results
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sort ascending by sequence
    pub fn ascending(mut self) -> Self {
        self.ascending = true;
        self
    }

    /// Sort descending by sequence
    pub fn descending(mut self) -> Self {
        self.ascending = false;
        self
    }
}

/// Result of an event query
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Matching events
    pub events: Vec<StoredEvent>,

    /// Total count of matching events (before limit/offset)
    pub total_count: usize,

    /// Whether there are more events available
    pub has_more: bool,

    /// The query that produced this result
    pub query: EventQuery,
}

impl QueryResult {
    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the number of events returned
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Iterate over events
    pub fn iter(&self) -> impl Iterator<Item = &StoredEvent> {
        self.events.iter()
    }

    /// Get the first event
    pub fn first(&self) -> Option<&StoredEvent> {
        self.events.first()
    }

    /// Get the last event
    pub fn last(&self) -> Option<&StoredEvent> {
        self.events.last()
    }
}

/// Aggregate statistics for a specific aggregate ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateStats {
    /// The aggregate ID
    pub aggregate_id: Uuid,

    /// Total events for this aggregate
    pub event_count: u64,

    /// First event sequence
    pub first_seq: u64,

    /// Last event sequence
    pub last_seq: u64,

    /// First event timestamp
    pub first_event_time: DateTime<Utc>,

    /// Last event timestamp
    pub last_event_time: DateTime<Utc>,

    /// Event type breakdown
    pub event_types: std::collections::HashMap<String, u64>,
}

/// Event type statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeStats {
    /// Event type name
    pub event_type: String,

    /// Total count of this event type
    pub count: u64,

    /// First occurrence timestamp
    pub first_occurrence: DateTime<Utc>,

    /// Last occurrence timestamp
    pub last_occurrence: DateTime<Utc>,
}

/// Time-series aggregation bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesBucket {
    /// Bucket start time
    pub start_time: DateTime<Utc>,

    /// Bucket end time
    pub end_time: DateTime<Utc>,

    /// Event count in this bucket
    pub count: u64,

    /// Event types in this bucket
    pub event_types: std::collections::HashMap<String, u64>,
}

/// Trait for executing event queries
#[async_trait::async_trait]
pub trait EventQueryExecutor: Send + Sync {
    /// Execute a query and return matching events
    async fn execute(&self, stream: &str, query: EventQuery) -> Result<QueryResult, EventReplayError>;

    /// Count events matching the query
    async fn count(&self, stream: &str, query: EventQuery) -> Result<u64, EventReplayError>;

    /// Get statistics for a specific aggregate
    async fn aggregate_stats(&self, stream: &str, aggregate_id: Uuid) -> Result<AggregateStats, EventReplayError>;

    /// Get event type statistics
    async fn event_type_stats(&self, stream: &str) -> Result<Vec<EventTypeStats>, EventReplayError>;

    /// Get time-series aggregation
    async fn time_series(
        &self,
        stream: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        bucket_duration: std::time::Duration,
    ) -> Result<Vec<TimeSeriesBucket>, EventReplayError>;
}

/// Execute a query against a replay result (in-memory)
pub fn execute_query(replay: &ReplayResult, query: &EventQuery) -> QueryResult {
    let mut events: Vec<StoredEvent> = replay.events.iter()
        .filter(|e| {
            // Filter by event type pattern
            if let Some(ref pattern) = query.event_type_pattern {
                let event_type = get_event_type(&e.event);
                if !matches_pattern(&event_type, pattern) {
                    return false;
                }
            }

            // Filter by aggregate ID
            if let Some(agg_id) = query.aggregate_id {
                if e.event.aggregate_id() != agg_id {
                    return false;
                }
            }

            // Filter by correlation ID
            if let Some(corr_id) = query.correlation_id {
                if e.correlation_id != Some(corr_id) {
                    return false;
                }
            }

            // Filter by causation ID
            if let Some(cause_id) = query.causation_id {
                if e.causation_id != Some(cause_id) {
                    return false;
                }
            }

            // Filter by time range
            if let Some(from) = query.from_time {
                if e.timestamp < from {
                    return false;
                }
            }
            if let Some(to) = query.to_time {
                if e.timestamp >= to {
                    return false;
                }
            }

            // Filter by sequence range
            if let Some(from_seq) = query.from_sequence {
                if e.sequence < from_seq {
                    return false;
                }
            }
            if let Some(to_seq) = query.to_sequence {
                if e.sequence >= to_seq {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect();

    // Sort
    if query.ascending {
        events.sort_by_key(|e| e.sequence);
    } else {
        events.sort_by_key(|e| std::cmp::Reverse(e.sequence));
    }

    let total_count = events.len();

    // Apply offset
    if let Some(offset) = query.offset {
        events = events.into_iter().skip(offset).collect();
    }

    // Apply limit
    let has_more = if let Some(limit) = query.limit {
        let has_more = events.len() > limit;
        events.truncate(limit);
        has_more
    } else {
        false
    };

    QueryResult {
        events,
        total_count,
        has_more,
        query: query.clone(),
    }
}

/// Get the event type name from a domain event
fn get_event_type(event: &DomainEvent) -> String {
    match event {
        DomainEvent::Person(e) => format!("Person.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::Organization(e) => format!("Organization.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::Location(e) => format!("Location.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::Certificate(e) => format!("Certificate.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::Key(e) => format!("Key.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::Delegation(e) => format!("Delegation.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::NatsOperator(e) => format!("NatsOperator.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::NatsAccount(e) => format!("NatsAccount.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::NatsUser(e) => format!("NatsUser.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::YubiKey(e) => format!("YubiKey.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::Relationship(e) => format!("Relationship.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::Manifest(e) => format!("Manifest.{}", std::any::type_name_of_val(e).split("::").last().unwrap_or("Unknown")),
        DomainEvent::Saga(e) => format!("Saga.{}", e.event_type()),
    }
}

/// Simple pattern matching (supports * as wildcard)
fn matches_pattern(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        text.starts_with(prefix)
    } else if pattern.starts_with("*") {
        let suffix = &pattern[1..];
        text.ends_with(suffix)
    } else if pattern.contains("*") {
        // Split on * and check prefix/suffix
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            text.starts_with(parts[0]) && text.ends_with(parts[1])
        } else {
            text == pattern
        }
    } else {
        text == pattern
    }
}

/// Compute aggregate statistics from a replay result
pub fn compute_aggregate_stats(replay: &ReplayResult, aggregate_id: Uuid) -> Option<AggregateStats> {
    let events: Vec<&StoredEvent> = replay.events.iter()
        .filter(|e| e.event.aggregate_id() == aggregate_id)
        .collect();

    if events.is_empty() {
        return None;
    }

    let mut event_types = std::collections::HashMap::new();
    for e in &events {
        let event_type = get_event_type(&e.event);
        *event_types.entry(event_type).or_insert(0u64) += 1;
    }

    let first = events.first()?;
    let last = events.last()?;

    Some(AggregateStats {
        aggregate_id,
        event_count: events.len() as u64,
        first_seq: first.sequence,
        last_seq: last.sequence,
        first_event_time: first.timestamp,
        last_event_time: last.timestamp,
        event_types,
    })
}

/// Compute event type statistics from a replay result
pub fn compute_event_type_stats(replay: &ReplayResult) -> Vec<EventTypeStats> {
    let mut stats_map: std::collections::HashMap<String, (u64, DateTime<Utc>, DateTime<Utc>)> =
        std::collections::HashMap::new();

    for event in &replay.events {
        let event_type = get_event_type(&event.event);
        stats_map.entry(event_type)
            .and_modify(|(count, first, last)| {
                *count += 1;
                if event.timestamp < *first {
                    *first = event.timestamp;
                }
                if event.timestamp > *last {
                    *last = event.timestamp;
                }
            })
            .or_insert((1, event.timestamp, event.timestamp));
    }

    stats_map.into_iter()
        .map(|(event_type, (count, first_occurrence, last_occurrence))| {
            EventTypeStats {
                event_type,
                count,
                first_occurrence,
                last_occurrence,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_options_builder() {
        let options = ReplayOptions::new()
            .with_subject_filter("keys.events.key.>")
            .with_start_sequence(100)
            .with_max_events(50);

        assert_eq!(options.subject_filter, Some("keys.events.key.>".to_string()));
        assert_eq!(options.start_sequence, Some(100));
        assert_eq!(options.max_events, Some(50));
    }

    #[test]
    fn test_replay_result_helpers() {
        let result = ReplayResult {
            events: vec![],
            first_sequence: None,
            last_sequence: None,
            stream_total: 0,
        };

        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_snapshot_should_snapshot() {
        let mut snapshot = Snapshot::new("test", 100);

        assert!(!snapshot.should_snapshot(10));

        snapshot.events_since_snapshot = 10;
        assert!(snapshot.should_snapshot(10));
    }

    #[test]
    fn test_stream_stats_creation() {
        let stats = StreamStats {
            name: "KEYS_EVENTS".to_string(),
            messages: 1000,
            bytes: 50000,
            first_seq: 1,
            last_seq: 1000,
            consumer_count: 2,
        };

        assert_eq!(stats.name, "KEYS_EVENTS");
        assert_eq!(stats.messages, 1000);
    }

    #[test]
    fn test_event_query_builder() {
        let id = Uuid::now_v7();
        let query = EventQuery::new()
            .with_event_type("Key.*")
            .with_aggregate(id)
            .limit(10)
            .offset(5)
            .descending();

        assert_eq!(query.event_type_pattern, Some("Key.*".to_string()));
        assert_eq!(query.aggregate_id, Some(id));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(5));
        assert!(!query.ascending);
    }

    #[test]
    fn test_query_result_helpers() {
        let result = QueryResult {
            events: vec![],
            total_count: 0,
            has_more: false,
            query: EventQuery::new(),
        };

        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
        assert!(result.first().is_none());
        assert!(result.last().is_none());
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("Key.KeyGenerated", "Key.KeyGenerated"));
        assert!(!matches_pattern("Key.KeyGenerated", "Key.KeyRevoked"));
    }

    #[test]
    fn test_matches_pattern_wildcard() {
        // Wildcard at end
        assert!(matches_pattern("Key.KeyGenerated", "Key.*"));
        assert!(matches_pattern("Key.KeyRevoked", "Key.*"));
        assert!(!matches_pattern("Certificate.Created", "Key.*"));

        // Wildcard at start
        assert!(matches_pattern("Key.KeyGenerated", "*Generated"));
        assert!(matches_pattern("Certificate.CertificateGenerated", "*Generated"));
        assert!(!matches_pattern("Key.KeyRevoked", "*Generated"));

        // Catch all
        assert!(matches_pattern("Any.Event.Type", "*"));
    }

    #[test]
    fn test_matches_pattern_middle_wildcard() {
        assert!(matches_pattern("Key.Something.Generated", "Key.*Generated"));
        assert!(!matches_pattern("Cert.Something.Generated", "Key.*Generated"));
    }

    #[test]
    fn test_aggregate_stats_structure() {
        let agg_id = Uuid::now_v7();
        let mut event_types = std::collections::HashMap::new();
        event_types.insert("Key.KeyGenerated".to_string(), 5);
        event_types.insert("Key.KeyRevoked".to_string(), 1);

        let stats = AggregateStats {
            aggregate_id: agg_id,
            event_count: 6,
            first_seq: 1,
            last_seq: 100,
            first_event_time: Utc::now(),
            last_event_time: Utc::now(),
            event_types,
        };

        assert_eq!(stats.aggregate_id, agg_id);
        assert_eq!(stats.event_count, 6);
        assert_eq!(stats.event_types.len(), 2);
    }

    #[test]
    fn test_event_type_stats_structure() {
        let stats = EventTypeStats {
            event_type: "Key.KeyGenerated".to_string(),
            count: 100,
            first_occurrence: Utc::now(),
            last_occurrence: Utc::now(),
        };

        assert_eq!(stats.event_type, "Key.KeyGenerated");
        assert_eq!(stats.count, 100);
    }

    #[test]
    fn test_time_series_bucket_structure() {
        let mut event_types = std::collections::HashMap::new();
        event_types.insert("Key.KeyGenerated".to_string(), 10);

        let bucket = TimeSeriesBucket {
            start_time: Utc::now(),
            end_time: Utc::now(),
            count: 10,
            event_types,
        };

        assert_eq!(bucket.count, 10);
        assert_eq!(bucket.event_types.len(), 1);
    }
}
