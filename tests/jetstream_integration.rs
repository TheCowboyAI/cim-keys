// Copyright (c) 2025 - Cowboy AI, LLC.

//! JetStream Integration Tests
//!
//! Tests for event replay, projection rebuilding, and query capabilities.
//! Uses a mock JetStreamPort implementation for testing without a real NATS server.

use chrono::{Duration, Utc};
use cim_keys::domain::nats::replay::{
    EventQuery, ReplayOptions,
    ReplayResult, Snapshot, StoredEvent, StreamStats, TimeSeriesBucket,
    compute_aggregate_stats, compute_event_type_stats, execute_query,
};
use cim_keys::events::{DomainEvent, KeyEvents};
use cim_keys::types::{KeyAlgorithm, KeyPurpose, KeyMetadata};
use cim_keys::value_objects::ActorId;
use uuid::Uuid;
use std::collections::HashMap;

// =============================================================================
// Test Helpers
// =============================================================================

fn create_test_key_event(key_id: Uuid, seq: u64) -> StoredEvent {
    use cim_keys::events::key::KeyGeneratedEvent;

    let event = KeyGeneratedEvent {
        key_id,
        algorithm: KeyAlgorithm::Ed25519,
        purpose: KeyPurpose::Signing,
        generated_at: Utc::now(),
        generated_by: ActorId::system("test-user"),
        hardware_backed: false,
        metadata: KeyMetadata {
            label: "Test Key".to_string(),
            description: Some("A test key".to_string()),
            tags: vec![],
            attributes: HashMap::new(),
            jwt_kid: None,
            jwt_alg: None,
            jwt_use: None,
        },
        ownership: None,
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    };

    StoredEvent {
        sequence: seq,
        subject: "keys.events.key.generated".to_string(),
        event: DomainEvent::Key(KeyEvents::KeyGenerated(event)),
        event_id: Some(Uuid::now_v7()),
        correlation_id: Some(Uuid::now_v7()),
        causation_id: None,
        timestamp: Utc::now() - Duration::seconds(100 - seq as i64),
        source: Some("cim-keys".to_string()),
    }
}

fn create_test_replay_result(count: usize) -> ReplayResult {
    let events: Vec<StoredEvent> = (1..=count as u64)
        .map(|seq| create_test_key_event(Uuid::now_v7(), seq))
        .collect();

    let first_sequence = events.first().map(|e| e.sequence);
    let last_sequence = events.last().map(|e| e.sequence);

    ReplayResult {
        events,
        first_sequence,
        last_sequence,
        stream_total: count as u64,
    }
}

// =============================================================================
// ReplayOptions Tests
// =============================================================================

#[test]
fn test_replay_options_default() {
    let options = ReplayOptions::default();
    assert!(options.subject_filter.is_none());
    assert!(options.start_sequence.is_none());
    assert!(options.start_time.is_none());
    assert!(options.max_events.is_none());
    assert!(options.correlation_id.is_none());
    assert!(options.aggregate_id.is_none());
}

#[test]
fn test_replay_options_builder_chain() {
    let now = Utc::now();
    let corr_id = Uuid::now_v7();
    let agg_id = Uuid::now_v7();

    let options = ReplayOptions::new()
        .with_subject_filter("keys.events.>")
        .with_start_sequence(100)
        .with_start_time(now)
        .with_max_events(50)
        .with_correlation_id(corr_id)
        .with_aggregate_id(agg_id);

    assert_eq!(options.subject_filter, Some("keys.events.>".to_string()));
    assert_eq!(options.start_sequence, Some(100));
    assert_eq!(options.start_time, Some(now));
    assert_eq!(options.max_events, Some(50));
    assert_eq!(options.correlation_id, Some(corr_id));
    assert_eq!(options.aggregate_id, Some(agg_id));
}

// =============================================================================
// ReplayResult Tests
// =============================================================================

#[test]
fn test_replay_result_empty() {
    let result = ReplayResult {
        events: vec![],
        first_sequence: None,
        last_sequence: None,
        stream_total: 0,
    };

    assert!(result.is_empty());
    assert_eq!(result.len(), 0);
    assert_eq!(result.domain_events().len(), 0);
}

#[test]
fn test_replay_result_with_events() {
    let result = create_test_replay_result(5);

    assert!(!result.is_empty());
    assert_eq!(result.len(), 5);
    assert_eq!(result.first_sequence, Some(1));
    assert_eq!(result.last_sequence, Some(5));
    assert_eq!(result.stream_total, 5);
    assert_eq!(result.domain_events().len(), 5);
}

#[test]
fn test_replay_result_iteration() {
    let result = create_test_replay_result(3);
    let sequences: Vec<u64> = result.iter().map(|e| e.sequence).collect();
    assert_eq!(sequences, vec![1, 2, 3]);
}

// =============================================================================
// StoredEvent Tests
// =============================================================================

