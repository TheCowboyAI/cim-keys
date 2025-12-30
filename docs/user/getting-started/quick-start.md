# Quick Start Guide

<!-- Copyright (c) 2025 - Cowboy AI, LLC. -->

Get started with cim-keys in 5 minutes.

## Architecture: Air-Gapped Only

**cim-keys operates EXCLUSIVELY in air-gapped mode.** There is no "online" mode.

- NATS runs on **localhost only** - no network connectivity
- All events are **projected to JSON files** on the encrypted SD card
- Private keys **never touch any network**

This is not a limitation - it's a security requirement for PKI bootstrap.

## Fastest Path

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

## Configuration

```toml
# config.toml - Air-gapped configuration

[nats]
# Localhost only - NO network connectivity
enabled = true
url = "nats://127.0.0.1:4222"

[storage]
# All output goes to encrypted SD card
keys_output_dir = "/mnt/encrypted/cim-keys/keys"
events_dir = "/mnt/encrypted/cim-keys/events"
```

```bash
# Run local NATS server (no network binding)
nats-server --addr 127.0.0.1

# Run GUI
cargo run --bin cim-keys-gui --features gui
```

## Essential Commands

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

## Output Structure (on Encrypted SD Card)

```
/mnt/encrypted/cim-keys/
├── manifest.json              # Current state projection
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

## Event Flow

```
User Action → Intent → Command → Aggregate → Event
                                              ↓
                              NATS (localhost) → JSON Projection
                                                    ↓
                                            Encrypted SD Card
```

All events flow through localhost NATS and are immediately projected to JSON files on the encrypted storage. The SD card becomes the portable, air-gapped state that can be physically transported to other systems.

## GUI Workflow

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
   - Generate manifest

## Verify Setup

```bash
# Check configuration
cargo run --bin cim-keys -- validate-config

# Expected output:
# ✅ Configuration is valid!
# 📋 Configuration Summary:
#    • NATS: localhost only (127.0.0.1:4222)
#    • Output: /mnt/encrypted/cim-keys

# Run GUI (should start without errors)
cargo run --bin cim-keys-gui --features gui

# Expected startup:
# 🔐 [CIM Keys] - Air-Gapped PKI Bootstrap
# 📁 [Output] /mnt/encrypted/cim-keys
# 🔒 [NATS] Localhost only (127.0.0.1:4222)
```

## Next Steps

- **Full Tutorial:** [End-to-End Workflow](../guides/end-to-end-workflow.md)
- **CLI Reference:** [CLI Commands](../guides/cli-reference.md)
- **Architecture:** [NATS Streaming](../../technical/architecture/nats-streaming.md)

## Troubleshooting

**GUI won't start:**
```bash
mkdir -p /mnt/encrypted/cim-keys
cargo run --bin cim-keys-gui --features gui
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

## Learning Path

1. ✅ **Quick Start** (you are here)
2. 📖 [End-to-End Workflow](../guides/end-to-end-workflow.md) - Complete workflow
3. 📋 [CLI Reference](../guides/cli-reference.md) - All commands
4. 🏗️ [NATS Architecture](../../technical/architecture/nats-streaming.md) - System design
5. 💻 [CLAUDE.md](../../../CLAUDE.md) - Development guidelines

---

**Ready to begin?**

```bash
nix develop
cargo run --bin cim-keys -- create-example-config
cp config.example.toml config.toml
cargo run --bin cim-keys-gui --features gui
```
