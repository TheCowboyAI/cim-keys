# Quick Start Guide

Get started with cim-keys in 5 minutes.

## 🚀 Fastest Path

```bash
# 1. Enter development environment
nix develop

# 2. Generate example config
cargo run --bin cim-keys -- create-example-config

# 3. Copy and customize
cp config.example.toml config.toml

# 4. Run GUI
cargo run --bin cim-keys-gui --features gui
```

## 📋 Common Workflows

### Offline Mode (Air-Gapped)

**Use Case:** Secure key generation without network

```bash
# config.toml
mode = "Offline"

[nats]
enabled = false

[storage]
keys_output_dir = "./cim-keys-output/keys"
offline_events_dir = "./cim-keys-output/events"
```

```bash
# Run
cargo run --bin cim-keys-gui --features gui
```

### Online Mode (Real-Time Publishing)

**Use Case:** Live event streaming to NATS

```bash
# config.toml
mode = "Online"

[nats]
enabled = true
url = "nats://leaf-node-1.local:4222"
credentials_file = "./creds/infra.creds"
```

```bash
# Validate
cargo run --bin cim-keys -- validate-config

# Run
cargo run --bin cim-keys-gui --features gui
```

### Hybrid Mode (Offline + Batch Upload)

**Use Case:** Work offline, publish later

```bash
# config.toml
mode = "Hybrid"

[nats]
enabled = false  # Disable for offline work
url = "nats://leaf-node-1.local:4222"
```

```bash
# Work offline
cargo run --bin cim-keys-gui --features gui

# Later, batch upload (v0.9.0)
cargo run --bin cim-keys -- batch-upload \
  --config config.toml \
  --events-dir ./cim-keys-output/events
```

## 🛠️ Essential Commands

```bash
# Configuration management
cim-keys create-example-config       # Generate example config
cim-keys validate-config              # Validate config.toml
cim-keys show-config                  # Display current config

# Run GUI
cim-keys-gui                          # Use default config.toml
cim-keys-gui --config path/to/cfg     # Specify config file
cim-keys-gui --verbose                # Enable debug logging

# Development
cargo check --features gui            # Verify compilation
cargo test --all-features            # Run tests
cargo build --release --features gui # Build optimized binary
```

## 📁 Output Structure

```
./cim-keys-output/
├── manifest.json              # Current state
├── domain/
│   ├── organization.json     # Org structure
│   ├── people.json          # All people
│   └── relationships.json   # Graph edges
├── keys/
│   └── {key-id}/
│       ├── metadata.json
│       └── public.pem
├── certificates/
│   ├── root-ca/
│   └── intermediate-ca/
├── nats/
│   ├── operator/
│   ├── accounts/
│   └── users/
└── events/
    └── 2025-01-20/
        ├── 001-org-created.json
        └── 002-person-created.json
```

## 🎯 GUI Workflow

1. **Welcome Tab**
   - Create organization
   - Set master passphrase

2. **Organization Tab**
   - Add people (nodes)
   - Establish relationships (edges)
   - Visualize organizational graph

3. **Keys Tab**
   - Generate root CA
   - Generate personal keys
   - Provision YubiKeys

4. **Export Tab**
   - Export to encrypted SD card
   - Configure export options

## 🔍 Verify Setup

```bash
# Check configuration
cargo run --bin cim-keys -- validate-config

# Expected output:
# ✅ Configuration is valid!
# 📋 Configuration Summary:
#    • Mode: Offline
#    • NATS enabled: false
#    • Keys output dir: ./cim-keys-output/keys

# Run GUI (should start without errors)
cargo run --bin cim-keys-gui --features gui

# Expected startup:
# 🔐 [CIM Keys] - Offline Domain Bootstrap
# 📁 [Output] Directory: ./cim-keys-output
# ⚙️  [Mode] Offline
# 📴 [NATS] Disabled - offline mode
```

## 📚 Next Steps

- **Full Tutorial:** [End-to-End Usage Example](./END_TO_END_USAGE_EXAMPLE.md)
- **CLI Reference:** [CLI Commands](./CLI_REFERENCE.md)
- **Configuration:** [Event Publishing Usage](./EVENT_PUBLISHING_USAGE.md)
- **Architecture:** [NATS Streaming](./NATS_STREAMING_ARCHITECTURE.md)

## 🆘 Troubleshooting

**GUI won't start:**
```bash
mkdir -p ./cim-keys-output
cargo run --bin cim-keys-gui --features gui -- ./cim-keys-output
```

**Config errors:**
```bash
cargo run --bin cim-keys -- create-example-config
cp config.example.toml config.toml
vim config.toml  # Fix errors
cargo run --bin cim-keys -- validate-config
```

**Can't see events:**
```bash
RUST_LOG=cim_keys=debug cargo run --bin cim-keys-gui --features gui -- --verbose
```

## 🎓 Learning Path

1. ✅ **Quick Start** (you are here)
2. 📖 [End-to-End Usage Example](./END_TO_END_USAGE_EXAMPLE.md) - Complete workflow
3. 📋 [CLI Reference](./CLI_REFERENCE.md) - All commands
4. ⚙️ [Event Publishing Usage](./EVENT_PUBLISHING_USAGE.md) - Configuration details
5. 🏗️ [NATS Architecture](./NATS_STREAMING_ARCHITECTURE.md) - System design
6. 💻 [CLAUDE.md](../CLAUDE.md) - Development guidelines

---

**Ready to begin?**

```bash
nix develop
cargo run --bin cim-keys -- create-example-config
cp config.example.toml config.toml
cargo run --bin cim-keys-gui --features gui
```
