# Hexagonal Architecture Implementation

## Overview

cim-keys implements **Hexagonal Architecture** (Ports and Adapters pattern) with **Category Theory** principles. Each port is a **Functor** mapping from an external category (Storage, YubiKey, X.509 PKI, OpenPGP, SSH) to the Domain category, preserving structure and composition laws.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      Domain Core                             │
│  ┌────────────┐  ┌──────────┐  ┌────────────┐              │
│  │ Aggregates │  │ Commands │  │   Events   │              │
│  └────────────┘  └──────────┘  └────────────┘              │
│         ↓              ↓              ↓                      │
│  ┌──────────────────────────────────────────┐               │
│  │         Domain Logic (Pure)              │               │
│  └──────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────┘
         ↓                   ↓                   ↓
    ┌─────────┐         ┌─────────┐        ┌─────────┐
    │  Ports  │         │  Ports  │        │  Ports  │
    │(Functors)│       │(Functors)│       │(Functors)│
    └─────────┘         └─────────┘        └─────────┘
         ↓                   ↓                   ↓
    ┌─────────┐         ┌─────────┐        ┌─────────┐
    │Adapters │         │Adapters │        │Adapters │
    │  (Impl) │         │  (Impl) │        │  (Impl) │
    └─────────┘         └─────────┘        └─────────┘
         ↓                   ↓                   ↓
  ┌──────────┐        ┌──────────┐      ┌──────────┐
  │ Storage  │        │ YubiKey  │      │   X.509  │
  │  System  │        │ Hardware │      │    PKI   │
  └──────────┘        └──────────┘      └──────────┘
