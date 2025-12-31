// Copyright (c) 2025 - Cowboy AI, LLC.

//! Visualization Bounded Context
//!
//! This module defines the coproduct for visualization-related entities
//! including manifests, policy roles, claims, and categories.
//!
//! ## Entities in this Context
//! - Manifest (tier 2)
//! - PolicyRole (tier 1)
//! - PolicyClaim (tier 2)
//! - PolicyCategory (tier 1)
//! - PolicyGroup/SeparationClass (tier 0)

use std::fmt;
use uuid::Uuid;

use crate::domain::visualization::{
    Manifest, PolicyRole, PolicyClaimView, PolicyCategory, PolicyGroup,
};

/// Injection tag for Visualization bounded context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualizationInjection {
    Manifest,
    PolicyRole,
    PolicyClaim,
    PolicyCategory,
    PolicyGroup,
}

impl VisualizationInjection {
    /// Display name for this entity type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Manifest => "Manifest",
            Self::PolicyRole => "Policy Role",
            Self::PolicyClaim => "Policy Claim",
            Self::PolicyCategory => "Policy Category",
            Self::PolicyGroup => "Separation Class",
        }
    }

    /// Layout tier for hierarchical visualization
    pub fn layout_tier(&self) -> u8 {
        match self {
            Self::PolicyGroup => 0,
            Self::PolicyRole | Self::PolicyCategory => 1,
            Self::Manifest | Self::PolicyClaim => 2,
        }
    }

    /// Check if this is a policy-related type
    pub fn is_policy(&self) -> bool {
        matches!(
            self,
            Self::PolicyRole | Self::PolicyClaim | Self::PolicyCategory | Self::PolicyGroup
        )
    }
}

impl fmt::Display for VisualizationInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Inner data for Visualization context entities
#[derive(Debug, Clone)]
pub enum VisualizationData {
    Manifest(Manifest),
    PolicyRole(PolicyRole),
    PolicyClaim(PolicyClaimView),
    PolicyCategory(PolicyCategory),
    PolicyGroup(PolicyGroup),
}

/// Visualization Entity - Coproduct of visualization-related types
#[derive(Debug, Clone)]
pub struct VisualizationEntity {
    injection: VisualizationInjection,
    data: VisualizationData,
}

impl VisualizationEntity {
    // ========================================================================
    // Injection Functions
    // ========================================================================

    /// Inject Manifest into coproduct
    pub fn inject_manifest(manifest: Manifest) -> Self {
        Self {
            injection: VisualizationInjection::Manifest,
            data: VisualizationData::Manifest(manifest),
        }
    }

    /// Inject PolicyRole into coproduct
    pub fn inject_policy_role(role: PolicyRole) -> Self {
        Self {
            injection: VisualizationInjection::PolicyRole,
            data: VisualizationData::PolicyRole(role),
        }
    }

    /// Inject PolicyClaim into coproduct
    pub fn inject_policy_claim(claim: PolicyClaimView) -> Self {
        Self {
            injection: VisualizationInjection::PolicyClaim,
            data: VisualizationData::PolicyClaim(claim),
        }
    }

    /// Inject PolicyCategory into coproduct
    pub fn inject_policy_category(category: PolicyCategory) -> Self {
        Self {
            injection: VisualizationInjection::PolicyCategory,
            data: VisualizationData::PolicyCategory(category),
        }
    }

    /// Inject PolicyGroup into coproduct
    pub fn inject_policy_group(group: PolicyGroup) -> Self {
        Self {
            injection: VisualizationInjection::PolicyGroup,
            data: VisualizationData::PolicyGroup(group),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the injection tag
    pub fn injection(&self) -> VisualizationInjection {
        self.injection
    }

    /// Get reference to inner data
    pub fn data(&self) -> &VisualizationData {
        &self.data
    }

    /// Get entity ID
    pub fn id(&self) -> Uuid {
        match &self.data {
            VisualizationData::Manifest(m) => m.id.as_uuid(),
            VisualizationData::PolicyRole(r) => r.id.as_uuid(),
            VisualizationData::PolicyClaim(c) => c.id.as_uuid(),
            VisualizationData::PolicyCategory(c) => c.id.as_uuid(),
            VisualizationData::PolicyGroup(g) => g.id.as_uuid(),
        }
    }

    /// Get entity name
    pub fn name(&self) -> &str {
        match &self.data {
            VisualizationData::Manifest(m) => &m.name,
            VisualizationData::PolicyRole(r) => &r.name,
            VisualizationData::PolicyClaim(c) => &c.name,
            VisualizationData::PolicyCategory(c) => &c.name,
            VisualizationData::PolicyGroup(g) => &g.name,
        }
    }

    // ========================================================================
    // Universal Property (Fold)
    // ========================================================================

    /// Apply a fold to this entity
    pub fn fold<F: FoldVisualizationEntity>(&self, folder: &F) -> F::Output {
        match &self.data {
            VisualizationData::Manifest(m) => folder.fold_manifest(m),
            VisualizationData::PolicyRole(r) => folder.fold_policy_role(r),
            VisualizationData::PolicyClaim(c) => folder.fold_policy_claim(c),
            VisualizationData::PolicyCategory(c) => folder.fold_policy_category(c),
            VisualizationData::PolicyGroup(g) => folder.fold_policy_group(g),
        }
    }
}

/// Universal property trait for VisualizationEntity coproduct
pub trait FoldVisualizationEntity {
    type Output;

    fn fold_manifest(&self, manifest: &Manifest) -> Self::Output;
    fn fold_policy_role(&self, role: &PolicyRole) -> Self::Output;
    fn fold_policy_claim(&self, claim: &PolicyClaimView) -> Self::Output;
    fn fold_policy_category(&self, category: &PolicyCategory) -> Self::Output;
    fn fold_policy_group(&self, group: &PolicyGroup) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct InjectionFolder;

    impl FoldVisualizationEntity for InjectionFolder {
        type Output = VisualizationInjection;

        fn fold_manifest(&self, _: &Manifest) -> Self::Output {
            VisualizationInjection::Manifest
        }
        fn fold_policy_role(&self, _: &PolicyRole) -> Self::Output {
            VisualizationInjection::PolicyRole
        }
        fn fold_policy_claim(&self, _: &PolicyClaimView) -> Self::Output {
            VisualizationInjection::PolicyClaim
        }
        fn fold_policy_category(&self, _: &PolicyCategory) -> Self::Output {
            VisualizationInjection::PolicyCategory
        }
        fn fold_policy_group(&self, _: &PolicyGroup) -> Self::Output {
            VisualizationInjection::PolicyGroup
        }
    }
}
