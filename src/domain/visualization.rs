// Copyright (c) 2025 - Cowboy AI, LLC.

//! Visualization Bounded Context
//!
//! This module provides visualization-specific domain types for cim-keys.
//! These types are used for graph rendering and export manifests.
//!
//! ## Domain Types
//!
//! **Manifest Types**:
//! - Export manifest for SD card/air-gapped operations
//!
//! **Policy Visualization Types**:
//! - PolicyRoleNode - Role display in graph
//! - PolicyClaimNode - Claim display in graph
//! - PolicyCategoryNode - Category grouping in graph
//! - PolicyGroupNode - Separation class grouping in graph
//!
//! ## Bounded Context Separation
//!
//! This context is responsible for:
//! - Export manifest tracking
//! - Policy visualization groupings
//! - Graph layout metadata
//!
//! It does NOT handle:
//! - Core domain logic (see other contexts)
//! - Actual policy evaluation (see `bootstrap::Policy`)

use std::path::PathBuf;

// Re-export phantom-typed IDs for visualization
pub use super::ids::{
    ManifestId,
    ManifestMarker,
    PolicyRoleId,
    PolicyRoleMarker,
    ClaimId,
    ClaimMarker,
    PolicyCategoryId,
    PolicyCategoryMarker,
    PolicyGroupId,
    PolicyGroupMarker,
};

// Re-export SeparationClass from policy module
pub use crate::policy::SeparationClass;

// ============================================================================
// MANIFEST TYPES
// ============================================================================

/// Export manifest for graph visualization
///
/// Represents an export operation to encrypted storage.
#[derive(Debug, Clone)]
pub struct Manifest {
    pub id: ManifestId,
    pub name: String,
    pub destination: Option<PathBuf>,
    pub checksum: Option<String>,
}

impl Manifest {
    /// Create a new manifest
    pub fn new(
        id: ManifestId,
        name: String,
        destination: Option<PathBuf>,
        checksum: Option<String>,
    ) -> Self {
        Self { id, name, destination, checksum }
    }

    /// Check if manifest has been written
    pub fn is_written(&self) -> bool {
        self.checksum.is_some()
    }

    /// Get destination path as string
    pub fn destination_str(&self) -> Option<String> {
        self.destination.as_ref().map(|p| p.display().to_string())
    }
}

// ============================================================================
// POLICY VISUALIZATION TYPES
// ============================================================================

/// Policy role for graph visualization
///
/// Represents a role with its claims in the policy graph.
#[derive(Debug, Clone)]
pub struct PolicyRole {
    pub id: PolicyRoleId,
    pub name: String,
    pub purpose: String,
    pub level: u8,
    pub separation_class: SeparationClass,
    pub claim_count: usize,
}

impl PolicyRole {
    /// Create a new policy role
    pub fn new(
        id: PolicyRoleId,
        name: String,
        purpose: String,
        level: u8,
        separation_class: SeparationClass,
        claim_count: usize,
    ) -> Self {
        Self { id, name, purpose, level, separation_class, claim_count }
    }

    /// Check if this is a high-privilege role (level 0-1)
    pub fn is_high_privilege(&self) -> bool {
        self.level <= 1
    }
}

/// Policy claim view for graph visualization
///
/// Represents an individual permission/claim in the policy graph.
/// Named PolicyClaimView to avoid conflict with domain::PolicyClaim enum.
#[derive(Debug, Clone)]
pub struct PolicyClaimView {
    pub id: ClaimId,
    pub name: String,
    pub category: String,
}

impl PolicyClaimView {
    /// Create a new policy claim view
    pub fn new(id: ClaimId, name: String, category: String) -> Self {
        Self { id, name, category }
    }
}

/// Policy category for grouping claims
///
/// Represents a category of claims (e.g., "Key Management", "Infrastructure").
#[derive(Debug, Clone)]
pub struct PolicyCategory {
    pub id: PolicyCategoryId,
    pub name: String,
    pub claim_count: usize,
    pub expanded: bool,
}

impl PolicyCategory {
    /// Create a new policy category
    pub fn new(
        id: PolicyCategoryId,
        name: String,
        claim_count: usize,
        expanded: bool,
    ) -> Self {
        Self { id, name, claim_count, expanded }
    }

    /// Toggle expansion state
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }
}

/// Policy group for separation class grouping
///
/// Groups roles by their separation class (e.g., "Administration", "Operations").
#[derive(Debug, Clone)]
pub struct PolicyGroup {
    pub id: PolicyGroupId,
    pub name: String,
    pub separation_class: SeparationClass,
    pub role_count: usize,
    pub expanded: bool,
}

impl PolicyGroup {
    /// Create a new policy group
    pub fn new(
        id: PolicyGroupId,
        name: String,
        separation_class: SeparationClass,
        role_count: usize,
        expanded: bool,
    ) -> Self {
        Self { id, name, separation_class, role_count, expanded }
    }

    /// Toggle expansion state
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }
}

// ============================================================================
// DISPLAY IMPLEMENTATIONS
// ============================================================================

impl std::fmt::Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Manifest: {}", self.name)
    }
}

impl std::fmt::Display for PolicyRole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} (L{})", self.name, self.level)
    }
}

impl std::fmt::Display for PolicyClaimView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Display for PolicyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.claim_count)
    }
}

impl std::fmt::Display for PolicyGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({} roles)", self.name, self.role_count)
    }
}
