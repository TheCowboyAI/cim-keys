# cim-keys

A comprehensive cryptographic key management library for Rust, providing support for YubiKey hardware tokens, GPG/OpenPGP, TLS/X.509 certificates, SSH keys, and PKI infrastructure.

## Features

- **YubiKey Support**: Full integration with YubiKey hardware tokens for secure key storage and operations
- **Multiple Key Types**: Support for RSA, ECDSA, Ed25519, and other algorithms
- **Certificate Management**: X.509 certificate generation, signing, and validation
- **SSH Key Operations**: Generate and manage SSH keys compatible with OpenSSH
- **GPG/OpenPGP**: Full OpenPGP support using Sequoia-PGP
- **PKI Infrastructure**: Complete PKI support including CA operations and certificate chains
- **Secure Storage**: Multiple storage backends with encryption support
- **Async/Await**: Fully asynchronous API using Tokio

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cim-keys = { path = "../cim-keys" }
```

## Quick Start

```rust
use cim_keys::{
    KeyManager, Signer, CertificateManager,
    KeyAlgorithm, KeyUsage, SignatureFormat,
    ssh::SshKeyManager,
    tls::TlsManager,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate an SSH key
    let ssh_manager = SshKeyManager::new();
    let key_id = ssh_manager.generate_key(
        KeyAlgorithm::Ed25519,
        "user@example.com".to_string(),
        KeyUsage::Signing,
    ).await?;

    // Sign data
    let signature = ssh_manager.sign(
        &key_id,
        b"Hello, World!",
        SignatureFormat::Raw,
    ).await?;

    // Generate a TLS certificate
    let tls_manager = TlsManager::new();
    let (key_id, cert_id) = tls_manager.generate_self_signed(
        "example.com",
        vec!["example.com".to_string()],
        KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
        365,
    ).await?;

    Ok(())
}
```

## Modules

### YubiKey (`yubikey`)
Hardware token support for secure key storage and cryptographic operations.

- PIV (Personal Identity Verification) support
- OpenPGP card functionality
- FIDO2/WebAuthn operations
- Secure key generation and storage

### SSH (`ssh`)
SSH key management compatible with OpenSSH.

- Key generation (RSA, ECDSA, Ed25519)
- OpenSSH format import/export
- SSH agent protocol support
- Certificate authority operations

### TLS (`tls`)
TLS and X.509 certificate management.

- Self-signed certificate generation
- CSR (Certificate Signing Request) creation
- Certificate validation and chain building
- Multiple format support (PEM, DER)

### GPG (`gpg`)
OpenPGP operations using Sequoia-PGP.

- Key generation and management
- Signing and verification
- Encryption and decryption
- Web of Trust operations

### PKI (`pki`)
Complete PKI infrastructure support.

- Root and intermediate CA creation
- Certificate issuance and revocation
- CRL (Certificate Revocation List) management
- Trust store operations

### Storage (`storage`)
Secure key storage backends.

- File-based storage with encryption
- In-memory storage for testing
- Hardware token storage
- Cloud HSM integration (planned)

## Security Considerations

- All private keys are stored encrypted at rest
- Hardware token support for maximum security
- Secure key generation using system randomness
- Memory is zeroed after use for sensitive data
- Comprehensive error handling without information leakage

## Examples

See the `examples/` directory for more detailed usage examples:

- `basic_usage.rs` - Basic key operations
- `cim_leaf_integration.rs` - CIM Leaf three-level PKI integration with YubiKeys
- `nats_tls_setup.rs` - NATS TLS configuration with YubiKey-backed certificates
- `yubikey_piv.rs` - YubiKey PIV operations
- `ssh_agent.rs` - SSH agent integration
- `tls_server.rs` - TLS server with generated certificates
- `gpg_signing.rs` - GPG signing and verification

### CIM Leaf Integration

The cim-keys module is designed to integrate seamlessly with the CIM Leaf three-level PKI infrastructure:

1. **Operator Level**: System operations and disk encryption
   - Operator Root CA and Intermediate CA
   - YubiKey with operator certificates
   - System-level operations

2. **Domain Level**: Domain administration and user certificate signing
   - Domain Root CA and Intermediate CA
   - YubiKey with domain certificates
   - Domain administration

3. **User Level**: Day-to-day operations
   - User certificates signed by Domain Intermediate CA
   - YubiKey with user certificates
   - Regular user operations

Each YubiKey is configured with:
- PIV slots (9A: Authentication, 9C: Digital Signature, 9D: Key Management)
- FIDO2 for SSH authentication
- OpenPGP for signing and encryption
- OATH for TOTP

## GUI Application

CIM Keys includes a beautiful GUI application for offline key generation and management.

### Building the GUI

```bash
# Enter Nix development shell (provides all dependencies)
nix develop

# Build native GUI application
cargo build --release --features gui