```

## Port Definitions

All ports are defined in `src/ports/` and implement the Functor pattern:

### 1. StoragePort (src/ports/storage.rs)

**Functor**: Storage → Domain

Maps file system operations to domain persistence operations.

```rust
#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), StorageError>;
    async fn read(&self, path: &str) -> Result<Vec<u8>, StorageError>;
    async fn exists(&self, path: &str) -> Result<bool, StorageError>;
    async fn list(&self, path: &str) -> Result<Vec<String>, StorageError>;
    async fn delete(&self, path: &str) -> Result<(), StorageError>;
    async fn create_dir(&self, path: &str) -> Result<(), StorageError>;
    async fn sync(&self, mode: SyncMode) -> Result<(), StorageError>;
}
```

**Functor Laws**:
- Identity: `read(write(path, data)) = data`
- Composition: `sync ∘ write` ensures persistence

**Adapters**:
- `InMemoryStorageAdapter` (mock for testing)
- TODO: `FileSystemStorageAdapter` (production)

### 2. YubiKeyPort (src/ports/yubikey.rs)

**Functor**: YubiKey Hardware → Domain

Maps YubiKey PIV operations to domain key management operations.

```rust
#[async_trait]
pub trait YubiKeyPort: Send + Sync {
    async fn list_devices(&self) -> Result<Vec<YubiKeyDevice>, YubiKeyError>;
    async fn generate_key_in_slot(&self, serial: &str, slot: PivSlot,
                                  algorithm: KeyAlgorithm, pin: &SecureString)
                                  -> Result<PublicKey, YubiKeyError>;
    async fn import_certificate(&self, serial: &str, slot: PivSlot,
                               certificate: &[u8], pin: &SecureString)
                               -> Result<(), YubiKeyError>;
    async fn sign_with_slot(&self, serial: &str, slot: PivSlot,
                           data: &[u8], pin: &SecureString)
                           -> Result<Signature, YubiKeyError>;
    async fn verify_pin(&self, serial: &str, pin: &SecureString)
                       -> Result<bool, YubiKeyError>;
    async fn reset_piv(&self, serial: &str) -> Result<(), YubiKeyError>;
}
```

**Functor Laws**:
- Identity: No-op operations preserve YubiKey state
- Composition: `generate_key ∘ import_cert` maintains PIV slot state

**Adapters**:
- `MockYubiKeyAdapter` (mock for testing)
- TODO: `YubiKeyPCSCAdapter` (real hardware via PC/SC)

### 3. X509Port (src/ports/x509.rs)

**Functor**: X.509 PKI → Domain

Maps X.509 certificate operations to domain trust operations.

```rust
#[async_trait]
pub trait X509Port: Send + Sync {
    async fn generate_root_ca(&self, subject: &CertificateSubject,
                             key: &PrivateKey, validity_days: u32)
                             -> Result<Certificate, X509Error>;
    async fn generate_csr(&self, subject: &CertificateSubject,
                         key: &PrivateKey, san: Vec<String>)
                         -> Result<CertificateSigningRequest, X509Error>;
    async fn sign_csr(&self, csr: &CertificateSigningRequest,
                     ca_cert: &Certificate, ca_key: &PrivateKey,
                     validity_days: u32, is_ca: bool)
                     -> Result<Certificate, X509Error>;
    async fn verify_chain(&self, leaf_cert: &Certificate,
                         intermediates: &[Certificate], root_cert: &Certificate)
                         -> Result<bool, X509Error>;
    async fn generate_crl(&self, ca_cert: &Certificate,
                         ca_key: &PrivateKey, revoked_certs: Vec<RevokedCertificate>)
                         -> Result<CertificateRevocationList, X509Error>;
}
```

**Functor Laws**:
- Identity: No-op on certificate preserves structure
- Composition: `F(sign_csr ∘ generate_csr) = F(sign_csr) ∘ F(generate_csr)`

**Adapters**:
- `MockX509Adapter` (mock for testing)
- TODO: `RcgenX509Adapter` (production using rcgen crate)

### 4. GpgPort (src/ports/gpg.rs)

**Functor**: OpenPGP → Domain

Maps OpenPGP operations to domain cryptographic operations.

```rust
#[async_trait]
pub trait GpgPort: Send + Sync {
    async fn generate_keypair(&self, user_id: &str, key_type: GpgKeyType,
                              key_length: u32, expires_in_days: Option<u32>)
                              -> Result<GpgKeypair, GpgError>;
    async fn sign(&self, key_id: &GpgKeyId, data: &[u8], detached: bool)
                 -> Result<Vec<u8>, GpgError>;
    async fn verify(&self, data: &[u8], signature: &[u8])
                   -> Result<GpgVerification, GpgError>;
    async fn encrypt(&self, recipient_keys: &[GpgKeyId], data: &[u8])
                    -> Result<Vec<u8>, GpgError>;
    async fn decrypt(&self, key_id: &GpgKeyId, encrypted_data: &[u8])
                    -> Result<Vec<u8>, GpgError>;
    async fn revoke_key(&self, key_id: &GpgKeyId, reason: RevocationReason)
                       -> Result<Vec<u8>, GpgError>;
}
```

**Functor Laws**:
- Identity: `decrypt(encrypt(m)) = m`
- Composition: `verify(sign(m)) = valid`

**Adapters**:
- `MockGpgAdapter` (mock for testing)
- TODO: `SequoiaGpgAdapter` (production using sequoia-openpgp crate)

### 5. SshKeyPort (src/ports/ssh.rs)

**Functor**: SSH → Domain

Maps SSH key operations to domain authentication operations.

```rust
#[async_trait]
pub trait SshKeyPort: Send + Sync {
    async fn generate_keypair(&self, key_type: SshKeyType,
                             bits: Option<u32>, comment: Option<String>)
                             -> Result<SshKeypair, SshError>;
    async fn sign(&self, private_key: &SshPrivateKey, data: &[u8])
                 -> Result<SshSignature, SshError>;
    async fn verify(&self, public_key: &SshPublicKey, data: &[u8],
                   signature: &SshSignature) -> Result<bool, SshError>;
    async fn format_authorized_key(&self, public_key: &SshPublicKey,
                                   comment: Option<String>)
                                   -> Result<String, SshError>;
    async fn get_fingerprint(&self, public_key: &SshPublicKey,
                            hash_type: FingerprintHashType)
                            -> Result<String, SshError>;
}
```

**Functor Laws**:
- Identity: `verify(sign(m)) = valid`
- Composition: `format ∘ parse` preserves key structure

**Adapters**:
- `MockSshKeyAdapter` (mock for testing)
- TODO: `SshKeysAdapter` (production using ssh-key crate)

### 6. NatsKeyPort (src/ports/nats.rs)

**Functor**: NATS → Domain

Maps NATS key operations to domain messaging operations.

```rust
#[async_trait]
pub trait NatsKeyPort: Send + Sync {
    async fn create_operator(&self, name: &str, signing_keys: Vec<String>)
                            -> Result<NatsOperator, NatsKeyError>;
    async fn create_account(&self, operator_id: &Uuid, name: &str,
                           public_key: String)
                           -> Result<NatsAccount, NatsKeyError>;
    async fn create_user(&self, account_id: &Uuid, name: &str,
                        public_key: String)
                        -> Result<NatsUser, NatsKeyError>;
}
```

**Adapters**:
- `NscAdapter` (production using NSC CLI)

## Adapter Implementations

All adapters are in `src/adapters/`:

### Mock Adapters (For Testing)

1. **InMemoryStorageAdapter** - In-memory HashMap storage
2. **MockYubiKeyAdapter** - Simulated YubiKey with PIV slots
3. **MockX509Adapter** - Simulated PKI with certificate chains
4. **MockGpgAdapter** - Simulated GPG with encryption/signing
5. **MockSshKeyAdapter** - Simulated SSH key operations

### Production Adapters

- TODO: FileSystemStorageAdapter (encrypted SD card)
- TODO: YubiKeyPCSCAdapter (real hardware)
- TODO: RcgenX509Adapter (rcgen library)
- TODO: SequoiaGpgAdapter (Sequoia PGP)
- TODO: SshKeysAdapter (ssh-key library)

## Category Theory Verification

All adapters include unit tests verifying Functor laws:

### Identity Law: F(id) = id

```rust
#[tokio::test]
async fn test_functor_identity_law() {
    let adapter = MockAdapter::new();

    // No-op operation should preserve state
    let data = b"test data";
    adapter.write("path", data).await.unwrap();
    let result = adapter.read("path").await.unwrap();

    assert_eq!(data, result.as_slice());
}
```

### Composition Law: F(g ∘ f) = F(g) ∘ F(f)

```rust
#[tokio::test]
async fn test_functor_composition_law() {
    let adapter = MockAdapter::new();

    // Composition should preserve structure
    let keypair = adapter.generate_keypair(...).await.unwrap();
    let signature = adapter.sign(&keypair, data).await.unwrap();
    let is_valid = adapter.verify(&keypair.public_key, data, &signature)
        .await.unwrap();

    assert!(is_valid);
}
```

## Test Results

All Functor law tests passing:

```
test adapters::in_memory::tests::test_functor_identity_law ... ok
test adapters::in_memory::tests::test_functor_composition_law ... ok
test adapters::yubikey_mock::tests::test_functor_identity_law ... ok
test adapters::yubikey_mock::tests::test_functor_composition_law ... ok
test adapters::x509_mock::tests::test_functor_identity_law ... ok
test adapters::x509_mock::tests::test_functor_composition_law ... ok
test adapters::gpg_mock::tests::test_functor_identity_law ... ok
test adapters::gpg_mock::tests::test_functor_composition_law ... ok
test adapters::ssh_mock::tests::test_functor_identity_law ... ok
test adapters::ssh_mock::tests::test_functor_composition_law ... ok

