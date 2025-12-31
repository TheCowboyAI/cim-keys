// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Bounded Context
//!
//! This module defines the coproduct for the Organization bounded context,
//! following DDD principles where each context owns its entity types.
//!
//! ## Entities in this Context
//! - Organization (root entity)
//! - OrganizationUnit (intermediate)
//! - Person (leaf)
//! - Location (leaf)
//! - Role (intermediate)
//! - Policy (intermediate)
//!
//! ## Categorical Structure
//!
//! OrganizationEntity is a coproduct with:
//! - Injection morphisms for each entity type
//! - Universal property via FoldOrganizationEntity trait

use std::fmt;
use uuid::Uuid;

use cim_domain::AggregateRoot;
use crate::domain::{
    Person, Organization, OrganizationUnit, Location, Policy, Role, KeyOwnerRole,
};

/// Injection tag for Organization bounded context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrganizationInjection {
    Person,
    Organization,
    OrganizationUnit,
    Location,
    Role,
    Policy,
}

impl OrganizationInjection {
    /// Display name for this entity type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Person => "Person",
            Self::Organization => "Organization",
            Self::OrganizationUnit => "Organizational Unit",
            Self::Location => "Location",
            Self::Role => "Role",
            Self::Policy => "Policy",
        }
    }

    /// Layout tier for hierarchical visualization
    pub fn layout_tier(&self) -> u8 {
        match self {
            Self::Organization => 0,
            Self::OrganizationUnit | Self::Role | Self::Policy => 1,
            Self::Person | Self::Location => 2,
        }
    }

    /// All creatable entity types in this context
    pub fn creatable() -> Vec<Self> {
        vec![
            Self::Organization,
            Self::OrganizationUnit,
            Self::Person,
            Self::Location,
            Self::Role,
            Self::Policy,
        ]
    }
}

impl fmt::Display for OrganizationInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Inner data for Organization context entities
#[derive(Debug, Clone)]
pub enum OrganizationData {
    Person { person: Person, role: KeyOwnerRole },
    Organization(Organization),
    OrganizationUnit(OrganizationUnit),
    Location(Location),
    Role(Role),
    Policy(Policy),
}

/// Organization Entity - Coproduct of organization-related types
#[derive(Debug, Clone)]
pub struct OrganizationEntity {
    injection: OrganizationInjection,
    data: OrganizationData,
}

impl OrganizationEntity {
    // ========================================================================
    // Injection Functions
    // ========================================================================

    /// Inject Person into coproduct
    pub fn inject_person(person: Person, role: KeyOwnerRole) -> Self {
        Self {
            injection: OrganizationInjection::Person,
            data: OrganizationData::Person { person, role },
        }
    }

    /// Inject Organization into coproduct
    pub fn inject_organization(org: Organization) -> Self {
        Self {
            injection: OrganizationInjection::Organization,
            data: OrganizationData::Organization(org),
        }
    }

    /// Inject OrganizationUnit into coproduct
    pub fn inject_unit(unit: OrganizationUnit) -> Self {
        Self {
            injection: OrganizationInjection::OrganizationUnit,
            data: OrganizationData::OrganizationUnit(unit),
        }
    }

    /// Inject Location into coproduct
    pub fn inject_location(loc: Location) -> Self {
        Self {
            injection: OrganizationInjection::Location,
            data: OrganizationData::Location(loc),
        }
    }

    /// Inject Role into coproduct
    pub fn inject_role(role: Role) -> Self {
        Self {
            injection: OrganizationInjection::Role,
            data: OrganizationData::Role(role),
        }
    }

    /// Inject Policy into coproduct
    pub fn inject_policy(policy: Policy) -> Self {
        Self {
            injection: OrganizationInjection::Policy,
            data: OrganizationData::Policy(policy),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get the injection tag
    pub fn injection(&self) -> OrganizationInjection {
        self.injection
    }

    /// Get reference to inner data
    pub fn data(&self) -> &OrganizationData {
        &self.data
    }

    /// Get entity ID
    pub fn id(&self) -> Uuid {
        match &self.data {
            OrganizationData::Person { person, .. } => person.id,
            OrganizationData::Organization(o) => o.id,
            OrganizationData::OrganizationUnit(u) => u.id,
            OrganizationData::Location(l) => *l.id().as_uuid(),
            OrganizationData::Role(r) => r.id,
            OrganizationData::Policy(p) => p.id,
        }
    }

    /// Get entity name
    pub fn name(&self) -> &str {
        match &self.data {
            OrganizationData::Person { person, .. } => &person.name,
            OrganizationData::Organization(o) => &o.name,
            OrganizationData::OrganizationUnit(u) => &u.name,
            OrganizationData::Location(l) => &l.name,
            OrganizationData::Role(r) => &r.name,
            OrganizationData::Policy(p) => &p.name,
        }
    }

    // ========================================================================
    // Universal Property (Fold)
    // ========================================================================

    /// Apply a fold to this entity (universal property of coproduct)
    pub fn fold<F: FoldOrganizationEntity>(&self, folder: &F) -> F::Output {
        match &self.data {
            OrganizationData::Person { person, role } => folder.fold_person(person, *role),
            OrganizationData::Organization(o) => folder.fold_organization(o),
            OrganizationData::OrganizationUnit(u) => folder.fold_unit(u),
            OrganizationData::Location(l) => folder.fold_location(l),
            OrganizationData::Role(r) => folder.fold_role(r),
            OrganizationData::Policy(p) => folder.fold_policy(p),
        }
    }
}

/// Universal property trait for OrganizationEntity coproduct
///
/// For any type X with morphisms from each component, this trait
/// captures the unique morphism OrganizationEntity → X.
pub trait FoldOrganizationEntity {
    type Output;

    fn fold_person(&self, person: &Person, role: KeyOwnerRole) -> Self::Output;
    fn fold_organization(&self, org: &Organization) -> Self::Output;
    fn fold_unit(&self, unit: &OrganizationUnit) -> Self::Output;
    fn fold_location(&self, loc: &Location) -> Self::Output;
    fn fold_role(&self, role: &Role) -> Self::Output;
    fn fold_policy(&self, policy: &Policy) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test fold that returns injection type
    struct InjectionFolder;

    impl FoldOrganizationEntity for InjectionFolder {
        type Output = OrganizationInjection;

        fn fold_person(&self, _: &Person, _: KeyOwnerRole) -> Self::Output {
            OrganizationInjection::Person
        }
        fn fold_organization(&self, _: &Organization) -> Self::Output {
            OrganizationInjection::Organization
        }
        fn fold_unit(&self, _: &OrganizationUnit) -> Self::Output {
            OrganizationInjection::OrganizationUnit
        }
        fn fold_location(&self, _: &Location) -> Self::Output {
            OrganizationInjection::Location
        }
        fn fold_role(&self, _: &Role) -> Self::Output {
            OrganizationInjection::Role
        }
        fn fold_policy(&self, _: &Policy) -> Self::Output {
            OrganizationInjection::Policy
        }
    }

    #[test]
    fn test_injection_preserves_type() {
        let org = Organization {
            id: Uuid::now_v7(),
            name: "test-org".to_string(),
            display_name: "Test Org".to_string(),
            description: None,
            parent_id: None,
            units: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        let entity = OrganizationEntity::inject_organization(org);
        let injection = entity.fold(&InjectionFolder);

        assert_eq!(injection, OrganizationInjection::Organization);
    }
}
