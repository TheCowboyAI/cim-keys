// Copyright (c) 2025 - Cowboy AI, LLC.

//! Domain Module - Bounded Context Organization
//!
//! This module provides DDD-compliant bounded context separation for cim-keys.
//! Each bounded context is isolated in its own submodule with clear interfaces.
//!
//! ## Bounded Contexts
//!
//! - **Organization**: Organizations, People, Units, Locations, Roles, Policies
//! - **PKI**: Certificates (Root, Intermediate, Leaf), Cryptographic Keys
//! - **NATS**: Operators, Accounts, Users (NATS security infrastructure)
//! - **YubiKey**: Hardware tokens, PIV Slots
//!
//! ## Type-Safe IDs
//!
//! All entity IDs use phantom types for compile-time safety:
//!
//! ```ignore
//! use cim_keys::domain::{OrganizationId, PersonId};
//!
//! // Compile error: cannot pass PersonId where OrganizationId expected
//! fn get_org(id: OrganizationId) { ... }
//! get_org(person_id); // ERROR!
//! ```

pub mod ids;
pub mod bootstrap;

// Re-export all ID types at module level for convenience
pub use ids::*;

// Re-export all bootstrap types for backward compatibility
// This maintains the existing API: cim_keys::domain::Organization, etc.
pub use bootstrap::*;

// Future: Bounded context modules will be added here
// pub mod organization;
// pub mod pki;
// pub mod nats;
// pub mod yubikey;
