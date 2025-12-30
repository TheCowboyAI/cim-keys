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
[FUTURE] Remote subscribers / Event replay
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

## Offline vs Online Operation

### Offline Mode (Current)
- Events generated and logged locally
- No NATS connection required
- Suitable for air-gapped key generation scenarios

### Online Mode (Proposed)
- Events published to NATS as they're generated
- Real-time event streaming to leaf nodes/clusters
- Enables distributed key management across organizations

### Hybrid Mode (Best for cim-keys)
- Generate events offline (air-gapped key generation)
- Batch export events to encrypted SD card
- Later upload to NATS when connected to secure network
- Replay protection via event_id deduplication

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

### Phase 2: Batch Operations (v0.10.0)
- [ ] Batch event export to JSON files
- [ ] Batch event import from JSON files
- [ ] Batch NATS publish from offline events
- [ ] Deduplication via event_id checking

### Phase 3: Advanced Features (v0.11.0)
- [ ] Event replay capabilities
- [ ] Distributed event sourcing across leaf nodes
- [ ] Real-time graph synchronization between GUI instances
- [ ] Event-driven projections to other CIM modules

## Testing Strategy

1. **Unit Tests**: NatsEventPublisher, IpldObjectStore in isolation
2. **Integration Tests**: NATS JetStream + IPLD roundtrip
3. **E2E Tests**: GUI → GraphProjector → NATS → Subscriber
4. **Performance Tests**: Batch publish throughput benchmarks
5. **Security Tests**: Authentication, authorization, encryption verification

## Configuration Example

```toml
[nats]
enabled = true
url = "nats://leaf-node-1.local:4222"
credentials = "/path/to/nats.creds"
tls_cert = "/path/to/client-cert.pem"
tls_key = "/path/to/client-key.pem"
tls_ca = "/path/to/ca-cert.pem"

[nats.jetstream]
stream_name = "CIM_GRAPH_EVENTS"
object_store_bucket = "cim-graph-payloads"

[offline]
# If true, events are logged but not published to NATS
offline_mode = false
# Where to store offline events for later batch upload
offline_storage = "/mnt/encrypted/cim-keys/events"
```

## Benefits

1. **Event Sourcing**: Complete audit trail of all key operations
2. **Distributed Systems**: Multiple leaf nodes can subscribe to events
3. **Replay**: Reconstruct state by replaying events from NATS
4. **Integration**: Other CIM modules can subscribe to graph events
5. **Scalability**: NATS JetStream handles high-throughput event streams
6. **Content Addressing**: IPLD ensures payload immutability and deduplication

## Future Enhancements

1. **Event Schema Registry**: Validate events against schemas before publishing
2. **Conflict Resolution**: Detect and resolve conflicting events from multiple sources
3. **Snapshot Mechanism**: Periodic snapshots to optimize replay performance
4. **Event Versioning**: Handle schema evolution gracefully
5. **Query Interface**: GraphQL/REST API over event store for external systems
