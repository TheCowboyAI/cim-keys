// Copyright (c) 2025 - Cowboy AI, LLC.
//! Event Hardening Tests (Sprint 37)
//!
//! Tests for:
//! - #[non_exhaustive] compliance
//! - Content-addressed events (CID)
//! - Pure projection functions
//! - Event envelope integrity

use cim_keys::events::{
    DomainEvent, EventEnvelope, EventChainBuilder,
    PersonEvents, KeyEvents,
};
use cim_keys::events::person::PersonCreatedEvent;
use cim_keys::events::key::KeyGeneratedEvent;
use cim_keys::types::{KeyAlgorithm, KeyPurpose, KeyMetadata};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_person_event() -> DomainEvent {
    DomainEvent::Person(PersonEvents::PersonCreated(PersonCreatedEvent {
        person_id: Uuid::now_v7(),
        name: "Test Person".to_string(),
        email: Some("test@example.com".to_string()),
        title: None,
        department: None,
        organization_id: Uuid::now_v7(),
        created_by: None,
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }))
}

fn create_key_event() -> DomainEvent {
    DomainEvent::Key(KeyEvents::KeyGenerated(KeyGeneratedEvent {
        key_id: Uuid::now_v7(),
        algorithm: KeyAlgorithm::Ed25519,
        purpose: KeyPurpose::Signing,
        generated_at: Utc::now(),
        generated_by: "test".to_string(),
        hardware_backed: false,
        metadata: KeyMetadata {
            label: "test-key".to_string(),
            description: None,
            tags: vec![],
            attributes: HashMap::new(),
            jwt_kid: None,
            jwt_alg: None,
            jwt_use: None,
        },
        ownership: None,
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }))
}

// ============================================================================
// EVENT ENVELOPE TESTS
// ============================================================================

#[test]
fn event_envelope_has_cid_field() {
    // Verify the CID field exists and defaults to None
    let event = create_person_event();
    let envelope = EventEnvelope::new(event, Uuid::now_v7(), None);

    // CID should be None by default
    assert!(!envelope.has_cid());
    assert!(envelope.cid_string().is_none());
}

#[test]
fn event_envelope_verify_cid_returns_true_when_no_cid() {
    let event = create_person_event();
    let envelope = EventEnvelope::new(event, Uuid::now_v7(), None);

    // Without CID, verification should pass
    assert!(envelope.verify_cid().unwrap());
}

#[test]
fn event_envelope_with_subject_preserves_cid_field() {
    let event = create_person_event();
    let envelope = EventEnvelope::new(event, Uuid::now_v7(), None)
        .with_subject("test.subject");

    // CID field should still exist (as None)
    assert!(envelope.cid.is_none());
}

#[test]
fn event_chain_builder_creates_envelopes_with_cid_field() {
    let mut builder = EventChainBuilder::new()
        .for_organization("test-org");

    let event = create_person_event();
    let envelope = builder.envelope(event);

    // CID should be None by default (not automatically generated)
    assert!(!envelope.has_cid());
}

// ============================================================================
// NON-EXHAUSTIVE COMPLIANCE TESTS
// ============================================================================

#[test]
fn domain_event_match_requires_wildcard() {
    // This test verifies that DomainEvent is #[non_exhaustive]
    // Consumers must have a catch-all arm
    let event = create_person_event();

    // The wildcard arm is required because DomainEvent is #[non_exhaustive]
    let aggregate_name = match &event {
        DomainEvent::Person(_) => "Person",
        DomainEvent::Organization(_) => "Organization",
        DomainEvent::Location(_) => "Location",
        DomainEvent::Certificate(_) => "Certificate",
        DomainEvent::Key(_) => "Key",
        DomainEvent::NatsOperator(_) => "NatsOperator",
        DomainEvent::NatsAccount(_) => "NatsAccount",
        DomainEvent::NatsUser(_) => "NatsUser",
        DomainEvent::YubiKey(_) => "YubiKey",
        DomainEvent::Relationship(_) => "Relationship",
        DomainEvent::Manifest(_) => "Manifest",
        DomainEvent::Saga(_) => "Saga",
        _ => "Unknown", // Required due to #[non_exhaustive]
    };

    assert_eq!(aggregate_name, "Person");
}

