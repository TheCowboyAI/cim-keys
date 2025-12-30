// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Bounded Context
//!
//! This module provides the Organization bounded context for cim-keys.
//! It includes organizational structure, people, and their relationships.
//!
//! ## Domain Types
//!
//! **From cim-domain-organization** (canonical runtime types):
//! - `DomainOrganization` - Runtime organization entity
//! - `DomainOrganizationUnit` - Department/team/project units
//! - `DomainRole` - Organizational roles
//!
//! **From cim-domain-person** (canonical identity types):
//! - `DomainPerson` - Core person identity
//! - `PersonId` - Phantom-typed person identifier
//! - `EmploymentRelationship` - Person-organization binding
//!
//! **From cim-domain-location** (canonical location types):
//! - `Location` - Physical/virtual/logical locations
//!
//! **Bootstrap types** (JSON configuration):
//! - `Organization` - Bootstrap config for initial setup
//! - `OrganizationUnit` - Bootstrap unit configuration
//! - `Person` - Bootstrap person configuration
//!
//! ## Bounded Context Separation
//!
//! This context is responsible for:
//! - Organizational hierarchy (orgs, units, teams)
//! - People and their identities
//! - Roles and permissions within the organization
//! - Location management for physical security
//!
//! It does NOT handle:
//! - Cryptographic keys (see `pki` context)
//! - NATS credentials (see `nats` context)
//! - Hardware tokens (see `yubikey` context)

// Re-export canonical domain types from cim-domain-* crates
#[cfg(feature = "policy")]
pub use cim_domain_organization::{
    Organization as DomainOrganization,
    OrganizationUnit as DomainOrganizationUnit,
    Department as DomainDepartment,
    Team as DomainTeam,
    Role as DomainRole,
    OrganizationType,
    OrganizationStatus,
};

#[cfg(feature = "policy")]
pub use cim_domain_organization::entity::OrganizationUnitType as DomainOrganizationUnitType;

#[cfg(feature = "policy")]
pub use cim_domain_person::{
    Person as DomainPerson,
    PersonId,
    PersonMarker,
    PersonName,
    EmploymentRelationship,
    EmploymentRole,
};

pub use cim_domain_location::{
    Location,
    LocationMarker,
    LocationType,
    Address,
    GeoCoordinates,
    VirtualLocation,
    DefineLocation,
    LocationDomainEvent,
};

// Re-export bootstrap types from bootstrap module
pub use super::bootstrap::{
    Organization,
    OrganizationUnit,
    OrganizationUnitType,
    Person,
    PersonRole,
    RoleType,
    RoleScope,
    Permission,
};

// Re-export organization-specific phantom-typed IDs
pub use super::ids::{
    BootstrapOrgId,
    BootstrapOrgMarker,
    UnitId,
    UnitMarker,
};
