# NATS Streaming Architecture for cim-keys

## Overview

This document describes the event streaming architecture for persisting cim-graph events from the cim-keys GUI to NATS JetStream and IPLD object storage.

## Current State (v0.8.0)

The functorial projection layer is complete and functional:

- **GraphProjector** lifts domain events (PersonEvent, OrganizationEvent) to cim-graph events
- **GUI Integration** demonstrates event generation in real-time
- **Test Coverage** validates functor properties and causation chains

**What's Missing**: Persistence layer to stream events to NATS and store payloads in IPLD.

## Proposed Architecture

### Event Flow

```
User Action (GUI)
  ↓
Local GUI Event (NodeCreated, EdgeCreated, etc.)
  ↓
Domain Event (PersonCreated, OrganizationCreated, etc.)
  ↓
GraphProjector.lift_*_event()
  ↓
cim-graph Events (BoundedContextCreated, AggregateAdded, etc.)
  ↓
[NEW] NatsEventPublisher
  ↓
NATS JetStream + IPLD Storage
  ↓
JSON Projection → Encrypted SD Card
```

### Components

#### 1. NatsEventPublisher (New)

Location: `src/adapters/nats_publisher.rs`

```rust
pub struct NatsEventPublisher {
    nats_client: async_nats::Client,
    jetstream: async_nats::jetstream::Context,
    ipld_store: IpldObjectStore,
}

impl NatsEventPublisher {
    /// Publish a single cim-graph event
    pub async fn publish_event(&self, event: &GraphEvent) -> Result<(), PublishError> {
        // 1. Serialize event payload to IPLD
        let cid = self.ipld_store.put(&event.payload).await?;

        // 2. Create event envelope with CID reference
        let envelope = EventEnvelope {
            event_id: event.event_id,
            aggregate_id: event.aggregate_id,
            correlation_id: event.correlation_id,
            causation_id: event.causation_id,
            payload_cid: cid,
            timestamp: Utc::now(),
        };

        // 3. Publish to NATS subject
        let subject = format!("cim.graph.{}.events", self.context_name());
        self.jetstream.publish(subject, serde_json::to_vec(&envelope)?).await?;

        Ok(())
    }

    /// Publish a batch of events (maintains causation order)
    pub async fn publish_batch(&self, events: Vec<GraphEvent>) -> Result<(), PublishError> {
        for event in events {
            self.publish_event(&event).await?;
        }
        Ok(())
    }
}
```

#### 2. IPLD Object Store Integration

Location: `src/adapters/ipld_store.rs`

```rust
pub struct IpldObjectStore {
    store: async_nats::jetstream::object_store::ObjectStore,
}

impl IpldObjectStore {
    /// Store event payload and return CID
    pub async fn put(&self, payload: &EventPayload) -> Result<Cid, StoreError> {
        let bytes = serde_json::to_vec(payload)?;
        let cid = generate_cid(&bytes, ContentType::DagJson)?;

        // Store in NATS object store with CID as key
        self.store.put(&cid.to_string(), bytes.into()).await?;

        Ok(cid)
    }

    /// Retrieve event payload by CID
    pub async fn get(&self, cid: &Cid) -> Result<EventPayload, StoreError> {
        let bytes = self.store.get(&cid.to_string()).await?;
        let payload = serde_json::from_slice(&bytes)?;
        Ok(payload)
    }
}
```

#### 3. GUI Integration Points

Location: `src/gui.rs` (modifications)

```rust
impl CimKeysApp {
    async fn publish_to_nats(&self, graph_events: Vec<GraphEvent>) -> Result<(), PublishError> {
        if let Some(publisher) = &self.nats_publisher {
            publisher.publish_batch(graph_events).await?;
            tracing::info!("✅ Published {} events to NATS", graph_events.len());
        } else {
            tracing::warn!("⚠️  NATS publisher not configured - events not persisted");
        }
        Ok(())
    }
}
```

### NATS Subject Hierarchy

Events are published to subjects following CIM naming conventions:

```
cim.graph.{context}.events.{event_type}

Examples:
- cim.graph.person.events.context.bounded_context_created
- cim.graph.person.events.context.aggregate_added
- cim.graph.organization.events.context.bounded_context_created
- cim.graph.organization.events.concept.concept_created
```

### JetStream Stream Configuration

Stream: `CIM_GRAPH_EVENTS`

