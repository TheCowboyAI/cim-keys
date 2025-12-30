# Event Publishing Usage Guide

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

This guide demonstrates how to use the event publishing system in cim-keys for air-gapped PKI bootstrap.

## Overview

The cim-keys event publishing system provides a functorial projection from domain events to cim-graph events, with localhost NATS as the event bus and JSON projection to encrypted SD card.

**IMPORTANT**: cim-keys operates EXCLUSIVELY in air-gapped mode. There is no "online mode".

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
NATS (localhost only) → JSON Projection
  ↓
Encrypted SD Card (physical transport to target)
```

## Configuration

### 1. Create Configuration File

```bash
cp config.example.toml config.toml
```

### 2. Configure for Air-Gapped Operation

**Air-Gapped Configuration (the only mode):**
```toml
[nats]
# Localhost NATS - event bus without network
enabled = true
url = "nats://127.0.0.1:4222"
stream_name = "CIM_KEYS_EVENTS"

[storage]
# Output to encrypted SD card
events_dir = "/mnt/encrypted/cim-keys/events"
keys_output_dir = "/mnt/encrypted/cim-keys/keys"
```

**Note**: There is no "online mode" or "hybrid mode". cim-keys always operates air-gapped with localhost NATS and JSON projections to the SD card.

## Usage Examples

### Basic Usage

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

// Events flow to localhost NATS and are projected to JSON
tracing::info!("Generated {} graph events", graph_events.len());
for (i, event) in graph_events.iter().enumerate() {
    tracing::debug!("Event {}: {:?}", i+1, event);
}
```

### JSON Projection to SD Card

Events published to localhost NATS are projected to JSON files:

```rust
use cim_keys::config::Config;
use std::fs;

let config = Config::from_file(&"config.toml".into())?;

// Events are automatically projected to JSON files
// on the encrypted SD card in the events_dir

// Read projected events
let events_dir = &config.storage.events_dir;
for entry in fs::read_dir(events_dir)? {
    let path = entry?.path();
    if path.extension() == Some("json".as_ref()) {
        let event_json = fs::read_to_string(&path)?;
        tracing::info!("Projected event: {}", path.display());
    }
}

// The SD card can then be physically transported to target systems
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
- NATS runs on localhost only (127.0.0.1) - no network binding
- Events projected to encrypted SD card
- Private keys never touch any network
- SD card physically transported to target systems

### Localhost NATS Security
- Bound to 127.0.0.1 only (not 0.0.0.0)
- No external network access
- Provides event bus functionality without network exposure

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

1. **Enhanced JSON Projection** (v0.9.0)
   - Complete event projection to SD card
   - Manifest generation with checksums
   - Event replay from JSON files

2. **IPLD Object Store** (v0.9.0)
   - Implement IpldObjectStore adapter
   - Content-addressed payload storage
   - Deduplication support

3. **Import/Export** (v0.10.0)
   - Import existing PKI from SD cards
   - Export for specific target systems
   - Merge multiple organizational PKIs

## Relationship to CIM Ecosystem

cim-keys is the **air-gapped genesis point** for distributed CIM infrastructure:

1. **Generate** - Create organization PKI offline with cim-keys
2. **Project** - Events flow through localhost NATS to JSON on SD card
3. **Transport** - Physically move encrypted SD card to target systems
4. **Bootstrap** - Target CIM leaf nodes/clusters import the PKI
5. **Operate** - Distributed CIM system uses the generated keys

The distributed nature of CIM comes AFTER keys are generated and transported.

## See Also

- [NATS_STREAMING_ARCHITECTURE.md](./NATS_STREAMING_ARCHITECTURE.md) - Complete architecture design
- [config.example.toml](../config.example.toml) - Example configuration
- [GraphProjector](../src/graph_projection.rs) - Functorial projection implementation
