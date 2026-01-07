//! Master domain model for CIM key infrastructure
//!
//! This module provides domain types for CIM key infrastructure by:
//! - Importing canonical types from cim-domain-* crates
//! - Defining cim-keys specific extensions (NatsPermissions, KeyOwnership, etc.)
//! - Providing bootstrap configuration types for JSON loading
//!
//! ## Architecture
//!
//! **Domain Types** (from cim-domain-*):
//! - Organization, Department, Team, Role from cim-domain-organization
//! - Person (core identity) from cim-domain-person
//! - Location from cim-domain-location
//! - EdgeConcept, RelationshipCategory from cim-domain-relationship
//!
//! **Bootstrap Types** (cim-keys specific):
//! - BootstrapOrganization - JSON config with embedded units
//! - BootstrapPerson - JSON config with roles/permissions
//!
//! **cim-keys Extensions**:
//! - KeyOwnership, KeyOwnerRole - PKI ownership model
//! - NatsPermissions, NatsIdentity - NATS security configuration
//! - Policy, PolicyClaim - Claims-based authorization

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Timelike};
use std::collections::HashMap;

// ============================================================================
// DOMAIN IMPORTS FROM CIM-DOMAIN-* CRATES
// ============================================================================

// Import Location domain from cim-domain-location
pub use cim_domain_location::{
    Location,
    LocationMarker,
    Address,
    GeoCoordinates,
    LocationType,  // Physical, Virtual, Logical, Hybrid
    VirtualLocation,
    DefineLocation,  // Command
    LocationDomainEvent,  // Events
};

// Import Organization domain types from cim-domain-organization
// These are the canonical DDD types for runtime operations
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

// OrganizationUnitType from entity submodule (not exported at crate root)
#[cfg(feature = "policy")]
pub use cim_domain_organization::entity::OrganizationUnitType as DomainOrganizationUnitType;

// Import Person domain types from cim-domain-person
// Person is core identity only (name, birth/death, lifecycle)
#[cfg(feature = "policy")]
pub use cim_domain_person::{
    Person as DomainPerson,
    PersonId,
    PersonMarker,
    PersonName,
    EmploymentRelationship,
    EmploymentRole,
};

// Import Relationship domain types from cim-domain-relationship
#[cfg(feature = "policy")]
pub use cim_domain_relationship::{
    EdgeConcept,
    RelationshipCategory,
    EntityRef,
    EntityType as RelationshipEntityType,
    RelationshipQuality,
};

// Import Agent domain from cim-domain-agent (optional feature)
#[cfg(feature = "cim-domain-agent")]
pub use cim_domain_agent::{
    Agent,
    AgentType,
};

// ============================================================================
// BOOTSTRAP CONFIGURATION TYPES
// ============================================================================
// These types are for JSON deserialization of bootstrap configuration files.
// They provide a "denormalized" view with embedded relationships for convenience.
// After loading, convert to proper domain types for runtime operations.

/// Bootstrap organization - a domain entity that projects to JSON.
///
/// CIM creates these entities; JSON is just the projection format.
/// EntityId auto-generates UUID v7 on creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: super::ids::BootstrapOrgId,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub parent_id: Option<super::ids::BootstrapOrgId>,
    /// Embedded units for bootstrap convenience (denormalized)
    pub units: Vec<OrganizationUnit>,
    pub metadata: HashMap<String, String>,
}

impl Organization {
    /// Create a new Organization with auto-generated ID.
    pub fn new(name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: super::ids::BootstrapOrgId::new(),
            name: name.into(),
            display_name: display_name.into(),
            description: None,
            parent_id: None,
            units: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Builder: set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Builder: set parent organization.
    pub fn with_parent(mut self, parent_id: super::ids::BootstrapOrgId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Builder: add a unit.
    pub fn with_unit(mut self, unit: OrganizationUnit) -> Self {
        self.units.push(unit);
        self
    }
}

/// Bootstrap organizational unit - a domain entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUnit {
    pub id: super::ids::UnitId,
    pub name: String,
    pub unit_type: OrganizationUnitType,
    pub parent_unit_id: Option<super::ids::UnitId>,
    pub responsible_person_id: Option<Uuid>,  // PersonId from external crate
    /// NATS account name for NSC export (e.g., "core", "media", "development")
    pub nats_account_name: Option<String>,
}

impl OrganizationUnit {
    /// Create a new OrganizationUnit with auto-generated ID.
    pub fn new(name: impl Into<String>, unit_type: OrganizationUnitType) -> Self {
        Self {
            id: super::ids::UnitId::new(),
            name: name.into(),
            unit_type,
            parent_unit_id: None,
            responsible_person_id: None,
            nats_account_name: None,
        }
    }

    /// Builder: set parent unit.
    pub fn with_parent(mut self, parent_id: super::ids::UnitId) -> Self {
        self.parent_unit_id = Some(parent_id);
        self
    }

    /// Builder: set responsible person.
    pub fn with_responsible_person(mut self, person_id: Uuid) -> Self {
        self.responsible_person_id = Some(person_id);
        self
    }

    /// Builder: set NATS account name.
    pub fn with_nats_account(mut self, account_name: impl Into<String>) -> Self {
        self.nats_account_name = Some(account_name.into());
        self
    }
}

/// Type of organizational unit (bootstrap version)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganizationUnitType {
    Division,
    Department,
    Team,
    Project,
    Service,
    Infrastructure,
}

/// Bootstrap person - a domain entity with auto-generated ID.
///
/// Contains both person identity AND organizational membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: super::ids::BootstrapPersonId,
    pub name: String,
    pub email: String,
    pub roles: Vec<PersonRole>,
    pub organization_id: super::ids::BootstrapOrgId,
    pub unit_ids: Vec<super::ids::UnitId>,
    pub active: bool,
    /// NATS permissions for NSC export (for Service role persons)
    pub nats_permissions: Option<NatsPermissions>,
    /// Owner/manager of this person or service account
    #[serde(default)]
    pub owner_id: Option<super::ids::BootstrapPersonId>,
}