```json
{
  "name": "CIM_GRAPH_EVENTS",
  "subjects": ["cim.graph.*.events.>"],
  "retention": "limits",
  "max_age": 31536000000000000,  // 1 year in nanoseconds
  "max_msgs": -1,  // Unlimited
  "storage": "file",
  "discard": "old",
  "num_replicas": 3,  // High availability
  "duplicate_window": 120000000000  // 2 minutes
}
```

### Event Envelope Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique event identifier (UUID v7 for time ordering)
    pub event_id: Uuid,

    /// Aggregate this event belongs to
    pub aggregate_id: Uuid,

    /// Correlation ID (shared across related events)
    pub correlation_id: Uuid,

    /// Causation ID (what caused this event)
    pub causation_id: Option<Uuid>,

    /// CID pointing to IPLD-stored payload
    pub payload_cid: Cid,

    /// When this event was published
    pub timestamp: DateTime<Utc>,

    /// Optional: Source system identifier
    pub source: Option<String>,
}
```

## Air-Gapped Operation (The Only Mode)

**cim-keys operates EXCLUSIVELY in air-gapped mode.** There is no "online mode".

### Architecture
- NATS runs on **localhost only** (127.0.0.1:4222)
- Events flow through localhost NATS as an event bus
- All events are **projected to JSON files** on the encrypted SD card
- The SD card is **physically transported** to target systems
- No network connectivity - ever

### Event Flow
```
GUI Event → Localhost NATS → JSON Projection → Encrypted SD Card
                                                      ↓
                                        Physical Transport to Target
```

### Why Localhost NATS?
- Provides JetStream for event ordering and replay
- Provides IPLD object store for content-addressed payloads
- Standard NATS API for consistency with rest of CIM ecosystem
- Event bus functionality without network exposure

## Security Considerations

1. **TLS Encryption**: All NATS connections use TLS 1.3+
2. **Authentication**: JWT-based authentication with operator/account/user hierarchy
3. **Authorization**: Subject-based permissions (only authorized users can publish to cim.graph.*)
4. **Audit Trail**: All event publications logged with publisher identity
5. **Payload Encryption**: Consider encrypting IPLD payloads for sensitive key material

## Implementation Roadmap

### Phase 1: Core Infrastructure (v0.9.0)
- [ ] Implement NatsEventPublisher
- [ ] Implement IpldObjectStore
- [ ] Add NATS connection configuration to CimKeysApp
- [ ] Add optional NATS publishing to GUI event handlers

### Phase 2: Enhanced Projections (v0.10.0)
- [ ] Batch event export to JSON files
- [ ] Batch event import from JSON files
- [ ] SD card manifest generation
- [ ] Event deduplication via event_id

### Phase 3: Advanced Features (v0.11.0)
- [ ] Event replay from JSON files
- [ ] State reconstruction from event history
- [ ] Import PKI from other SD cards
- [ ] YubiKey state synchronization

## Testing Strategy

1. **Unit Tests**: NatsEventPublisher, IpldObjectStore in isolation
2. **Integration Tests**: NATS JetStream + IPLD roundtrip
3. **E2E Tests**: GUI → GraphProjector → NATS → Subscriber
4. **Performance Tests**: Batch publish throughput benchmarks
5. **Security Tests**: Authentication, authorization, encryption verification

## Configuration Example

```toml
[nats]
# Localhost NATS - no network binding
enabled = true
url = "nats://127.0.0.1:4222"

[nats.jetstream]
stream_name = "CIM_KEYS_EVENTS"
object_store_bucket = "cim-keys-payloads"

[storage]
# All output to encrypted SD card
keys_output_dir = "/mnt/encrypted/cim-keys/keys"
events_dir = "/mnt/encrypted/cim-keys/events"
```

## Benefits

1. **Event Sourcing**: Complete audit trail of all key operations on SD card
2. **Portable State**: Encrypted SD card can be physically transported
3. **Replay**: Reconstruct state by replaying events from JSON files
4. **Standard API**: NATS API for consistency with CIM ecosystem
5. **Content Addressing**: IPLD ensures payload immutability and deduplication
6. **Air-Gapped Security**: Private keys never touch any network

## Future Enhancements

1. **Event Schema Registry**: Validate events against schemas before publishing
2. **Conflict Resolution**: Detect and resolve conflicting events from multiple sources
3. **Snapshot Mechanism**: Periodic snapshots to optimize replay performance
4. **Event Versioning**: Handle schema evolution gracefully
5. **Query Interface**: GraphQL/REST API over event store for external systems
