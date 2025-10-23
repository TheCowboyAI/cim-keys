//! Policy engine for cim-keys
//!
//! This module provides the main policy evaluation engine that integrates
//! with cim-keys operations to enforce organizational policies.

use cim_domain_policy::{
    PolicyEvaluator, PolicyConflictResolver,
    Policy, PolicySet, PolicyExemption,
    EvaluationContext, ComplianceResult, ConflictResolution,
    PolicyError as DomainPolicyError,
};
use cim_domain_policy::services::{PolicyTemplateEngine, EvaluationError};
use cim_domain_policy::value_objects::PolicyId;
use crate::domain::{Organization, Person, KeyContext};
use crate::policy::PkiPolicySet;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Policy evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    #[error("Policy violations: {violations:?}")]
    PolicyViolations { violations: Vec<String> },

    #[error("Exemption required: {0}")]
    ExemptionRequired(String),

    #[error("Domain policy error: {0}")]
    DomainError(#[from] DomainPolicyError),

    #[error("Evaluation error: {0}")]
    EvaluationError(#[from] EvaluationError),
}

/// Main policy engine for cim-keys
pub struct KeyPolicyEngine {
    /// Organization-specific PKI policies
    pki_policies: PkiPolicySet,
    /// Policy evaluator
    evaluator: PolicyEvaluator,
    /// Active exemptions
    exemptions: Vec<PolicyExemption>,
    /// Conflict resolver
    conflict_resolver: PolicyConflictResolver,
    /// Template engine
    template_engine: PolicyTemplateEngine,
}

impl KeyPolicyEngine {
    /// Create a new policy engine for an organization
    pub fn new(organization: &Organization) -> Self {
        Self {
            pki_policies: PkiPolicySet::default_for_organization(&organization.name),
            evaluator: PolicyEvaluator::new(),
            exemptions: Vec::new(),
            conflict_resolver: PolicyConflictResolver::new(ConflictResolution::MostRestrictive),
            template_engine: PolicyTemplateEngine::new(),
        }
    }

    /// Register an exemption
    pub fn add_exemption(&mut self, exemption: PolicyExemption) {
        if exemption.is_valid() {
            self.exemptions.push(exemption);
        }
    }

    /// Clear expired exemptions
    pub fn clear_expired_exemptions(&mut self) {
        self.exemptions.retain(|e| e.is_valid());
    }

    /// Evaluate key generation against policies
    pub fn evaluate_key_generation(
        &mut self,
        person: &Person,
        algorithm: &str,
        key_size: u32,
        purpose: &str,
    ) -> Result<(), PolicyError> {
        // Clear expired exemptions first
        self.clear_expired_exemptions();

        // Build evaluation context
        let context = EvaluationContext::new()
            .with_field("algorithm", algorithm)
            .with_field("key_size", key_size as i64)
            .with_field("purpose", purpose)
            .with_field("requester", person.id.to_string())
            .with_field("organization", person.organization_id.to_string());

        // Register exemptions with evaluator
        let mut evaluator = PolicyEvaluator::new();
        evaluator.register_exemptions(self.exemptions.clone());

        // Evaluate against key generation policy
        let result = evaluator.evaluate(&self.pki_policies.key_generation_policy, &context)?;

        if result.is_compliant() {
            Ok(())
        } else {
            let violations: Vec<String> = result.violations()
                .into_iter()
                .map(|v| format!("{}: {}", v.rule_description, v.details))
                .collect();
            Err(PolicyError::PolicyViolations { violations })
        }
    }

    /// Evaluate certificate issuance against policies
    pub fn evaluate_certificate_issuance(
        &mut self,
        person: &Person,
        validity_days: u32,
        subject_org: &str,
        key_usage: Vec<String>,
    ) -> Result<(), PolicyError> {
        // Clear expired exemptions first
        self.clear_expired_exemptions();

        // Build evaluation context
        let mut context = EvaluationContext::new()
            .with_field("validity_days", validity_days as i64)
            .with_field("subject.organization", subject_org)
            .with_field("requester", person.id.to_string());

        if !key_usage.is_empty() {
            context = context.with_field("key_usage", key_usage.join(","));
        }

        // Register exemptions with evaluator
        let mut evaluator = PolicyEvaluator::new();
        evaluator.register_exemptions(self.exemptions.clone());

        // Evaluate against certificate policy
        let result = evaluator.evaluate(&self.pki_policies.certificate_issuance_policy, &context)?;

        if result.is_compliant() {
            Ok(())
        } else {
            let violations: Vec<String> = result.violations()
                .into_iter()
                .map(|v| format!("{}: {}", v.rule_description, v.details))
                .collect();
            Err(PolicyError::PolicyViolations { violations })
        }
    }

    /// Evaluate YubiKey provisioning against policies
    pub fn evaluate_yubikey_provisioning(
        &mut self,
        person: &Person,
        yubikey_config: &YubikeyConfig,
    ) -> Result<(), PolicyError> {
        // Clear expired exemptions first
        self.clear_expired_exemptions();

        // Build evaluation context
        let context = EvaluationContext::new()
            .with_field("pin_configured", yubikey_config.pin_configured)
            .with_field("puk_configured", yubikey_config.puk_configured)
            .with_field("touch_policy", yubikey_config.touch_policy.clone())
            .with_field("management_key", yubikey_config.management_key_status.clone())
            .with_field("firmware_version", yubikey_config.firmware_version.clone())
            .with_field("requester", person.id.to_string());

        // Register exemptions with evaluator
        let mut evaluator = PolicyEvaluator::new();
        evaluator.register_exemptions(self.exemptions.clone());

        // Evaluate against YubiKey policy
        let result = evaluator.evaluate(&self.pki_policies.yubikey_policy, &context)?;

        if result.is_compliant() {
            Ok(())
        } else {
            let violations: Vec<String> = result.violations()
                .into_iter()
                .map(|v| format!("{}: {}", v.rule_description, v.details))
                .collect();
            Err(PolicyError::PolicyViolations { violations })
        }
    }

    /// Evaluate NATS operator key generation
    pub fn evaluate_nats_operator_key(
        &mut self,
        person: &Person,
        algorithm: &str,
        storage_location: &str,
        required_signatures: u32,
        backup_exists: bool,
    ) -> Result<(), PolicyError> {
        // Clear expired exemptions first
        self.clear_expired_exemptions();

        // Build evaluation context
        let context = EvaluationContext::new()
            .with_field("algorithm", algorithm)
            .with_field("storage_location", storage_location)
            .with_field("required_signatures", required_signatures as i64)
            .with_field("backup_exists", backup_exists)
            .with_field("requester", person.id.to_string());

        // Register exemptions with evaluator
        let mut evaluator = PolicyEvaluator::new();
        evaluator.register_exemptions(self.exemptions.clone());

        // Evaluate against NATS operator policy
        let result = evaluator.evaluate(&self.pki_policies.nats_operator_policy, &context)?;

        if result.is_compliant() {
            Ok(())
        } else {
            let violations: Vec<String> = result.violations()
                .into_iter()
                .map(|v| format!("{}: {}", v.rule_description, v.details))
                .collect();
            Err(PolicyError::PolicyViolations { violations })
        }
    }

    /// Evaluate root CA generation
    pub fn evaluate_root_ca_generation(
        &mut self,
        person: &Person,
        algorithm: &str,
        key_size: u32,
        validity_years: u32,
        storage_location: &str,
        approval_count: u32,
    ) -> Result<(), PolicyError> {
        // Clear expired exemptions first
        self.clear_expired_exemptions();

        // Build evaluation context
        let context = EvaluationContext::new()
            .with_field("algorithm", algorithm)
            .with_field("key_size", key_size as i64)
            .with_field("validity_years", validity_years as i64)
            .with_field("storage_location", storage_location)
            .with_field("approval_count", approval_count as i64)
            .with_field("requester", person.id.to_string());

        // Register exemptions with evaluator
        let mut evaluator = PolicyEvaluator::new();
        evaluator.register_exemptions(self.exemptions.clone());

        // Evaluate against root CA policy
        let result = evaluator.evaluate(&self.pki_policies.root_ca_policy, &context)?;

        if result.is_compliant() {
            Ok(())
        } else {
            let violations: Vec<String> = result.violations()
                .into_iter()
                .map(|v| format!("{}: {}", v.rule_description, v.details))
                .collect();
            Err(PolicyError::PolicyViolations { violations })
        }
    }

    /// Create a custom policy from template
    pub fn create_policy_from_template(
        &self,
        template_name: &str,
        parameters: HashMap<String, cim_domain_policy::Value>,
        policy_name: String,
        policy_description: String,
    ) -> Result<Policy, PolicyError> {
        self.template_engine
            .instantiate(template_name, parameters, policy_name, policy_description)
            .map_err(|e| PolicyError::EvaluationFailed(e.to_string()))
    }

    /// Resolve conflicts between multiple policies
    pub fn resolve_conflicts(&self, policies: Vec<Policy>) -> Result<Vec<Policy>, PolicyError> {
        let conflicts = self.conflict_resolver.detect_conflicts(&policies);

        if conflicts.is_empty() {
            Ok(policies)
        } else {
            self.conflict_resolver
                .resolve_conflicts(policies, conflicts)
                .map_err(|e| PolicyError::EvaluationFailed(e.to_string()))
        }
    }
}

/// YubiKey configuration for policy evaluation
#[derive(Debug, Clone)]
pub struct YubikeyConfig {
    pub pin_configured: bool,
    pub puk_configured: bool,
    pub touch_policy: String,
    pub management_key_status: String,
    pub firmware_version: String,
}

impl Default for YubikeyConfig {
    fn default() -> Self {
        Self {
            pin_configured: false,
            puk_configured: false,
            touch_policy: "never".to_string(),
            management_key_status: "default".to_string(),
            firmware_version: "0.0".to_string(),
        }
    }
}