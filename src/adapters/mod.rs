//! Adapters (implementations) for external system integration
//!
//! This module contains the concrete implementations of our ports.
//! These adapters handle the actual integration with external systems.
//!
//! **Category Theory Perspective:**
//! Each adapter is a **Functor** mapping from an external category (Storage, YubiKey, etc.)
//! to the Domain category, preserving the structure and composition laws.

pub mod nsc;
pub mod in_memory;
pub mod yubikey_mock;
pub mod x509_mock;
pub mod gpg_mock;
pub mod ssh_mock;

pub use nsc::NscAdapter;
pub use in_memory::InMemoryStorageAdapter;
pub use yubikey_mock::MockYubiKeyAdapter;
pub use x509_mock::MockX509Adapter;
pub use gpg_mock::MockGpgAdapter;
pub use ssh_mock::MockSshKeyAdapter;

// TODO: Implement real adapters for production use
// - FileSystemStorageAdapter for StoragePort
// - YubiKeyPCSCAdapter for YubiKeyPort (real hardware)
// - RcgenX509Adapter for X509Port (using rcgen crate)
// - SequoiaGpgAdapter for GpgPort (using sequoia-openpgp crate)
// - SshKeysAdapter for SshKeyPort (using ssh-key crate)