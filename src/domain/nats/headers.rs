// Copyright (c) 2025 - Cowboy AI, LLC.

//! CIM NATS Header Specification
//!
//! This module defines the standard headers used in CIM event messages.
//! Headers provide metadata for correlation, causation tracking, and
//! deduplication without requiring parsing of the message body.
//!
//! ## Required Headers
//!
//! All CIM events MUST include these headers:
//!
//! | Header | Description | Example |
//! |--------|-------------|---------|
//! | `CIM-Correlation-Id` | Groups related events | `01JXYZ...` (UUID v7) |
//! | `CIM-Causation-Id` | Parent event that caused this | `01JXYZ...` or same as event ID for roots |
//! | `CIM-Event-Type` | Qualified event type name | `KeyGenerated` |
//! | `CIM-Timestamp` | ISO 8601 timestamp | `2026-01-03T12:00:00Z` |
//! | `CIM-Source` | Source system identifier | `cim-keys` |
//! | `Nats-Msg-Id` | Deduplication ID (event ID) | `01JXYZ...` |
//!
//! ## Optional Headers
//!
//! | Header | Description | Example |
//! |--------|-------------|---------|
//! | `CIM-Aggregate-Id` | Aggregate root ID | `01JXYZ...` |
//! | `CIM-Aggregate-Type` | Aggregate type name | `Organization` |
//! | `CIM-Content-Type` | Payload format | `application/json` |
//! | `CIM-Schema-Version` | Event schema version | `1.0` |
//!
//! ## Usage
//!
//! ```ignore
//! use cim_keys::domain::nats::headers::{CimHeaders, CimHeaderBuilder};
//!
//! let headers = CimHeaderBuilder::new("KeyGenerated")
//!     .correlation_id(correlation_id)
//!     .causation_id(causation_id)
//!     .event_id(event_id)
//!     .aggregate("Organization", org_id)
//!     .build();
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Header key constants for CIM events
pub mod keys {
    /// Correlation ID header - groups related events
    pub const CORRELATION_ID: &str = "CIM-Correlation-Id";

    /// Causation ID header - parent event that caused this one
    pub const CAUSATION_ID: &str = "CIM-Causation-Id";

    /// Event type header - qualified event type name
    pub const EVENT_TYPE: &str = "CIM-Event-Type";

    /// Timestamp header - ISO 8601 timestamp
    pub const TIMESTAMP: &str = "CIM-Timestamp";

    /// Source header - source system identifier
    pub const SOURCE: &str = "CIM-Source";

    /// Aggregate ID header - aggregate root ID
    pub const AGGREGATE_ID: &str = "CIM-Aggregate-Id";

    /// Aggregate type header - aggregate type name
    pub const AGGREGATE_TYPE: &str = "CIM-Aggregate-Type";

    /// Content type header - payload format
    pub const CONTENT_TYPE: &str = "CIM-Content-Type";

    /// Schema version header - event schema version
    pub const SCHEMA_VERSION: &str = "CIM-Schema-Version";

    /// NATS deduplication ID header
    pub const NATS_MSG_ID: &str = "Nats-Msg-Id";
}

/// Default source identifier for cim-keys
pub const CIM_KEYS_SOURCE: &str = "cim-keys";

/// Default content type for JSON payloads
pub const CONTENT_TYPE_JSON: &str = "application/json";

/// Current schema version
pub const SCHEMA_VERSION: &str = "1.0";

/// CIM message headers for NATS events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimHeaders {
    /// Unique event identifier (also used as Nats-Msg-Id)
    pub event_id: Uuid,

    /// Correlation ID - groups related events in a workflow
    pub correlation_id: Uuid,

    /// Causation ID - the event that directly caused this one
    /// For root events, this equals event_id
    pub causation_id: Uuid,

    /// Qualified event type name
    pub event_type: String,

    /// ISO 8601 timestamp when event was created
    pub timestamp: DateTime<Utc>,

    /// Source system identifier
    pub source: String,

    /// Optional aggregate ID
    pub aggregate_id: Option<Uuid>,

    /// Optional aggregate type name
    pub aggregate_type: Option<String>,

    /// Content type (default: application/json)
    pub content_type: String,

    /// Schema version
    pub schema_version: String,
}

impl CimHeaders {
    /// Create new CIM headers with required fields
    pub fn new(
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Uuid,
        event_type: impl Into<String>,
    ) -> Self {
        Self {
            event_id,
            correlation_id,
            causation_id,
            event_type: event_type.into(),
            timestamp: Utc::now(),
            source: CIM_KEYS_SOURCE.to_string(),
            aggregate_id: None,
            aggregate_type: None,
            content_type: CONTENT_TYPE_JSON.to_string(),
            schema_version: SCHEMA_VERSION.to_string(),
        }
    }

