// Copyright (c) 2025 - Cowboy AI, LLC.
//! Claims Vocabulary for Policy Ontology
//!
//! Claims are atomic permission primitives that compose into Policies.
//! When Claims + Conditions + Bindings are evaluated together, they form
//! an Ontology - a formal knowledge representation of authorization.
//!
//! # Ontological Composition
//!
//! ```text
//! Claim (atomic term)
//!   ↓ composed in
//! Policy (claims + conditions)
//!   ↓ bound to
//! PolicyBinding (policy + entity)
//!   ↓ evaluated as
//! PolicyEvaluation (inference result)
//! ```
//!
//! # Claim Categories
//!
//! Claims are organized into domains reflecting IT/knowledge worker operations:
//!
//! - **Identity & Access** - User, group, role, session management
//! - **Infrastructure** - Servers, containers, networks, storage
//! - **Development** - Code, CI/CD, deployments, releases
//! - **Security** - Keys, certificates, incidents, compliance
//! - **Data** - Databases, backups, PII, exports
//! - **Observability** - Logs, metrics, alerts, traces
//! - **Communication** - Email, chat, documents, wikis
//! - **Project** - Tasks, sprints, roadmaps, budgets
//! - **Finance** - Invoices, expenses, procurement
//! - **HR** - Employees, onboarding, reviews, training

use serde::{Deserialize, Serialize};
use std::fmt;

