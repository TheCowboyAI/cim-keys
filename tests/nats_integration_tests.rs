// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Integration Tests
//!
//! These tests verify real NATS connectivity and JetStream operations.
//! They require a running NATS server with JetStream enabled.
//!
//! ## Running the Tests
//!
//! 1. Start NATS with JetStream:
//!    ```bash
//!    nats-server -js
//!    ```
//!
//! 2. Run the tests:
//!    ```bash
//!    NATS_URL=nats://localhost:4222 cargo test --test nats_integration_tests --features nats-client
//!    ```
//!
//! ## Environment Variables
//!
//! - `NATS_URL`: NATS server URL (default: nats://localhost:4222)
//! - `NATS_STREAM_PREFIX`: Stream prefix for test isolation (default: TEST_CIM_KEYS)

#![cfg(feature = "nats-client")]

use std::env;
use std::time::Duration;
use async_nats::jetstream::{self, stream::Config as StreamConfig};
use chrono::Utc;
use uuid::Uuid;

/// Get NATS URL from environment or use default
fn nats_url() -> String {
    env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

/// Get stream prefix for test isolation
fn stream_prefix() -> String {
    env::var("NATS_STREAM_PREFIX").unwrap_or_else(|_| "TEST_CIM_KEYS".to_string())
}

/// Generate a unique stream name for each test run
fn unique_stream_name(base: &str) -> String {
    let uuid_suffix = Uuid::now_v7().to_string()[..8].to_string();
    format!("{}_{}", base, uuid_suffix)
}

/// Skip tests if NATS is not available
macro_rules! skip_if_no_nats {
    () => {
        match check_nats_available().await {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Skipping test: NATS not available - {}", e);
                return;
            }
        }
    };
}

/// Check if NATS server is reachable
async fn check_nats_available() -> Result<(), String> {
    let url = nats_url();
    let client = async_nats::connect(&url)
        .await
        .map_err(|e| format!("Failed to connect to NATS at {}: {}", url, e))?;

    // Verify JetStream is available
    let _js = jetstream::new(client);
    Ok(())
}

// =============================================================================
// Connection Tests
// =============================================================================

#[tokio::test]
async fn test_nats_connection() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();

    // Connection should be established
    assert!(client.connection_state() == async_nats::connection::State::Connected);

    // Flush to ensure connection is active
    client.flush().await.unwrap();
}

#[tokio::test]
async fn test_jetstream_available() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    // Get account info to verify JetStream
    let account_info = js.get_stream("nonexistent_stream_12345").await;

    // Should get a "stream not found" error, not a "JetStream not available" error
    assert!(account_info.is_err());
    let err = account_info.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("stream not found") || err_str.contains("not found"),
        "Expected 'stream not found' error, got: {}", err_str
    );
}

// =============================================================================
// Stream Creation Tests
// =============================================================================

#[tokio::test]
async fn test_create_stream() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    let stream_name = unique_stream_name(&format!("{}_CREATE", stream_prefix()));

    // Create a test stream
    let mut stream = js.create_stream(StreamConfig {
        name: stream_name.clone(),
        subjects: vec![format!("{}.>", stream_name.to_lowercase())],
        max_messages: 1000,
        max_bytes: 1024 * 1024,  // 1MB
        ..Default::default()
    }).await.unwrap();

    // Verify stream exists
    let info = stream.info().await.unwrap();
    assert_eq!(info.config.name, stream_name);
    assert_eq!(info.state.messages, 0);

    // Cleanup
    js.delete_stream(&stream_name).await.unwrap();
}

// =============================================================================
// Publish/Subscribe Tests
// =============================================================================

#[tokio::test]
async fn test_publish_to_stream() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    let stream_name = unique_stream_name(&format!("{}_PUB", stream_prefix()));
    let subject = format!("{}.events.test", stream_name.to_lowercase());

    // Create stream
    js.create_stream(StreamConfig {
        name: stream_name.clone(),
        subjects: vec![format!("{}.>", stream_name.to_lowercase())],
        ..Default::default()
    }).await.unwrap();

    // Publish a message
    let payload = serde_json::json!({
        "event_id": Uuid::now_v7().to_string(),
        "event_type": "TestEvent",
        "timestamp": Utc::now().to_rfc3339(),
        "data": {"message": "Hello from integration test"}
    });

    let ack = js.publish(subject, payload.to_string().into())
        .await
        .unwrap()
        .await
        .unwrap();

    // Verify acknowledgement
    assert_eq!(ack.stream, stream_name);
    assert_eq!(ack.sequence, 1);

    // Verify message count
    let mut stream = js.get_stream(&stream_name).await.unwrap();
    let info = stream.info().await.unwrap();
    assert_eq!(info.state.messages, 1);

    // Cleanup
    js.delete_stream(&stream_name).await.unwrap();
}

