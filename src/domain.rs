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
use chrono::{DateTime, Utc, Timelike};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// FIPS 140-2 Level 4 (highest)
    Fips140Level4,

    /// FIPS 140-2 Level 3
    Fips140Level3,

    /// FIPS 140-2 Level 2
    Fips140Level2,

    /// FIPS 140-2 Level 1
    Fips140Level1,

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

impl std::fmt::Display for LocationType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LocationType::DataCenter => write!(f, "Data Center"),
            LocationType::Office => write!(f, "Office"),
            LocationType::CloudRegion => write!(f, "Cloud Region"),
            LocationType::SafeDeposit => write!(f, "Safe Deposit"),
            LocationType::SecureStorage => write!(f, "Secure Storage"),
            LocationType::HardwareToken => write!(f, "Hardware Token"),
        }
    }
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SecurityLevel::Fips140Level4 => write!(f, "FIPS 140-2 Level 4"),
            SecurityLevel::Fips140Level3 => write!(f, "FIPS 140-2 Level 3"),
            SecurityLevel::Fips140Level2 => write!(f, "FIPS 140-2 Level 2"),
            SecurityLevel::Fips140Level1 => write!(f, "FIPS 140-2 Level 1"),
            SecurityLevel::Commercial => write!(f, "Commercial"),
            SecurityLevel::Basic => write!(f, "Basic"),
        }
    }
}

// ========================================================================
// CLAIMS-BASED SECURITY: POLICY SYSTEM
// ========================================================================

/// Policy entity with claims-based permissions
///
/// Policies define what actions are permitted based on conditions.
/// Multiple policies can be composed (claims are unioned).
/// Policies are evaluated in priority order (higher priority wins).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub claims: Vec<PolicyClaim>,
    pub conditions: Vec<PolicyCondition>,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub metadata: HashMap<String, String>,
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
        .filter(|p| applicable_policy_ids.contains(&p.id))
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
                active_policies.push(policy.id);
                all_claims.extend(policy.claims.clone());
            } else {
                let reasons = policy.conditions
                    .iter()
                    .filter(|c| !c.is_satisfied(context))
                    .map(|c| format!("{:?} not satisfied", c))
                    .collect();
                inactive_policies.push((policy.id, reasons));
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

/// Role/Position in the organization
///
/// Roles represent positions that people can fill.
/// Each role has required policies that must be active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub organization_id: Uuid,
    pub unit_id: Option<Uuid>, // Optional: role specific to unit
    pub required_policies: Vec<Uuid>,
    pub responsibilities: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub active: bool,
}

/// Assignment of a role to a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub id: Uuid,
    pub person_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Uuid,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub active: bool,
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