/// Atomic permission claim - vocabulary term in the authorization ontology
///
/// Claims compose additively: Policy A claims ∪ Policy B claims = effective claims.
/// The ontology emerges when claims are bound to entities with conditions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Claim {
    // ========================================================================
    // IDENTITY & ACCESS MANAGEMENT
    // ========================================================================

    // --- User Management ---
    /// Create new user accounts
    CreateUser,
    /// Read user profile information
    ReadUser,
    /// Update user profile information
    UpdateUser,
    /// Deactivate user accounts (soft delete)
    DeactivateUser,
    /// Permanently delete user accounts
    DeleteUser,
    /// Reset user passwords
    ResetUserPassword,
    /// Unlock locked user accounts
    UnlockUser,
    /// Impersonate another user (admin debugging)
    ImpersonateUser,

    // --- Group Management ---
    /// Create groups
    CreateGroup,
    /// Read group membership
    ReadGroup,
    /// Update group settings
    UpdateGroup,
    /// Delete groups
    DeleteGroup,
    /// Add members to groups
    AddGroupMember,
    /// Remove members from groups
    RemoveGroupMember,

    // --- Role Management ---
    /// Create roles
    CreateRole,
    /// Read role definitions
    ReadRole,
    /// Update role permissions
    UpdateRole,
    /// Delete roles
    DeleteRole,
    /// Assign roles to users
    AssignRole,
    /// Revoke roles from users
    RevokeRole,

    // --- Session Management ---
    /// View active sessions
    ViewSessions,
    /// Terminate user sessions
    TerminateSession,
    /// Configure session policies
    ConfigureSessionPolicy,

    // --- SSO & Federation ---
    /// Configure SSO providers
    ConfigureSSO,
    /// Manage identity federation
    ManageFederation,
    /// Configure MFA settings
    ConfigureMFA,

    // ========================================================================
    // INFRASTRUCTURE
    // ========================================================================

    // --- Compute ---
    /// Create virtual machines/servers
    CreateServer,
    /// Read server configuration
    ReadServer,
    /// Update server configuration
    UpdateServer,
    /// Delete servers
    DeleteServer,
    /// Start servers
    StartServer,
    /// Stop servers
    StopServer,
    /// Restart servers
    RestartServer,
    /// Access server console/SSH
    AccessServerConsole,

    // --- Containers & Orchestration ---
    /// Create containers/pods
    CreateContainer,
    /// Read container status
    ReadContainer,
    /// Update container configuration
    UpdateContainer,
    /// Delete containers
    DeleteContainer,
    /// Execute commands in containers
    ExecInContainer,
    /// View container logs
    ViewContainerLogs,
    /// Manage Kubernetes namespaces
    ManageNamespace,
    /// Deploy Helm charts
    DeployHelmChart,

    // --- Networks ---
    /// Create networks/VPCs
    CreateNetwork,
    /// Read network configuration
    ReadNetwork,
    /// Update network configuration
    UpdateNetwork,
    /// Delete networks
    DeleteNetwork,
    /// Manage firewall rules
    ManageFirewall,
    /// Configure load balancers
    ConfigureLoadBalancer,
    /// Manage DNS records
    ManageDNS,
    /// Configure VPN
    ConfigureVPN,

    // --- Storage ---
    /// Create storage volumes
    CreateStorage,
    /// Read storage configuration
    ReadStorage,
    /// Update storage configuration
    UpdateStorage,
    /// Delete storage
    DeleteStorage,
    /// Manage storage snapshots
    ManageSnapshots,
    /// Configure backup policies
    ConfigureBackupPolicy,

    // --- Cloud Resources ---
    /// Manage cloud provider accounts
    ManageCloudAccount,
    /// View cloud billing
    ViewCloudBilling,
    /// Configure cloud quotas
    ConfigureCloudQuota,

    // ========================================================================
    // DEVELOPMENT & DEVOPS
    // ========================================================================

    // --- Source Code ---
    /// Read source code repositories
    ReadRepository,
    /// Write to source code repositories
    WriteRepository,
    /// Create repositories
    CreateRepository,
    /// Delete repositories
    DeleteRepository,
    /// Manage repository settings
    ManageRepositorySettings,
    /// Create branches
    CreateBranch,
    /// Delete branches
    DeleteBranch,
    /// Merge branches
    MergeBranch,
    /// Approve pull requests
    ApprovePullRequest,
    /// Force push to protected branches
    ForcePush,

    // --- CI/CD ---
    /// View pipeline status
    ViewPipeline,
    /// Trigger pipelines
    TriggerPipeline,
    /// Cancel pipelines
    CancelPipeline,
    /// Configure pipelines
    ConfigurePipeline,
    /// View build logs
    ViewBuildLogs,
    /// Manage pipeline secrets
    ManagePipelineSecrets,
    /// Approve deployments
    ApproveDeployment,

    // --- Deployments ---
    /// Deploy to development environment
    DeployToDevelopment,
    /// Deploy to staging environment
    DeployToStaging,
    /// Deploy to production environment
    DeployToProduction,
    /// Rollback deployments
    RollbackDeployment,
    /// View deployment history
    ViewDeploymentHistory,
    /// Configure deployment strategies
    ConfigureDeploymentStrategy,

    // --- Releases ---
    /// Create releases
    CreateRelease,
    /// Read release information
    ReadRelease,
    /// Publish releases
    PublishRelease,
    /// Deprecate releases
    DeprecateRelease,

    // --- Artifacts ---
    /// Upload artifacts
    UploadArtifact,
    /// Download artifacts
    DownloadArtifact,
    /// Delete artifacts
    DeleteArtifact,
    /// Manage artifact retention
    ManageArtifactRetention,

    // --- Feature Flags ---
    /// Create feature flags
    CreateFeatureFlag,
    /// Read feature flags
    ReadFeatureFlag,
    /// Update feature flags
    UpdateFeatureFlag,
    /// Delete feature flags
    DeleteFeatureFlag,
    /// Toggle feature flags in production
    ToggleProductionFlag,

    // ========================================================================
    // SECURITY & CRYPTOGRAPHY
    // ========================================================================

    // --- Key Management ---
    /// Generate cryptographic keys
    GenerateKey,
    /// Read key metadata
    ReadKeyMetadata,
    /// Export private keys
    ExportPrivateKey,
    /// Import keys
    ImportKey,
    /// Rotate keys
    RotateKey,
    /// Revoke keys
    RevokeKey,
    /// Backup keys
    BackupKey,
    /// Delegate key usage
    DelegateKey,

    // --- Certificates ---
    /// Request certificates
    RequestCertificate,
    /// Issue certificates (act as CA)
    IssueCertificate,
    /// Renew certificates
    RenewCertificate,
    /// Revoke certificates
    RevokeCertificate,
    /// View certificate details
    ViewCertificate,

    // --- Secrets Management ---
    /// Create secrets
    CreateSecret,
    /// Read secrets
    ReadSecret,
    /// Update secrets
    UpdateSecret,
    /// Delete secrets
    DeleteSecret,
    /// Rotate secrets
    RotateSecret,

    // --- Code Signing ---
    /// Sign code/binaries
    SignCode,
    /// Sign container images
    SignContainerImage,
    /// Verify signatures
    VerifySignature,

    // --- Security Operations ---
    /// View security alerts
    ViewSecurityAlerts,
    /// Acknowledge security alerts
    AcknowledgeSecurityAlert,
    /// Escalate security incidents
    EscalateIncident,
    /// Close security incidents
    CloseIncident,
    /// Run security scans
    RunSecurityScan,
    /// View vulnerability reports
    ViewVulnerabilityReport,
    /// Override security controls (break glass)
    OverrideSecurityControl,

    // --- Compliance ---
    /// View compliance reports
    ViewComplianceReport,
    /// Configure compliance policies
    ConfigureCompliancePolicy,
    /// Acknowledge compliance exceptions
    AcknowledgeComplianceException,

    // ========================================================================
    // DATA & DATABASES
    // ========================================================================

    // --- Database Administration ---
    /// Create databases
    CreateDatabase,
    /// Read database schema
    ReadDatabaseSchema,
    /// Modify database schema
    ModifyDatabaseSchema,
    /// Delete databases
    DeleteDatabase,
    /// Execute raw SQL
    ExecuteSQL,
    /// View query plans
    ViewQueryPlan,

    // --- Data Access Levels ---
    /// Read public data
    ReadPublicData,
    /// Read internal data
    ReadInternalData,
    /// Read confidential data
    ReadConfidentialData,
    /// Read restricted/PII data
    ReadRestrictedData,
    /// Write to data stores
    WriteData,
    /// Delete data
    DeleteData,

    // --- Data Operations ---
    /// Export data
    ExportData,
    /// Import data
    ImportData,
    /// Run data migrations
    RunDataMigration,
    /// Manage data retention
    ManageDataRetention,
    /// Anonymize data
    AnonymizeData,
    /// Restore from backup
    RestoreFromBackup,

    // ========================================================================
    // OBSERVABILITY & MONITORING
    // ========================================================================

    // --- Logs ---
    /// View application logs
    ViewLogs,
    /// View audit logs
    ViewAuditLogs,
    /// Export logs
    ExportLogs,
    /// Configure log retention
    ConfigureLogRetention,
    /// Delete logs
    DeleteLogs,

    // --- Metrics ---
    /// View metrics
    ViewMetrics,
    /// Configure metrics collection
    ConfigureMetrics,
    /// Create custom metrics
    CreateCustomMetric,

    // --- Alerts ---
    /// View alerts
    ViewAlerts,
    /// Create alert rules
    CreateAlertRule,
    /// Update alert rules
    UpdateAlertRule,
    /// Delete alert rules
    DeleteAlertRule,
    /// Acknowledge alerts
    AcknowledgeAlert,
    /// Silence alerts
    SilenceAlert,

    // --- Dashboards ---
    /// View dashboards
    ViewDashboard,
    /// Create dashboards
    CreateDashboard,
    /// Update dashboards
    UpdateDashboard,
    /// Delete dashboards
    DeleteDashboard,
    /// Share dashboards
    ShareDashboard,

    // --- Tracing ---
    /// View traces
    ViewTraces,
    /// Configure tracing
    ConfigureTracing,

    // ========================================================================
    // COMMUNICATION & COLLABORATION
    // ========================================================================

    // --- Email ---
    /// Send email as organization
    SendOrganizationEmail,
    /// Access shared mailboxes
    AccessSharedMailbox,
    /// Configure email routing
    ConfigureEmailRouting,
    /// View email logs
    ViewEmailLogs,

    // --- Chat & Messaging ---
    /// Create chat channels
    CreateChannel,
    /// Archive chat channels
    ArchiveChannel,
    /// Delete chat messages
    DeleteChatMessage,
    /// Export chat history
    ExportChatHistory,
    /// Manage chat integrations
    ManageChatIntegration,

    // --- Documents ---
    /// Create documents
    CreateDocument,
    /// Read documents
    ReadDocument,
    /// Update documents
    UpdateDocument,
    /// Delete documents
    DeleteDocument,
    /// Share documents externally
    ShareDocumentExternal,
    /// Manage document permissions
    ManageDocumentPermission,

    // --- Wiki & Knowledge Base ---
    /// Create wiki pages
    CreateWikiPage,
    /// Edit wiki pages
    EditWikiPage,
    /// Delete wiki pages
    DeleteWikiPage,
    /// Manage wiki structure
    ManageWikiStructure,

    // --- Calendar ---
    /// View team calendars
    ViewTeamCalendar,
    /// Manage room bookings
    ManageRoomBooking,
    /// Create organization events
    CreateOrganizationEvent,

    // ========================================================================
    // PROJECT MANAGEMENT
    // ========================================================================

    // --- Tasks & Issues ---
    /// Create tasks/issues
    CreateTask,
    /// Read tasks/issues
    ReadTask,
    /// Update tasks/issues
    UpdateTask,
    /// Delete tasks/issues
    DeleteTask,
    /// Assign tasks
    AssignTask,
    /// Close tasks
    CloseTask,

    // --- Sprints & Iterations ---
    /// Create sprints
    CreateSprint,
    /// Manage sprint backlog
    ManageSprintBacklog,
    /// Close sprints
    CloseSprint,
    /// View sprint reports
    ViewSprintReport,

    // --- Roadmaps ---
    /// View roadmaps
    ViewRoadmap,
    /// Edit roadmaps
    EditRoadmap,
    /// Publish roadmaps
    PublishRoadmap,

    // --- Time Tracking ---
    /// Log time
    LogTime,
    /// View team time logs
    ViewTeamTimeLogs,
    /// Approve timesheets
    ApproveTimesheet,
    /// Generate time reports
    GenerateTimeReport,

    // --- Project Budgets ---
    /// View project budgets
    ViewProjectBudget,
    /// Update project budgets
    UpdateProjectBudget,
    /// Approve budget requests
    ApproveBudgetRequest,

    // ========================================================================
    // FINANCE & BILLING
    // ========================================================================

    // --- Invoices ---
    /// Create invoices
    CreateInvoice,
    /// View invoices
    ViewInvoice,
    /// Approve invoices
    ApproveInvoice,
    /// Void invoices
    VoidInvoice,

    // --- Expenses ---
    /// Submit expenses
    SubmitExpense,
    /// View expenses
    ViewExpense,
    /// Approve expenses
    ApproveExpense,
    /// Reject expenses
    RejectExpense,

    // --- Procurement ---
    /// Create purchase requests
    CreatePurchaseRequest,
    /// Approve purchase requests
    ApprovePurchaseRequest,
    /// Manage vendors
    ManageVendor,
    /// Sign contracts
    SignContract,

    // --- Financial Reports ---
    /// View financial reports
    ViewFinancialReport,
    /// Generate financial reports
    GenerateFinancialReport,
    /// Export financial data
    ExportFinancialData,

    // ========================================================================
    // HR & PEOPLE OPERATIONS
    // ========================================================================

    // --- Employee Records ---
    /// View employee records
    ViewEmployeeRecord,
    /// Update employee records
    UpdateEmployeeRecord,
    /// View compensation data
    ViewCompensation,
    /// Update compensation
    UpdateCompensation,

    // --- Onboarding/Offboarding ---
    /// Initiate onboarding
    InitiateOnboarding,
    /// Complete onboarding tasks
    CompleteOnboardingTask,
    /// Initiate offboarding
    InitiateOffboarding,
    /// Revoke access (offboarding)
    RevokeAllAccess,

    // --- Performance ---
    /// Create performance reviews
    CreatePerformanceReview,
    /// View performance reviews
    ViewPerformanceReview,
    /// Submit performance feedback
    SubmitPerformanceFeedback,
    /// Approve performance reviews
    ApprovePerformanceReview,

    // --- Training ---
    /// Create training courses
    CreateTrainingCourse,
    /// Assign training
    AssignTraining,
    /// Complete training
    CompleteTraining,
    /// View training records
    ViewTrainingRecord,
    /// Issue certifications
    IssueCertification,

    // --- Leave Management ---
    /// Request leave
    RequestLeave,
    /// View team leave
    ViewTeamLeave,
    /// Approve leave
    ApproveLeave,
    /// Configure leave policies
    ConfigureLeavePolicy,

    // ========================================================================
    // NATS MESSAGING INFRASTRUCTURE
    // ========================================================================

    /// Create NATS operators
    CreateNATSOperator,
    /// Create NATS accounts
    CreateNATSAccount,
    /// Create NATS users
    CreateNATSUser,
    /// Manage NATS subjects
    ManageNATSSubject,
    /// Publish to any subject
    PublishAnySubject,
    /// Subscribe to any subject
    SubscribeAnySubject,
    /// Manage JetStream streams
    ManageJetStream,
    /// View NATS metrics
    ViewNATSMetrics,

    // ========================================================================
    // ORGANIZATION MANAGEMENT
    // ========================================================================

    /// Create organizational units
    CreateOrganizationalUnit,
    /// Update organizational structure
    UpdateOrganizationalStructure,
    /// Delete organizational units
    DeleteOrganizationalUnit,
    /// Manage organization settings
    ManageOrganizationSettings,
    /// View organization analytics
    ViewOrganizationAnalytics,

    // ========================================================================
    // POLICY MANAGEMENT (META)
    // ========================================================================

    /// Create policies
    CreatePolicy,
    /// Read policies
    ReadPolicy,
    /// Update policies
    UpdatePolicy,
    /// Delete policies
    DeletePolicy,
    /// Bind policies to entities
    BindPolicy,
    /// Unbind policies
    UnbindPolicy,
    /// View policy evaluation logs
    ViewPolicyEvaluationLog,

    // ========================================================================
    // EMERGENCY & ADMINISTRATIVE
    // ========================================================================

    /// Initiate emergency procedures
    InitiateEmergency,
    /// Access emergency controls
    AccessEmergencyControl,
    /// Declare incident
    DeclareIncident,
    /// All read operations (super reader)
    SuperRead,
    /// All write operations (super admin)
    SuperAdmin,

    // ========================================================================
    // CUSTOM/EXTENSION
    // ========================================================================

    /// Custom claim for domain-specific extensions
    Custom {
        /// Domain/namespace for the claim
        domain: String,
        /// Resource being accessed
        resource: String,
        /// Action being performed
        action: String,
        /// Optional scope limitation
        scope: Option<String>,
    },
}