#[tokio::test]
async fn test_publish_multiple_messages() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    let stream_name = unique_stream_name(&format!("{}_MULTI", stream_prefix()));
    let subject_base = stream_name.to_lowercase();

    // Create stream
    js.create_stream(StreamConfig {
        name: stream_name.clone(),
        subjects: vec![format!("{}.>", subject_base)],
        ..Default::default()
    }).await.unwrap();

    // Publish multiple messages to different subjects
    let subjects = vec![
        format!("{}.events.key.generated", subject_base),
        format!("{}.events.key.rotated", subject_base),
        format!("{}.events.certificate.issued", subject_base),
    ];

    for (i, subject) in subjects.iter().enumerate() {
        let payload = serde_json::json!({
            "sequence": i + 1,
            "subject": subject,
        });
        js.publish(subject.clone(), payload.to_string().into())
            .await
            .unwrap()
            .await
            .unwrap();
    }

    // Verify message count
    let mut stream = js.get_stream(&stream_name).await.unwrap();
    let info = stream.info().await.unwrap();
    assert_eq!(info.state.messages, 3);

    // Cleanup
    js.delete_stream(&stream_name).await.unwrap();
}

// =============================================================================
// Consumer Tests
// =============================================================================

#[tokio::test]
async fn test_consume_messages() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    let stream_name = unique_stream_name(&format!("{}_CONSUME", stream_prefix()));
    let subject = format!("{}.events.test", stream_name.to_lowercase());

    // Create stream
    js.create_stream(StreamConfig {
        name: stream_name.clone(),
        subjects: vec![format!("{}.>", stream_name.to_lowercase())],
        ..Default::default()
    }).await.unwrap();

    // Publish messages
    for i in 1..=5 {
        let payload = format!(r#"{{"sequence": {}}}"#, i);
        js.publish(subject.clone(), payload.into())
            .await
            .unwrap()
            .await
            .unwrap();
    }

    // Create consumer
    let stream = js.get_stream(&stream_name).await.unwrap();
    let consumer = stream.create_consumer(jetstream::consumer::pull::Config {
        name: Some(format!("{}_consumer", stream_name)),
        durable_name: Some(format!("{}_consumer", stream_name)),
        ..Default::default()
    }).await.unwrap();

    // Consume messages
    let mut messages = consumer.fetch().max_messages(10).messages().await.unwrap();
    let mut count = 0;

    while let Some(msg) = messages.next().await {
        let msg = msg.unwrap();
        count += 1;
        msg.ack().await.unwrap();
    }

    assert_eq!(count, 5);

    // Cleanup
    js.delete_stream(&stream_name).await.unwrap();
}

// =============================================================================
// Subject Pattern Tests
// =============================================================================

#[tokio::test]
async fn test_subject_wildcard_subscription() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    let stream_name = unique_stream_name(&format!("{}_WILD", stream_prefix()));
    let subject_base = stream_name.to_lowercase();

    // Create stream with wildcard subject
    js.create_stream(StreamConfig {
        name: stream_name.clone(),
        subjects: vec![format!("{}.events.>", subject_base)],
        ..Default::default()
    }).await.unwrap();

    // Publish to various subjects
    let event_subjects = vec![
        format!("{}.events.key.generated", subject_base),
        format!("{}.events.key.rotated", subject_base),
        format!("{}.events.certificate.issued", subject_base),
        format!("{}.events.certificate.revoked", subject_base),
    ];

    for subject in &event_subjects {
        let payload = format!(r#"{{"subject": "{}"}}"#, subject);
        js.publish(subject.clone(), payload.into())
            .await
            .unwrap()
            .await
            .unwrap();
    }

    // Verify all messages captured
    let mut stream = js.get_stream(&stream_name).await.unwrap();
    let info = stream.info().await.unwrap();
    assert_eq!(info.state.messages, 4);

    // Cleanup
    js.delete_stream(&stream_name).await.unwrap();
}

// =============================================================================
// Message Deduplication Tests
// =============================================================================