impl Person {
    /// Create a new Person with auto-generated ID.
    pub fn new(
        name: impl Into<String>,
        email: impl Into<String>,
        organization_id: super::ids::BootstrapOrgId,
    ) -> Self {
        Self {
            id: super::ids::BootstrapPersonId::new(),
            name: name.into(),
            email: email.into(),
            roles: Vec::new(),
            organization_id,
            unit_ids: Vec::new(),
            active: true,
            nats_permissions: None,
            owner_id: None,
        }
    }

    /// Builder: add a role.
    pub fn with_role(mut self, role: PersonRole) -> Self {
        self.roles.push(role);
        self
    }

    /// Builder: add to a unit.
    pub fn in_unit(mut self, unit_id: super::ids::UnitId) -> Self {
        self.unit_ids.push(unit_id);
        self
    }

    /// Builder: set owner/manager.
    pub fn with_owner(mut self, owner_id: super::ids::BootstrapPersonId) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    /// Builder: set NATS permissions.
    pub fn with_nats_permissions(mut self, perms: NatsPermissions) -> Self {
        self.nats_permissions = Some(perms);
        self
    }
}

/// Role a person can have
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonRole {
    pub role_type: RoleType,
    pub scope: RoleScope,
    pub permissions: Vec<Permission>,
}

/// Type of role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RoleType {
    Executive,
    Administrator,
    Developer,
    Operator,
    Auditor,
    Service,
}

/// Scope of a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleScope {
    Organization,
    Unit(Uuid),
    System,
}

/// Permission that can be granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    CreateKeys,
    SignCertificates,
    SignContract,
    RevokeKeys,
    ManageInfrastructure,
    ViewAuditLogs,
    ModifyConfiguration,
    ConfigureCompliancePolicy,
    InitiateHold,
    ManageDataRetention,
}

/// NATS permissions for publish/subscribe operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NatsPermissions {
    /// Subjects this entity can publish to (e.g., ["thecowboyai.org.person.>"])
    pub publish: Vec<String>,
    /// Subjects this entity can subscribe to
    pub subscribe: Vec<String>,
    /// Allow request-reply responses
    pub allow_responses: bool,
    /// Maximum message payload size in bytes
    pub max_payload: Option<usize>,
}

// ============================================================================
// CONVERSION IMPLEMENTATIONS
// ============================================================================
// These implementations allow converting between bootstrap types (for JSON loading)
// and domain types (for runtime operations with cim-domain-* crates).

#[cfg(feature = "policy")]
impl Organization {
    /// Convert bootstrap Organization to domain Organization
    ///
    /// Note: The embedded `units` are NOT included in the domain Organization.
    /// They should be converted separately to DomainOrganizationUnit entities.
    pub fn to_domain(&self) -> DomainOrganization {
        use cim_domain::EntityId;
        let now = Utc::now();
        DomainOrganization {
            id: EntityId::from_uuid(self.id.as_uuid()),
            name: self.name.clone(),
            display_name: self.display_name.clone(),
            description: self.description.clone(),
            parent_id: self.parent_id.map(|p| EntityId::from_uuid(p.as_uuid())),
            organization_type: OrganizationType::Corporation, // Default
            status: OrganizationStatus::Active,
            founded_date: None,
            metadata: serde_json::to_value(&self.metadata).unwrap_or_default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Extract units as domain OrganizationUnit entities
    pub fn units_to_domain(&self) -> Vec<DomainOrganizationUnit> {
        use cim_domain::EntityId;
        let now = Utc::now();
        self.units.iter().map(|unit| {
            DomainOrganizationUnit {
                id: EntityId::from_uuid(unit.id.as_uuid()),
                organization_id: EntityId::from_uuid(self.id.as_uuid()),
                parent_id: unit.parent_unit_id.map(|p| EntityId::from_uuid(p.as_uuid())),
                unit_type: match unit.unit_type {
                    OrganizationUnitType::Division => DomainOrganizationUnitType::Division,
                    OrganizationUnitType::Department => DomainOrganizationUnitType::Branch,
                    OrganizationUnitType::Team => DomainOrganizationUnitType::Office,
                    OrganizationUnitType::Project => DomainOrganizationUnitType::BusinessUnit,
                    OrganizationUnitType::Service => DomainOrganizationUnitType::CostCenter,
                    OrganizationUnitType::Infrastructure => DomainOrganizationUnitType::Other("Infrastructure".to_string()),
                },
                name: unit.name.clone(),
                code: unit.name.to_lowercase().replace(' ', "-"),
                description: unit.nats_account_name.clone(),
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: now,
                updated_at: now,
            }
        }).collect()
    }
}

#[cfg(feature = "policy")]
impl Person {
    /// Convert bootstrap Person to domain Person (core identity only)
    ///
    /// Note: Organization membership (roles, unit_ids) are NOT included.
    /// They should be represented as EmploymentRelationship from cim-domain-person.
    pub fn to_domain_person(&self) -> DomainPerson {
        // Parse name into first/last
        let parts: Vec<&str> = self.name.split_whitespace().collect();
        let (first, last) = if parts.len() >= 2 {
            (parts[0].to_string(), parts[1..].join(" "))
        } else {
            (self.name.clone(), String::new())
        };

        DomainPerson::new(
            PersonId::from_uuid(self.id.as_uuid()),
            PersonName::new(first, last),
        )
    }

    /// Create an EmploymentRelationship for this person's org membership
    pub fn to_employment(&self) -> EmploymentRelationship {
        use chrono::NaiveDate;
        use cim_domain_person::cross_domain::person_organization::{
            EmploymentType, EmploymentMetadata, RemoteWorkArrangement,
        };
        EmploymentRelationship {
            person_id: PersonId::from_uuid(self.id.as_uuid()),
            organization_id: self.organization_id.as_uuid(),
            role: EmploymentRole {
                title: self.roles.first()
                    .map(|r| format!("{:?}", r.role_type))
                    .unwrap_or_else(|| "Member".to_string()),
                level: None,
                category: None,
            },
            department_id: self.unit_ids.first().map(|u| u.as_uuid()),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), // Default
            end_date: None,
            employment_type: EmploymentType::FullTime,
            reporting_to: self.owner_id.map(|o| PersonId::from_uuid(o.as_uuid())),
            is_primary: true,
            metadata: EmploymentMetadata {
                compensation: None,
                work_location_id: None,
                remote_work: RemoteWorkArrangement::OnSite,
                agreement_id: None,
                custom_attributes: std::collections::HashMap::new(),
            },
        }
    }
}

/// Key ownership tied to a person in the organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyOwnership {
    /// The person who owns/controls this key
    pub person_id: Uuid,

