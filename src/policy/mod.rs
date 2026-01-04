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
//! - **CIM Claims** (`cim_claims.rs`) - CIM-specific claims (NATS, PKI, YubiKey, Trust)
//! - **Roles** (`roles.rs`) - Aggregates of claims with semantic purpose
//! - **CIM Roles** (`cim_role.rs`) - CIM-specific role compositions
//! - **Subject** (`subject.rs`) - NATS subject patterns for pub/sub
//! - **Capability** (`capability.rs`) - Capability = fold(Policies, Role × Subject)
//! - **Standard Roles** (`standard_roles.rs`) - Pre-defined role templates
//! - **Policies** - Roles + Conditions for contextual authorization
//!
//! # Mathematical Structure
//!
//! ```text
//! Capability = fold(Policies, Role × Subject)
//!
//! where:
//!   - Role × Subject is the categorical product (base capability)
//!   - fold is a list catamorphism over Policy
//!   - Subject = Command ⊎ Query ⊎ Event (CQRS coproduct)
//! ```
//!
//! See `docs/CLAIMS_POLICY_ONTOLOGY.md` for full documentation.

// ============================================================================
// CLAIMS-BASED ONTOLOGY (always available)
// ============================================================================

/// Claims vocabulary - atomic permission terms (legacy/generic)
pub mod claims;

/// CIM-specific claims - NATS, PKI, YubiKey, Trust
pub mod cim_claims;

/// Role aggregates - semantic claim compositions (legacy/generic)
pub mod roles;

/// CIM-specific roles using CimClaim
pub mod cim_role;

/// NATS subject patterns for pub/sub authorization
pub mod subject;

/// Capability - the categorical product Role × Subject with policy fold
pub mod capability;

/// Standard role definitions for IT/knowledge worker organizations
pub mod standard_roles;

// Legacy exports
pub use claims::{Claim, ClaimCategory};
pub use roles::{Role, RoleAssignment, RolePurpose, RoleStatus, SeparationClass, RoleError};
pub use standard_roles::{StandardRole, ALL_STANDARD_ROLES, get_standard_role};

// CIM-specific exports
pub use cim_claims::{CimClaim, ClaimSet, ClaimDomain, NatsClaim, PkiClaim, YubiKeyClaim, TrustClaim};
pub use cim_role::{CimRole, CimRoleStatus, CimRoleError, StandardCimRoles};
pub use subject::{Subject, SubjectPattern, SubjectToken, SubjectError, MessageType, TypedSubject, CimSubjects};
pub use capability::{
    BaseCapability, EffectiveCapability, Policy, PolicyConstraint,
    fold_policies, compute_capability,
    CqrsClaimRequirement, WriteAction, WriteScope, ReadScope, FieldAccess, ReplayPermission,
    ClaimValidationResult, ClaimViolation, validate_cqrs_operation,
    TypedCapability, CapabilityContext,
};

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