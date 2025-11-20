# CIM Keys CLI Reference

Complete command-line interface reference for cim-keys.

## Installation

```bash
# Build from source
cargo build --release

# Install to PATH
cargo install --path .

# Or run directly
cargo run --bin cim-keys -- <command>
```

## Global Options

### `--config <PATH>`

Load configuration from a custom file path.

**Default**: `config.toml`

**Example**:
```bash
cim-keys --config production.toml bootstrap
```

### `--verbose` / `-v`

Enable verbose output with debug logging.

**Example**:
```bash
cim-keys -v validate-config
```

## Configuration Commands

### `create-example-config`

Generate an example configuration file with all available options.

**Usage**:
```bash
cim-keys create-example-config [OPTIONS]
```

**Options**:
- `-o, --output <PATH>` - Output path for example config (default: `config.example.toml`)

**Example**:
```bash
# Create example config
cim-keys create-example-config

# Create with custom name
cim-keys create-example-config --output my-config.toml

# Copy and customize
cp config.example.toml config.toml
vim config.toml
```

**Output**:
```
📝 Creating example configuration: config.example.toml
✅ Example configuration created!

📋 Next steps:
   1. Copy to config.toml: cp config.example.toml config.toml
   2. Edit config.toml to match your environment
   3. Validate: cim-keys validate-config
```

### `validate-config`

Validate a configuration file for correctness.

**Usage**:
```bash
cim-keys validate-config [OPTIONS]
```

**Options**:
- `--path <PATH>` - Path to configuration file (default: uses `--config` or `config.toml`)

**Validation Checks**:
- NATS URL format when enabled
- Stream name not empty
- Credentials file exists if specified
- TLS certificate files exist if configured
- Backup directory specified when backup enabled

**Example**:
```bash
# Validate default config
cim-keys validate-config

# Validate specific file
cim-keys validate-config --path production.toml

# Validate and show details
cim-keys -v validate-config
```

**Success Output**:
```
🔍 Validating configuration: config.toml

✓ Configuration loaded successfully

✅ Configuration is valid!

📋 Configuration Summary:
   • Mode: Offline
   • NATS enabled: false
   • Offline events dir: /mnt/encrypted/cim-keys/events
   • Keys output dir: /mnt/encrypted/cim-keys/keys
```

**Error Output**:
```
❌ Configuration validation failed:
   Credentials file not found: /path/to/nats.creds
```

### `show-config`

Display the current configuration in TOML format.

**Usage**:
```bash
cim-keys show-config [OPTIONS]
```

**Options**:
- `--path <PATH>` - Path to configuration file (default: uses `--config` or `config.toml`)

**Example**:
```bash
# Show default config
cim-keys show-config

# Show specific config
cim-keys show-config --path production.toml

# Pipe to file
cim-keys show-config > current-config.toml
```

**Output**:
```toml
📋 Configuration from: config.toml

mode = "Offline"

[nats]
enabled = false
url = "nats://localhost:4222"
stream_name = "CIM_GRAPH_EVENTS"
# ... rest of configuration
```

## NATS Infrastructure Commands

### `bootstrap`

Bootstrap complete NATS infrastructure from domain configuration.

Generates Operator, Account, and User identities for NATS from organizational structure.

**Usage**:
```bash
cim-keys bootstrap [OPTIONS]
```

**Options**:
- `--domain <PATH>` - Path to domain configuration JSON (default: `secrets/domain-bootstrap.json`)
- `--people <PATH>` - Path to people configuration JSON (optional)
- `-o, --output <DIR>` - Output directory for credentials (default: `./nats-credentials`)
- `--nats-format` - Format credentials for NATS server import

**Example**:
```bash
# Bootstrap with defaults
cim-keys bootstrap

# Custom domain and output
cim-keys bootstrap \
  --domain org-structure.json \
  --people team-members.json \
  --output /mnt/encrypted/nats-creds

# With NATS format
cim-keys bootstrap --nats-format
```