    /// The organization this key belongs to
    pub organization_id: Uuid,

    /// Role of the person in the organization
    pub role: KeyOwnerRole,

    /// Delegation permissions
    pub delegations: Vec<KeyDelegation>,
}

/// Role of a key owner
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyOwnerRole {
    /// Root key holder (highest authority)
    RootAuthority,

    /// Security administrator
    SecurityAdmin,

    /// Developer with signing rights
    Developer,

    /// Service account
    ServiceAccount,

    /// Backup key holder
    BackupHolder,

    /// External auditor
    Auditor,
}

/// Key delegation to another person
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyDelegation {
    pub delegated_to: Uuid,

    pub permissions: Vec<KeyPermission>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Permissions that can be delegated
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyPermission {
    Sign,
    Encrypt,
    Decrypt,
    CertifyOthers,
    RevokeOthers,
    BackupAccess,
}

/// YubiKey configuration for a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiKeyConfig {
    pub serial: String,
    pub name: String,
    pub owner_email: String,
    pub role: YubiKeyRole,
    pub piv: PivConfig,
    pub pgp: Option<PgpConfig>,
    pub fido: Option<FidoConfig>,
    pub ssl: Option<SslConfig>,
}

/// Role of the YubiKey in the organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YubiKeyRole {
    RootCA,      // Holds root CA private key
    Backup,      // Backup root CA key
    User,        // Regular user authentication/signing
    Service,     // Service account key
}

/// PIV (Personal Identity Verification) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivConfig {
    pub default_pin: String,
    pub default_puk: String,
    pub pin: String,
    pub puk: String,
    pub mgmt_key: String,
    pub mgmt_key_old: Option<String>,
    pub piv_alg: PivAlgorithm,
}

/// PIV algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PivAlgorithm {
    #[serde(rename = "aes128")]
    Aes128,
    #[serde(rename = "aes192")]
    Aes192,
    #[serde(rename = "aes256")]
    Aes256,
    #[serde(rename = "tdes")]
    TripleDes,
}

/// PGP/GPG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgpConfig {
    pub user_pin: String,
    pub user_pin_old: Option<String>,
    pub admin_pin: String,
    pub admin_pin_old: Option<String>,
    pub reset_code: Option<String>,
    pub key_type_auth: String,  // e.g., "ed25519"
    pub key_type_sign: String,  // e.g., "ed25519"
    pub key_type_encr: String,  // e.g., "cv25519"
    pub expiration: String,     // e.g., "5y"
}

/// FIDO/U2F configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidoConfig {
    pub pin: String,
    pub retries: u8,
}

/// SSL/TLS certificate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub common_name: Option<String>,
    pub email: Option<String>,
    pub key_type: String,     // e.g., "prime256v1", "rsa2048"
    pub expiration: String,   // in days, e.g., "1825"
}

/// NATS identity tied to organizational structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsIdentity {
    /// The operator represents the organization
    pub operator_org_id: Uuid,

    /// Accounts map to organizational units
    pub account_units: Vec<(String, Uuid)>,

    /// Users map to people
    pub user_people: Vec<(String, Uuid)>,

    /// Service accounts for automated systems
    pub service_accounts: Vec<ServiceAccount>,
}

/// Service account for automated systems
///
/// CRITICAL ACCOUNTABILITY REQUIREMENT:
/// Every ServiceAccount MUST report to a single Person who is responsible
/// for its operations, security, and lifecycle management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    pub id: Uuid,
    pub name: String,
    pub purpose: String,

    /// Which unit owns this service account
    pub owning_unit_id: Uuid,

    /// REQUIRED: Person responsible for this service account
    /// This person is accountable for:
    /// - Security and access control
    /// - Key rotation and credential management
    /// - Incident response and audit compliance
    /// - Lifecycle (creation, updates, deactivation)
    pub responsible_person_id: Uuid,

    /// Created timestamp

    /// Active status
    pub active: bool,
}

impl ServiceAccount {
    /// Create a new service account with required accountability
    pub fn new(
        name: String,
        purpose: String,
        owning_unit_id: Uuid,
        responsible_person_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            purpose,
            owning_unit_id,
            responsible_person_id,
            active: true,
        }
    }

    /// Validate that this service account has proper accountability
    pub fn validate_accountability(&self, organization: &Organization) -> Result<(), String> {
        // Ensure responsible person exists in organization
        let person_exists = organization
            .units
            .iter()
            .any(|_unit| {
                // TODO: Check if responsible_person_id exists in org's people
                true // Placeholder
            });

        if !person_exists {
            return Err(format!(
                "ServiceAccount {} responsible_person_id {} not found in organization",
                self.name, self.responsible_person_id
            ));
        }

        Ok(())
    }
}

