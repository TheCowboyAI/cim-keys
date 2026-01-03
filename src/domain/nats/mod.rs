// Copyright (c) 2025 - Cowboy AI, LLC.

//! NATS Bounded Context Module
//!
//! This module organizes the NATS bounded context with proper separation:
//! - **Entities**: NATS hierarchy types (Operator, Account, User)
//! - **Subjects**: Type-safe NATS subject naming algebra
//!
//! ## Subject Algebra
//!
//! NATS subjects follow semantic naming: `organization.unit.entity.operation`
//!
//! ```ignore
//! use cim_keys::domain::nats::subjects;
//!
//! let subject = subjects::keys::certificate_generate("cowboyai", "root");
//! // => "cowboyai.security.keys.certificate.generate.root"
//! ```

pub mod entities;
pub mod headers;
pub mod jetstream;
pub mod publisher;
pub mod replay;
pub mod subjects;

// Re-export entity types for backward compatibility
pub use entities::*;

// Re-export header types for event publishing
pub use headers::{
    CimHeaders,
    CimHeaderBuilder,
    keys as header_keys,
    CIM_KEYS_SOURCE,
    CONTENT_TYPE_JSON,
    SCHEMA_VERSION,
};

// Re-export JetStream configuration types
pub use jetstream::{
    StreamConfig,
    ConsumerConfig,
    RetentionPolicy,
    StorageType,
    AckPolicy,
    DeliverPolicy,
    KEYS_EVENTS_STREAM,
    KEYS_COMMANDS_STREAM,
    KEYS_SUBJECT_PREFIX,
    events as js_events,
    commands as js_commands,
};

// Re-export event publisher
pub use publisher::{EventPublisher, EventPublishError, PublishableEvents};

// Re-export event replay
pub use replay::{
    EventReplay,
    EventReplayError,
    JetStreamReplay,
    ReplayOptions,
    ReplayResult,
    StoredEvent,
    StreamStats,
    Rebuildable,
    RebuildError,
    Snapshot,
    rebuild_projection,
};

// Re-export event query capabilities
pub use replay::{
    EventQuery,
    QueryResult,
    AggregateStats,
    EventTypeStats,
    TimeSeriesBucket,
    EventQueryExecutor,
    execute_query,
    compute_aggregate_stats,
    compute_event_type_stats,
};

// Re-export subject algebra at module level
pub use subjects::{
    Subject,
    SubjectBuilder,
    SubjectToken,
    SubjectError,
    PermissionSet,
    // Factory modules
    organization as org_subjects,
    keys as key_subjects,
    infrastructure as infra_subjects,
    audit as audit_subjects,
    services as service_subjects,
};