    /// Create headers for a root event (no parent causation)
    pub fn root(event_id: Uuid, correlation_id: Uuid, event_type: impl Into<String>) -> Self {
        Self::new(event_id, correlation_id, event_id, event_type)
    }

    /// Set aggregate information
    pub fn with_aggregate(mut self, aggregate_type: impl Into<String>, aggregate_id: Uuid) -> Self {
        self.aggregate_type = Some(aggregate_type.into());
        self.aggregate_id = Some(aggregate_id);
        self
    }

    /// Set source system
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }

    /// Convert to a HashMap suitable for NATS headers
    #[cfg(feature = "nats-client")]
    pub fn to_nats_headers(&self) -> async_nats::HeaderMap {
        use async_nats::HeaderMap;

        let mut headers = HeaderMap::new();

        // Required headers
        headers.insert(keys::CORRELATION_ID, self.correlation_id.to_string().as_str());
        headers.insert(keys::CAUSATION_ID, self.causation_id.to_string().as_str());
        headers.insert(keys::EVENT_TYPE, self.event_type.as_str());
        headers.insert(keys::TIMESTAMP, self.timestamp.to_rfc3339().as_str());
        headers.insert(keys::SOURCE, self.source.as_str());
        headers.insert(keys::NATS_MSG_ID, self.event_id.to_string().as_str());

        // Optional headers
        if let Some(agg_id) = &self.aggregate_id {
            headers.insert(keys::AGGREGATE_ID, agg_id.to_string().as_str());
        }
        if let Some(agg_type) = &self.aggregate_type {
            headers.insert(keys::AGGREGATE_TYPE, agg_type.as_str());
        }

        headers.insert(keys::CONTENT_TYPE, self.content_type.as_str());
        headers.insert(keys::SCHEMA_VERSION, self.schema_version.as_str());

        headers
    }

    /// Convert to a standard HashMap (for non-NATS use)
    pub fn to_hashmap(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();

        map.insert(keys::CORRELATION_ID.to_string(), self.correlation_id.to_string());
        map.insert(keys::CAUSATION_ID.to_string(), self.causation_id.to_string());
        map.insert(keys::EVENT_TYPE.to_string(), self.event_type.clone());
        map.insert(keys::TIMESTAMP.to_string(), self.timestamp.to_rfc3339());
        map.insert(keys::SOURCE.to_string(), self.source.clone());
        map.insert(keys::NATS_MSG_ID.to_string(), self.event_id.to_string());

        if let Some(agg_id) = &self.aggregate_id {
            map.insert(keys::AGGREGATE_ID.to_string(), agg_id.to_string());
        }
        if let Some(agg_type) = &self.aggregate_type {
            map.insert(keys::AGGREGATE_TYPE.to_string(), agg_type.clone());
        }

        map.insert(keys::CONTENT_TYPE.to_string(), self.content_type.clone());
        map.insert(keys::SCHEMA_VERSION.to_string(), self.schema_version.clone());

        map
    }
}

/// Builder for CIM headers with fluent API
#[derive(Debug, Clone)]
pub struct CimHeaderBuilder {
    event_type: String,
    event_id: Option<Uuid>,
    correlation_id: Option<Uuid>,
    causation_id: Option<Uuid>,
    aggregate_id: Option<Uuid>,
    aggregate_type: Option<String>,
    source: String,
    content_type: String,
}