/// User identity for NATS authentication
///
/// Maps to NATS User (U prefix) NKeys:
/// - Person: Human user with email and roles (self-accountable)
/// - Agent: Automated agent (AI, automation, etc.) from cim-domain-agent (MUST have responsible_person_id)
/// - ServiceAccount: System service account (databases, APIs, etc.) (MUST have responsible_person_id)
///
/// CRITICAL ACCOUNTABILITY REQUIREMENT:
/// - Person: Self-accountable human
/// - Agent: MUST report to a single responsible Person
/// - ServiceAccount: MUST report to a single responsible Person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserIdentity {
    /// Human user (self-accountable)
    Person(Person),

    /// Automated agent (requires 'agent' feature)
    /// MUST have responsible_person_id set in Agent structure
    #[cfg(feature = "cim-domain-agent")]
    Agent(Agent),

    /// Service account (non-agent automated system)
    /// MUST have responsible_person_id set
    ServiceAccount(ServiceAccount),
}

impl UserIdentity {
    /// Get the unique ID for this user identity
    pub fn id(&self) -> Uuid {
        match self {
            UserIdentity::Person(p) => p.id.as_uuid(),
            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(a) => a.id().into(), // Convert AgentId to Uuid
            UserIdentity::ServiceAccount(sa) => sa.id,
        }
    }

    /// Get the name for this user identity
    pub fn name(&self) -> &str {
        match self {
            UserIdentity::Person(p) => &p.name,
            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(a) => a.metadata().name(),
            UserIdentity::ServiceAccount(sa) => &sa.name,
        }
    }

    /// Get the organization ID for this user identity
    pub fn organization_id(&self) -> Uuid {
        match self {
            UserIdentity::Person(p) => p.organization_id.as_uuid(),
            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(a) => a.metadata().owner_id(), // Using owner_id as org for now
            UserIdentity::ServiceAccount(sa) => sa.owning_unit_id, // TODO: Get org from unit
        }
    }

    /// Check if this identity is active
    pub fn is_active(&self) -> bool {
        match self {
            UserIdentity::Person(p) => p.active,
            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(a) => a.status().can_execute(), // Active status means can execute
            UserIdentity::ServiceAccount(sa) => sa.active,
        }
    }

    /// Get a descriptive identifier for NATS credential naming
    pub fn credential_identifier(&self) -> String {
        match self {
            UserIdentity::Person(p) => format!("person-{}", p.email),
            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(a) => format!("agent-{}", a.metadata().name().to_lowercase().replace(' ', "-")),
            UserIdentity::ServiceAccount(sa) => format!("service-{}", sa.name.to_lowercase().replace(' ', "-")),
        }
    }

    /// Get the responsible person ID for this user identity
    ///
    /// Returns:
    /// - Person: None (self-accountable)
    /// - Agent: Some(responsible_person_id) - REQUIRED
    /// - ServiceAccount: Some(responsible_person_id) - REQUIRED
    ///
    /// CRITICAL: Agents and ServiceAccounts MUST have accountability to a human!
    pub fn responsible_person_id(&self) -> Option<Uuid> {
        match self {
            UserIdentity::Person(_) => None, // Self-accountable
            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(a) => Some(a.metadata().owner_id()),
            UserIdentity::ServiceAccount(sa) => Some(sa.responsible_person_id),
        }
    }

    /// Validate accountability requirements
    ///
    /// Ensures that:
    /// - Agents have a responsible person set
    /// - ServiceAccounts have a responsible person set
    /// - The responsible person exists in the organization
    pub fn validate_accountability(&self, organization: &Organization) -> Result<(), String> {
        match self {
            UserIdentity::Person(_) => Ok(()), // Self-accountable, no validation needed

            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(agent) => {
                // Verify responsible person exists
                let person_exists = organization.units.iter().any(|_unit| {
                    // TODO: Check if agent.metadata().owner_id() exists in org's people
                    true // Placeholder
                });

                if !person_exists {
                    return Err(format!(
                        "Agent {} responsible_person_id {} not found in organization",
                        agent.metadata().name(), agent.metadata().owner_id()
                    ));
                }

                Ok(())
            }

            UserIdentity::ServiceAccount(sa) => sa.validate_accountability(organization),
        }
    }

    /// Get accountability information for audit logging
    pub fn accountability_info(&self) -> String {
        match self {
            UserIdentity::Person(p) => format!("Person {} (self-accountable)", p.name),
            #[cfg(feature = "cim-domain-agent")]
            UserIdentity::Agent(a) => format!(
                "Agent {} (responsible: person-{})",
                a.metadata().name(), a.metadata().owner_id()
            ),
            UserIdentity::ServiceAccount(sa) => format!(
                "ServiceAccount {} (responsible: person-{})",
                sa.name, sa.responsible_person_id
            ),
        }
    }
}

/// Account identity for NATS authentication
///
/// Maps to NATS Account (A prefix) NKeys:
/// - Organization: Top-level org becomes an account
/// - OrganizationUnit: Departments/teams/projects become sub-accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountIdentity {
    /// Top-level organization
    Organization(Organization),

    /// Organizational unit (department, team, project)
    OrganizationUnit(OrganizationUnit),
}

impl AccountIdentity {
    /// Get the unique ID for this account identity
    pub fn id(&self) -> Uuid {
        match self {
            AccountIdentity::Organization(o) => o.id.as_uuid(),
            AccountIdentity::OrganizationUnit(u) => u.id.as_uuid(),
        }
    }

    /// Get the name for this account identity
    pub fn name(&self) -> &str {
        match self {
            AccountIdentity::Organization(o) => &o.name,
            AccountIdentity::OrganizationUnit(u) => &u.name,
        }
    }

    /// Get account type description
    pub fn account_type(&self) -> &'static str {
        match self {
            AccountIdentity::Organization(_) => "Organization",
            AccountIdentity::OrganizationUnit(u) => match u.unit_type {
                OrganizationUnitType::Division => "Division",
                OrganizationUnitType::Department => "Department",
                OrganizationUnitType::Team => "Team",
                OrganizationUnitType::Project => "Project",
                OrganizationUnitType::Service => "Service",
                OrganizationUnitType::Infrastructure => "Infrastructure",
            },
        }
    }
}

