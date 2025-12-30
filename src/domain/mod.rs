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

// ============================================================================
// CORE MODULES
// ============================================================================

/// Phantom-typed entity IDs for compile-time type safety
pub mod ids;

/// Bootstrap configuration types for JSON loading
pub mod bootstrap;

// ============================================================================
// BOUNDED CONTEXT MODULES (DDD Compliant)
// ============================================================================

/// Organization Bounded Context - Orgs, People, Units, Locations
pub mod organization;

/// PKI Bounded Context - Certificates, Keys, Trust Hierarchy
pub mod pki;

/// NATS Bounded Context - Operators, Accounts, Users
pub mod nats;

/// YubiKey Bounded Context - Hardware Tokens, PIV Slots
pub mod yubikey;

// ============================================================================
// RE-EXPORTS FOR CONVENIENCE
// ============================================================================

// Re-export all ID types at module level for convenience
pub use ids::*;

// Re-export all bootstrap types for backward compatibility
// This maintains the existing API: cim_keys::domain::Organization, etc.
pub use bootstrap::*;