impl CimHeaderBuilder {
    /// Create a new header builder with event type
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            event_id: None,
            correlation_id: None,
            causation_id: None,
            aggregate_id: None,
            aggregate_type: None,
            source: CIM_KEYS_SOURCE.to_string(),
            content_type: CONTENT_TYPE_JSON.to_string(),
        }
    }

    /// Set the event ID (defaults to new UUID v7)
    pub fn event_id(mut self, id: Uuid) -> Self {
        self.event_id = Some(id);
        self
    }

    /// Set the correlation ID (defaults to event_id)
    pub fn correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Set the causation ID (defaults to event_id for root events)
    pub fn causation_id(mut self, id: Uuid) -> Self {
        self.causation_id = Some(id);
        self
    }

    /// Set aggregate information
    pub fn aggregate(mut self, aggregate_type: impl Into<String>, aggregate_id: Uuid) -> Self {
        self.aggregate_type = Some(aggregate_type.into());
        self.aggregate_id = Some(aggregate_id);
        self
    }

    /// Set source system
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set content type
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }

    /// Build the CIM headers
    pub fn build(self) -> CimHeaders {
        let event_id = self.event_id.unwrap_or_else(Uuid::now_v7);
        let correlation_id = self.correlation_id.unwrap_or(event_id);
        let causation_id = self.causation_id.unwrap_or(event_id);

        CimHeaders {
            event_id,
            correlation_id,
            causation_id,
            event_type: self.event_type,
            timestamp: Utc::now(),
            source: self.source,
            aggregate_id: self.aggregate_id,
            aggregate_type: self.aggregate_type,
            content_type: self.content_type,
            schema_version: SCHEMA_VERSION.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cim_headers_new() {
        let event_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();
        let causation_id = Uuid::now_v7();

        let headers = CimHeaders::new(event_id, correlation_id, causation_id, "KeyGenerated");

        assert_eq!(headers.event_id, event_id);
        assert_eq!(headers.correlation_id, correlation_id);
        assert_eq!(headers.causation_id, causation_id);
        assert_eq!(headers.event_type, "KeyGenerated");
        assert_eq!(headers.source, CIM_KEYS_SOURCE);
    }

    #[test]
    fn test_cim_headers_root() {
        let event_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();

        let headers = CimHeaders::root(event_id, correlation_id, "BootstrapStarted");

        assert_eq!(headers.event_id, event_id);
        assert_eq!(headers.correlation_id, correlation_id);
        // Root events have causation_id == event_id
        assert_eq!(headers.causation_id, event_id);
    }

    #[test]
    fn test_cim_headers_with_aggregate() {
        let event_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();
        let aggregate_id = Uuid::now_v7();

        let headers = CimHeaders::root(event_id, correlation_id, "OrganizationCreated")
            .with_aggregate("Organization", aggregate_id);

        assert_eq!(headers.aggregate_type, Some("Organization".to_string()));
        assert_eq!(headers.aggregate_id, Some(aggregate_id));
    }

    #[test]
    fn test_cim_header_builder() {
        let event_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();
        let causation_id = Uuid::now_v7();
        let aggregate_id = Uuid::now_v7();

        let headers = CimHeaderBuilder::new("KeyGenerated")
            .event_id(event_id)
            .correlation_id(correlation_id)
            .causation_id(causation_id)
            .aggregate("Organization", aggregate_id)
            .source("cim-keys-test")
            .build();

        assert_eq!(headers.event_id, event_id);
        assert_eq!(headers.correlation_id, correlation_id);
        assert_eq!(headers.causation_id, causation_id);
        assert_eq!(headers.event_type, "KeyGenerated");
        assert_eq!(headers.aggregate_type, Some("Organization".to_string()));
        assert_eq!(headers.aggregate_id, Some(aggregate_id));
        assert_eq!(headers.source, "cim-keys-test");
    }

    #[test]
    fn test_cim_header_builder_defaults() {
        let headers = CimHeaderBuilder::new("TestEvent").build();

        // event_id should be auto-generated
        assert!(!headers.event_id.is_nil());
        // correlation_id defaults to event_id
        assert_eq!(headers.correlation_id, headers.event_id);
        // causation_id defaults to event_id (root event)
        assert_eq!(headers.causation_id, headers.event_id);
        assert_eq!(headers.source, CIM_KEYS_SOURCE);
        assert_eq!(headers.content_type, CONTENT_TYPE_JSON);
    }

    #[test]
    fn test_to_hashmap() {
        let event_id = Uuid::now_v7();
        let correlation_id = Uuid::now_v7();

        let headers = CimHeaders::root(event_id, correlation_id, "TestEvent");
        let map = headers.to_hashmap();

        assert_eq!(map.get(keys::CORRELATION_ID), Some(&correlation_id.to_string()));
        assert_eq!(map.get(keys::CAUSATION_ID), Some(&event_id.to_string()));
        assert_eq!(map.get(keys::EVENT_TYPE), Some(&"TestEvent".to_string()));
        assert_eq!(map.get(keys::SOURCE), Some(&CIM_KEYS_SOURCE.to_string()));
        assert_eq!(map.get(keys::NATS_MSG_ID), Some(&event_id.to_string()));
    }

    #[test]
    fn test_header_key_constants() {
        // Ensure header keys follow expected format
        assert!(keys::CORRELATION_ID.starts_with("CIM-"));
        assert!(keys::CAUSATION_ID.starts_with("CIM-"));
        assert!(keys::EVENT_TYPE.starts_with("CIM-"));
        assert!(keys::TIMESTAMP.starts_with("CIM-"));
        assert!(keys::SOURCE.starts_with("CIM-"));
        assert!(keys::NATS_MSG_ID.starts_with("Nats-"));
    }
}