/// Certificate authority hierarchy mapped to organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalPKI {
    /// Root CA for the organization
    pub root_ca_org_id: Uuid,

    /// Intermediate CAs for organizational units
    pub intermediate_cas: Vec<(Uuid, Uuid)>,

    /// Policy CA for special purposes
    pub policy_cas: Vec<PolicyCA>,

    /// Cross-certifications with partner orgs
    pub cross_certifications: Vec<(Uuid, Uuid)>,
}

/// Policy-specific certificate authority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCA {
    pub name: String,
    pub purpose: PolicyPurpose,
    pub constraints: Vec<PolicyConstraint>,
}

/// Purpose of a policy CA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyPurpose {
    CodeSigning,
    EmailEncryption,
    DocumentSigning,
    TimestampAuthority,
    DeviceAuthentication,
}

/// Constraints on policy CA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyConstraint {
    MaxPathLength(u32),
    NameConstraints { permitted: Vec<String>, excluded: Vec<String> },
    KeyUsageRestriction(Vec<String>),
    ValidityPeriodMax { days: u32 },
}

/// Integration context for key operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyContext {
    /// Who is performing the operation
    pub actor: KeyOwnership,

    /// Organizational context
    pub org_context: Option<OrganizationalPKI>,

    /// NATS identity mapping
    pub nats_identity: Option<NatsIdentity>,

    /// Audit trail requirements
    pub audit_requirements: Vec<AuditRequirement>,
}

/// Audit requirements for key operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditRequirement {
    /// Log to secure audit trail
    SecureLogging { log_level: String },

    /// Notify security team
    SecurityNotification { channels: Vec<String> },

    /// Require witness
    WitnessRequired { witnesses: Vec<Uuid> },

    /// Video recording of ceremony
    VideoRecording { camera_ids: Vec<String> },

    /// Compliance reporting
    ComplianceReport { standards: Vec<String> },
}

// Helper functions to create proper domain integrations

/// Create a key ownership record for a person in an organization
pub fn create_key_ownership(
    person_id: Uuid,
    org_id: Uuid,
    role: KeyOwnerRole,
) -> KeyOwnership {
    KeyOwnership {
        person_id,
        organization_id: org_id,
        role,
        delegations: Vec::new(),
    }
}

/// Create NATS identity mapping for an organization
pub fn create_nats_identity(
    org_id: Uuid,
    accounts: Vec<(String, Uuid)>,
    users: Vec<(String, Uuid)>,
) -> NatsIdentity {
    NatsIdentity {
        operator_org_id: org_id,
        account_units: accounts,
        user_people: users,
        service_accounts: Vec::new(),
    }
}

// Display implementation for GUI
impl std::fmt::Display for KeyOwnerRole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KeyOwnerRole::RootAuthority => write!(f, "Root Authority"),
            KeyOwnerRole::SecurityAdmin => write!(f, "Security Admin"),
            KeyOwnerRole::Developer => write!(f, "Developer"),
            KeyOwnerRole::ServiceAccount => write!(f, "Service Account"),
            KeyOwnerRole::BackupHolder => write!(f, "Backup Holder"),
            KeyOwnerRole::Auditor => write!(f, "Auditor"),
        }
    }
}

// ========================================================================
// CLAIMS-BASED SECURITY: POLICY SYSTEM
// ========================================================================

/// Policy entity with claims-based permissions - a domain entity.
///
/// Policies define what actions are permitted based on conditions.
/// Multiple policies can be composed (claims are unioned).
/// Policies are evaluated in priority order (higher priority wins).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: super::ids::BootstrapPolicyId,
    pub name: String,
    pub description: String,
    pub claims: Vec<PolicyClaim>,
    pub conditions: Vec<PolicyCondition>,
    pub priority: i32,
    pub enabled: bool,
    pub created_by: super::ids::BootstrapPersonId,
    pub metadata: HashMap<String, String>,
}

impl Policy {
    /// Create a new Policy with auto-generated ID.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        created_by: super::ids::BootstrapPersonId,
    ) -> Self {
        Self {
            id: super::ids::BootstrapPolicyId::new(),
            name: name.into(),
            description: description.into(),
            claims: Vec::new(),
            conditions: Vec::new(),
            priority: 0,
            enabled: true,
            created_by,
            metadata: HashMap::new(),
        }
    }

    /// Builder: add a claim.
    pub fn with_claim(mut self, claim: PolicyClaim) -> Self {
        self.claims.push(claim);
        self
    }

    /// Builder: set priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Individual claim (capability/permission)
///
/// Claims represent atomic permissions. They compose additively:
/// Policy A: [CanSignCode] + Policy B: [CanAccessProd] = [CanSignCode, CanAccessProd]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyClaim {
    // ===== Key Management Claims =====
    /// Generate new cryptographic keys
    CanGenerateKeys,

    /// Sign code artifacts (binaries, containers, etc.)
    CanSignCode,

    /// Sign certificates (act as CA)
    CanSignCertificates,

    /// Revoke keys or certificates
    CanRevokeKeys,

    /// Delegate key permissions to others
    CanDelegateKeys,

    /// Export private keys from secure storage
    CanExportKeys,

    /// Backup keys to offline storage
    CanBackupKeys,

    /// Rotate keys (generate new, revoke old)
    CanRotateKeys,

    // ===== Infrastructure Claims =====
    /// Access production infrastructure
    CanAccessProduction,

    /// Access staging infrastructure
    CanAccessStaging,

    /// Access development infrastructure
    CanAccessDevelopment,

    /// Modify infrastructure configuration
    CanModifyInfrastructure,

    /// Deploy services to infrastructure
    CanDeployServices,

    /// Create new infrastructure resources
    CanCreateInfrastructure,

    /// Delete infrastructure resources
    CanDeleteInfrastructure,

    // ===== Administrative Claims =====
    /// Manage organizational structure
    CanManageOrganization,

    /// Create and modify policies
    CanManagePolicies,

    /// Assign roles to people
    CanAssignRoles,

    /// Create user accounts
    CanCreateAccounts,

    /// Disable user accounts
    CanDisableAccounts,

    /// Delete user accounts
    CanDeleteAccounts,

    /// View audit logs
    CanViewAuditLogs,

    /// Export audit logs
    CanExportAuditLogs,

    /// Modify audit log settings
    CanModifyAuditSettings,

    // ===== NATS Claims =====
    /// Create NATS operators
    CanCreateNATSOperators,

    /// Create NATS accounts
    CanCreateNATSAccounts,

    /// Create NATS users
    CanCreateNATSUsers,

    /// Manage NATS subjects
    CanManageNATSSubjects,

    /// Publish to sensitive NATS subjects
    CanPublishSensitiveSubjects,

    /// Subscribe to sensitive NATS subjects
    CanSubscribeSensitiveSubjects,

    // ===== Data Claims =====
    /// Read sensitive data
    CanReadSensitiveData,

    /// Write sensitive data
    CanWriteSensitiveData,

    /// Delete data
    CanDeleteData,

    /// Export data
    CanExportData,

    /// Import data
    CanImportData,

    // ===== Security Claims =====
    /// Perform security audits
    CanPerformAudits,

    /// Review security incidents
    CanReviewIncidents,

    /// Initiate emergency procedures
    CanInitiateEmergency,

    /// Override security controls (break glass)
    CanOverrideSecurityControls,

    // ===== Custom Claims =====
    /// Custom claim for domain-specific permissions
    Custom {
        name: String,
        scope: String,
        description: String,
    },
}

