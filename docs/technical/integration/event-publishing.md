# Event Publishing Usage Guide

This guide demonstrates how to use the event publishing system in cim-keys for distributed event sourcing.

## Overview

The cim-keys event publishing system provides a functorial projection from domain events to cim-graph events, with optional publishing to NATS JetStream for distributed systems.

## Event Flow

```
User Action (GUI)
  ↓
GUI Event (NodeCreated, EdgeCreated, etc.)
  ↓
Domain Event (PersonCreated, OrganizationCreated, etc.)
  ↓
GraphProjector.lift_*_event()
  ↓
cim-graph Events (BoundedContextCreated, AggregateAdded, etc.)
  ↓
[When NATS enabled] EventEnvelope → NATS Subject
  ↓
[When NATS enabled] EventPayload → IPLD Object Store
```

## Configuration

### 1. Create Configuration File

```bash
cp config.example.toml config.toml
```

### 2. Configure for Your Environment

**Offline Mode (Air-Gapped Key Generation):**
```toml
mode = "Offline"

[nats]
enabled = false

[storage]
offline_events_dir = "/mnt/encrypted/cim-keys/events"
keys_output_dir = "/mnt/encrypted/cim-keys/keys"
```

**Online Mode (Real-Time Event Publishing):**
```toml
mode = "Online"

[nats]
enabled = true
url = "nats://leaf-node-1.local:4222"
stream_name = "CIM_GRAPH_EVENTS"
credentials_file = "/path/to/nats.creds"
```

**Hybrid Mode (Local + Queued Upload):**
```toml
mode = "Hybrid"

[nats]
enabled = false  # Enable later for batch upload

[storage]
offline_events_dir = "/mnt/encrypted/cim-keys/events"
enable_backup = true
backup_dir = "/backup/cim-keys"
```

## Usage Examples

### Basic Usage (Offline Mode)

```rust
use cim_keys::config::Config;
use cim_keys::graph_projection::GraphProjector;
use cim_domain_person::events::{PersonEvent, PersonCreated};

// Load configuration
let config = Config::from_file(&"config.toml".into())?;

// Create graph projector
let projector = GraphProjector::new();

// Create domain event
let domain_event = PersonEvent::PersonCreated(PersonCreated {
    person_id: EntityId::new(),
    name: PersonName::new("Alice".to_string(), "Smith".to_string()),
    source: "gui".to_string(),
    created_at: Utc::now(),
});

// Lift to cim-graph events
let graph_events = projector.lift_person_event(&domain_event)?;

// In offline mode, events are logged but not published
tracing::info!("Generated {} graph events", graph_events.len());
for (i, event) in graph_events.iter().enumerate() {
    tracing::debug!("Event {}: {:?}", i+1, event);
}
```

### NATS Publishing (Online Mode)

**NOTE: Full implementation pending. Current stub demonstrates architecture.**

```rust
use cim_keys::config::Config;
use cim_keys::adapters::{EventEnvelope, build_subject};

// Load configuration
let config = Config::from_file(&"config.toml".into())?;

if config.nats.enabled {
    // TODO: Initialize NATS publisher
    // let publisher = NatsEventPublisher::new(
    //     &config.nats.url,
    //     config.nats.stream_name,
    //     config.nats.source_id,
    // ).await?;

    // Build subject for event
    let subject = build_subject(
        &config.nats.subject_prefix,
        "person",
        &graph_event.payload,
    );

    tracing::info!("Would publish to subject: {}", subject);
    // subject: "cim.graph.person.events.context.bounded_context_created"
}
```

### Batch Upload (Hybrid Mode)

```rust
use cim_keys::config::Config;
use std::fs;

// 1. Generate keys offline (NATS disabled)
let config = Config::from_file(&"config.toml".into())?;
assert_eq!(config.mode, OperationalMode::Hybrid);

// ... generate keys and events offline ...

// 2. Later, when connected to secure network, enable NATS
let mut online_config = config.clone();
online_config.nats.enabled = true;

// 3. Read offline events and publish in batch
let events_dir = config.storage.offline_events_dir;
for entry in fs::read_dir(events_dir)? {
    let path = entry?.path();
    if path.extension() == Some("json".as_ref()) {
        // TODO: Parse event and publish
        // let event = serde_json::from_str(&fs::read_to_string(path)?)?;
        // publisher.publish_event(&event, "person").await?;
    }
}
```

