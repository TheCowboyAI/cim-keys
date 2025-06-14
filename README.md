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
- `yubikey_piv.rs` - YubiKey PIV operations
- `ssh_agent.rs` - SSH agent integration
- `tls_server.rs` - TLS server with generated certificates
- `gpg_signing.rs` - GPG signing and verification

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built on top of excellent Rust cryptography libraries
- YubiKey support via `yubikey` crate
- OpenPGP support via `sequoia-openpgp`
- SSH support via `ssh-key` and `ssh-encoding`