/// Conditions that must be met for policy to be active
///
/// ALL conditions must be satisfied for the policy to activate.
/// If any condition fails, the policy is inactive (claims don't apply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    /// Minimum security clearance level required
    MinimumSecurityClearance(SecurityClearance),

    /// MFA must be enabled and verified
    MFAEnabled(bool),

    /// YubiKey must be present
    YubiKeyRequired(bool),

    /// Must be at one of these physical locations
    LocationRestriction(Vec<Uuid>),

    /// Must be within time window
    TimeWindow {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },

    /// Must have witness(es) present
    RequiresWitness {
        count: u32,
        witness_clearance: Option<SecurityClearance>,
    },

    /// Must be member of specific organizational units
    MemberOfUnits(Vec<Uuid>),

    /// Must have specific role
    HasRole(Uuid),

    /// Must have been employed for minimum duration
    MinimumEmploymentDuration {
        days: u32,
    },

    /// Must have completed specific training
    CompletedTraining {
        training_ids: Vec<String>,
    },

    /// IP address must be in whitelist
    IPWhitelist(Vec<String>),

    /// Must be during business hours
    BusinessHoursOnly {
        timezone: String,
        start_hour: u8,
        end_hour: u8,
    },

    /// Custom condition (evaluated by external system)
    Custom {
        name: String,
        parameters: HashMap<String, String>,
    },
}

/// Security clearance levels (hierarchical)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityClearance {
    /// Public information
    Public,

    /// Internal company information
    Internal,

    /// Confidential business data
    Confidential,

    /// Secret operations data
    Secret,

    /// Top secret strategic data
    TopSecret,
}

/// Binds a policy to entities it governs
///
/// Policies can govern:
/// - Organizations (all members)
/// - Organizational Units (all members)
/// - People (specific individuals)
/// - Locations (when accessed)
/// - Keys (when used)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBinding {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: PolicyEntityType,
    pub bound_at: DateTime<Utc>,
    pub bound_by: Uuid,
    pub active: bool,
}

/// Types of entities that policies can govern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEntityType {
    Organization,
    OrganizationalUnit,
    Person,
    Location,
    Key,
    Role,
}

/// Result of policy evaluation for an entity
///
/// Collects all active claims from all applicable policies.
#[derive(Debug, Clone)]
pub struct PolicyEvaluation {
    /// Entity being evaluated
    pub entity_id: Uuid,
    pub entity_type: PolicyEntityType,

    /// All active policies that apply
    pub active_policies: Vec<Uuid>,

    /// All inactive policies (conditions not met)
    pub inactive_policies: Vec<(Uuid, Vec<String>)>, // (policy_id, reasons)

    /// Union of all claims from active policies
    pub granted_claims: Vec<PolicyClaim>,

    /// Timestamp of evaluation
    pub evaluated_at: DateTime<Utc>,
}

impl Policy {
    /// Check if all conditions are satisfied for this policy
    pub fn evaluate_conditions(&self, context: &PolicyEvaluationContext) -> bool {
        self.conditions.iter().all(|condition| {
            condition.is_satisfied(context)
        })
    }

    /// Get all claims if policy is enabled and conditions are met
    pub fn get_active_claims(&self, context: &PolicyEvaluationContext) -> Vec<PolicyClaim> {
        if self.enabled && self.evaluate_conditions(context) {
            self.claims.clone()
        } else {
            Vec::new()
        }
    }
}