#[test]
fn test_stored_event_metadata() {
    let key_id = Uuid::now_v7();
    let stored = create_test_key_event(key_id, 42);

    assert_eq!(stored.sequence, 42);
    assert_eq!(stored.subject, "keys.events.key.generated");
    assert!(stored.event_id.is_some());
    assert!(stored.correlation_id.is_some());
    assert!(stored.source.is_some());
}

// =============================================================================
// EventQuery Tests
// =============================================================================

#[test]
fn test_event_query_default() {
    let query = EventQuery::new();
    assert!(query.event_type_pattern.is_none());
    assert!(query.aggregate_id.is_none());
    assert!(query.correlation_id.is_none());
    assert!(query.causation_id.is_none());
    assert!(query.from_time.is_none());
    assert!(query.to_time.is_none());
    assert!(query.from_sequence.is_none());
    assert!(query.to_sequence.is_none());
    assert!(query.limit.is_none());
    assert!(query.offset.is_none());
    assert!(query.ascending);
}

#[test]
fn test_event_query_full_builder() {
    let now = Utc::now();
    let agg_id = Uuid::now_v7();
    let corr_id = Uuid::now_v7();
    let cause_id = Uuid::now_v7();

    let query = EventQuery::new()
        .with_event_type("Key.*")
        .with_aggregate(agg_id)
        .with_correlation(corr_id)
        .with_causation(cause_id)
        .from_time(now - Duration::hours(1))
        .to_time(now)
        .from_sequence(10)
        .to_sequence(100)
        .limit(50)
        .offset(5)
        .descending();

    assert_eq!(query.event_type_pattern, Some("Key.*".to_string()));
    assert_eq!(query.aggregate_id, Some(agg_id));
    assert_eq!(query.correlation_id, Some(corr_id));
    assert_eq!(query.causation_id, Some(cause_id));
    assert!(query.from_time.is_some());
    assert!(query.to_time.is_some());
    assert_eq!(query.from_sequence, Some(10));
    assert_eq!(query.to_sequence, Some(100));
    assert_eq!(query.limit, Some(50));
    assert_eq!(query.offset, Some(5));
    assert!(!query.ascending);
}

#[test]
fn test_event_query_ascending_toggle() {
    let query = EventQuery::new().descending().ascending();
    assert!(query.ascending);

    let query = EventQuery::new().ascending().descending();
    assert!(!query.ascending);
}

// =============================================================================
// execute_query Tests
// =============================================================================

#[test]
fn test_execute_query_no_filter() {
    let replay = create_test_replay_result(10);
    let query = EventQuery::new();

    let result = execute_query(&replay, &query);

    assert_eq!(result.len(), 10);
    assert_eq!(result.total_count, 10);
    assert!(!result.has_more);
}

#[test]
fn test_execute_query_with_limit() {
    let replay = create_test_replay_result(10);
    let query = EventQuery::new().limit(5);

    let result = execute_query(&replay, &query);

    assert_eq!(result.len(), 5);
    assert_eq!(result.total_count, 10);
    assert!(result.has_more);
}

#[test]
fn test_execute_query_with_offset() {
    let replay = create_test_replay_result(10);
    let query = EventQuery::new().offset(5);

    let result = execute_query(&replay, &query);

    assert_eq!(result.len(), 5);
    // Sequences should be 6-10
    let first_seq = result.first().map(|e| e.sequence);
    assert_eq!(first_seq, Some(6));
}

#[test]
fn test_execute_query_with_limit_and_offset() {
    let replay = create_test_replay_result(20);
    let query = EventQuery::new().limit(5).offset(10);

    let result = execute_query(&replay, &query);

    assert_eq!(result.len(), 5);
    // Sequences should be 11-15
    let first_seq = result.first().map(|e| e.sequence);
    assert_eq!(first_seq, Some(11));
}

#[test]
fn test_execute_query_descending() {
    let replay = create_test_replay_result(5);
    let query = EventQuery::new().descending();

    let result = execute_query(&replay, &query);

    let sequences: Vec<u64> = result.iter().map(|e| e.sequence).collect();
    assert_eq!(sequences, vec![5, 4, 3, 2, 1]);
}

#[test]
fn test_execute_query_sequence_range() {
    let replay = create_test_replay_result(10);
    let query = EventQuery::new()
        .from_sequence(3)
        .to_sequence(8);

    let result = execute_query(&replay, &query);

    // Should include 3, 4, 5, 6, 7 (to_sequence is exclusive)
    assert_eq!(result.len(), 5);
    let sequences: Vec<u64> = result.iter().map(|e| e.sequence).collect();
    assert_eq!(sequences, vec![3, 4, 5, 6, 7]);
}

// =============================================================================
// QueryResult Tests
// =============================================================================