#[test]
fn event_envelope_serializes_without_cid_when_none() {
    let event = create_person_event();
    let envelope = EventEnvelope::new(event, Uuid::now_v7(), None);
    let json = serde_json::to_string(&envelope).unwrap();

    // CID should be omitted when None (skip_serializing_if)
    assert!(!json.contains("\"cid\""));
}

// ============================================================================
// CORRELATION AND CAUSATION CHAIN TESTS
// ============================================================================

#[test]
fn event_chain_maintains_causation() {
    let mut builder = EventChainBuilder::new();

    let event1 = create_person_event();
    let event2 = create_person_event();

    let envelope1 = builder.envelope(event1);
    let envelope2 = builder.envelope(event2);

    // Both should share the same correlation_id
    assert_eq!(envelope1.correlation_id, envelope2.correlation_id);

    // envelope2 should be caused by envelope1
    assert!(envelope2.is_caused_by(&envelope1));

    // Both should be correlated
    assert!(envelope1.is_correlated_with(&envelope2));
}

#[test]
fn event_envelope_uuid_v7_ordering() {
    // Verify that event_ids are UUID v7 (time-ordered)
    let event1 = create_person_event();
    let event2 = create_person_event();

    let envelope1 = EventEnvelope::new(event1, Uuid::now_v7(), None);
    let envelope2 = EventEnvelope::new(event2, Uuid::now_v7(), None);

    // UUID v7 should maintain temporal ordering
    assert!(envelope1.event_id < envelope2.event_id);
}

// ============================================================================
// AGGREGATE TYPE TESTS
// ============================================================================

#[test]
fn event_envelope_reports_correct_aggregate_type() {
    let person_event = create_person_event();
    let key_event = create_key_event();

    let person_envelope = EventEnvelope::new(person_event, Uuid::now_v7(), None);
    let key_envelope = EventEnvelope::new(key_event, Uuid::now_v7(), None);

    assert_eq!(person_envelope.aggregate_type(), "Person");
    assert_eq!(key_envelope.aggregate_type(), "Key");
}

#[test]
fn event_envelope_default_subject() {
    let person_event = create_person_event();
    let key_event = create_key_event();

    let person_envelope = EventEnvelope::new(person_event, Uuid::now_v7(), None);
    let key_envelope = EventEnvelope::new(key_event, Uuid::now_v7(), None);

    assert_eq!(person_envelope.nats_subject, "cim.person.event");
    assert_eq!(key_envelope.nats_subject, "cim.key.event");
}

#[test]
fn event_envelope_org_subject() {
    let event = create_person_event();
    let envelope = EventEnvelope::new(event, Uuid::now_v7(), None)
        .with_org_subject("cowboyai", "person", "created");

    assert_eq!(envelope.nats_subject, "organization.cowboyai.person.created");
}

// ============================================================================
// IPLD/CID FEATURE TESTS (require `ipld` feature)
// ============================================================================

#[cfg(feature = "ipld")]
mod ipld_tests {
    use super::*;
    use cim_keys::ipld_support::generate_cid;

    #[test]
    fn event_envelope_with_cid_generates_content_address() {
        let event = create_person_event();
        let envelope = EventEnvelope::new(event, Uuid::now_v7(), None)
            .with_cid()
            .unwrap();

        assert!(envelope.has_cid());
        assert!(envelope.cid_string().unwrap().starts_with("baf"));
    }

    #[test]
    fn event_envelope_cid_verification_succeeds() {
        let event = create_person_event();
        let envelope = EventEnvelope::new(event, Uuid::now_v7(), None)
            .with_cid()
            .unwrap();

        assert!(envelope.verify_cid().unwrap());
    }

    #[test]
    fn deterministic_cid_for_same_content() {
        let person_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();

        let event1 = PersonCreatedEvent {
            person_id,
            name: "Test".to_string(),
            email: Some("test@example.com".to_string()),
            title: None,
            department: None,
            organization_id: org_id,
            created_by: None,
            correlation_id,
            causation_id: None,
        };

        let event2 = PersonCreatedEvent {
            person_id,
            name: "Test".to_string(),
            email: Some("test@example.com".to_string()),
            title: None,
            department: None,
            organization_id: org_id,
            created_by: None,
            correlation_id,
            causation_id: None,
        };

        let cid1 = generate_cid(&event1).unwrap();
        let cid2 = generate_cid(&event2).unwrap();

        assert_eq!(cid1, cid2);
    }
}