impl PolicyCondition {
    /// Check if this condition is satisfied in the given context
    pub fn is_satisfied(&self, context: &PolicyEvaluationContext) -> bool {
        match self {
            PolicyCondition::MinimumSecurityClearance(required) => {
                context.person_clearance >= *required
            }

            PolicyCondition::MFAEnabled(required) => {
                context.mfa_verified == *required
            }

            PolicyCondition::YubiKeyRequired(required) => {
                context.yubikey_present == *required
            }

            PolicyCondition::LocationRestriction(allowed_locations) => {
                context.current_location
                    .map(|loc| allowed_locations.contains(&loc))
                    .unwrap_or(false)
            }

            PolicyCondition::TimeWindow { start, end } => {
                let now = context.current_time;
                now >= *start && now <= *end
            }

            PolicyCondition::RequiresWitness { count, witness_clearance } => {
                let sufficient_count = context.witnesses.len() >= *count as usize;
                let sufficient_clearance = witness_clearance
                    .map(|required| {
                        context.witnesses.iter().all(|w| w.clearance >= required)
                    })
                    .unwrap_or(true);
                sufficient_count && sufficient_clearance
            }

            PolicyCondition::MemberOfUnits(required_units) => {
                required_units.iter().any(|unit| context.person_units.contains(unit))
            }

            PolicyCondition::HasRole(required_role) => {
                context.person_roles.contains(required_role)
            }

            PolicyCondition::MinimumEmploymentDuration { days } => {
                let employment_duration = context.current_time
                    .signed_duration_since(context.employment_start_date)
                    .num_days();
                employment_duration >= *days as i64
            }

            PolicyCondition::CompletedTraining { training_ids } => {
                training_ids.iter().all(|tid| context.completed_training.contains(tid))
            }

            PolicyCondition::IPWhitelist(allowed_ips) => {
                context.source_ip
                    .as_ref()
                    .map(|ip| allowed_ips.contains(ip))
                    .unwrap_or(false)
            }

            PolicyCondition::BusinessHoursOnly { timezone: _, start_hour, end_hour } => {
                // TODO: Implement timezone-aware business hours check
                // For now, simplified version using UTC
                let hour = context.current_time.hour();
                hour >= *start_hour as u32 && hour < *end_hour as u32
            }

            PolicyCondition::Custom { name: _, parameters: _ } => {
                // Custom conditions evaluated by external system
                // For now, return false (requires external evaluation)
                false
            }
        }
    }
}

/// Context for evaluating policy conditions
#[derive(Debug, Clone)]
pub struct PolicyEvaluationContext {
    /// Person being evaluated
    pub person_id: Uuid,
    pub person_clearance: SecurityClearance,
    pub person_units: Vec<Uuid>,
    pub person_roles: Vec<Uuid>,
    pub employment_start_date: DateTime<Utc>,
    pub completed_training: Vec<String>,

    /// Current context
    pub current_time: DateTime<Utc>,
    pub current_location: Option<Uuid>,
    pub source_ip: Option<String>,

    /// Security context
    pub mfa_verified: bool,
    pub yubikey_present: bool,
    pub witnesses: Vec<WitnessInfo>,
}

/// Information about a witness
#[derive(Debug, Clone)]
pub struct WitnessInfo {
    pub person_id: Uuid,
    pub clearance: SecurityClearance,
}

/// Evaluate all policies applicable to an entity
pub fn evaluate_policies(
    policies: &[Policy],
    bindings: &[PolicyBinding],
    entity_id: Uuid,
    entity_type: PolicyEntityType,
    context: &PolicyEvaluationContext,
) -> PolicyEvaluation {
    // Find all policies bound to this entity
    let applicable_policy_ids: Vec<Uuid> = bindings
        .iter()
        .filter(|b| b.entity_id == entity_id && b.entity_type == entity_type && b.active)
        .map(|b| b.policy_id)
        .collect();

    let applicable_policies: Vec<&Policy> = policies
        .iter()
        .filter(|p| applicable_policy_ids.contains(&p.id.as_uuid()))
        .collect();

    // Sort by priority (higher priority first)
    let mut sorted_policies = applicable_policies.clone();
    sorted_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

    let mut active_policies = Vec::new();
    let mut inactive_policies = Vec::new();
    let mut all_claims = Vec::new();

    for policy in sorted_policies {
        if policy.enabled {
            if policy.evaluate_conditions(context) {
                active_policies.push(policy.id.as_uuid());
                all_claims.extend(policy.claims.clone());
            } else {
                let reasons = policy.conditions
                    .iter()
                    .filter(|c| !c.is_satisfied(context))
                    .map(|c| format!("{:?} not satisfied", c))
                    .collect();
                inactive_policies.push((policy.id.as_uuid(), reasons));
            }
        }
    }

    // Deduplicate claims (union)
    all_claims.sort_by_key(|c| format!("{:?}", c));
    all_claims.dedup();

    PolicyEvaluation {
        entity_id,
        entity_type,
        active_policies,
        inactive_policies,
        granted_claims: all_claims,
        evaluated_at: Utc::now(),
    }
}

// ========================================================================
// ROLE SYSTEM
// ========================================================================

/// Role/Position in the organization - a domain entity.
///
/// Roles represent positions that people can fill.
/// Each role has required policies that must be active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: super::ids::BootstrapRoleId,
    pub name: String,
    pub description: String,
    pub organization_id: super::ids::BootstrapOrgId,
    pub unit_id: Option<super::ids::UnitId>,
    pub required_policies: Vec<super::ids::BootstrapPolicyId>,
    pub responsibilities: Vec<String>,
    pub created_by: super::ids::BootstrapPersonId,
    pub active: bool,
}

impl Role {
    /// Create a new Role with auto-generated ID.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        organization_id: super::ids::BootstrapOrgId,
        created_by: super::ids::BootstrapPersonId,
    ) -> Self {
        Self {
            id: super::ids::BootstrapRoleId::new(),
            name: name.into(),
            description: description.into(),
            organization_id,
            unit_id: None,
            required_policies: Vec::new(),
            responsibilities: Vec::new(),
            created_by,
            active: true,
        }
    }

    /// Builder: set unit scope.
    pub fn in_unit(mut self, unit_id: super::ids::UnitId) -> Self {
        self.unit_id = Some(unit_id);
        self
    }

    /// Builder: add a required policy.
    pub fn requires_policy(mut self, policy_id: super::ids::BootstrapPolicyId) -> Self {
        self.required_policies.push(policy_id);
        self
    }

    /// Builder: add a responsibility.
    pub fn with_responsibility(mut self, resp: impl Into<String>) -> Self {
        self.responsibilities.push(resp.into());
        self
    }
}

/// Assignment of a role to a person - a domain entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub id: super::ids::BootstrapRoleId,  // Using role marker for assignment too
    pub person_id: super::ids::BootstrapPersonId,
    pub role_id: super::ids::BootstrapRoleId,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: super::ids::BootstrapPersonId,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub active: bool,
}