#[test]
fn test_query_result_helpers() {
    let replay = create_test_replay_result(5);
    let query = EventQuery::new();
    let result = execute_query(&replay, &query);

    assert!(!result.is_empty());
    assert_eq!(result.len(), 5);
    assert!(result.first().is_some());
    assert!(result.last().is_some());
    assert_eq!(result.first().unwrap().sequence, 1);
    assert_eq!(result.last().unwrap().sequence, 5);
}

// =============================================================================
// AggregateStats Tests
// =============================================================================

#[test]
fn test_compute_aggregate_stats_found() {
    // Create events with same aggregate ID (key_id)
    let agg_id = Uuid::now_v7();
    let events: Vec<StoredEvent> = (1..=5)
        .map(|seq| create_test_key_event(agg_id, seq))
        .collect();

    let replay = ReplayResult {
        events,
        first_sequence: Some(1),
        last_sequence: Some(5),
        stream_total: 5,
    };

    // For KeyEvents, aggregate_id returns key_id
    // The test passes agg_id as key_id to create_test_key_event
    let stats = compute_aggregate_stats(&replay, agg_id);
    assert!(stats.is_some());

    let stats = stats.unwrap();
    assert_eq!(stats.event_count, 5);
}

#[test]
fn test_compute_aggregate_stats_not_found() {
    let replay = create_test_replay_result(5);
    let random_id = Uuid::now_v7();

    let stats = compute_aggregate_stats(&replay, random_id);
    assert!(stats.is_none());
}

// =============================================================================
// EventTypeStats Tests
// =============================================================================

#[test]
fn test_compute_event_type_stats() {
    let replay = create_test_replay_result(10);
    let stats = compute_event_type_stats(&replay);

    // All events are KeyGenerated, so should have one entry
    assert_eq!(stats.len(), 1);
    assert!(stats[0].event_type.contains("KeyEvents"));
    assert_eq!(stats[0].count, 10);
}

// =============================================================================
// Snapshot Tests
// =============================================================================

#[test]
fn test_snapshot_creation() {
    let data = "test_projection_state";
    let snapshot = Snapshot::new(data, 100);

    assert_eq!(snapshot.data, "test_projection_state");
    assert_eq!(snapshot.sequence, 100);
    assert_eq!(snapshot.events_since_snapshot, 0);
}

#[test]
fn test_snapshot_should_snapshot_threshold() {
    let mut snapshot = Snapshot::new("data", 100);

    // Below threshold
    snapshot.events_since_snapshot = 9;
    assert!(!snapshot.should_snapshot(10));

    // At threshold
    snapshot.events_since_snapshot = 10;
    assert!(snapshot.should_snapshot(10));

    // Above threshold
    snapshot.events_since_snapshot = 15;
    assert!(snapshot.should_snapshot(10));
}

// =============================================================================
// StreamStats Tests
// =============================================================================

#[test]
fn test_stream_stats() {
    let stats = StreamStats {
        name: "KEYS_EVENTS".to_string(),
        messages: 1000,
        bytes: 500_000,
        first_seq: 1,
        last_seq: 1000,
        consumer_count: 3,
    };

    assert_eq!(stats.name, "KEYS_EVENTS");
    assert_eq!(stats.messages, 1000);
    assert_eq!(stats.bytes, 500_000);
    assert_eq!(stats.first_seq, 1);
    assert_eq!(stats.last_seq, 1000);
    assert_eq!(stats.consumer_count, 3);
}

// =============================================================================
// TimeSeriesBucket Tests
// =============================================================================

#[test]
fn test_time_series_bucket() {
    let now = Utc::now();
    let mut event_types = std::collections::HashMap::new();
    event_types.insert("Key.KeyGenerated".to_string(), 50);
    event_types.insert("Key.KeyRevoked".to_string(), 5);

    let bucket = TimeSeriesBucket {
        start_time: now - Duration::hours(1),
        end_time: now,
        count: 55,
        event_types,
    };

    assert_eq!(bucket.count, 55);
    assert_eq!(bucket.event_types.len(), 2);
    assert_eq!(*bucket.event_types.get("Key.KeyGenerated").unwrap(), 50);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_replay_result() {
    let replay = ReplayResult {
        events: vec![],
        first_sequence: None,
        last_sequence: None,
        stream_total: 0,
    };

    let query = EventQuery::new();
    let result = execute_query(&replay, &query);

    assert!(result.is_empty());
    assert_eq!(result.total_count, 0);
    assert!(!result.has_more);
}

#[test]
fn test_query_with_no_matches() {
    let replay = create_test_replay_result(10);
    let random_id = Uuid::now_v7();

    let query = EventQuery::new().with_correlation(random_id);
    let result = execute_query(&replay, &query);

    assert!(result.is_empty());
    assert_eq!(result.total_count, 0);
}

#[test]
fn test_large_offset_beyond_results() {
    let replay = create_test_replay_result(5);
    let query = EventQuery::new().offset(100);

    let result = execute_query(&replay, &query);

    assert!(result.is_empty());
}
