# NATS Integration with IPLD/CID Support

## Overview

cim-keys now includes **real NATS client integration** with content-addressed events using IPLD/CID. This enables:

- ✅ Real-time event publishing to NATS JetStream
- ✅ Content-addressed events with cryptographic integrity
- ✅ Offline-first design with local queue persistence
- ✅ Automatic retry and reconnection
- ✅ Subject-based routing for different event types

## Architecture

```
KeyEvent → IPLD CID → ContentAddressedEvent
                    ↓
              NATS Subject (cim.keys.{aggregate}.{event})
                    ↓
         JetStream (Persistent Stream)
                    ↓
              Subscribers (Other CIM Components)
```

## Features

### 1. Real NATS Client (`NatsClientAdapter`)

Replaces the stub implementation with real async-nats connectivity:

```rust
use cim_keys::{adapters::NatsClientAdapter, config::NatsConfig};
use std::path::PathBuf;

// Create configuration
let config = NatsConfig {
    enabled: true,
    url: "nats://localhost:4222".to_string(),
    enable_jetstream: true,
    enable_ipld: true,
    ..Default::default()
};

// Create adapter
let adapter = NatsClientAdapter::new(config, PathBuf::from("./queue.json"));

// Connect to NATS
adapter.connect().await?;

// Publish events
adapter.publish_event(&event).await?;
```

### 2. IPLD/CID Content Addressing

Every event can be content-addressed using IPLD CIDs:

```rust
use cim_keys::ipld_support::{generate_cid, ContentAddressedEvent};

// Generate CID for any event
let cid = generate_cid(&event)?;
println!("Event CID: {}", cid); // bafyreig...

// Create content-addressed event
let ca_event = ContentAddressedEvent::new(event)?;

// Verify integrity
assert!(ca_event.verify()?);

// CIDs are deterministic - same content = same CID
let cid1 = generate_cid(&event1)?;
let cid2 = generate_cid(&event2)?;
assert_eq!(cid1, cid2); // If event1 == event2
```

### 3. Offline-First Queue

Events are queued locally when NATS is unavailable:

```rust
// Events automatically queue if NATS is down
adapter.publish_event(&event).await?;

// Check queue size
let size = adapter.queue_size().await;
println!("Queued events: {}", size);

// Flush queue when connection restored
let flushed = adapter.flush_queue().await?;
println!("Published {} events", flushed);
```

The queue persists to disk and survives restarts.

### 4. Subject-Based Routing

Events are published to semantic subjects:

```
cim.keys.{aggregate_type}.{event_type}

Examples:
- cim.keys.key.keygenerated
- cim.keys.certificate.certificategenerated
- cim.keys.yubikey.yubikeyprovisioned
- cim.keys.nats.operator.natsoperatorcreated
- cim.keys.organization.organizationcreated
- cim.keys.person.personcreated
```

This enables subscribers to filter by aggregate or event type.

## Configuration

### TOML Configuration File

```toml
[nats]
enabled = true
url = "nats://leaf-node-1.local:4222"
stream_name = "CIM_KEYS_EVENTS"
object_store_bucket = "cim-keys-objects"
source_id = "cim-keys-v0.8.0"
subject_prefix = "cim.keys"
enable_jetstream = true
enable_ipld = true
max_retries = 3
connection_timeout_secs = 10

# Optional TLS
[nats.tls]
ca_cert = "/path/to/ca-cert.pem"
client_cert = "/path/to/client-cert.pem"
client_key = "/path/to/client-key.pem"

# Optional credentials
credentials_file = "/path/to/nats.creds"

[storage]
offline_events_dir = "/mnt/encrypted/cim-keys/events"
keys_output_dir = "/mnt/encrypted/cim-keys/keys"
enable_backup = true
backup_dir = "/backup/cim-keys"

[mode]
mode = "Hybrid"  # Offline, Online, or Hybrid
```

### Programmatic Configuration

```rust
use cim_keys::config::{Config, NatsConfig, OperationalMode};

let mut config = Config::default();

// Enable NATS
config.nats.enabled = true;
config.nats.url = "nats://localhost:4222".to_string();

// Enable IPLD
config.nats.enable_ipld = true;

// Set operational mode
config.mode = OperationalMode::Hybrid;

// Validate
config.validate()?;
```

## Usage Examples

### Example 1: Basic Event Publishing

```rust
use cim_keys::{adapters::NatsClientAdapter, events::*, config::NatsConfig};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let config = NatsConfig::default();
    let adapter = NatsClientAdapter::new(config, "./queue.json".into());
    adapter.connect().await?;

    // Create event
    let event = KeyEvent::KeyGenerated(KeyGeneratedEvent {
        key_id: Uuid::now_v7(),
        algorithm: KeyAlgorithm::Ed25519,
        purpose: KeyPurpose::Signing,
        generated_at: chrono::Utc::now(),
        generated_by: "alice@example.com".to_string(),
        hardware_backed: false,
        metadata: KeyMetadata {
            label: "Test Key".to_string(),
            description: None,
            tags: vec![],
            attributes: Default::default(),
            jwt_kid: None,
            jwt_alg: None,
            jwt_use: None,
        },
        owner: None,
    });

    // Publish with automatic CID generation
    adapter.publish_event(&event).await?;

    println!("✅ Event published!");
    Ok(())
}
```

### Example 2: Offline Queue Management

```rust
// Queue events while offline
for event in events {
    adapter.publish_event(&event).await?;
}

println!("Queued {} events", adapter.queue_size().await);

// Later, when connection restored
adapter.connect().await?;
let flushed = adapter.flush_queue().await?;
println!("Published {} queued events", flushed);
```