**Output Structure**:
```
nats-credentials/
├── operator.jwt        # Operator JWT token
├── operator.nk         # Operator seed (private key)
├── accounts/
│   ├── engineering.jwt
│   ├── engineering.nk
│   ├── operations.jwt
│   └── operations.nk
└── users/
    ├── alice_smith.creds
    ├── bob_jones.creds
    └── ...
```

**Success Output**:
```
🔐 CIM Keys - NATS Infrastructure Bootstrap

📖 Loading domain configuration from: secrets/domain-bootstrap.json
   Organization: CowboyAI
   Units: 3
   People: 12

🔑 Generating NATS identities...
   ✓ Operator: CowboyAI
   ✓ Accounts: 3
   ✓ Users: 12
   ✓ Total identities: 16

📁 Writing credentials to: ./nats-credentials
   ✓ Operator JWT: ./nats-credentials/operator.jwt
   ✓ Operator seed: ./nats-credentials/operator.nk
   ✓ Account 'Engineering': ./nats-credentials/accounts/engineering.jwt
   ✓ User 'Alice Smith': ./nats-credentials/users/alice_smith.creds
   ...

✅ Bootstrap complete!

📋 Summary:
   • 1 operator identity
   • 3 account identities
   • 12 user identities
   • 22 total files written

🔒 Security Notes:
   • Store operator.nk and account *.nk files securely
   • Distribute user *.creds files via secure channels
   • Consider encrypting the entire output directory
```

### `list`

List organizations in domain configuration.

**Usage**:
```bash
cim-keys list [OPTIONS]
```

**Options**:
- `--domain <PATH>` - Path to domain configuration JSON (default: `secrets/domain-bootstrap.json`)

**Example**:
```bash
# List default domain
cim-keys list

# List specific domain
cim-keys list --domain my-org.json
```

**Output**:
```
📋 Organizations in: secrets/domain-bootstrap.json

Organization: CowboyAI
  ID: 01936f3e-9d42-7b3c-8e1a-2f5d8c4a9b7e
  Display Name: CowboyAI Inc.
  Description: AI-powered infrastructure automation

Organizational Units (3):
  • Engineering (01936f3e-9d42-7b3c-8e1a-2f5d8c4a9b7f)
    Type: Department
  • Operations (01936f3e-9d42-7b3c-8e1a-2f5d8c4a9b80)
    Type: Department
  • Security (01936f3e-9d42-7b3c-8e1a-2f5d8c4a9b81)
    Type: Department
```

### `validate`

Validate domain configuration without generating keys.

**Usage**:
```bash
cim-keys validate [OPTIONS]
```

**Options**:
- `--domain <PATH>` - Path to domain configuration JSON (default: `secrets/domain-bootstrap.json`)
- `--people <PATH>` - Path to people configuration JSON (optional)

**Example**:
```bash
# Validate domain only
cim-keys validate

# Validate domain and people
cim-keys validate \
  --domain org-structure.json \
  --people team-members.json
```

**Output**:
```
✓ Validating domain configuration...

✓ Organization valid: CowboyAI
  • 3 units
✓ People valid: 12 persons

✅ Configuration is valid!
```

**With Warnings**:
```
✓ Validating domain configuration...

✓ Organization valid: CowboyAI
  • 3 units
✓ People valid: 12 persons
⚠  Warning: 2 people reference different organization IDs

✅ Configuration is valid!
```

### `version`

Show version and build information.

**Usage**:
```bash
cim-keys version
```

**Output**:
```
cim-keys version 0.8.0
Event-sourced NATS infrastructure bootstrap tool
```

## GUI Commands

### `cim-keys-gui`

Launch the graphical user interface for visual key management.

**Usage**:
```bash
cim-keys-gui [OUTPUT_DIR]
```

**Options**:
- `OUTPUT_DIR` - Directory for generated keys (default: `./keys-output`)
- `--config <PATH>` - Configuration file to load (when implemented)

**Example**:
```bash
# Launch with default output
cim-keys-gui

# Launch with custom output directory
cim-keys-gui /mnt/encrypted/keys

# (Future) With configuration
cim-keys-gui --config production.toml
```

## Common Workflows

### Initial Setup

