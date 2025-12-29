//! Policy integration for cim-keys
//!
//! This module provides policy-driven key management, enforcing organizational
//! policies for PKI operations including key generation, certificate issuance,
//! and YubiKey provisioning.
//!
//! # Claims-Based Policy Ontology
//!
//! The policy system is built on a Claims-Based Ontology:
//!
//! - **Claims** (`claims.rs`) - Atomic permission vocabulary (200+ standard claims)
//! - **Roles** (`roles.rs`) - Aggregates of claims with semantic purpose
//! - **Standard Roles** (`standard_roles.rs`) - Pre-defined role templates
//! - **Policies** - Roles + Conditions for contextual authorization
//!
//! See `docs/CLAIMS_POLICY_ONTOLOGY.md` for full documentation.

// ============================================================================
// CLAIMS-BASED ONTOLOGY (always available)
// ============================================================================

/// Claims vocabulary - atomic permission terms
pub mod claims;

/// Role aggregates - semantic claim compositions
pub mod roles;

/// Standard role definitions for IT/knowledge worker organizations
pub mod standard_roles;

pub use claims::{Claim, ClaimCategory};
pub use roles::{Role, RoleAssignment, RolePurpose, RoleStatus, SeparationClass, RoleError};
pub use standard_roles::{StandardRole, ALL_STANDARD_ROLES, get_standard_role};

// ============================================================================
// PKI POLICY ENGINE (requires "policy" feature)
// ============================================================================

#[cfg(feature = "policy")]
pub mod pki_policies;

#[cfg(feature = "policy")]
pub mod policy_engine;

#[cfg(feature = "policy")]
pub mod policy_commands;

#[cfg(feature = "policy")]
pub mod policy_events;

#[cfg(feature = "policy")]
pub use pki_policies::*;

#[cfg(feature = "policy")]
pub use policy_engine::*;

#[cfg(feature = "policy")]
pub use policy_commands::*;

#[cfg(feature = "policy")]
pub use policy_events::*;