### Example 3: IPLD/CID Verification

```rust
use cim_keys::ipld_support::{generate_cid, verify_cid};

// Generate CID
let cid = generate_cid(&event)?;
println!("Event CID: {}", cid);

// Verify later
assert!(verify_cid(&event, &cid)?);

// Use content-addressed wrapper
let ca_event = ContentAddressedEvent::new(event)?;
println!("CID: {}", ca_event.cid);
assert!(ca_event.verify()?);
```

## NATS Server Setup

### Local Development

```bash
# Install NATS server
curl -L https://github.com/nats-io/nats-server/releases/download/v2.10.7/nats-server-v2.10.7-linux-amd64.tar.gz | tar xz
sudo mv nats-server-v2.10.7-linux-amd64/nats-server /usr/local/bin/

# Start with JetStream
nats-server -js

# Or with config file
nats-server -c nats-server.conf
```

### Production Configuration

```conf
# nats-server.conf
port: 4222
jetstream {
    store_dir: /var/lib/nats/jetstream
    max_mem: 1G
    max_file: 10G
}

accounts {
    CIM_KEYS: {
        jetstream: enabled
        users: [
            {user: cim-keys, password: $NATS_PASSWORD}
        ]
    }
}
```

### Creating JetStream Stream

```bash
# Create stream for cim-keys events
nats stream add CIM_KEYS_EVENTS \
    --subjects "cim.keys.>" \
    --storage file \
    --retention limits \
    --max-age 720h \
    --max-msgs 1000000

# Create object store for IPLD
nats object add cim-keys-objects \
    --storage file \
    --max-age 2160h
```

## Integration with Aggregate

Wire up NATS publishing in your aggregate:

```rust
use cim_keys::adapters::NatsClientAdapter;

pub struct KeyAggregate {
    nats: Arc<NatsClientAdapter>,
}

impl KeyAggregate {
    pub async fn handle_command(&self, cmd: KeyCommand) -> Result<Vec<KeyEvent>, Error> {
        // Generate events
        let events = self.process_command(cmd)?;

        // Publish to NATS
        for event in &events {
            self.nats.publish_event(event).await?;
        }

        Ok(events)
    }
}
```

## Testing

### Unit Tests

```bash
# Test without real NATS
cargo test --lib

# Test with IPLD features
cargo test --features ipld
```

### Integration Tests

```bash
# Start local NATS server
nats-server -js &

# Run integration example
cargo run --example nats_integration --features nats-client,ipld
```

### Expected Output

```
🚀 CIM-Keys NATS Integration Example

📋 Configuration:
  NATS URL: nats://localhost:4222
  JetStream: true
  IPLD/CID: true
  Subject Prefix: cim.keys

🔌 Connecting to NATS...
✅ Connected to NATS server

📝 Creating example events...

1️⃣  Publishing: OrganizationCreated
2️⃣  Publishing: PersonCreated
3️⃣  Publishing: KeyGenerated
4️⃣  Publishing: NatsOperatorCreated

📊 Status:
  Connected: true
  Queue Size: 0

🔍 IPLD/CID Verification:
  Event CID: bafyreig3v7d5qxh7xmxqxqxqxqxqxqxqxqxqxqx
  Verified: true
  Deterministic: true

✅ Example complete!
```

## Troubleshooting

### Connection Refused

```
⚠️  Could not connect to NATS: Connection error: connection refused
```

**Solution**: Start NATS server:
```bash
nats-server -js
```

### Credentials Error

```
⚠️  Could not connect to NATS: Authorization violation
```

**Solution**: Provide valid credentials:
```toml
[nats]
credentials_file = "/path/to/valid.creds"
```

### Queue Not Flushing

**Check connection**:
```rust
if !adapter.is_connected().await {
    adapter.connect().await?;
}
adapter.flush_queue().await?;
```

## Performance Considerations

### Local Queue Size

Events queue in memory and on disk. Monitor size:

```rust
let size = adapter.queue_size().await;
if size > 1000 {
    warn!("Large queue detected: {} events", size);
}
```

### Batch Flushing

Flush periodically in a background task:

```rust
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        if let Err(e) = adapter.flush_queue().await {
            error!("Queue flush failed: {}", e);
        }
    }
});
```

### IPLD Overhead

CID generation adds ~1-2ms per event. Disable if not needed:

```toml
[nats]
enable_ipld = false
```

## Security Considerations

### TLS

Always use TLS in production:

```toml
[nats.tls]
ca_cert = "/etc/cim/ca-cert.pem"
client_cert = "/etc/cim/client-cert.pem"
client_key = "/etc/cim/client-key.pem"
```

### Credentials

Use NATS credentials files (not passwords):

```bash
# Generate credentials with NSC
nsc generate creds -a CIM_KEYS -n cim-keys-publisher
```

### Event Encryption

Events contain sensitive key metadata. Consider:

1. Encrypting event payloads before publishing
2. Using NATS encryption at rest
3. Network-level encryption (TLS/VPN)

## Future Enhancements

- [ ] Object Store integration for large payloads
- [ ] KV Store for domain metadata
- [ ] Stream replay for event sourcing
- [ ] NATS-based saga coordination
- [ ] Distributed tracing integration

## See Also

- [NATS JetStream Documentation](https://docs.nats.io/nats-concepts/jetstream)
- [IPLD Specification](https://ipld.io/)
- [CID Specification](https://github.com/multiformats/cid)
- [cim-keys Architecture](../README.md)
