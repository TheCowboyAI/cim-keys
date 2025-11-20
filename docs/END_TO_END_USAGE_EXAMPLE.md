# End-to-End Usage Example

This guide walks through a complete workflow using cim-keys, from initial configuration to event publishing demonstration.

## Table of Contents

1. [Environment Setup](#environment-setup)
2. [Configuration Creation](#configuration-creation)
3. [Configuration Validation](#configuration-validation)
4. [Running the GUI](#running-the-gui)
5. [Creating Domain Objects](#creating-domain-objects)
6. [Observing Event Flow](#observing-event-flow)
7. [Operational Modes](#operational-modes)
8. [Export and Deployment](#export-and-deployment)

---

## Environment Setup

### Prerequisites

```bash
# Ensure you're in a Nix development environment
nix develop

# Verify cargo is available
cargo --version

# Create output directory
mkdir -p ./cim-keys-output
```

### Optional: NATS Server (for Online Mode)

```bash
# Start a local NATS server for testing
nats-server -js

# In another terminal, verify NATS is running
nats server info
```

---

## Configuration Creation

### Step 1: Generate Example Configuration

```bash
# Create example configuration file
cargo run --bin cim-keys -- create-example-config

# This creates config.example.toml in the current directory
```

### Step 2: Customize Configuration

Copy and customize the example config:

```bash
# Copy example to active config
cp config.example.toml config.toml

# Edit configuration
vim config.toml
```

**Example Configuration for Offline Mode:**

```toml
# Offline mode - no NATS, events logged locally only
mode = "Offline"

[nats]
enabled = false
url = "nats://localhost:4222"
stream_name = "CIM_GRAPH_EVENTS"
subject_prefix = "cim.graph"

[storage]
keys_output_dir = "./cim-keys-output/keys"
offline_events_dir = "./cim-keys-output/events"
backup_dir = "./cim-keys-output/backup"
```

**Example Configuration for Online Mode:**

```toml
# Online mode - real-time NATS publishing
mode = "Online"

[nats]
enabled = true
url = "nats://leaf-node-1.local:4222"
stream_name = "CIM_GRAPH_EVENTS"
subject_prefix = "cim.graph.person"
credentials_file = "../cim-keys/nsc/nkeys/creds/CowboyAI/InfraUser/infrastructure.creds"

[nats.tls]
enabled = true
ca_file = "./ca.pem"
cert_file = "./cert.pem"
key_file = "./key.pem"

[storage]
keys_output_dir = "/mnt/encrypted/cim-keys/keys"
offline_events_dir = "/mnt/encrypted/cim-keys/events"
backup_dir = "/backup/cim-keys"
```

**Example Configuration for Hybrid Mode:**

```toml
# Hybrid mode - queue events for batch upload
mode = "Hybrid"

[nats]
enabled = false  # Initially disabled, will enable for batch upload
url = "nats://leaf-node-1.local:4222"
stream_name = "CIM_GRAPH_EVENTS"
subject_prefix = "cim.graph.person"

[storage]
keys_output_dir = "./cim-keys-output/keys"
offline_events_dir = "./cim-keys-output/events"  # Events queued here
backup_dir = "./cim-keys-output/backup"
```

---

## Configuration Validation

### Step 3: Validate Configuration

```bash
# Validate the configuration
cargo run --bin cim-keys -- validate-config

# Expected output:
# 🔍 Validating configuration: config.toml
# ✓ Configuration loaded successfully
# ✅ Configuration is valid!
# 📋 Configuration Summary:
#    • Mode: Offline
#    • NATS enabled: false
#    • Offline events dir: ./cim-keys-output/events
#    • Keys output dir: ./cim-keys-output/keys
#    • Backup dir: ./cim-keys-output/backup
```

### Step 4: Show Configuration

```bash
# Display the current configuration
cargo run --bin cim-keys -- show-config

# Expected output shows TOML content
```

---

## Running the GUI

### Step 5: Launch GUI with Configuration

```bash
# Run GUI with default config.toml (if exists)
cargo run --bin cim-keys-gui --features gui

# Or specify config explicitly
cargo run --bin cim-keys-gui --features gui -- \
  --config config.toml \
  ./cim-keys-output

# Expected startup output:
# 🔐 [CIM Keys] - Offline Domain Bootstrap
# 📁 [Output] Directory: ./cim-keys-output
# ⚙️  [Mode] Offline
# 📴 [NATS] Disabled - offline mode
# ⚠️  [WARNING] Ensure this computer is air-gapped for secure key generation!
```

### Step 6: Run with Verbose Logging

```bash
# Enable verbose logging to see event flow
RUST_LOG=cim_keys=debug cargo run --bin cim-keys-gui --features gui -- \
  --config config.toml \
  --verbose \
  ./cim-keys-output
```

---

## Creating Domain Objects

### Step 7: Create Organization

In the GUI:

1. Navigate to **Welcome** tab
2. Enter organization details:
   - **Organization Name**: `cowboyai`
   - **Organization Domain**: `cowboyai.com`
   - **Master Passphrase**: (enter secure passphrase)
3. Click **"Create Organization"**

**Observe in logs:**
```
[INFO] Creating organization: cowboyai
[DEBUG] ✨ Generated 2 cim-graph events for OrganizationCreated
[DEBUG]   Event 1: BoundedContextCreated { ... }
[DEBUG]   Event 2: AggregateAdded { ... }
[DEBUG] 📴 No configuration loaded - events logged locally only
```

### Step 8: Add People to Organization

Navigate to **Organization** tab → **Graph View**:

1. Click **"Add Node"** dropdown
2. Select **"Person"**
3. Right-click on canvas to place node
4. In context menu, select **"Create Person"**
5. Enter person details inline

**Observe in logs:**
```
[DEBUG] ✨ Generated 2 cim-graph events for PersonCreated
[DEBUG]   Event 1: BoundedContextCreated { context: "cim.person" }
[DEBUG]   Event 2: AggregateAdded { aggregate_id: ... }
[DEBUG] 📴 NATS disabled - events logged locally only
```

### Step 9: Establish Relationships

1. Click **"Add Edge"** button
2. Click on first person (source)
3. Click on second person (target)
4. Select relationship type: **"Reports To"**

---

## Observing Event Flow

### Step 10: Examine Event Files

```bash
# List generated events (if offline mode)
ls -lah ./cim-keys-output/events/

# Expected structure:
# ./cim-keys-output/events/
# ├── 2025-01-20/
# │   ├── 001-organization-created.json
# │   ├── 002-person-created.json
# │   ├── 003-person-created.json
# │   └── 004-relationship-established.json
```

### Step 11: Inspect Event Content

```bash
# View an event file
cat ./cim-keys-output/events/2025-01-20/001-organization-created.json | jq .

# Expected structure:
# {
#   "event_id": "01943b8a-7f8e-7c4d-9e1f-2a3b4c5d6e7f",
#   "correlation_id": "01943b8a-7f8e-7c4d-9e1f-2a3b4c5d6e7f",
#   "causation_id": "01943b8a-7f8e-7c4d-9e1f-2a3b4c5d6e7f",
#   "occurred_at": "2025-01-20T15:30:45Z",
#   "payload": {
#     "type": "context",
#     "event": "bounded_context_created",
#     "context": "cim.organization",
#     "data": { ... }
#   }
# }
```

### Step 12: Verify Projection State

```bash
# Check manifest for current state
cat ./cim-keys-output/manifest.json | jq .

# Expected manifest:
# {
#   "version": "1.0",
#   "organization": {
#     "id": "...",
#     "name": "cowboyai",
#     "domain": "cowboyai.com"
#   },
#   "people": [
#     { "id": "...", "name": "Alice Smith", "email": "alice@cowboyai.com" },
#     { "id": "...", "name": "Bob Jones", "email": "bob@cowboyai.com" }
#   ],
#   "relationships": [
#     { "from": "...", "to": "...", "type": "reports_to" }
#   ]
# }
```

---

## Operational Modes

### Offline Mode Workflow

**Use Case:** Air-gapped key generation, no network

```bash
# 1. Configure for offline mode
cat > config.toml <<EOF
mode = "Offline"

[nats]
enabled = false

[storage]
keys_output_dir = "./cim-keys-output/keys"
offline_events_dir = "./cim-keys-output/events"
backup_dir = "./cim-keys-output/backup"
EOF

# 2. Run GUI
cargo run --bin cim-keys-gui --features gui

# 3. Generate keys and export
# ... (use GUI to create domain and generate keys)

# 4. Events are logged to ./cim-keys-output/events/
# 5. Keys are stored in ./cim-keys-output/keys/
# 6. Export to encrypted SD card when ready
```

### Online Mode Workflow

**Use Case:** Real-time event publishing to NATS infrastructure

```bash
# 1. Ensure NATS server is running
nats server info

# 2. Configure for online mode
cat > config.toml <<EOF
mode = "Online"

[nats]
enabled = true
url = "nats://leaf-node-1.local:4222"
stream_name = "CIM_GRAPH_EVENTS"
subject_prefix = "cim.graph.person"
credentials_file = "./creds/infrastructure.creds"
EOF

# 3. Validate configuration
cargo run --bin cim-keys -- validate-config

# 4. Run GUI
cargo run --bin cim-keys-gui --features gui

# 5. Create domain objects - events published in real-time
# Observe in NATS:
nats stream view CIM_GRAPH_EVENTS

# 6. Query events by subject
nats stream get CIM_GRAPH_EVENTS --subject "cim.graph.person.events.context.bounded_context_created"
```

### Hybrid Mode Workflow

**Use Case:** Work offline, batch publish later

```bash
# 1. Configure for hybrid mode (NATS disabled initially)
cat > config.toml <<EOF
mode = "Hybrid"

[nats]
enabled = false
url = "nats://leaf-node-1.local:4222"
stream_name = "CIM_GRAPH_EVENTS"
subject_prefix = "cim.graph.person"

[storage]
offline_events_dir = "./cim-keys-output/events"
EOF

# 2. Run GUI offline
cargo run --bin cim-keys-gui --features gui

# 3. Create domain objects (events queued locally)
# ... use GUI ...

# 4. Later, when network available, batch upload:
# TODO (v0.9.0): Implement batch upload command
cargo run --bin cim-keys -- batch-upload \
  --config config.toml \
  --events-dir ./cim-keys-output/events

# 5. Verify events published to NATS
nats stream info CIM_GRAPH_EVENTS
```

---

## Export and Deployment

### Step 13: Export Domain Configuration

In the GUI:

1. Navigate to **Export** tab
2. Configure export options:
   - ✓ Include public keys
   - ✓ Include certificates
   - ✓ Include NATS config
   - ✗ Include private keys (for security)
3. Set export path: `/mnt/encrypted/cim-keys-export`
4. Enter export password
5. Click **"Export Domain"**

### Step 14: Verify Export

```bash
# Mount encrypted SD card
sudo cryptsetup open /dev/sdb1 cim-keys-export
sudo mount /dev/mapper/cim-keys-export /mnt/encrypted

# Verify export structure
tree /mnt/encrypted/cim-keys-export/

# Expected structure:
# /mnt/encrypted/cim-keys-export/
# ├── manifest.json
# ├── domain/
# │   ├── organization.json
# │   ├── people.json
# │   └── relationships.json
# ├── keys/
# │   ├── {key-id}/
# │   │   ├── metadata.json
# │   │   └── public.pem
# ├── certificates/
# │   └── root-ca/
# ├── nats/
# │   ├── operator/
# │   ├── accounts/
# │   └── users/
# └── events/
#     └── 2025-01-20/
```

### Step 15: Deploy to Leaf Node

```bash
# Copy export to leaf node
rsync -avz /mnt/encrypted/cim-keys-export/ \
  leaf-node-1.local:/opt/cim/keys/

# On leaf node, import configuration
ssh leaf-node-1.local
cd /opt/cim/keys/
cim-keys import --manifest manifest.json

# Verify NATS configuration imported
nsc describe operator
nsc describe account
nsc describe user
```

---

## Troubleshooting

### Configuration Errors

```bash
# Error: Configuration file not found
# Solution: Create config.toml or specify path
cargo run --bin cim-keys -- create-example-config
cp config.example.toml config.toml

# Error: Invalid NATS URL
# Solution: Check URL format (must start with nats:// or tls://)
vim config.toml  # Fix [nats] url field

# Error: Credentials file not found
# Solution: Verify credentials path exists
ls -lah ./creds/infrastructure.creds
```

### Event Publishing Issues

```bash
# Issue: Events not appearing in NATS
# Check: NATS connection
nats server ping

# Check: JetStream enabled
nats stream ls

# Check: Credentials valid
nats account info --creds ./creds/infrastructure.creds

# Check: GUI logs
RUST_LOG=cim_keys=debug cargo run --bin cim-keys-gui --features gui
```

### GUI Issues

```bash
# Issue: GUI crashes on startup
# Solution: Check output directory exists
mkdir -p ./cim-keys-output

# Issue: Cannot save domain
# Solution: Check directory permissions
chmod -R u+w ./cim-keys-output

# Issue: YubiKey not detected
# Solution: Ensure pcscd is running
systemctl status pcscd
```

---

## Next Steps

### v0.9.0 Features (Upcoming)

- **Full NATS Integration**: Real event publishing when `config.nats.enabled = true`
- **Batch Upload**: Upload queued events from Hybrid mode
- **Event Replay**: Reconstruct state from event history
- **IPLD Object Store**: Store event payloads as content-addressed objects

### v0.10.0 Features (Roadmap)

- **Multi-Node Support**: Synchronize events across multiple leaf nodes
- **Conflict Resolution**: Handle concurrent updates with CRDTs
- **Event Subscriptions**: React to remote events in real-time
- **Distributed Queries**: Query events across cluster

---

## Complete Example Session

```bash
# 1. Setup
nix develop
mkdir -p ./cim-keys-output

# 2. Configuration
cargo run --bin cim-keys -- create-example-config
cp config.example.toml config.toml
vim config.toml  # Set mode = "Offline"

# 3. Validation
cargo run --bin cim-keys -- validate-config

# 4. Run GUI
RUST_LOG=cim_keys=debug cargo run --bin cim-keys-gui --features gui -- \
  --config config.toml \
  --verbose \
  ./cim-keys-output

# 5. In GUI:
#    - Create organization "cowboyai"
#    - Add people: Alice, Bob, Carol
#    - Establish relationships
#    - Generate root CA key
#    - Export to encrypted SD card

# 6. Verify output
tree ./cim-keys-output/
cat ./cim-keys-output/manifest.json | jq .
cat ./cim-keys-output/events/2025-01-20/001-organization-created.json | jq .

# 7. Deploy
rsync -avz ./cim-keys-output/ leaf-node-1.local:/opt/cim/keys/
```

---

## Reference Documentation

- [CLI Reference](./CLI_REFERENCE.md) - Complete command documentation
- [Configuration Guide](./EVENT_PUBLISHING_USAGE.md) - Detailed configuration options
- [NATS Architecture](./NATS_STREAMING_ARCHITECTURE.md) - Event streaming design
- [GUI User Guide](../CLAUDE.md) - GUI usage and patterns

---

## Support

For issues or questions:
1. Check [Troubleshooting](#troubleshooting) section
2. Review logs with `RUST_LOG=cim_keys=debug`
3. Validate configuration with `validate-config` command
4. Open issue at: https://github.com/thecowboyai/cim-keys/issues
