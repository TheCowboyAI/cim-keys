//! Policy-related events for cim-keys

use cim_domain::{DomainEvent, MessageIdentity};
use cim_domain_policy::{ComplianceResult, Severity};
use cim_domain_policy::value_objects::{PolicyId, ExemptionId};
use crate::domain::Person;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Events related to policy enforcement in cim-keys
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum KeyPolicyEvent {
    /// Key generation was evaluated against policies
    KeyGenerationEvaluated(KeyGenerationEvaluated),
    /// Certificate issuance was evaluated against policies
    CertificateIssuanceEvaluated(CertificateIssuanceEvaluated),
    /// Policy violation detected during key operation
    KeyPolicyViolationDetected(KeyPolicyViolationDetected),
    /// Policy exemption requested for key operation
    KeyPolicyExemptionRequested(KeyPolicyExemptionRequested),
    /// Policy exemption approved
    KeyPolicyExemptionApproved(KeyPolicyExemptionApproved),
    /// Policy enforced on key operation
    KeyPolicyEnforced(KeyPolicyEnforced),
    /// Organization policy created
    OrganizationPolicyCreated(OrganizationPolicyCreated),
    /// YubiKey provisioning evaluated
    YubikeyProvisioningEvaluated(YubikeyProvisioningEvaluated),
    /// Root CA generation evaluated
    RootCaGenerationEvaluated(RootCaGenerationEvaluated),
}

impl DomainEvent for KeyPolicyEvent {
    fn event_type(&self) -> &'static str {
        match self {
            KeyPolicyEvent::KeyGenerationEvaluated(_) => "KeyGenerationEvaluated",
            KeyPolicyEvent::CertificateIssuanceEvaluated(_) => "CertificateIssuanceEvaluated",
            KeyPolicyEvent::KeyPolicyViolationDetected(_) => "KeyPolicyViolationDetected",
            KeyPolicyEvent::KeyPolicyExemptionRequested(_) => "KeyPolicyExemptionRequested",
            KeyPolicyEvent::KeyPolicyExemptionApproved(_) => "KeyPolicyExemptionApproved",
            KeyPolicyEvent::KeyPolicyEnforced(_) => "KeyPolicyEnforced",
            KeyPolicyEvent::OrganizationPolicyCreated(_) => "OrganizationPolicyCreated",
            KeyPolicyEvent::YubikeyProvisioningEvaluated(_) => "YubikeyProvisioningEvaluated",
            KeyPolicyEvent::RootCaGenerationEvaluated(_) => "RootCaGenerationEvaluated",
        }
    }

    fn aggregate_id(&self) -> Uuid {
        match self {
            KeyPolicyEvent::KeyGenerationEvaluated(e) => e.key_id.unwrap_or(e.evaluation_id),
            KeyPolicyEvent::CertificateIssuanceEvaluated(e) => e.certificate_id,
            KeyPolicyEvent::KeyPolicyViolationDetected(e) => e.key_id.unwrap_or(e.violation_id),
            KeyPolicyEvent::KeyPolicyExemptionRequested(e) => e.exemption_request_id,
            KeyPolicyEvent::KeyPolicyExemptionApproved(e) => e.exemption_id.0,
            KeyPolicyEvent::KeyPolicyEnforced(e) => e.key_id.unwrap_or(e.enforcement_id),
            KeyPolicyEvent::OrganizationPolicyCreated(e) => e.policy_id.0,
            KeyPolicyEvent::YubikeyProvisioningEvaluated(e) => e.yubikey_id,
            KeyPolicyEvent::RootCaGenerationEvaluated(e) => e.evaluation_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGenerationEvaluated {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub evaluation_id: Uuid,
    pub key_id: Option<Uuid>,
    pub policy_id: PolicyId,
    pub evaluated_by: Person,
    pub algorithm: String,
    pub key_size: u32,
    pub result: ComplianceResult,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateIssuanceEvaluated {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub certificate_id: Uuid,
    pub key_id: Uuid,
    pub policy_id: PolicyId,
    pub evaluated_by: Person,
    pub validity_days: u32,
    pub result: ComplianceResult,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPolicyViolationDetected {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub violation_id: Uuid,
    pub key_id: Option<Uuid>,
    pub policy_id: PolicyId,
    pub violations: Vec<PolicyViolation>,
    pub severity: Severity,
    pub detected_at: DateTime<Utc>,
    pub operation_blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule_name: String,
    pub rule_description: String,
    pub severity: Severity,
    pub details: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPolicyExemptionRequested {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub exemption_request_id: Uuid,
    pub policy_id: PolicyId,
    pub key_id: Option<Uuid>,
    pub requested_by: Person,
    pub reason: String,
    pub justification: String,
    pub requested_at: DateTime<Utc>,
    pub requested_duration_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPolicyExemptionApproved {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub exemption_id: ExemptionId,
    pub exemption_request_id: Uuid,
    pub policy_id: PolicyId,
    pub approved_by: Person,
    pub approval_notes: String,
    pub risk_acceptance: Option<String>,
    pub approved_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPolicyEnforced {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub enforcement_id: Uuid,
    pub policy_id: PolicyId,
    pub key_id: Option<Uuid>,
    pub enforcement_action: String,
    pub enforced_by: Person,
    pub enforced_at: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationPolicyCreated {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub policy_id: PolicyId,
    pub organization_id: Uuid,
    pub policy_name: String,
    pub policy_description: String,
    pub policy_type: String,
    pub created_by: Person,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubikeyProvisioningEvaluated {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub yubikey_id: Uuid,
    pub policy_id: PolicyId,
    pub evaluated_by: Person,
    pub pin_configured: bool,
    pub puk_configured: bool,
    pub touch_policy: String,
    pub result: ComplianceResult,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCaGenerationEvaluated {
    pub event_id: Uuid,
    pub identity: MessageIdentity,
    pub evaluation_id: Uuid,
    pub policy_id: PolicyId,
    pub evaluated_by: Person,
    pub algorithm: String,
    pub key_size: u32,
    pub validity_years: u32,
    pub approval_count: u32,
    pub result: ComplianceResult,
    pub evaluated_at: DateTime<Utc>,
}