impl Claim {
    /// Get the category this claim belongs to
    pub fn category(&self) -> ClaimCategory {
        use Claim::*;
        match self {
            // Identity & Access
            CreateUser | ReadUser | UpdateUser | DeactivateUser | DeleteUser |
            ResetUserPassword | UnlockUser | ImpersonateUser |
            CreateGroup | ReadGroup | UpdateGroup | DeleteGroup |
            AddGroupMember | RemoveGroupMember |
            CreateRole | ReadRole | UpdateRole | DeleteRole |
            AssignRole | RevokeRole |
            ViewSessions | TerminateSession | ConfigureSessionPolicy |
            ConfigureSSO | ManageFederation | ConfigureMFA => ClaimCategory::Identity,

            // Infrastructure
            CreateServer | ReadServer | UpdateServer | DeleteServer |
            StartServer | StopServer | RestartServer | AccessServerConsole |
            CreateContainer | ReadContainer | UpdateContainer | DeleteContainer |
            ExecInContainer | ViewContainerLogs | ManageNamespace | DeployHelmChart |
            CreateNetwork | ReadNetwork | UpdateNetwork | DeleteNetwork |
            ManageFirewall | ConfigureLoadBalancer | ManageDNS | ConfigureVPN |
            CreateStorage | ReadStorage | UpdateStorage | DeleteStorage |
            ManageSnapshots | ConfigureBackupPolicy |
            ManageCloudAccount | ViewCloudBilling | ConfigureCloudQuota => ClaimCategory::Infrastructure,

            // Development
            ReadRepository | WriteRepository | CreateRepository | DeleteRepository |
            ManageRepositorySettings | CreateBranch | DeleteBranch | MergeBranch |
            ApprovePullRequest | ForcePush |
            ViewPipeline | TriggerPipeline | CancelPipeline | ConfigurePipeline |
            ViewBuildLogs | ManagePipelineSecrets | ApproveDeployment |
            DeployToDevelopment | DeployToStaging | DeployToProduction |
            RollbackDeployment | ViewDeploymentHistory | ConfigureDeploymentStrategy |
            CreateRelease | ReadRelease | PublishRelease | DeprecateRelease |
            UploadArtifact | DownloadArtifact | DeleteArtifact | ManageArtifactRetention |
            CreateFeatureFlag | ReadFeatureFlag | UpdateFeatureFlag | DeleteFeatureFlag |
            ToggleProductionFlag => ClaimCategory::Development,

            // Security
            GenerateKey | ReadKeyMetadata | ExportPrivateKey | ImportKey |
            RotateKey | RevokeKey | BackupKey | DelegateKey |
            RequestCertificate | IssueCertificate | RenewCertificate |
            RevokeCertificate | ViewCertificate |
            CreateSecret | ReadSecret | UpdateSecret | DeleteSecret | RotateSecret |
            SignCode | SignContainerImage | VerifySignature |
            ViewSecurityAlerts | AcknowledgeSecurityAlert | EscalateIncident |
            CloseIncident | RunSecurityScan | ViewVulnerabilityReport |
            OverrideSecurityControl |
            ViewComplianceReport | ConfigureCompliancePolicy |
            AcknowledgeComplianceException => ClaimCategory::Security,

            // Data
            CreateDatabase | ReadDatabaseSchema | ModifyDatabaseSchema | DeleteDatabase |
            ExecuteSQL | ViewQueryPlan |
            ReadPublicData | ReadInternalData | ReadConfidentialData | ReadRestrictedData |
            WriteData | DeleteData |
            ExportData | ImportData | RunDataMigration | ManageDataRetention |
            AnonymizeData | RestoreFromBackup => ClaimCategory::Data,

            // Observability
            ViewLogs | ViewAuditLogs | ExportLogs | ConfigureLogRetention | DeleteLogs |
            ViewMetrics | ConfigureMetrics | CreateCustomMetric |
            ViewAlerts | CreateAlertRule | UpdateAlertRule | DeleteAlertRule |
            AcknowledgeAlert | SilenceAlert |
            ViewDashboard | CreateDashboard | UpdateDashboard | DeleteDashboard | ShareDashboard |
            ViewTraces | ConfigureTracing => ClaimCategory::Observability,

            // Communication
            SendOrganizationEmail | AccessSharedMailbox | ConfigureEmailRouting | ViewEmailLogs |
            CreateChannel | ArchiveChannel | DeleteChatMessage | ExportChatHistory | ManageChatIntegration |
            CreateDocument | ReadDocument | UpdateDocument | DeleteDocument |
            ShareDocumentExternal | ManageDocumentPermission |
            CreateWikiPage | EditWikiPage | DeleteWikiPage | ManageWikiStructure |
            ViewTeamCalendar | ManageRoomBooking | CreateOrganizationEvent => ClaimCategory::Communication,

            // Project
            CreateTask | ReadTask | UpdateTask | DeleteTask | AssignTask | CloseTask |
            CreateSprint | ManageSprintBacklog | CloseSprint | ViewSprintReport |
            ViewRoadmap | EditRoadmap | PublishRoadmap |
            LogTime | ViewTeamTimeLogs | ApproveTimesheet | GenerateTimeReport |
            ViewProjectBudget | UpdateProjectBudget | ApproveBudgetRequest => ClaimCategory::Project,

            // Finance
            CreateInvoice | ViewInvoice | ApproveInvoice | VoidInvoice |
            SubmitExpense | ViewExpense | ApproveExpense | RejectExpense |
            CreatePurchaseRequest | ApprovePurchaseRequest | ManageVendor | SignContract |
            ViewFinancialReport | GenerateFinancialReport | ExportFinancialData => ClaimCategory::Finance,

            // HR
            ViewEmployeeRecord | UpdateEmployeeRecord | ViewCompensation | UpdateCompensation |
            InitiateOnboarding | CompleteOnboardingTask | InitiateOffboarding | RevokeAllAccess |
            CreatePerformanceReview | ViewPerformanceReview | SubmitPerformanceFeedback | ApprovePerformanceReview |
            CreateTrainingCourse | AssignTraining | CompleteTraining | ViewTrainingRecord | IssueCertification |
            RequestLeave | ViewTeamLeave | ApproveLeave | ConfigureLeavePolicy => ClaimCategory::HR,

            // NATS
            CreateNATSOperator | CreateNATSAccount | CreateNATSUser |
            ManageNATSSubject | PublishAnySubject | SubscribeAnySubject |
            ManageJetStream | ViewNATSMetrics => ClaimCategory::NATS,

            // Organization
            CreateOrganizationalUnit | UpdateOrganizationalStructure |
            DeleteOrganizationalUnit | ManageOrganizationSettings |
            ViewOrganizationAnalytics => ClaimCategory::Organization,

            // Policy (meta)
            CreatePolicy | ReadPolicy | UpdatePolicy | DeletePolicy |
            BindPolicy | UnbindPolicy | ViewPolicyEvaluationLog => ClaimCategory::Policy,

            // Emergency
            InitiateEmergency | AccessEmergencyControl | DeclareIncident |
            SuperRead | SuperAdmin => ClaimCategory::Emergency,

            // Custom
            Custom { .. } => ClaimCategory::Custom,
        }
    }

