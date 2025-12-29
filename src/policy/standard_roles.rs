// Copyright (c) 2025 - Cowboy AI, LLC.
//! Standard Role Definitions for IT/Knowledge Worker Organizations
//!
//! This module provides pre-defined roles that can be used as templates
//! for common organizational structures. These roles are designed based on:
//!
//! - Industry best practices for separation of duties
//! - SOC2, HIPAA, PCI-DSS compliance requirements
//! - Common IT organizational hierarchies
//!
//! # Role Tracks
//!
//! - **Development**: Junior Dev → Dev → Senior → Lead → Staff → Principal
//! - **Operations**: Junior SRE → SRE → Senior SRE → Platform → Staff Platform
//! - **Security**: Analyst → Engineer → Architect → CISO
//! - **Data**: Analyst → Engineer → Senior Engineer → Architect
//! - **Support**: Specialist → Engineer → Lead
//! - **Management**: Team Lead → Manager → Director → VP
//! - **Compliance**: Analyst → Auditor → Officer
//!
//! # Separation of Duties
//!
//! Roles are marked as incompatible when holding both would violate
//! separation of duties principles (e.g., Auditor cannot be Developer).

use crate::policy::claims::Claim;
use crate::policy::roles::{Role, RolePurpose, SeparationClass};
use crate::policy::claims::ClaimCategory;
use std::collections::HashSet;
use uuid::Uuid;

/// Standard role template for creating organization roles
#[derive(Debug, Clone)]
pub struct StandardRole {
    /// Role name
    pub name: &'static str,
    /// Purpose description
    pub purpose: &'static str,
    /// Primary domain category
    pub domain: ClaimCategory,
    /// Seniority level (0-5)
    pub level: u8,
    /// Separation class
    pub separation_class: SeparationClass,
    /// Claims granted by this role
    pub claims: &'static [Claim],
    /// Names of incompatible roles
    pub incompatible_with: &'static [&'static str],
}

impl StandardRole {
    /// Convert to a full Role aggregate
    pub fn to_role(&self, created_by: Uuid) -> Result<Role, crate::policy::roles::RoleError> {
        let claims: HashSet<Claim> = self.claims.iter().cloned().collect();
        let purpose = RolePurpose {
            domain: self.domain,
            description: self.purpose.to_string(),
            separation_class: self.separation_class,
            level: self.level,
        };
        Role::new(self.name, purpose, claims, created_by)
    }
}

// ============================================================================
// DEVELOPMENT TRACK
// ============================================================================

