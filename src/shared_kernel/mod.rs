// Copyright (c) 2025 - Cowboy AI, LLC.

//! Shared Kernel - Minimal Cross-Context Communication Types
//!
//! This module provides the **Published Language** for cross-context communication
//! in the cim-keys bounded contexts. Following DDD principles:
//!
//! 1. **Minimal Surface** - Only essential types are shared
//! 2. **Reference Types** - Lightweight references, not full entities
//! 3. **Immutable DTOs** - No behavior, just data transfer
//! 4. **Context Independence** - No context-specific imports
//!
//! ## Context Map
//!
//! ```text
//! Organization Context [Upstream]
//!         │
//!         ▼ publishes
//! ┌─────────────────────────────┐
//! │   Published Language        │
//! │   - OrganizationRef         │
//! │   - PersonRef               │
//! │   - LocationRef             │
//! │   - KeyRef                  │
//! │   - CertificateRef          │
//! └─────────────────────────────┘
//!         │
//!         ▼ consumes via ACL
//! ┌───────────────┬───────────────┐
//! │ PKI Context   │ NATS Context  │
//! │ [Downstream]  │ [Downstream]  │
//! └───────────────┴───────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cim_keys::shared_kernel::{PersonRef, KeyRef};
//!
//! // Cross-context reference - no direct Person import needed
//! struct KeyOwnership {
//!     key: KeyRef,
//!     owner: PersonRef,
//! }
//! ```

pub mod published;
pub mod acl;

pub use published::{
    // Organization context references
    OrganizationRef,
    PersonRef,
    LocationRef,
    UnitRef,
    RoleRef,
    // PKI context references
    KeyRef,
    CertificateRef,
    // NATS context references
    OperatorRef,
    AccountRef,
    UserRef,
    // YubiKey context references
    DeviceRef,
    SlotRef,
};

// Re-export ACL traits and types
pub use acl::{
    // Adapter traits
    OrgContextAdapter,
    PersonAdapter,
    LocationAdapter,
    UnitAdapter,
    RoleAdapter,
    KeyAdapter,
    CertificateAdapter,
    OperatorAdapter,
    AccountAdapter,
    NatsUserAdapter,
    DeviceAdapter,
    SlotAdapter,
    // Helper functions for context-dependent conversions
    unit_to_ref,
    role_to_ref,
    // Cross-context compositions
    KeyOwnership,
    NatsUserMapping,
    YubiKeyAssignment,
    CertificateChain,
};
