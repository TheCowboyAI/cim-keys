//! Ports (interfaces) for external system integration
//!
//! This module defines the interfaces that our domain uses to interact
//! with external systems. The domain only knows about these interfaces,
//! not the concrete implementations.
//!
//! **Category Theory Perspective:**
//! Each port is a **Functor** interface that maps from an external category
//! to the Domain category, preserving structure and composition laws.

pub mod nats;
pub mod storage;
pub mod yubikey;

pub use nats::{NatsKeyPort, NatsKeyOperations};
pub use storage::{StoragePort, StorageConfig, StorageMetadata, StorageError, SyncMode};
pub use yubikey::{
    YubiKeyPort, YubiKeyDevice, YubiKeyError, PivSlot, KeyAlgorithm,
    PublicKey, Signature, SecureString,
};