```bash
# 1. Create example configuration
cim-keys create-example-config

# 2. Customize configuration
cp config.example.toml config.toml
vim config.toml

# 3. Validate configuration
cim-keys validate-config

# 4. Verify settings
cim-keys show-config
```

### Offline Key Generation

```bash
# 1. Prepare domain configuration
vim secrets/domain-bootstrap.json

# 2. Validate domain structure
cim-keys validate --domain secrets/domain-bootstrap.json

# 3. Bootstrap NATS infrastructure
cim-keys bootstrap \
  --domain secrets/domain-bootstrap.json \
  --output /mnt/encrypted/nats-creds

# 4. Launch GUI for key generation
cim-keys-gui /mnt/encrypted/keys
```

### Online Event Publishing

```bash
# 1. Configure for online mode
vim config.toml
# Set mode = "Online" and nats.enabled = true

# 2. Validate configuration
cim-keys validate-config

# 3. Launch GUI with config
cim-keys-gui --config config.toml

# Events will now be published to NATS in real-time
```

### Hybrid Mode (Offline + Later Upload)

```bash
# 1. Configure for hybrid mode
vim config.toml
# Set mode = "Hybrid" and nats.enabled = false

# 2. Generate keys offline
cim-keys-gui /mnt/encrypted/keys

# Events logged to offline_events_dir

# 3. Later, when connected to secure network
vim config.toml
# Set nats.enabled = true

# 4. Upload offline events (future feature)
# cim-keys upload-offline-events
```

## Configuration File Format

### Complete Example

```toml
# Operational mode
mode = "Offline"  # or "Online" or "Hybrid"

[nats]
# Enable NATS event publishing
enabled = false

# NATS server URL
url = "nats://leaf-node-1.local:4222"

# JetStream stream name
stream_name = "CIM_GRAPH_EVENTS"

# Object store bucket for IPLD payloads
object_store_bucket = "cim-graph-payloads"

# Source identifier
source_id = "cim-keys-v0.8.0"

# Subject prefix
subject_prefix = "cim.graph"

# Optional: NATS credentials file
# credentials_file = "/path/to/nats.creds"

# Optional: TLS configuration
# [nats.tls]
# ca_cert = "/path/to/ca-cert.pem"
# client_cert = "/path/to/client-cert.pem"
# client_key = "/path/to/client-key.pem"

[storage]
# Directory for offline event storage
offline_events_dir = "/mnt/encrypted/cim-keys/events"

# Directory for generated keys
keys_output_dir = "/mnt/encrypted/cim-keys/keys"

# Enable automatic backup
enable_backup = false

# Backup directory (required if enable_backup = true)
# backup_dir = "/backup/cim-keys"
```

## Environment Variables

### `RUST_LOG`

Control logging level.

**Values**:
- `error` - Errors only
- `warn` - Warnings and errors
- `info` - Informational messages (default)
- `debug` - Debug messages
- `trace` - Trace messages

**Example**:
```bash
# Enable debug logging
RUST_LOG=cim_keys=debug cim-keys bootstrap

# Enable trace logging for specific module
RUST_LOG=cim_keys::graph_projection=trace cim-keys-gui
```

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Validation error

## Troubleshooting

### Configuration Not Found

```bash
# Error
Configuration file not found: config.toml

# Solution
cim-keys create-example-config
cp config.example.toml config.toml
```

### Invalid Configuration

```bash
# Error
Invalid configuration: NATS URL cannot be empty when enabled

# Solution
vim config.toml
# Set nats.url or disable NATS: nats.enabled = false
```

### Credentials File Not Found

```bash
# Error
Credentials file not found: /path/to/nats.creds

# Solution
# Either create the credentials file or comment out in config:
# credentials_file = "/path/to/nats.creds"
```

## See Also

- [EVENT_PUBLISHING_USAGE.md](./EVENT_PUBLISHING_USAGE.md) - Event publishing guide
- [NATS_STREAMING_ARCHITECTURE.md](./NATS_STREAMING_ARCHITECTURE.md) - Architecture design
- [config.example.toml](../config.example.toml) - Example configuration