# Run the GUI
cargo run --bin cim-keys-gui --release -- /path/to/output/dir
```

### Using the GUI

#### 1. Import from Secrets (Recommended)

The GUI can import your complete organizational configuration from JSON files:

**Required files:**
- `secrets/secrets.json` - Organization and people configuration with master passphrase
- `secrets/cowboyai.json` - YubiKey configurations and roles

**What gets imported:**
- ✅ Organization name and domain
- ✅ Master passphrase for encryption
- ✅ All people with roles and email addresses
- ✅ YubiKey configurations (serials, PINs, PUKs, management keys)
- ✅ Role assignments (Root CA, Backup, User, Service)

Click **"Import from Secrets"** on the Welcome screen to load everything automatically.

#### 2. Manual Configuration

Alternatively, create your domain manually:

1. **Welcome Tab** - Create new domain or load existing
2. **Organization Tab**
   - Set organization name and domain
   - Create master passphrase
   - Add people to organization graph
   - Define relationships (reports to, delegates to, trusts)
   - Drag nodes to arrange graph layout
3. **Locations Tab**
   - Add physical/logical locations for key storage
   - Set security levels (FIPS 140 Level 1-4, Commercial, Basic)
   - Location types: Data Center, Office, Cloud Region, Safe Deposit, etc.
4. **Keys Tab**
   - Detect YubiKeys (shows all connected devices)
   - Generate Root CA
   - Generate Intermediate CAs
   - Generate Server Certificates
   - Provision YubiKeys with keys
   - View all imported YubiKey configurations
   - View generated certificates and keys from manifest
5. **Export Tab**
   - Export to encrypted SD card
   - Generate manifest with all configuration

#### 3. Data Persistence

All data is automatically saved to `manifest.json` in your output directory:

**On startup, the GUI loads:**
- ✅ Organization information
- ✅ All saved locations
- ✅ All people entries
- ✅ All generated certificates (with validity dates, issuers)
- ✅ All generated keys (with hardware backing status)

**The manifest persists:**
- Organization metadata
- Location database
- People registry
- Certificate inventory with PKI details
- Key inventory with YubiKey associations
- Complete event log for audit trail

#### 4. YubiKey Operations

**Detection:**
```bash
# Click "Detect YubiKeys" in Keys tab
# Shows: Model, Version, Serial, PIV status
```

**Provisioning:**
```bash
# After importing secrets with YubiKey configs
# Click "Provision YubiKeys"
# Automatically:
#   - Matches detected hardware with configs by serial
#   - Generates keys in appropriate PIV slots based on role
#   - Root CA → Signature slot (9C)
#   - Backup → Key Management slot (9D)
#   - User → Authentication slot (9A)
#   - Service → Card Authentication slot (9E)
```

### Example Workflow

```bash
# 1. Prepare your secrets files
cat secrets/secrets.json    # Contains certify_pass (master passphrase)
cat secrets/cowboyai.json   # Contains YubiKey configurations

# 2. Run the GUI
cargo run --bin cim-keys-gui --release -- ./my-keys-output

# 3. Import configuration
#    - Click "Import from Secrets"
#    - All data loads automatically
#    - Master passphrase appears in Organization tab
#    - People appear in organization graph
#    - YubiKey configs appear in Keys tab

# 4. Insert YubiKeys and detect
#    - Keys Tab → "Detect YubiKeys"
#    - Should match your imported configurations by serial

# 5. Generate PKI hierarchy
#    - Generate Root CA (stored on Root CA YubiKey)
#    - Generate Intermediate CAs for departments
#    - Generate Server Certificates

# 6. Provision YubiKeys
#    - Click "Provision YubiKeys"
#    - Keys generated in correct slots with imported PINs

# 7. Export to SD card
#    - Export Tab → Select options
#    - Creates encrypted partition structure
#    - Manifest contains complete audit trail

# 8. Restart the GUI
#    - All data reloads from manifest.json
#    - Organization, locations, people, certificates, keys all restored
```

### Manifest Structure

```
output-directory/
├── manifest.json           # Master index
├── events/                 # Event log (audit trail)
├── keys/                   # Key material
├── certificates/           # X.509 certificates
├── yubikeys/              # YubiKey configurations
└── pki/                   # PKI hierarchies
```

### Graph Interaction

The organizational graph is fully interactive:

- **Drag nodes** - Click and drag to reposition people
- **Zoom** - Mouse wheel to zoom in/out
- **Pan** - Click empty space and drag to pan
- **Auto-layout** - Click "Auto Layout" for automatic positioning

Each node shows:
- Role (above): Root CA, Security Admin, Developer, etc.
- Name (center): Person's full name
- Email (below): Contact email

Edges show relationships with labels:
- "reports to" - Organizational hierarchy
- "delegates to" - Permission delegation
- "trusts" - Trust relationships

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built on top of excellent Rust cryptography libraries
- YubiKey support via `yubikey` crate
- OpenPGP support via `sequoia-openpgp`
- SSH support via `ssh-key` and `ssh-encoding`
