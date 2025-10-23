//! Master domain model for CIM key infrastructure
//!
//! This module defines the foundational domain entities for CIM:
//! - Organizations and their structure
//! - People and their roles
//! - Physical and logical locations
//! - Key ownership and delegation
//!
//! cim-keys is the genesis point that creates the initial Domain
//! for a business infrastructure. These models are projected to
//! encrypted storage and imported by CIM deployments.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// We define our own complete domain models here since cim-keys
// is the master that creates the initial infrastructure domain

/// Organization in the CIM infrastructure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub units: Vec<OrganizationUnit>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Organizational unit within an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUnit {
    pub id: Uuid,
    pub name: String,
    pub unit_type: OrganizationUnitType,
    pub parent_unit_id: Option<Uuid>,
    pub responsible_person_id: Option<Uuid>,
}

/// Type of organizational unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationUnitType {
    Division,
    Department,
    Team,
    Project,
    Service,
    Infrastructure,
}

/// Person in the organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub roles: Vec<PersonRole>,
    pub organization_id: Uuid,
    pub unit_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub active: bool,
}

/// Role a person can have
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonRole {
    pub role_type: RoleType,
    pub scope: RoleScope,
    pub permissions: Vec<Permission>,
}

/// Type of role
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    RevokeKeys,
    ManageInfrastructure,
    ViewAuditLogs,
    ModifyConfiguration,
}

/// Physical or logical location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub name: String,
    pub location_type: LocationType,
    pub security_level: SecurityLevel,
    pub address: Option<String>,
    pub coordinates: Option<(f64, f64)>,
    pub metadata: HashMap<String, String>,
}

/// Type of location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    DataCenter,
    Office,
    CloudRegion,
    SafeDeposit,
    SecureStorage,
    HardwareToken,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyPermission {
    Sign,
    Encrypt,
    Decrypt,
    CertifyOthers,
    RevokeOthers,
    BackupAccess,
}

/// Physical storage location of keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStorageLocation {
    /// Physical location where key material is stored
    pub location_id: Uuid,

    /// Type of storage at this location
    pub storage_type: KeyStorageType,

    /// Security level of the location
    pub security_level: SecurityLevel,

    /// Access controls for this location
    pub access_controls: Vec<AccessControl>,
}

/// Type of key storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStorageType {
    /// Hardware Security Module
    HSM { model: String, serial: String },

    /// YubiKey hardware token
    YubiKey { serial: String },

    /// Encrypted SD card
    EncryptedSDCard { device_id: String },

    /// Safe deposit box
    SafeDeposit { box_number: String, bank: String },

    /// Cloud HSM
    CloudHSM { provider: String, region: String },

    /// Paper backup
    PaperBackup { copies: u32 },
}

/// Security level of storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// FIPS 140-2 Level 4 (highest)
    FIPS140_Level4,

    /// FIPS 140-2 Level 3
    FIPS140_Level3,

    /// FIPS 140-2 Level 2
    FIPS140_Level2,

    /// FIPS 140-2 Level 1
    FIPS140_Level1,

    /// Commercial grade encryption
    Commercial,

    /// Basic protection
    Basic,
}

/// Access control for key storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    /// Who can access
    pub authorized_person_id: Uuid,

    /// Type of access
    pub access_type: AccessType,

    /// Multi-factor requirements
    pub mfa_required: Vec<MfaRequirement>,
}

/// Type of access to key storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessType {
    Physical,
    Remote,
    Emergency,
    Audit,
}

/// Multi-factor authentication requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaRequirement {
    Biometric { type_: String },
    PinCode { min_length: u8 },
    HardwareToken { token_type: String },
    TimeBasedOTP,
    DualControl { other_person: Uuid },
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    pub name: String,
    pub purpose: String,

    /// Which unit owns this service account
    pub owning_unit_id: Uuid,

    /// Technical contact
    pub technical_contact_id: Uuid,
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

    /// Where the operation is happening
    pub location: Option<KeyStorageLocation>,

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

/// Create a storage location for keys
pub fn create_storage_location(
    location_id: Uuid,
    storage_type: KeyStorageType,
    security_level: SecurityLevel,
) -> KeyStorageLocation {
    KeyStorageLocation {
        location_id,
        storage_type,
        security_level,
        access_controls: Vec::new(),
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