#[tokio::test]
async fn test_message_deduplication() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    let stream_name = unique_stream_name(&format!("{}_DEDUP", stream_prefix()));
    let subject = format!("{}.events.test", stream_name.to_lowercase());

    // Create stream with deduplication window
    js.create_stream(StreamConfig {
        name: stream_name.clone(),
        subjects: vec![format!("{}.>", stream_name.to_lowercase())],
        duplicate_window: Duration::from_secs(60),
        ..Default::default()
    }).await.unwrap();

    // Publish same message twice with same Nats-Msg-Id
    let msg_id = Uuid::now_v7().to_string();
    let payload = r#"{"data": "test"}"#;

    for _ in 0..2 {
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Nats-Msg-Id", msg_id.as_str());

        js.publish_with_headers(subject.clone(), headers, payload.into())
            .await
            .unwrap()
            .await
            .unwrap();
    }

    // Only one message should be stored (deduplicated)
    let mut stream = js.get_stream(&stream_name).await.unwrap();
    let info = stream.info().await.unwrap();
    assert_eq!(info.state.messages, 1, "Expected deduplication to result in 1 message");

    // Cleanup
    js.delete_stream(&stream_name).await.unwrap();
}

// =============================================================================
// Event Envelope Tests
// =============================================================================

#[tokio::test]
async fn test_event_envelope_roundtrip() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    let stream_name = unique_stream_name(&format!("{}_ENV", stream_prefix()));
    let subject = format!("{}.events.test", stream_name.to_lowercase());

    // Create stream
    js.create_stream(StreamConfig {
        name: stream_name.clone(),
        subjects: vec![format!("{}.>", stream_name.to_lowercase())],
        ..Default::default()
    }).await.unwrap();

    // Create a structured event envelope
    let event_id = Uuid::now_v7();
    let correlation_id = Uuid::now_v7();
    let envelope = serde_json::json!({
        "event_id": event_id.to_string(),
        "correlation_id": correlation_id.to_string(),
        "causation_id": null,
        "timestamp": Utc::now().to_rfc3339(),
        "event_type": "KeyGenerated",
        "aggregate_type": "Key",
        "aggregate_id": Uuid::now_v7().to_string(),
        "data": {
            "key_id": Uuid::now_v7().to_string(),
            "algorithm": "Ed25519",
            "purpose": "Signing"
        }
    });

    // Publish
    js.publish(subject.clone(), serde_json::to_vec(&envelope).unwrap().into())
        .await
        .unwrap()
        .await
        .unwrap();

    // Consume and verify
    let stream = js.get_stream(&stream_name).await.unwrap();
    let consumer = stream.create_consumer(jetstream::consumer::pull::Config {
        name: Some("test_consumer".to_string()),
        ..Default::default()
    }).await.unwrap();

    let mut messages = consumer.fetch().max_messages(1).messages().await.unwrap();
    if let Some(msg) = messages.next().await {
        let msg = msg.unwrap();
        let received: serde_json::Value = serde_json::from_slice(&msg.payload).unwrap();

        assert_eq!(received["event_id"], event_id.to_string());
        assert_eq!(received["correlation_id"], correlation_id.to_string());
        assert_eq!(received["event_type"], "KeyGenerated");

        msg.ack().await.unwrap();
    } else {
        panic!("Expected to receive a message");
    }

    // Cleanup
    js.delete_stream(&stream_name).await.unwrap();
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_publish_to_nonexistent_stream() {
    skip_if_no_nats!();

    let url = nats_url();
    let client = async_nats::connect(&url).await.unwrap();
    let js = jetstream::new(client);

    // Try to publish to a subject with no matching stream
    let result = js.publish("nonexistent.subject.12345", "test".into()).await;

    // Should fail because no stream matches this subject
    // The publish itself might succeed but the ack will fail
    if let Ok(ack_future) = result {
        let ack_result = ack_future.await;
        assert!(ack_result.is_err(), "Expected error when publishing to non-existent stream");
    }
}

// =============================================================================
// Cleanup Helper
// =============================================================================

/// Cleanup test streams (call after test suite if needed)
#[allow(dead_code)]
async fn cleanup_test_streams() {
    if let Ok(()) = check_nats_available().await {
        let url = nats_url();
        let client = async_nats::connect(&url).await.unwrap();
        let js = jetstream::new(client);

        let prefix = stream_prefix();

        // List and delete all test streams
        let mut streams = js.streams();
        while let Some(stream) = streams.try_next().await.unwrap() {
            if stream.config.name.starts_with(&prefix) {
                let _ = js.delete_stream(&stream.config.name).await;
            }
        }
    }
}

use futures::StreamExt;
use futures::TryStreamExt;
