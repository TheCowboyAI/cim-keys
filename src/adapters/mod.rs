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

pub use nsc::NscAdapter;
pub use in_memory::InMemoryStorageAdapter;