/// Junior Developer - Entry-level code contributor
pub const JUNIOR_DEVELOPER: StandardRole = StandardRole {
    name: "Junior Developer",
    purpose: "Entry-level contributor who writes code under supervision and learns organizational practices",
    domain: ClaimCategory::Development,
    level: 0,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadRepository,
        Claim::WriteRepository,
        Claim::CreateBranch,
        Claim::CreateTask,
        Claim::ViewLogs,
        Claim::ReadUser,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Developer - Independent code contributor
pub const DEVELOPER: StandardRole = StandardRole {
    name: "Developer",
    purpose: "Independent contributor who implements features, fixes bugs, and participates in code reviews",
    domain: ClaimCategory::Development,
    level: 1,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadRepository,
        Claim::WriteRepository,
        Claim::CreateBranch,
        Claim::MergeBranch,
        Claim::CreateTask,
        Claim::ViewLogs,
        Claim::ViewPipeline,
        Claim::TriggerPipeline,
        Claim::ReadUser,
        Claim::ViewContainerLogs,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Senior Developer - Experienced contributor with code review authority
pub const SENIOR_DEVELOPER: StandardRole = StandardRole {
    name: "Senior Developer",
    purpose: "Experienced contributor who mentors others, leads technical decisions, and owns subsystems",
    domain: ClaimCategory::Development,
    level: 2,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadRepository,
        Claim::WriteRepository,
        Claim::CreateBranch,
        Claim::MergeBranch,
        Claim::ApprovePullRequest,
        Claim::CreateTask,
        Claim::ViewLogs,
        Claim::ViewPipeline,
        Claim::TriggerPipeline,
        Claim::CreateAlertRule,
        Claim::ConfigureTracing,
        Claim::SignCode,
        Claim::ManageSprintBacklog,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Lead Developer - Technical team leader
pub const LEAD_DEVELOPER: StandardRole = StandardRole {
    name: "Lead Developer",
    purpose: "Technical leader who coordinates team delivery, architects solutions, and approves production deployments",
    domain: ClaimCategory::Development,
    level: 3,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadRepository,
        Claim::WriteRepository,
        Claim::MergeBranch,
        Claim::ApprovePullRequest,
        Claim::DeployToStaging,
        Claim::DeployToProduction,
        Claim::SignCode,
        Claim::CreateAlertRule,
        Claim::ConfigureTracing,
        Claim::ManageSprintBacklog,
        Claim::AssignRole,
        Claim::ViewAuditLogs,
        Claim::ManagePipelineSecrets,
    ],
    incompatible_with: &["Auditor", "Compliance Officer", "CISO"],
};

/// Staff Engineer - Organization-wide technical leader
pub const STAFF_ENGINEER: StandardRole = StandardRole {
    name: "Staff Engineer",
    purpose: "Organization-wide technical leader who sets architectural direction and mentors across teams",
    domain: ClaimCategory::Development,
    level: 4,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadRepository,
        Claim::WriteRepository,
        Claim::MergeBranch,
        Claim::ApprovePullRequest,
        Claim::DeployToProduction,
        Claim::SignCode,
        Claim::CreateAlertRule,
        Claim::ConfigureTracing,
        Claim::ViewSecurityAlerts,
        Claim::OverrideSecurityControl,
        Claim::ManagePipelineSecrets,
        Claim::ConfigurePipeline,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Principal Engineer - Executive-level technologist
pub const PRINCIPAL_ENGINEER: StandardRole = StandardRole {
    name: "Principal Engineer",
    purpose: "Executive-level technologist who shapes company-wide technical strategy and represents engineering externally",
    domain: ClaimCategory::Development,
    level: 5,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadRepository,
        Claim::WriteRepository,
        Claim::MergeBranch,
        Claim::ApprovePullRequest,
        Claim::DeployToProduction,
        Claim::SignCode,
        Claim::GenerateKey,
        Claim::ViewSecurityAlerts,
        Claim::OverrideSecurityControl,
        Claim::ViewFinancialReport,
        Claim::ConfigurePipeline,
        Claim::ManagePipelineSecrets,
    ],
    incompatible_with: &["Auditor", "Compliance Officer", "CISO"],
};

// ============================================================================
// OPERATIONS TRACK
// ============================================================================

/// Junior SRE - Entry-level reliability engineer
pub const JUNIOR_SRE: StandardRole = StandardRole {
    name: "Junior SRE",
    purpose: "Entry-level reliability engineer who monitors systems and responds to alerts under supervision",
    domain: ClaimCategory::Observability,
    level: 0,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ViewLogs,
        Claim::ViewMetrics,
        Claim::ViewAlerts,
        Claim::ViewSecurityAlerts,
        Claim::CreateTask,
        Claim::ReadRepository,
        Claim::ViewTraces,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// SRE - Site Reliability Engineer
pub const SRE: StandardRole = StandardRole {
    name: "SRE",
    purpose: "Reliability engineer who maintains production systems, responds to incidents, and automates operations",
    domain: ClaimCategory::Infrastructure,
    level: 1,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ViewLogs,
        Claim::ViewAuditLogs,
        Claim::CreateAlertRule,
        Claim::AcknowledgeAlert,
        Claim::ConfigureTracing,
        Claim::ManageNamespace,
        Claim::ReadRepository,
        Claim::DeployToProduction,
        Claim::ReadServer,
        Claim::RestartServer,
        Claim::ViewContainerLogs,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Senior SRE - Experienced reliability engineer
pub const SENIOR_SRE: StandardRole = StandardRole {
    name: "Senior SRE",
    purpose: "Experienced reliability engineer who leads incident response, designs resilient systems, and mentors juniors",
    domain: ClaimCategory::Infrastructure,
    level: 2,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ViewLogs,
        Claim::ViewAuditLogs,
        Claim::CreateAlertRule,
        Claim::AcknowledgeAlert,
        Claim::ConfigureTracing,
        Claim::ManageNamespace,
        Claim::ManageFirewall,
        Claim::DeployToProduction,
        Claim::CreateServer,
        Claim::UpdateServer,
        Claim::RestartServer,
        Claim::ConfigureLoadBalancer,
        Claim::OverrideSecurityControl,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Platform Engineer - Internal platform builder
pub const PLATFORM_ENGINEER: StandardRole = StandardRole {
    name: "Platform Engineer",
    purpose: "Engineer who builds and maintains internal developer platforms, tooling, and infrastructure abstractions",
    domain: ClaimCategory::Infrastructure,
    level: 3,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::CreateServer,
        Claim::DeleteServer,
        Claim::ManageNamespace,
        Claim::ManageFirewall,
        Claim::ConfigureLoadBalancer,
        Claim::DeployToProduction,
        Claim::CreateAlertRule,
        Claim::ConfigureTracing,
        Claim::GenerateKey,
        Claim::ViewSecurityAlerts,
        Claim::DeployHelmChart,
        Claim::ManageDNS,
    ],
    incompatible_with: &["Auditor", "Compliance Officer", "Security Architect"],
};

/// Staff Platform Engineer - Senior platform leader
pub const STAFF_PLATFORM_ENGINEER: StandardRole = StandardRole {
    name: "Staff Platform Engineer",
    purpose: "Senior platform leader who architects organization-wide infrastructure and sets operational standards",
    domain: ClaimCategory::Infrastructure,
    level: 4,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::CreateServer,
        Claim::DeleteServer,
        Claim::ManageNamespace,
        Claim::ManageFirewall,
        Claim::ConfigureLoadBalancer,
        Claim::DeployToProduction,
        Claim::GenerateKey,
        Claim::RevokeKey,
        Claim::OverrideSecurityControl,
        Claim::ViewAuditLogs,
        Claim::SignCode,
        Claim::ManageCloudAccount,
        Claim::ConfigureVPN,
    ],
    incompatible_with: &["Auditor", "Compliance Officer", "CISO"],
};

// ============================================================================
// SECURITY TRACK
// ============================================================================

/// Security Analyst - Entry-level security professional
pub const SECURITY_ANALYST: StandardRole = StandardRole {
    name: "Security Analyst",
    purpose: "Entry-level security professional who monitors alerts, triages vulnerabilities, and documents findings",
    domain: ClaimCategory::Security,
    level: 1,
    separation_class: SeparationClass::Audit,
    claims: &[
        Claim::ViewSecurityAlerts,
        Claim::AcknowledgeSecurityAlert,
        Claim::ViewLogs,
        Claim::ViewAuditLogs,
        Claim::ReadRepository,
        Claim::CreateTask,
        Claim::ViewVulnerabilityReport,
    ],
    incompatible_with: &["Developer", "SRE", "Platform Engineer"],
};

/// Security Engineer - Security implementation specialist
pub const SECURITY_ENGINEER: StandardRole = StandardRole {
    name: "Security Engineer",
    purpose: "Security practitioner who implements security controls, conducts assessments, and responds to incidents",
    domain: ClaimCategory::Security,
    level: 2,
    separation_class: SeparationClass::Audit,
    claims: &[
        Claim::ViewSecurityAlerts,
        Claim::AcknowledgeSecurityAlert,
        Claim::ViewAuditLogs,
        Claim::GenerateKey,
        Claim::RevokeKey,
        Claim::ManageFirewall,
        Claim::SignCode,
        Claim::ConfigureMFA,
        Claim::OverrideSecurityControl,
        Claim::ReadRestrictedData,
        Claim::RunSecurityScan,
    ],
    incompatible_with: &["Developer", "Lead Developer", "Platform Engineer"],
};

/// Security Architect - Senior security leader
pub const SECURITY_ARCHITECT: StandardRole = StandardRole {
    name: "Security Architect",
    purpose: "Senior security leader who designs security architecture, sets policy, and approves security exceptions",
    domain: ClaimCategory::Security,
    level: 4,
    separation_class: SeparationClass::Audit,
    claims: &[
        Claim::ViewSecurityAlerts,
        Claim::ViewAuditLogs,
        Claim::GenerateKey,
        Claim::RevokeKey,
        Claim::OverrideSecurityControl,
        Claim::ManageFirewall,
        Claim::ConfigureMFA,
        Claim::SignCode,
        Claim::ReadRestrictedData,
        Claim::ConfigureCompliancePolicy,
        Claim::CreatePolicy,
    ],
    incompatible_with: &["Developer", "Platform Engineer", "HR Manager"],
};

/// CISO - Chief Information Security Officer
pub const CISO: StandardRole = StandardRole {
    name: "CISO",
    purpose: "Executive responsible for organizational security strategy, risk management, and regulatory compliance",
    domain: ClaimCategory::Security,
    level: 5,
    separation_class: SeparationClass::Audit,
    claims: &[
        Claim::ViewSecurityAlerts,
        Claim::ViewAuditLogs,
        Claim::OverrideSecurityControl,
        Claim::RevokeKey,
        Claim::ViewFinancialReport,
        Claim::AcknowledgeComplianceException,
        Claim::AccessEmergencyControl,
        Claim::ViewEmployeeRecord,
        Claim::SignContract,
        Claim::CreatePolicy,
        Claim::DeletePolicy,
    ],
    incompatible_with: &["Developer", "SRE", "Platform Engineer", "VP Engineering"],
};

// ============================================================================
// DATA TRACK
// ============================================================================

/// Data Analyst - Business intelligence analyst
pub const DATA_ANALYST: StandardRole = StandardRole {
    name: "Data Analyst",
    purpose: "Analyst who queries data, builds reports, and provides business insights",
    domain: ClaimCategory::Data,
    level: 1,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadDatabaseSchema,
        Claim::ViewQueryPlan,
        Claim::ViewLogs,
        Claim::CreateDocument,
        Claim::ExportData,
        Claim::ReadRepository,
        Claim::ReadPublicData,
        Claim::ReadInternalData,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Data Engineer - Data pipeline builder
pub const DATA_ENGINEER: StandardRole = StandardRole {
    name: "Data Engineer",
    purpose: "Engineer who builds and maintains data pipelines, warehouses, and processing infrastructure",
    domain: ClaimCategory::Data,
    level: 2,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::CreateDatabase,
        Claim::ReadDatabaseSchema,
        Claim::ModifyDatabaseSchema,
        Claim::ReadRepository,
        Claim::WriteRepository,
        Claim::ConfigureTracing,
        Claim::ViewLogs,
        Claim::ManageNamespace,
        Claim::ExportData,
        Claim::ImportData,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Senior Data Engineer - Experienced data platform engineer
pub const SENIOR_DATA_ENGINEER: StandardRole = StandardRole {
    name: "Senior Data Engineer",
    purpose: "Experienced data engineer who architects data systems and ensures data quality and governance",
    domain: ClaimCategory::Data,
    level: 3,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::CreateDatabase,
        Claim::ModifyDatabaseSchema,
        Claim::ExecuteSQL,
        Claim::ReadRestrictedData,
        Claim::AnonymizeData,
        Claim::ExportData,
        Claim::ImportData,
        Claim::DeployToProduction,
        Claim::ViewAuditLogs,
        Claim::ManageNamespace,
        Claim::ConfigureTracing,
        Claim::RunDataMigration,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Data Architect - Enterprise data leader
pub const DATA_ARCHITECT: StandardRole = StandardRole {
    name: "Data Architect",
    purpose: "Senior leader who designs enterprise data architecture, governance policies, and data strategy",
    domain: ClaimCategory::Data,
    level: 4,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::CreateDatabase,
        Claim::DeleteDatabase,
        Claim::ExecuteSQL,
        Claim::ReadRestrictedData,
        Claim::AnonymizeData,
        Claim::ExportData,
        Claim::ViewAuditLogs,
        Claim::ManageDataRetention,
        Claim::ViewFinancialReport,
        Claim::CreatePolicy,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

// ============================================================================
// SUPPORT TRACK
// ============================================================================

/// Support Specialist - Front-line customer support
pub const SUPPORT_SPECIALIST: StandardRole = StandardRole {
    name: "Support Specialist",
    purpose: "Front-line support who handles customer issues, documents problems, and escalates when needed",
    domain: ClaimCategory::Communication,
    level: 0,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadUser,
        Claim::CreateTask,
        Claim::ViewLogs,
        Claim::CreateDocument,
        Claim::ShareDocumentExternal,
        Claim::ReadDocument,
    ],
    incompatible_with: &["Auditor", "Security Engineer"],
};

/// Support Engineer - Technical support specialist
pub const SUPPORT_ENGINEER: StandardRole = StandardRole {
    name: "Support Engineer",
    purpose: "Technical support engineer who diagnoses complex issues, accesses systems for troubleshooting, and coordinates fixes",
    domain: ClaimCategory::Communication,
    level: 1,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadUser,
        Claim::ViewLogs,
        Claim::ViewAuditLogs,
        Claim::ReadDatabaseSchema,
        Claim::CreateTask,
        Claim::CreateDocument,
        Claim::ViewTraces,
        Claim::ViewAlerts,
    ],
    incompatible_with: &["Auditor", "Security Architect"],
};

/// Support Lead - Support team leader
pub const SUPPORT_LEAD: StandardRole = StandardRole {
    name: "Support Lead",
    purpose: "Support team leader who manages escalations, coordinates with engineering, and improves support processes",
    domain: ClaimCategory::Communication,
    level: 2,
    separation_class: SeparationClass::Operational,
    claims: &[
        Claim::ReadUser,
        Claim::ViewLogs,
        Claim::ViewAuditLogs,
        Claim::CreateTask,
        Claim::ManageSprintBacklog,
        Claim::AssignRole,
        Claim::ApproveTimesheet,
        Claim::CreateDocument,
        Claim::ShareDocumentExternal,
    ],
    incompatible_with: &["Auditor", "Security Architect"],
};

// ============================================================================
// MANAGEMENT TRACK
// ============================================================================

/// Team Lead - First-line manager
pub const TEAM_LEAD: StandardRole = StandardRole {
    name: "Team Lead",
    purpose: "First-line manager who leads a small team, coordinates work, and handles personnel matters",
    domain: ClaimCategory::Organization,
    level: 2,
    separation_class: SeparationClass::Personnel,
    claims: &[
        Claim::AssignRole,
        Claim::RevokeRole,
        Claim::ManageSprintBacklog,
        Claim::ApproveTimesheet,
        Claim::ApproveLeave,
        Claim::ViewEmployeeRecord,
        Claim::CreateTask,
        Claim::ViewLogs,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// Engineering Manager - Multi-team manager
pub const ENGINEERING_MANAGER: StandardRole = StandardRole {
    name: "Engineering Manager",
    purpose: "Manager who leads multiple teams, hires engineers, manages performance, and owns delivery outcomes",
    domain: ClaimCategory::Organization,
    level: 3,
    separation_class: SeparationClass::Personnel,
    claims: &[
        Claim::AssignRole,
        Claim::RevokeRole,
        Claim::ViewEmployeeRecord,
        Claim::UpdateEmployeeRecord,
        Claim::InitiateOnboarding,
        Claim::ApproveLeave,
        Claim::ApproveTimesheet,
        Claim::ApproveExpense,
        Claim::ViewFinancialReport,
        Claim::ApproveDeployment,
    ],
    incompatible_with: &["Auditor", "Compliance Officer", "CISO"],
};

/// Director - Senior department leader
pub const DIRECTOR: StandardRole = StandardRole {
    name: "Director",
    purpose: "Senior leader who manages managers, sets department strategy, and owns significant budget and headcount",
    domain: ClaimCategory::Organization,
    level: 4,
    separation_class: SeparationClass::Personnel,
    claims: &[
        Claim::AssignRole,
        Claim::RevokeRole,
        Claim::ViewEmployeeRecord,
        Claim::UpdateEmployeeRecord,
        Claim::UpdateCompensation,
        Claim::InitiateOnboarding,
        Claim::InitiateOffboarding,
        Claim::ApproveExpense,
        Claim::ViewFinancialReport,
        Claim::SignContract,
    ],
    incompatible_with: &["Auditor", "Compliance Officer", "CISO"],
};

/// VP Engineering - Engineering executive
pub const VP_ENGINEERING: StandardRole = StandardRole {
    name: "VP Engineering",
    purpose: "Executive who leads engineering organization, sets technical direction, and owns engineering budget",
    domain: ClaimCategory::Organization,
    level: 5,
    separation_class: SeparationClass::Personnel,
    claims: &[
        Claim::AssignRole,
        Claim::RevokeRole,
        Claim::ViewEmployeeRecord,
        Claim::UpdateEmployeeRecord,
        Claim::UpdateCompensation,
        Claim::InitiateOnboarding,
        Claim::InitiateOffboarding,
        Claim::RevokeAllAccess,
        Claim::ApproveExpense,
        Claim::SignContract,
        Claim::ViewFinancialReport,
        Claim::OverrideSecurityControl,
    ],
    incompatible_with: &["Auditor", "Compliance Officer", "CISO"],
};

// ============================================================================
// COMPLIANCE TRACK
// ============================================================================

/// Compliance Analyst - Entry-level compliance professional
pub const COMPLIANCE_ANALYST: StandardRole = StandardRole {
    name: "Compliance Analyst",
    purpose: "Entry-level compliance professional who monitors controls, documents findings, and assists audits",
    domain: ClaimCategory::Security,
    level: 1,
    separation_class: SeparationClass::Audit,
    claims: &[
        Claim::ViewAuditLogs,
        Claim::ViewSecurityAlerts,
        Claim::ViewLogs,
        Claim::CreateDocument,
        Claim::CreateTask,
        Claim::ViewFinancialReport,
        Claim::ViewComplianceReport,
    ],
    incompatible_with: &["Developer", "SRE", "Platform Engineer", "Data Engineer"],
};

/// Auditor - Internal auditor
pub const AUDITOR: StandardRole = StandardRole {
    name: "Auditor",
    purpose: "Independent auditor who conducts internal audits, assesses controls, and reports to leadership",
    domain: ClaimCategory::Security,
    level: 2,
    separation_class: SeparationClass::Audit,
    claims: &[
        Claim::ViewAuditLogs,
        Claim::ViewSecurityAlerts,
        Claim::ViewLogs,
        Claim::ViewEmployeeRecord,
        Claim::ViewFinancialReport,
        Claim::ReadRestrictedData,
        Claim::ReadDatabaseSchema,
        Claim::ExportData,
        Claim::ViewComplianceReport,
    ],
    incompatible_with: &[
        "Developer", "Senior Developer", "Lead Developer", "Staff Engineer", "Principal Engineer",
        "SRE", "Senior SRE", "Platform Engineer", "Staff Platform Engineer",
        "Data Engineer", "Senior Data Engineer", "Data Architect",
        "Engineering Manager", "Director", "VP Engineering",
    ],
};

/// Compliance Officer - Senior compliance leader
pub const COMPLIANCE_OFFICER: StandardRole = StandardRole {
    name: "Compliance Officer",
    purpose: "Senior compliance leader who owns compliance program, certifies controls, and reports to board",
    domain: ClaimCategory::Security,
    level: 4,
    separation_class: SeparationClass::Audit,
    claims: &[
        Claim::ViewAuditLogs,
        Claim::ViewSecurityAlerts,
        Claim::ViewLogs,
        Claim::ViewEmployeeRecord,
        Claim::ViewFinancialReport,
        Claim::ReadRestrictedData,
        Claim::AcknowledgeComplianceException,
        Claim::SignContract,
        Claim::DeclareIncident,
        Claim::ViewComplianceReport,
        Claim::ConfigureCompliancePolicy,
    ],
    incompatible_with: &[
        "Developer", "Lead Developer", "Staff Engineer", "Principal Engineer",
        "Platform Engineer", "Staff Platform Engineer",
        "Data Architect", "VP Engineering", "Director",
    ],
};

// ============================================================================
// C-LEVEL EXECUTIVE TRACK
// ============================================================================

/// CEO - Chief Executive Officer
pub const CEO: StandardRole = StandardRole {
    name: "CEO",
    purpose: "Chief Executive Officer who leads the organization, sets vision, and has ultimate authority",
    domain: ClaimCategory::Organization,
    level: 5,
    separation_class: SeparationClass::Personnel,
    claims: &[
        // Organization
        Claim::ManageOrganizationSettings,
        Claim::CreateOrganizationalUnit,
        Claim::UpdateOrganizationalStructure,
        Claim::ViewOrganizationAnalytics,
        // Personnel
        Claim::AssignRole,
        Claim::RevokeRole,
        Claim::ViewEmployeeRecord,
        Claim::UpdateCompensation,
        Claim::InitiateOnboarding,
        Claim::InitiateOffboarding,
        Claim::RevokeAllAccess,
        // Financial
        Claim::ViewFinancialReport,
        Claim::GenerateFinancialReport,
        Claim::ApproveExpense,
        Claim::SignContract,
        // Policy
        Claim::CreatePolicy,
        Claim::UpdatePolicy,
        Claim::DeletePolicy,
        // Emergency
        Claim::AccessEmergencyControl,
        Claim::InitiateEmergency,
        Claim::OverrideSecurityControl,
        // Audit
        Claim::ViewAuditLogs,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// COO - Chief Operating Officer
pub const COO: StandardRole = StandardRole {
    name: "COO",
    purpose: "Chief Operating Officer who oversees daily operations, infrastructure, and organizational efficiency",
    domain: ClaimCategory::Organization,
    level: 5,
    separation_class: SeparationClass::Personnel,
    claims: &[
        // Organization
        Claim::ManageOrganizationSettings,
        Claim::CreateOrganizationalUnit,
        Claim::UpdateOrganizationalStructure,
        Claim::ViewOrganizationAnalytics,
        // Personnel
        Claim::AssignRole,
        Claim::RevokeRole,
        Claim::ViewEmployeeRecord,
        Claim::UpdateEmployeeRecord,
        Claim::InitiateOnboarding,
        Claim::InitiateOffboarding,
        Claim::ApproveLeave,
        Claim::ApproveTimesheet,
        // Infrastructure
        Claim::ManageCloudAccount,
        Claim::ViewCloudBilling,
        Claim::ConfigureCloudQuota,
        // Financial (operational)
        Claim::ViewFinancialReport,
        Claim::ApproveExpense,
        Claim::ApprovePurchaseRequest,
        Claim::ManageVendor,
        // Policy
        Claim::CreatePolicy,
        Claim::UpdatePolicy,
        // Emergency
        Claim::AccessEmergencyControl,
        Claim::OverrideSecurityControl,
        // Audit
        Claim::ViewAuditLogs,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

/// CFO - Chief Financial Officer
pub const CFO: StandardRole = StandardRole {
    name: "CFO",
    purpose: "Chief Financial Officer who manages finances, budgets, investments, and financial compliance",
    domain: ClaimCategory::Finance,
    level: 5,
    separation_class: SeparationClass::Financial,
    claims: &[
        // Financial - Full Authority
        Claim::ViewFinancialReport,
        Claim::GenerateFinancialReport,
        Claim::ExportFinancialData,
        Claim::CreateInvoice,
        Claim::ViewInvoice,
        Claim::ApproveInvoice,
        Claim::VoidInvoice,
        Claim::ApproveExpense,
        Claim::ApprovePurchaseRequest,
        Claim::ManageVendor,
        Claim::SignContract,
        // Budget
        Claim::ViewProjectBudget,
        Claim::UpdateProjectBudget,
        Claim::ApproveBudgetRequest,
        // Cloud/Infrastructure Costs
        Claim::ViewCloudBilling,
        Claim::ConfigureCloudQuota,
        // Personnel (compensation)
        Claim::ViewCompensation,
        Claim::UpdateCompensation,
        Claim::ViewEmployeeRecord,
        // Compliance
        Claim::ViewComplianceReport,
        Claim::ViewAuditLogs,
        // Policy
        Claim::CreatePolicy,
    ],
    incompatible_with: &["Auditor", "Developer", "SRE"],
};

/// CLO - Chief Legal Officer
pub const CLO: StandardRole = StandardRole {
    name: "CLO",
    purpose: "Chief Legal Officer who manages legal affairs, contracts, compliance, and corporate governance",
    domain: ClaimCategory::Organization,
    level: 5,
    separation_class: SeparationClass::Audit,
    claims: &[
        // Contracts & Legal
        Claim::SignContract,
        Claim::ManageVendor,
        // Compliance
        Claim::ViewComplianceReport,
        Claim::ConfigureCompliancePolicy,
        Claim::AcknowledgeComplianceException,
        // Audit & Investigation
        Claim::ViewAuditLogs,
        Claim::ViewSecurityAlerts,
        Claim::DeclareIncident,
        // Data (legal hold, discovery)
        Claim::ReadRestrictedData,
        Claim::ExportData,
        Claim::ManageDataRetention,
        // Personnel (legal matters)
        Claim::ViewEmployeeRecord,
        Claim::InitiateOffboarding,
        // Policy
        Claim::CreatePolicy,
        Claim::UpdatePolicy,
        Claim::DeletePolicy,
        // Financial (legal review)
        Claim::ViewFinancialReport,
        Claim::ViewInvoice,
    ],
    incompatible_with: &["Developer", "SRE", "Platform Engineer"],
};

/// CSO - Chief Science Officer
pub const CSO: StandardRole = StandardRole {
    name: "CSO",
    purpose: "Chief Science Officer who leads research, innovation, technical strategy, and scientific methodology",
    domain: ClaimCategory::Development,
    level: 5,
    separation_class: SeparationClass::Operational,
    claims: &[
        // Development - Technical Leadership
        Claim::ReadRepository,
        Claim::WriteRepository,
        Claim::MergeBranch,
        Claim::ApprovePullRequest,
        Claim::DeployToProduction,
        Claim::SignCode,
        Claim::ConfigurePipeline,
        // Research Infrastructure
        Claim::CreateServer,
        Claim::ManageNamespace,
        Claim::ConfigureTracing,
        Claim::ViewMetrics,
        // Data & Analysis
        Claim::CreateDatabase,
        Claim::ExecuteSQL,
        Claim::ReadRestrictedData,
        Claim::ExportData,
        Claim::AnonymizeData,
        // Security (research security)
        Claim::GenerateKey,
        Claim::ViewSecurityAlerts,
        // Documentation & Knowledge
        Claim::CreateDocument,
        Claim::CreateWikiPage,
        Claim::EditWikiPage,
        Claim::ManageWikiStructure,
        // Policy (research governance)
        Claim::CreatePolicy,
        Claim::ViewAuditLogs,
        // Budget (research budget)
        Claim::ViewProjectBudget,
        Claim::ViewFinancialReport,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

// ============================================================================
// EMERGENCY ROLE
// ============================================================================

/// Emergency Responder - Break-glass access for critical incidents
pub const EMERGENCY_RESPONDER: StandardRole = StandardRole {
    name: "Emergency Responder",
    purpose: "Break-glass role for critical incidents requiring elevated access with full audit trail",
    domain: ClaimCategory::Emergency,
    level: 4,
    separation_class: SeparationClass::Emergency,
    claims: &[
        Claim::OverrideSecurityControl,
        Claim::DeployToProduction,
        Claim::RollbackDeployment,
        Claim::ManageFirewall,
        Claim::RevokeKey,
        Claim::DeleteServer,
        Claim::ViewLogs,
        Claim::ViewAuditLogs,
        Claim::ReadRestrictedData,
        Claim::AccessEmergencyControl,
        Claim::InitiateEmergency,
        Claim::DeclareIncident,
    ],
    incompatible_with: &["Auditor", "Compliance Officer"],
};

// ============================================================================
// ALL STANDARD ROLES
// ============================================================================

/// All standard roles for easy iteration
pub const ALL_STANDARD_ROLES: &[&StandardRole] = &[
    // Development Track
    &JUNIOR_DEVELOPER,
    &DEVELOPER,
    &SENIOR_DEVELOPER,
    &LEAD_DEVELOPER,
    &STAFF_ENGINEER,
    &PRINCIPAL_ENGINEER,
    // Operations Track
    &JUNIOR_SRE,
    &SRE,
    &SENIOR_SRE,
    &PLATFORM_ENGINEER,
    &STAFF_PLATFORM_ENGINEER,
    // Security Track
    &SECURITY_ANALYST,
    &SECURITY_ENGINEER,
    &SECURITY_ARCHITECT,
    &CISO,
    // Data Track
    &DATA_ANALYST,
    &DATA_ENGINEER,
    &SENIOR_DATA_ENGINEER,
    &DATA_ARCHITECT,
    // Support Track
    &SUPPORT_SPECIALIST,
    &SUPPORT_ENGINEER,
    &SUPPORT_LEAD,
    // Management Track
    &TEAM_LEAD,
    &ENGINEERING_MANAGER,
    &DIRECTOR,
    &VP_ENGINEERING,
    // C-Level Executive Track
    &CEO,
    &COO,
    &CFO,
    &CLO,
    &CSO,
    // Compliance Track
    &COMPLIANCE_ANALYST,
    &AUDITOR,
    &COMPLIANCE_OFFICER,
    // Emergency
    &EMERGENCY_RESPONDER,
];

/// Get a standard role by name
pub fn get_standard_role(name: &str) -> Option<&'static StandardRole> {
    ALL_STANDARD_ROLES.iter().find(|r| r.name == name).copied()
}

/// Get all roles in a specific track
pub fn get_roles_by_track(domain: ClaimCategory) -> Vec<&'static StandardRole> {
    ALL_STANDARD_ROLES.iter()
        .filter(|r| r.domain == domain)
        .copied()
        .collect()
}

/// Get all roles at a specific level
pub fn get_roles_by_level(level: u8) -> Vec<&'static StandardRole> {
    ALL_STANDARD_ROLES.iter()
        .filter(|r| r.level == level)
        .copied()
        .collect()
}

/// Get all roles in a separation class
pub fn get_roles_by_separation_class(class: SeparationClass) -> Vec<&'static StandardRole> {
    ALL_STANDARD_ROLES.iter()
        .filter(|r| r.separation_class == class)
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_roles_have_claims() {
        for role in ALL_STANDARD_ROLES {
            assert!(!role.claims.is_empty(), "Role {} has no claims", role.name);
        }
    }

    #[test]
    fn test_all_roles_convertible() {
        let created_by = Uuid::now_v7();
        for std_role in ALL_STANDARD_ROLES {
            let result = std_role.to_role(created_by);
            assert!(result.is_ok(), "Failed to convert role {}: {:?}", std_role.name, result.err());
        }
    }

    #[test]
    fn test_get_standard_role() {
        assert!(get_standard_role("Developer").is_some());
        assert!(get_standard_role("CISO").is_some());
        assert!(get_standard_role("Nonexistent").is_none());
    }

    #[test]
    fn test_development_track() {
        let dev_roles = get_roles_by_track(ClaimCategory::Development);
        assert!(dev_roles.len() >= 5);
        assert!(dev_roles.iter().any(|r| r.name == "Junior Developer"));
        assert!(dev_roles.iter().any(|r| r.name == "Principal Engineer"));
    }

    #[test]
    fn test_audit_separation_class() {
        let audit_roles = get_roles_by_separation_class(SeparationClass::Audit);
        assert!(audit_roles.iter().any(|r| r.name == "Auditor"));
        assert!(audit_roles.iter().any(|r| r.name == "CISO"));
    }

    #[test]
    fn test_level_hierarchy() {
        let entry_level = get_roles_by_level(0);
        let executive_level = get_roles_by_level(5);

        assert!(entry_level.iter().any(|r| r.name == "Junior Developer"));
        assert!(executive_level.iter().any(|r| r.name == "Principal Engineer"));
        assert!(executive_level.iter().any(|r| r.name == "CISO"));
    }

    #[test]
    fn test_c_level_roles() {
        // Verify all C-Level roles exist
        assert!(get_standard_role("CEO").is_some());
        assert!(get_standard_role("COO").is_some());
        assert!(get_standard_role("CFO").is_some());
        assert!(get_standard_role("CLO").is_some());
        assert!(get_standard_role("CSO").is_some());

        // All C-Level roles are level 5
        let executive_level = get_roles_by_level(5);
        assert!(executive_level.iter().any(|r| r.name == "CEO"));
        assert!(executive_level.iter().any(|r| r.name == "COO"));
        assert!(executive_level.iter().any(|r| r.name == "CFO"));
        assert!(executive_level.iter().any(|r| r.name == "CLO"));
        assert!(executive_level.iter().any(|r| r.name == "CSO"));
    }

    #[test]
    fn test_c_level_convertible() {
        let created_by = Uuid::now_v7();

        let ceo = CEO.to_role(created_by);
        assert!(ceo.is_ok(), "Failed to convert CEO: {:?}", ceo.err());

        let coo = COO.to_role(created_by);
        assert!(coo.is_ok(), "Failed to convert COO: {:?}", coo.err());

        let cfo = CFO.to_role(created_by);
        assert!(cfo.is_ok(), "Failed to convert CFO: {:?}", cfo.err());

        let clo = CLO.to_role(created_by);
        assert!(clo.is_ok(), "Failed to convert CLO: {:?}", clo.err());

        let cso = CSO.to_role(created_by);
        assert!(cso.is_ok(), "Failed to convert CSO: {:?}", cso.err());
    }
}
