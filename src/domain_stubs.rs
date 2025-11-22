//! Temporary stub types to allow compilation during migration to graph-based architecture
//!
//! These types are TEMPORARY placeholders to allow the codebase to compile while we migrate
//! to the new DomainObject-based architecture. They should be replaced with proper DomainObject
//! usage in each module.
//!
//! DO NOT ADD NEW USAGE OF THESE TYPES - use graph_ui::types::DomainObject instead.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Organizational Entity Stubs
// ============================================================================

/// STUB: Organization - replace with DomainObject::new("Organization")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub units: Vec<OrganizationUnit>,
    pub metadata: HashMap<String, String>,
}

/// STUB: OrganizationUnit - replace with DomainObject::new("OrganizationUnit")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUnit {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub parent_unit_id: Option<Uuid>,
    pub unit_type: Option<OrganizationUnitType>,
    pub responsible_person_id: Option<Uuid>,
}

/// STUB: OrganizationUnitType
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationUnitType {
    Division,
    Department,
    Team,
    Project,
    Service,
    Infrastructure,
}

/// STUB: Person - replace with DomainObject::new("Person")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub title: Option<String>,
    pub organization_id: Option<Uuid>,
    pub unit_ids: Vec<Uuid>,
}

/// STUB: ServiceAccount - replace with DomainObject::new("ServiceAccount")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    pub id: Uuid,
    pub name: String,
    pub purpose: String,
    pub owning_unit_id: Uuid,
    pub responsible_person_id: Uuid,
}

// ============================================================================
// Key Ownership and Context Stubs
// ============================================================================

/// STUB: KeyOwnership - replace with DomainRelationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyOwnership {
    pub person_id: Uuid,
    pub key_id: Uuid,
    pub role: String,
    pub organization_id: Option<Uuid>,
    pub delegations: Vec<String>, // Simplified
}

/// STUB: KeyOwnerRole
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyOwnerRole {
    RootAuthority,
    SecurityAdmin,
    Developer,
    ServiceAccount,
    DevOpsEngineer,
    Auditor,
}

/// STUB: KeyContext - replace with event correlation patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyContext {
    pub actor: KeyOwnership,
    pub org_id: Option<Uuid>,
    pub org_context: Option<OrganizationalPKI>,
    pub nats_identity: Option<String>,
    pub audit_requirements: Vec<AuditRequirement>,
}

/// STUB: OrganizationalPKI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalPKI {
    pub root_ca_org_id: Uuid,
    pub intermediate_cas: Vec<(Uuid, Uuid)>,
    pub policy_cas: Vec<String>, // Simplified
    pub cross_certifications: Vec<String>, // Simplified
}

/// STUB: AuditRequirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditRequirement {
    SecureLogging { log_level: String },
    SecurityNotification { channels: Vec<String> },
    WitnessRequired { witnesses: Vec<Uuid> },
    VideoRecording { camera_ids: Vec<String> },
    ComplianceReport { standards: Vec<String> },
}

// ============================================================================
// NATS Identity Stubs
// ============================================================================

/// STUB: AccountIdentity - replace with DomainObject relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountIdentity {
    Organization(Organization),
    OrganizationUnit(OrganizationUnit),
}

impl AccountIdentity {
    pub fn id(&self) -> Uuid {
        match self {
            AccountIdentity::Organization(o) => o.id,
            AccountIdentity::OrganizationUnit(u) => u.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AccountIdentity::Organization(o) => &o.name,
            AccountIdentity::OrganizationUnit(u) => &u.name,
        }
    }
}

/// STUB: UserIdentity - replace with DomainObject relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserIdentity {
    Person(Person),
    ServiceAccount(ServiceAccount),
}

impl UserIdentity {
    pub fn id(&self) -> Uuid {
        match self {
            UserIdentity::Person(p) => p.id,
            UserIdentity::ServiceAccount(s) => s.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            UserIdentity::Person(p) => &p.name,
            UserIdentity::ServiceAccount(s) => &s.name,
        }
    }

    pub fn responsible_person_id(&self) -> Option<Uuid> {
        match self {
            UserIdentity::Person(_) => None,
            UserIdentity::ServiceAccount(s) => Some(s.responsible_person_id),
        }
    }

    pub fn validate_accountability(&self) -> Result<(), String> {
        match self {
            UserIdentity::Person(_) => Ok(()),
            UserIdentity::ServiceAccount(_) => Ok(()), // Simplified validation
        }
    }
}