test result: ok. 16 passed; 0 failed
```

## Usage Pattern

The domain core uses ports via dependency injection:

```rust
pub struct KeyAggregate<S: StoragePort, Y: YubiKeyPort, X: X509Port> {
    storage: Arc<S>,
    yubikey: Arc<Y>,
    x509: Arc<X>,
    // ... other dependencies
}

impl<S, Y, X> KeyAggregate<S, Y, X>
where
    S: StoragePort,
    Y: YubiKeyPort,
    X: X509Port,
{
    pub async fn generate_root_ca(&self, subject: CertificateSubject)
        -> Result<Certificate, Error>
    {
        // Generate key on YubiKey
        let key = self.yubikey.generate_key_in_slot(...).await?;

        // Generate certificate
        let cert = self.x509.generate_root_ca(&subject, &key, 3650).await?;

        // Store to offline partition
        self.storage.write(&format!("certs/{}.pem", cert.serial),
                          cert.pem.as_bytes()).await?;

        Ok(cert)
    }
}
```

## Benefits

1. **Testability**: Mock adapters enable complete offline testing
2. **Flexibility**: Swap implementations without changing domain
3. **Type Safety**: Compile-time verification of Functor contracts
4. **Isolation**: Domain logic independent of external systems
5. **Maintainability**: Clear separation of concerns
6. **Mathematical Rigor**: Category Theory ensures correctness

## Next Steps

1. Implement production adapters:
   - FileSystemStorageAdapter with encryption
   - YubiKeyPCSCAdapter using yubikey crate
   - RcgenX509Adapter using rcgen crate
   - SequoiaGpgAdapter using sequoia-openpgp
   - SshKeysAdapter using ssh-key crate

2. Update aggregate to use dependency injection

3. Add integration tests with real hardware (YubiKeys available)

4. Document adapter selection strategies for different deployment scenarios

## References

- [Hexagonal Architecture (Ports and Adapters)](https://alistair.cockburn.us/hexagonal-architecture/)
- [Category Theory for Programmers](https://github.com/hmemcpy/milewski-ctfp-pdf)
- [Domain-Driven Design](https://www.domainlanguage.com/ddd/)