    /// Check if this is a read-only claim
    pub fn is_read_only(&self) -> bool {
        use Claim::*;
        matches!(self,
            ReadUser | ReadGroup | ReadRole | ViewSessions |
            ReadServer | ReadContainer | ReadNetwork | ReadStorage |
            ReadRepository | ViewPipeline | ViewBuildLogs | ViewDeploymentHistory | ReadRelease | ReadFeatureFlag |
            ReadKeyMetadata | ViewCertificate | ViewSecurityAlerts | ViewVulnerabilityReport | ViewComplianceReport |
            ReadDatabaseSchema | ViewQueryPlan | ReadPublicData | ReadInternalData |
            ViewLogs | ViewAuditLogs | ViewMetrics | ViewAlerts | ViewDashboard | ViewTraces |
            ReadDocument | ViewTeamCalendar |
            ReadTask | ViewSprintReport | ViewRoadmap | ViewTeamTimeLogs | ViewProjectBudget |
            ViewInvoice | ViewExpense | ViewFinancialReport |
            ViewEmployeeRecord | ViewCompensation | ViewPerformanceReview | ViewTrainingRecord | ViewTeamLeave |
            ViewNATSMetrics | ViewOrganizationAnalytics | ReadPolicy | ViewPolicyEvaluationLog |
            SuperRead
        )
    }