## Subject Hierarchy

Events are published to semantic NATS subjects:

```
{prefix}.{context}.events.{payload_type}.{event_type}
```

### Examples:

**Person Events:**
- `cim.graph.person.events.context.bounded_context_created`
- `cim.graph.person.events.context.aggregate_added`

**Organization Events:**
- `cim.graph.organization.events.context.bounded_context_created`
- `cim.graph.organization.events.context.aggregate_added`

**Concept Events:**
- `cim.graph.person.events.concept.concept_defined`
- `cim.graph.pki.events.concept.properties_added`

## Event Envelope Structure

```json
{
  "event_id": "01936f3e-9d42-7b3c-8e1a-2f5d8c4a9b7e",
  "aggregate_id": "01936f3e-9d42-7b3c-8e1a-2f5d8c4a9b7e",
  "correlation_id": "01936f3e-9d42-7b3c-8e1a-2f5d8c4a9b7e",
  "causation_id": null,
  "payload_cid": "bafyreib3e4f5g6h7i8j9k0l1m2n3o4p",
  "timestamp": "2025-11-20T18:45:30.123456Z",
  "source": "cim-keys-v0.8.0",
  "event_type": "context.bounded_context_created"
}
```

## Testing Event Flow

```bash
# 1. Run with offline mode
cargo run --bin cim-keys-gui -- --config config.toml

# 2. Create a person node in the GUI
# - Events are generated and logged
# - Check logs for "Generated N cim-graph events"

# 3. Verify event structure
# Events are logged with tracing::debug!

# 4. (Future) Enable NATS and verify publishing
# - Start NATS server with JetStream
# - Update config.toml to enable NATS
# - Subscribe to subjects: nats sub 'cim.graph.>.events.>'
# - Create nodes and verify events published
```

## Security Considerations

### Air-Gapped Operation
- Offline mode ensures no network communication
- Events stored on encrypted partitions only
- Keys never leave secure environment

### NATS Security
- Always use TLS for production
- Use JWT-based authentication
- Configure subject-based authorization
- Rotate credentials regularly

### Payload Encryption
Consider encrypting IPLD payloads for sensitive key material:
- Use authenticated encryption (AES-256-GCM)
- Store encryption keys in HSM or YubiKey
- Include encryption metadata in EventEnvelope

## Troubleshooting

### Events Not Generated
1. Check tracing level: `RUST_LOG=cim_keys=debug`
2. Verify GraphProjector initialization
3. Check domain event construction

### NATS Connection Issues
1. Verify NATS server is running: `nats server check`
2. Test credentials: `nats account info`
3. Check TLS configuration
4. Verify firewall rules allow port 4222

### Configuration Errors
```bash
# Validate configuration
cargo run --bin cim-keys -- --validate-config config.toml

# Create example configuration
cargo run --bin cim-keys -- --create-example-config
```

## Next Steps

1. **Implement Full NATS Integration** (v0.9.0)
   - Add async-nats dependency
   - Implement NatsEventPublisher
   - Add integration tests

2. **IPLD Object Store** (v0.9.0)
   - Implement IpldObjectStore adapter
   - Content-addressed payload storage
   - Deduplication support

3. **Batch Operations** (v0.10.0)
   - Offline event queue management
   - Batch NATS publish
   - Conflict resolution

4. **Advanced Features** (v0.11.0)
   - Event replay
   - Distributed projections
   - Real-time graph sync

## See Also

- [NATS_STREAMING_ARCHITECTURE.md](./NATS_STREAMING_ARCHITECTURE.md) - Complete architecture design
- [config.example.toml](../config.example.toml) - Example configuration
- [GraphProjector](../src/graph_projection.rs) - Functorial projection implementation
