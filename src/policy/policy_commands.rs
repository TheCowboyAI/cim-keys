//! Policy-related commands for cim-keys

use cim_domain::{Command, MessageIdentity, EntityId};
use cim_domain_policy::value_objects::PolicyId;
use crate::aggregate::KeyManagementAggregate;
use crate::domain::{Person, KeyContext};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Commands related to policy enforcement in cim-keys
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command_type")]
pub enum KeyPolicyCommand {
    /// Evaluate key generation against policies
    EvaluateKeyGeneration(EvaluateKeyGeneration),
    /// Evaluate certificate issuance against policies
    EvaluateCertificateIssuance(EvaluateCertificateIssuance),
    /// Request policy exemption for key operation
    RequestKeyPolicyExemption(RequestKeyPolicyExemption),
    /// Approve policy exemption
    ApproveKeyPolicyExemption(ApproveKeyPolicyExemption),
    /// Enforce policy on key operation
    EnforceKeyPolicy(EnforceKeyPolicy),
    /// Create custom policy for organization
    CreateOrganizationPolicy(CreateOrganizationPolicy),
}

impl Command for KeyPolicyCommand {
    type Aggregate = KeyManagementAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        match self {
            KeyPolicyCommand::EvaluateKeyGeneration(cmd) => cmd.key_id.map(EntityId::from_uuid),
            KeyPolicyCommand::EvaluateCertificateIssuance(cmd) => Some(EntityId::from_uuid(cmd.certificate_id)),
            KeyPolicyCommand::RequestKeyPolicyExemption(cmd) => cmd.key_id.map(EntityId::from_uuid),
            KeyPolicyCommand::ApproveKeyPolicyExemption(_cmd) => None, // Exemption aggregate (cmd fields used in handler)
            KeyPolicyCommand::EnforceKeyPolicy(cmd) => cmd.key_id.map(EntityId::from_uuid),
            KeyPolicyCommand::CreateOrganizationPolicy(_) => None, // Policy aggregate
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateKeyGeneration {
    pub identity: MessageIdentity,
    pub key_id: Option<Uuid>,
    pub person: Person,
    pub algorithm: String,
    pub key_size: u32,
    pub purpose: String,
    pub context: KeyContext,
}

impl Command for EvaluateKeyGeneration {
    type Aggregate = KeyManagementAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        self.key_id.map(EntityId::from_uuid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateCertificateIssuance {
    pub identity: MessageIdentity,
    pub certificate_id: Uuid,
    pub key_id: Uuid,
    pub person: Person,
    pub validity_days: u32,
    pub subject_organization: String,
    pub key_usage: Vec<String>,
}

impl Command for EvaluateCertificateIssuance {
    type Aggregate = KeyManagementAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.key_id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestKeyPolicyExemption {
    pub identity: MessageIdentity,
    pub policy_id: PolicyId,
    pub key_id: Option<Uuid>,
    pub requester: Person,
    pub reason: String,
    pub justification: String,
    pub requested_duration: Duration,
    pub operation_type: KeyOperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyOperationType {
    KeyGeneration,
    CertificateIssuance,
    KeyRotation,
    YubikeyProvisioning,
    NatsOperatorKey,
    RootCaGeneration,
}

impl Command for RequestKeyPolicyExemption {
    type Aggregate = KeyManagementAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        self.key_id.map(EntityId::from_uuid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveKeyPolicyExemption {
    pub identity: MessageIdentity,
    pub exemption_request_id: Uuid,
    pub policy_id: PolicyId,
    pub approver: Person,
    pub approval_notes: String,
    pub risk_acceptance: Option<String>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub conditions: Vec<ExemptionCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExemptionCondition {
    pub field: String,
    pub operator: String,
    pub value: String,
}

impl Command for ApproveKeyPolicyExemption {
    type Aggregate = KeyManagementAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        // Note: This operates on exemption aggregate, not key aggregate
        // The exemption_request_id and policy_id are used in the handler
        tracing::debug!("Processing exemption for policy {:?} and request {:?}",
            self.policy_id, self.exemption_request_id);
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforceKeyPolicy {
    pub identity: MessageIdentity,
    pub policy_id: PolicyId,
    pub key_id: Option<Uuid>,
    pub enforcement_action: EnforcementAction,
    pub enforced_by: Person,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementAction {
    Block,
    Allow,
    AllowWithWarning,
    RequireApproval,
    RequireExemption,
}

impl Command for EnforceKeyPolicy {
    type Aggregate = KeyManagementAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        self.key_id.map(EntityId::from_uuid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganizationPolicy {
    pub identity: MessageIdentity,
    pub organization_id: Uuid,
    pub policy_name: String,
    pub policy_description: String,
    pub policy_type: OrganizationPolicyType,
    pub rules: Vec<PolicyRuleDefinition>,
    pub enforcement_level: String,
    pub created_by: Person,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationPolicyType {
    KeyGeneration,
    CertificateIssuance,
    YubikeyProvisioning,
    AccessControl,
    Compliance,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRuleDefinition {
    pub name: String,
    pub description: String,
    pub expression_type: String,
    pub parameters: serde_json::Value,
    pub severity: String,
}