    /// Check if this is a destructive/dangerous claim
    pub fn is_destructive(&self) -> bool {
        use Claim::*;
        matches!(self,
            DeleteUser | DeleteGroup | DeleteRole |
            DeleteServer | DeleteContainer | DeleteNetwork | DeleteStorage |
            DeleteRepository | DeleteBranch | ForcePush | DeleteArtifact | DeleteFeatureFlag |
            RevokeKey | RevokeCertificate | DeleteSecret | OverrideSecurityControl |
            DeleteDatabase | DeleteData | DeleteLogs |
            DeleteDashboard | DeleteAlertRule |
            DeleteDocument | DeleteWikiPage | DeleteChatMessage |
            DeleteTask |
            VoidInvoice |
            RevokeAllAccess |
            DeleteOrganizationalUnit | DeletePolicy |
            SuperAdmin
        )
    }

    /// Check if this claim requires elevated privilege
    pub fn requires_elevation(&self) -> bool {
        use Claim::*;
        matches!(self,
            ImpersonateUser |
            AccessServerConsole | ExecInContainer |
            ForcePush | DeployToProduction | ToggleProductionFlag |
            ExportPrivateKey | OverrideSecurityControl |
            ReadRestrictedData | ExecuteSQL | AnonymizeData |
            DeleteLogs |
            SignContract |
            UpdateCompensation |
            InitiateEmergency | AccessEmergencyControl |
            SuperRead | SuperAdmin
        )
    }

    /// Get a URI-style identifier for this claim (for serialization/comparison)
    pub fn uri(&self) -> String {
        match self {
            Claim::Custom { domain, resource, action, scope } => {
                match scope {
                    Some(s) => format!("claim:{}:{}:{}:{}", domain, resource, action, s),
                    None => format!("claim:{}:{}:{}", domain, resource, action),
                }
            }
            _ => format!("claim:cim:{}", self.to_snake_case()),
        }
    }

    /// Convert claim name to snake_case for URIs
    fn to_snake_case(&self) -> String {
        let debug = format!("{:?}", self);
        // Handle Custom variant specially
        if debug.starts_with("Custom") {
            return "custom".to_string();
        }
        // Convert CamelCase to snake_case
        let mut result = String::new();
        for (i, c) in debug.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }
}

/// Categories of claims for organization and filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimCategory {
    Identity,
    Infrastructure,
    Development,
    Security,
    Data,
    Observability,
    Communication,
    Project,
    Finance,
    HR,
    NATS,
    Organization,
    Policy,
    Emergency,
    Custom,
}

impl fmt::Display for ClaimCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaimCategory::Identity => write!(f, "Identity & Access"),
            ClaimCategory::Infrastructure => write!(f, "Infrastructure"),
            ClaimCategory::Development => write!(f, "Development & DevOps"),
            ClaimCategory::Security => write!(f, "Security & Cryptography"),
            ClaimCategory::Data => write!(f, "Data & Databases"),
            ClaimCategory::Observability => write!(f, "Observability & Monitoring"),
            ClaimCategory::Communication => write!(f, "Communication & Collaboration"),
            ClaimCategory::Project => write!(f, "Project Management"),
            ClaimCategory::Finance => write!(f, "Finance & Billing"),
            ClaimCategory::HR => write!(f, "HR & People Operations"),
            ClaimCategory::NATS => write!(f, "NATS Messaging"),
            ClaimCategory::Organization => write!(f, "Organization Management"),
            ClaimCategory::Policy => write!(f, "Policy Management"),
            ClaimCategory::Emergency => write!(f, "Emergency & Administrative"),
            ClaimCategory::Custom => write!(f, "Custom"),
        }
    }
}