impl RoleAssignment {
    /// Create a new RoleAssignment with auto-generated ID.
    pub fn new(
        person_id: super::ids::BootstrapPersonId,
        role_id: super::ids::BootstrapRoleId,
        assigned_by: super::ids::BootstrapPersonId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: super::ids::BootstrapRoleId::new(),
            person_id,
            role_id,
            assigned_at: now,
            assigned_by,
            valid_from: now,
            valid_until: None,
            active: true,
        }
    }

    /// Builder: set validity period.
    pub fn valid_until(mut self, until: DateTime<Utc>) -> Self {
        self.valid_until = Some(until);
        self
    }
}

impl Role {
    /// Check if a person can fulfill this role (has all required policies active)
    pub fn can_person_fulfill(
        &self,
        _person_id: Uuid,
        policy_evaluation: &PolicyEvaluation,
        policies: &[Policy],
    ) -> bool {
        // Get all required policy claims
        let required_claims: Vec<PolicyClaim> = self.required_policies
            .iter()
            .filter_map(|policy_id| {
                policies.iter().find(|p| p.id == *policy_id)
            })
            .flat_map(|p| p.claims.clone())
            .collect();

        // Check if person has all required claims
        required_claims.iter().all(|claim| {
            policy_evaluation.granted_claims.contains(claim)
        })
    }
}

// ========================================================================
// DISPLAY IMPLEMENTATIONS
// ========================================================================

impl std::fmt::Display for PolicyClaim {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PolicyClaim::CanGenerateKeys => write!(f, "Can Generate Keys"),
            PolicyClaim::CanSignCode => write!(f, "Can Sign Code"),
            PolicyClaim::CanSignCertificates => write!(f, "Can Sign Certificates"),
            PolicyClaim::CanRevokeKeys => write!(f, "Can Revoke Keys"),
            PolicyClaim::CanDelegateKeys => write!(f, "Can Delegate Keys"),
            PolicyClaim::CanExportKeys => write!(f, "Can Export Keys"),
            PolicyClaim::CanBackupKeys => write!(f, "Can Backup Keys"),
            PolicyClaim::CanRotateKeys => write!(f, "Can Rotate Keys"),
            PolicyClaim::CanAccessProduction => write!(f, "Can Access Production"),
            PolicyClaim::CanAccessStaging => write!(f, "Can Access Staging"),
            PolicyClaim::CanAccessDevelopment => write!(f, "Can Access Development"),
            PolicyClaim::CanModifyInfrastructure => write!(f, "Can Modify Infrastructure"),
            PolicyClaim::CanDeployServices => write!(f, "Can Deploy Services"),
            PolicyClaim::CanCreateInfrastructure => write!(f, "Can Create Infrastructure"),
            PolicyClaim::CanDeleteInfrastructure => write!(f, "Can Delete Infrastructure"),
            PolicyClaim::CanManageOrganization => write!(f, "Can Manage Organization"),
            PolicyClaim::CanManagePolicies => write!(f, "Can Manage Policies"),
            PolicyClaim::CanAssignRoles => write!(f, "Can Assign Roles"),
            PolicyClaim::CanCreateAccounts => write!(f, "Can Create Accounts"),
            PolicyClaim::CanDisableAccounts => write!(f, "Can Disable Accounts"),
            PolicyClaim::CanDeleteAccounts => write!(f, "Can Delete Accounts"),
            PolicyClaim::CanViewAuditLogs => write!(f, "Can View Audit Logs"),
            PolicyClaim::CanExportAuditLogs => write!(f, "Can Export Audit Logs"),
            PolicyClaim::CanModifyAuditSettings => write!(f, "Can Modify Audit Settings"),
            PolicyClaim::CanCreateNATSOperators => write!(f, "Can Create NATS Operators"),
            PolicyClaim::CanCreateNATSAccounts => write!(f, "Can Create NATS Accounts"),
            PolicyClaim::CanCreateNATSUsers => write!(f, "Can Create NATS Users"),
            PolicyClaim::CanManageNATSSubjects => write!(f, "Can Manage NATS Subjects"),
            PolicyClaim::CanPublishSensitiveSubjects => write!(f, "Can Publish Sensitive Subjects"),
            PolicyClaim::CanSubscribeSensitiveSubjects => write!(f, "Can Subscribe Sensitive Subjects"),
            PolicyClaim::CanReadSensitiveData => write!(f, "Can Read Sensitive Data"),
            PolicyClaim::CanWriteSensitiveData => write!(f, "Can Write Sensitive Data"),
            PolicyClaim::CanDeleteData => write!(f, "Can Delete Data"),
            PolicyClaim::CanExportData => write!(f, "Can Export Data"),
            PolicyClaim::CanImportData => write!(f, "Can Import Data"),
            PolicyClaim::CanPerformAudits => write!(f, "Can Perform Audits"),
            PolicyClaim::CanReviewIncidents => write!(f, "Can Review Incidents"),
            PolicyClaim::CanInitiateEmergency => write!(f, "Can Initiate Emergency"),
            PolicyClaim::CanOverrideSecurityControls => write!(f, "Can Override Security Controls"),
            PolicyClaim::Custom { name, .. } => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for SecurityClearance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SecurityClearance::Public => write!(f, "Public"),
            SecurityClearance::Internal => write!(f, "Internal"),
            SecurityClearance::Confidential => write!(f, "Confidential"),
            SecurityClearance::Secret => write!(f, "Secret"),
            SecurityClearance::TopSecret => write!(f, "Top Secret"),
        }
    }
}

impl std::fmt::Display for PolicyEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PolicyEntityType::Organization => write!(f, "Organization"),
            PolicyEntityType::OrganizationalUnit => write!(f, "Organizational Unit"),
            PolicyEntityType::Person => write!(f, "Person"),
            PolicyEntityType::Location => write!(f, "Location"),
            PolicyEntityType::Key => write!(f, "Key"),
            PolicyEntityType::Role => write!(f, "Role"),
        }
    }
}