impl fmt::Display for Claim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Identity
            Claim::CreateUser => write!(f, "Create User"),
            Claim::ReadUser => write!(f, "Read User"),
            Claim::UpdateUser => write!(f, "Update User"),
            Claim::DeactivateUser => write!(f, "Deactivate User"),
            Claim::DeleteUser => write!(f, "Delete User"),
            Claim::ResetUserPassword => write!(f, "Reset User Password"),
            Claim::UnlockUser => write!(f, "Unlock User"),
            Claim::ImpersonateUser => write!(f, "Impersonate User"),
            Claim::CreateGroup => write!(f, "Create Group"),
            Claim::ReadGroup => write!(f, "Read Group"),
            Claim::UpdateGroup => write!(f, "Update Group"),
            Claim::DeleteGroup => write!(f, "Delete Group"),
            Claim::AddGroupMember => write!(f, "Add Group Member"),
            Claim::RemoveGroupMember => write!(f, "Remove Group Member"),
            Claim::CreateRole => write!(f, "Create Role"),
            Claim::ReadRole => write!(f, "Read Role"),
            Claim::UpdateRole => write!(f, "Update Role"),
            Claim::DeleteRole => write!(f, "Delete Role"),
            Claim::AssignRole => write!(f, "Assign Role"),
            Claim::RevokeRole => write!(f, "Revoke Role"),
            Claim::ViewSessions => write!(f, "View Sessions"),
            Claim::TerminateSession => write!(f, "Terminate Session"),
            Claim::ConfigureSessionPolicy => write!(f, "Configure Session Policy"),
            Claim::ConfigureSSO => write!(f, "Configure SSO"),
            Claim::ManageFederation => write!(f, "Manage Federation"),
            Claim::ConfigureMFA => write!(f, "Configure MFA"),

            // Infrastructure
            Claim::CreateServer => write!(f, "Create Server"),
            Claim::ReadServer => write!(f, "Read Server"),
            Claim::UpdateServer => write!(f, "Update Server"),
            Claim::DeleteServer => write!(f, "Delete Server"),
            Claim::StartServer => write!(f, "Start Server"),
            Claim::StopServer => write!(f, "Stop Server"),
            Claim::RestartServer => write!(f, "Restart Server"),
            Claim::AccessServerConsole => write!(f, "Access Server Console"),
            Claim::CreateContainer => write!(f, "Create Container"),
            Claim::ReadContainer => write!(f, "Read Container"),
            Claim::UpdateContainer => write!(f, "Update Container"),
            Claim::DeleteContainer => write!(f, "Delete Container"),
            Claim::ExecInContainer => write!(f, "Execute in Container"),
            Claim::ViewContainerLogs => write!(f, "View Container Logs"),
            Claim::ManageNamespace => write!(f, "Manage Namespace"),
            Claim::DeployHelmChart => write!(f, "Deploy Helm Chart"),
            Claim::CreateNetwork => write!(f, "Create Network"),
            Claim::ReadNetwork => write!(f, "Read Network"),
            Claim::UpdateNetwork => write!(f, "Update Network"),
            Claim::DeleteNetwork => write!(f, "Delete Network"),
            Claim::ManageFirewall => write!(f, "Manage Firewall"),
            Claim::ConfigureLoadBalancer => write!(f, "Configure Load Balancer"),
            Claim::ManageDNS => write!(f, "Manage DNS"),
            Claim::ConfigureVPN => write!(f, "Configure VPN"),
            Claim::CreateStorage => write!(f, "Create Storage"),
            Claim::ReadStorage => write!(f, "Read Storage"),
            Claim::UpdateStorage => write!(f, "Update Storage"),
            Claim::DeleteStorage => write!(f, "Delete Storage"),
            Claim::ManageSnapshots => write!(f, "Manage Snapshots"),
            Claim::ConfigureBackupPolicy => write!(f, "Configure Backup Policy"),
            Claim::ManageCloudAccount => write!(f, "Manage Cloud Account"),
            Claim::ViewCloudBilling => write!(f, "View Cloud Billing"),
            Claim::ConfigureCloudQuota => write!(f, "Configure Cloud Quota"),

            // Development
            Claim::ReadRepository => write!(f, "Read Repository"),
            Claim::WriteRepository => write!(f, "Write Repository"),
            Claim::CreateRepository => write!(f, "Create Repository"),
            Claim::DeleteRepository => write!(f, "Delete Repository"),
            Claim::ManageRepositorySettings => write!(f, "Manage Repository Settings"),
            Claim::CreateBranch => write!(f, "Create Branch"),
            Claim::DeleteBranch => write!(f, "Delete Branch"),
            Claim::MergeBranch => write!(f, "Merge Branch"),
            Claim::ApprovePullRequest => write!(f, "Approve Pull Request"),
            Claim::ForcePush => write!(f, "Force Push"),
            Claim::ViewPipeline => write!(f, "View Pipeline"),
            Claim::TriggerPipeline => write!(f, "Trigger Pipeline"),
            Claim::CancelPipeline => write!(f, "Cancel Pipeline"),
            Claim::ConfigurePipeline => write!(f, "Configure Pipeline"),
            Claim::ViewBuildLogs => write!(f, "View Build Logs"),
            Claim::ManagePipelineSecrets => write!(f, "Manage Pipeline Secrets"),
            Claim::ApproveDeployment => write!(f, "Approve Deployment"),
            Claim::DeployToDevelopment => write!(f, "Deploy to Development"),
            Claim::DeployToStaging => write!(f, "Deploy to Staging"),
            Claim::DeployToProduction => write!(f, "Deploy to Production"),
            Claim::RollbackDeployment => write!(f, "Rollback Deployment"),
            Claim::ViewDeploymentHistory => write!(f, "View Deployment History"),
            Claim::ConfigureDeploymentStrategy => write!(f, "Configure Deployment Strategy"),
            Claim::CreateRelease => write!(f, "Create Release"),
            Claim::ReadRelease => write!(f, "Read Release"),
            Claim::PublishRelease => write!(f, "Publish Release"),
            Claim::DeprecateRelease => write!(f, "Deprecate Release"),
            Claim::UploadArtifact => write!(f, "Upload Artifact"),
            Claim::DownloadArtifact => write!(f, "Download Artifact"),
            Claim::DeleteArtifact => write!(f, "Delete Artifact"),
            Claim::ManageArtifactRetention => write!(f, "Manage Artifact Retention"),
            Claim::CreateFeatureFlag => write!(f, "Create Feature Flag"),
            Claim::ReadFeatureFlag => write!(f, "Read Feature Flag"),
            Claim::UpdateFeatureFlag => write!(f, "Update Feature Flag"),
            Claim::DeleteFeatureFlag => write!(f, "Delete Feature Flag"),
            Claim::ToggleProductionFlag => write!(f, "Toggle Production Flag"),

            // Security
            Claim::GenerateKey => write!(f, "Generate Key"),
            Claim::ReadKeyMetadata => write!(f, "Read Key Metadata"),
            Claim::ExportPrivateKey => write!(f, "Export Private Key"),
            Claim::ImportKey => write!(f, "Import Key"),
            Claim::RotateKey => write!(f, "Rotate Key"),
            Claim::RevokeKey => write!(f, "Revoke Key"),
            Claim::BackupKey => write!(f, "Backup Key"),
            Claim::DelegateKey => write!(f, "Delegate Key"),
            Claim::RequestCertificate => write!(f, "Request Certificate"),
            Claim::IssueCertificate => write!(f, "Issue Certificate"),
            Claim::RenewCertificate => write!(f, "Renew Certificate"),
            Claim::RevokeCertificate => write!(f, "Revoke Certificate"),
            Claim::ViewCertificate => write!(f, "View Certificate"),
            Claim::CreateSecret => write!(f, "Create Secret"),
            Claim::ReadSecret => write!(f, "Read Secret"),
            Claim::UpdateSecret => write!(f, "Update Secret"),
            Claim::DeleteSecret => write!(f, "Delete Secret"),
            Claim::RotateSecret => write!(f, "Rotate Secret"),
            Claim::SignCode => write!(f, "Sign Code"),
            Claim::SignContainerImage => write!(f, "Sign Container Image"),
            Claim::VerifySignature => write!(f, "Verify Signature"),
            Claim::ViewSecurityAlerts => write!(f, "View Security Alerts"),
            Claim::AcknowledgeSecurityAlert => write!(f, "Acknowledge Security Alert"),
            Claim::EscalateIncident => write!(f, "Escalate Incident"),
            Claim::CloseIncident => write!(f, "Close Incident"),
            Claim::RunSecurityScan => write!(f, "Run Security Scan"),
            Claim::ViewVulnerabilityReport => write!(f, "View Vulnerability Report"),
            Claim::OverrideSecurityControl => write!(f, "Override Security Control"),
            Claim::ViewComplianceReport => write!(f, "View Compliance Report"),
            Claim::ConfigureCompliancePolicy => write!(f, "Configure Compliance Policy"),
            Claim::AcknowledgeComplianceException => write!(f, "Acknowledge Compliance Exception"),

            // Data
            Claim::CreateDatabase => write!(f, "Create Database"),
            Claim::ReadDatabaseSchema => write!(f, "Read Database Schema"),
            Claim::ModifyDatabaseSchema => write!(f, "Modify Database Schema"),
            Claim::DeleteDatabase => write!(f, "Delete Database"),
            Claim::ExecuteSQL => write!(f, "Execute SQL"),
            Claim::ViewQueryPlan => write!(f, "View Query Plan"),
            Claim::ReadPublicData => write!(f, "Read Public Data"),
            Claim::ReadInternalData => write!(f, "Read Internal Data"),
            Claim::ReadConfidentialData => write!(f, "Read Confidential Data"),
            Claim::ReadRestrictedData => write!(f, "Read Restricted/PII Data"),
            Claim::WriteData => write!(f, "Write Data"),
            Claim::DeleteData => write!(f, "Delete Data"),
            Claim::ExportData => write!(f, "Export Data"),
            Claim::ImportData => write!(f, "Import Data"),
            Claim::RunDataMigration => write!(f, "Run Data Migration"),
            Claim::ManageDataRetention => write!(f, "Manage Data Retention"),
            Claim::AnonymizeData => write!(f, "Anonymize Data"),
            Claim::RestoreFromBackup => write!(f, "Restore from Backup"),

            // Observability
            Claim::ViewLogs => write!(f, "View Logs"),
            Claim::ViewAuditLogs => write!(f, "View Audit Logs"),
            Claim::ExportLogs => write!(f, "Export Logs"),
            Claim::ConfigureLogRetention => write!(f, "Configure Log Retention"),
            Claim::DeleteLogs => write!(f, "Delete Logs"),
            Claim::ViewMetrics => write!(f, "View Metrics"),
            Claim::ConfigureMetrics => write!(f, "Configure Metrics"),
            Claim::CreateCustomMetric => write!(f, "Create Custom Metric"),
            Claim::ViewAlerts => write!(f, "View Alerts"),
            Claim::CreateAlertRule => write!(f, "Create Alert Rule"),
            Claim::UpdateAlertRule => write!(f, "Update Alert Rule"),
            Claim::DeleteAlertRule => write!(f, "Delete Alert Rule"),
            Claim::AcknowledgeAlert => write!(f, "Acknowledge Alert"),
            Claim::SilenceAlert => write!(f, "Silence Alert"),
            Claim::ViewDashboard => write!(f, "View Dashboard"),
            Claim::CreateDashboard => write!(f, "Create Dashboard"),
            Claim::UpdateDashboard => write!(f, "Update Dashboard"),
            Claim::DeleteDashboard => write!(f, "Delete Dashboard"),
            Claim::ShareDashboard => write!(f, "Share Dashboard"),
            Claim::ViewTraces => write!(f, "View Traces"),
            Claim::ConfigureTracing => write!(f, "Configure Tracing"),

            // Communication
            Claim::SendOrganizationEmail => write!(f, "Send Organization Email"),
            Claim::AccessSharedMailbox => write!(f, "Access Shared Mailbox"),
            Claim::ConfigureEmailRouting => write!(f, "Configure Email Routing"),
            Claim::ViewEmailLogs => write!(f, "View Email Logs"),
            Claim::CreateChannel => write!(f, "Create Channel"),
            Claim::ArchiveChannel => write!(f, "Archive Channel"),
            Claim::DeleteChatMessage => write!(f, "Delete Chat Message"),
            Claim::ExportChatHistory => write!(f, "Export Chat History"),
            Claim::ManageChatIntegration => write!(f, "Manage Chat Integration"),
            Claim::CreateDocument => write!(f, "Create Document"),
            Claim::ReadDocument => write!(f, "Read Document"),
            Claim::UpdateDocument => write!(f, "Update Document"),
            Claim::DeleteDocument => write!(f, "Delete Document"),
            Claim::ShareDocumentExternal => write!(f, "Share Document Externally"),
            Claim::ManageDocumentPermission => write!(f, "Manage Document Permission"),
            Claim::CreateWikiPage => write!(f, "Create Wiki Page"),
            Claim::EditWikiPage => write!(f, "Edit Wiki Page"),
            Claim::DeleteWikiPage => write!(f, "Delete Wiki Page"),
            Claim::ManageWikiStructure => write!(f, "Manage Wiki Structure"),
            Claim::ViewTeamCalendar => write!(f, "View Team Calendar"),
            Claim::ManageRoomBooking => write!(f, "Manage Room Booking"),
            Claim::CreateOrganizationEvent => write!(f, "Create Organization Event"),

            // Project
            Claim::CreateTask => write!(f, "Create Task"),
            Claim::ReadTask => write!(f, "Read Task"),
            Claim::UpdateTask => write!(f, "Update Task"),
            Claim::DeleteTask => write!(f, "Delete Task"),
            Claim::AssignTask => write!(f, "Assign Task"),
            Claim::CloseTask => write!(f, "Close Task"),
            Claim::CreateSprint => write!(f, "Create Sprint"),
            Claim::ManageSprintBacklog => write!(f, "Manage Sprint Backlog"),
            Claim::CloseSprint => write!(f, "Close Sprint"),
            Claim::ViewSprintReport => write!(f, "View Sprint Report"),
            Claim::ViewRoadmap => write!(f, "View Roadmap"),
            Claim::EditRoadmap => write!(f, "Edit Roadmap"),
            Claim::PublishRoadmap => write!(f, "Publish Roadmap"),
            Claim::LogTime => write!(f, "Log Time"),
            Claim::ViewTeamTimeLogs => write!(f, "View Team Time Logs"),
            Claim::ApproveTimesheet => write!(f, "Approve Timesheet"),
            Claim::GenerateTimeReport => write!(f, "Generate Time Report"),
            Claim::ViewProjectBudget => write!(f, "View Project Budget"),
            Claim::UpdateProjectBudget => write!(f, "Update Project Budget"),
            Claim::ApproveBudgetRequest => write!(f, "Approve Budget Request"),

            // Finance
            Claim::CreateInvoice => write!(f, "Create Invoice"),
            Claim::ViewInvoice => write!(f, "View Invoice"),
            Claim::ApproveInvoice => write!(f, "Approve Invoice"),
            Claim::VoidInvoice => write!(f, "Void Invoice"),
            Claim::SubmitExpense => write!(f, "Submit Expense"),
            Claim::ViewExpense => write!(f, "View Expense"),
            Claim::ApproveExpense => write!(f, "Approve Expense"),
            Claim::RejectExpense => write!(f, "Reject Expense"),
            Claim::CreatePurchaseRequest => write!(f, "Create Purchase Request"),
            Claim::ApprovePurchaseRequest => write!(f, "Approve Purchase Request"),
            Claim::ManageVendor => write!(f, "Manage Vendor"),
            Claim::SignContract => write!(f, "Sign Contract"),
            Claim::ViewFinancialReport => write!(f, "View Financial Report"),
            Claim::GenerateFinancialReport => write!(f, "Generate Financial Report"),
            Claim::ExportFinancialData => write!(f, "Export Financial Data"),

            // HR
            Claim::ViewEmployeeRecord => write!(f, "View Employee Record"),
            Claim::UpdateEmployeeRecord => write!(f, "Update Employee Record"),
            Claim::ViewCompensation => write!(f, "View Compensation"),
            Claim::UpdateCompensation => write!(f, "Update Compensation"),
            Claim::InitiateOnboarding => write!(f, "Initiate Onboarding"),
            Claim::CompleteOnboardingTask => write!(f, "Complete Onboarding Task"),
            Claim::InitiateOffboarding => write!(f, "Initiate Offboarding"),
            Claim::RevokeAllAccess => write!(f, "Revoke All Access"),
            Claim::CreatePerformanceReview => write!(f, "Create Performance Review"),
            Claim::ViewPerformanceReview => write!(f, "View Performance Review"),
            Claim::SubmitPerformanceFeedback => write!(f, "Submit Performance Feedback"),
            Claim::ApprovePerformanceReview => write!(f, "Approve Performance Review"),
            Claim::CreateTrainingCourse => write!(f, "Create Training Course"),
            Claim::AssignTraining => write!(f, "Assign Training"),
            Claim::CompleteTraining => write!(f, "Complete Training"),
            Claim::ViewTrainingRecord => write!(f, "View Training Record"),
            Claim::IssueCertification => write!(f, "Issue Certification"),
            Claim::RequestLeave => write!(f, "Request Leave"),
            Claim::ViewTeamLeave => write!(f, "View Team Leave"),
            Claim::ApproveLeave => write!(f, "Approve Leave"),
            Claim::ConfigureLeavePolicy => write!(f, "Configure Leave Policy"),

            // NATS
            Claim::CreateNATSOperator => write!(f, "Create NATS Operator"),
            Claim::CreateNATSAccount => write!(f, "Create NATS Account"),
            Claim::CreateNATSUser => write!(f, "Create NATS User"),
            Claim::ManageNATSSubject => write!(f, "Manage NATS Subject"),
            Claim::PublishAnySubject => write!(f, "Publish to Any Subject"),
            Claim::SubscribeAnySubject => write!(f, "Subscribe to Any Subject"),
            Claim::ManageJetStream => write!(f, "Manage JetStream"),
            Claim::ViewNATSMetrics => write!(f, "View NATS Metrics"),

            // Organization
            Claim::CreateOrganizationalUnit => write!(f, "Create Organizational Unit"),
            Claim::UpdateOrganizationalStructure => write!(f, "Update Organizational Structure"),
            Claim::DeleteOrganizationalUnit => write!(f, "Delete Organizational Unit"),
            Claim::ManageOrganizationSettings => write!(f, "Manage Organization Settings"),
            Claim::ViewOrganizationAnalytics => write!(f, "View Organization Analytics"),

            // Policy
            Claim::CreatePolicy => write!(f, "Create Policy"),
            Claim::ReadPolicy => write!(f, "Read Policy"),
            Claim::UpdatePolicy => write!(f, "Update Policy"),
            Claim::DeletePolicy => write!(f, "Delete Policy"),
            Claim::BindPolicy => write!(f, "Bind Policy"),
            Claim::UnbindPolicy => write!(f, "Unbind Policy"),
            Claim::ViewPolicyEvaluationLog => write!(f, "View Policy Evaluation Log"),

            // Emergency
            Claim::InitiateEmergency => write!(f, "Initiate Emergency"),
            Claim::AccessEmergencyControl => write!(f, "Access Emergency Control"),
            Claim::DeclareIncident => write!(f, "Declare Incident"),
            Claim::SuperRead => write!(f, "Super Read (All Resources)"),
            Claim::SuperAdmin => write!(f, "Super Admin (All Operations)"),

            // Custom
            Claim::Custom { domain, resource, action, .. } => {
                write!(f, "Custom: {}:{}:{}", domain, resource, action)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_categories() {
        assert_eq!(Claim::CreateUser.category(), ClaimCategory::Identity);
        assert_eq!(Claim::CreateServer.category(), ClaimCategory::Infrastructure);
        assert_eq!(Claim::DeployToProduction.category(), ClaimCategory::Development);
        assert_eq!(Claim::GenerateKey.category(), ClaimCategory::Security);
        assert_eq!(Claim::ExecuteSQL.category(), ClaimCategory::Data);
        assert_eq!(Claim::ViewLogs.category(), ClaimCategory::Observability);
        assert_eq!(Claim::CreateDocument.category(), ClaimCategory::Communication);
        assert_eq!(Claim::CreateTask.category(), ClaimCategory::Project);
        assert_eq!(Claim::ApproveExpense.category(), ClaimCategory::Finance);
        assert_eq!(Claim::ViewEmployeeRecord.category(), ClaimCategory::HR);
        assert_eq!(Claim::CreateNATSOperator.category(), ClaimCategory::NATS);
        assert_eq!(Claim::CreateOrganizationalUnit.category(), ClaimCategory::Organization);
        assert_eq!(Claim::CreatePolicy.category(), ClaimCategory::Policy);
        assert_eq!(Claim::SuperAdmin.category(), ClaimCategory::Emergency);
    }

    #[test]
    fn test_read_only_claims() {
        assert!(Claim::ReadUser.is_read_only());
        assert!(Claim::ViewLogs.is_read_only());
        assert!(Claim::SuperRead.is_read_only());
        assert!(!Claim::CreateUser.is_read_only());
        assert!(!Claim::DeleteUser.is_read_only());
    }

    #[test]
    fn test_destructive_claims() {
        assert!(Claim::DeleteUser.is_destructive());
        assert!(Claim::RevokeKey.is_destructive());
        assert!(Claim::SuperAdmin.is_destructive());
        assert!(!Claim::ReadUser.is_destructive());
        assert!(!Claim::CreateUser.is_destructive());
    }

    #[test]
    fn test_elevation_required() {
        assert!(Claim::ImpersonateUser.requires_elevation());
        assert!(Claim::DeployToProduction.requires_elevation());
        assert!(Claim::ExportPrivateKey.requires_elevation());
        assert!(Claim::SuperAdmin.requires_elevation());
        assert!(!Claim::ReadUser.requires_elevation());
        assert!(!Claim::CreateTask.requires_elevation());
    }

    #[test]
    fn test_claim_uri() {
        assert_eq!(Claim::CreateUser.uri(), "claim:cim:create_user");
        assert_eq!(Claim::DeployToProduction.uri(), "claim:cim:deploy_to_production");

        let custom = Claim::Custom {
            domain: "myapp".to_string(),
            resource: "widget".to_string(),
            action: "manufacture".to_string(),
            scope: Some("factory-1".to_string()),
        };
        assert_eq!(custom.uri(), "claim:myapp:widget:manufacture:factory-1");
    }

    #[test]
    fn test_claim_display() {
        assert_eq!(format!("{}", Claim::CreateUser), "Create User");
        assert_eq!(format!("{}", Claim::DeployToProduction), "Deploy to Production");
        assert_eq!(format!("{}", Claim::SuperAdmin), "Super Admin (All Operations)");
    }
}
