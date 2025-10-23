//! PKI-specific policies for cim-keys
//!
//! This module defines the standard policies that govern PKI operations
//! including key generation, certificate issuance, and YubiKey provisioning.

use cim_domain_policy::{
    Policy, PolicyRule, RuleExpression, Severity, EnforcementLevel, PolicyTarget, Value,
};
use cim_domain_policy::entities::PolicyTemplate;
use cim_domain_policy::services::PolicyTemplateEngine;
use cim_domain_policy::value_objects::{PolicyId, OperationType, ResourceType};
use std::collections::HashMap;
use uuid::Uuid;

/// Standard PKI policies for cim-keys
pub struct PkiPolicySet {
    /// Key generation policy
    pub key_generation_policy: Policy,
    /// Certificate issuance policy
    pub certificate_issuance_policy: Policy,
    /// YubiKey provisioning policy
    pub yubikey_policy: Policy,
    /// NATS operator key policy
    pub nats_operator_policy: Policy,
    /// Root CA policy
    pub root_ca_policy: Policy,
}

impl PkiPolicySet {
    /// Create the default PKI policy set for an organization
    pub fn default_for_organization(organization_name: &str) -> Self {
        Self {
            key_generation_policy: Self::create_key_generation_policy(organization_name),
            certificate_issuance_policy: Self::create_certificate_policy(organization_name),
            yubikey_policy: Self::create_yubikey_policy(organization_name),
            nats_operator_policy: Self::create_nats_operator_policy(organization_name),
            root_ca_policy: Self::create_root_ca_policy(organization_name),
        }
    }

    /// Create a key generation policy
    fn create_key_generation_policy(org: &str) -> Policy {
        let mut policy = Policy::new(
            format!("{} Key Generation Policy", org),
            "Enforces key generation requirements"
        );

        // Minimum key size for RSA
        policy.add_rule(PolicyRule::new(
            "RSA Minimum Key Size",
            "RSA keys must be at least 2048 bits",
            RuleExpression::And(vec![
                RuleExpression::Equal {
                    field: "algorithm".to_string(),
                    value: Value::String("RSA".to_string()),
                },
                RuleExpression::GreaterThanOrEqual {
                    field: "key_size".to_string(),
                    value: Value::Integer(2048),
                },
            ]),
            Severity::Critical,
        ));

        // Minimum key size for ECDSA
        policy.add_rule(PolicyRule::new(
            "ECDSA Minimum Key Size",
            "ECDSA keys must be at least 256 bits",
            RuleExpression::And(vec![
                RuleExpression::Equal {
                    field: "algorithm".to_string(),
                    value: Value::String("ECDSA".to_string()),
                },
                RuleExpression::GreaterThanOrEqual {
                    field: "key_size".to_string(),
                    value: Value::Integer(256),
                },
            ]),
            Severity::Critical,
        ));

        // Allowed algorithms
        policy.add_rule(PolicyRule::new(
            "Allowed Algorithms",
            "Only approved algorithms may be used",
            RuleExpression::In {
                field: "algorithm".to_string(),
                values: vec![
                    Value::String("RSA".to_string()),
                    Value::String("ECDSA".to_string()),
                    Value::String("Ed25519".to_string()),
                ],
            },
            Severity::Critical,
        ));

        policy.enforcement_level = EnforcementLevel::Hard;
        policy.target = PolicyTarget::Operation(OperationType::KeyGeneration);
        policy
    }

    /// Create a certificate issuance policy
    fn create_certificate_policy(org: &str) -> Policy {
        let mut policy = Policy::new(
            format!("{} Certificate Issuance Policy", org),
            "Governs certificate generation and validity"
        );

        // Maximum validity period
        policy.add_rule(PolicyRule::new(
            "Maximum Certificate Validity",
            "Certificates cannot be valid for more than 365 days",
            RuleExpression::LessThanOrEqual {
                field: "validity_days".to_string(),
                value: Value::Integer(365),
            },
            Severity::High,
        ));

        // Minimum validity period
        policy.add_rule(PolicyRule::new(
            "Minimum Certificate Validity",
            "Certificates must be valid for at least 7 days",
            RuleExpression::GreaterThanOrEqual {
                field: "validity_days".to_string(),
                value: Value::Integer(7),
            },
            Severity::Medium,
        ));

        // Required extensions
        policy.add_rule(PolicyRule::new(
            "Key Usage Extension Required",
            "Certificates must include key usage extension",
            RuleExpression::Exists {
                field: "key_usage".to_string(),
            },
            Severity::High,
        ));

        // Subject requirements
        policy.add_rule(PolicyRule::new(
            "Subject Organization Required",
            "Certificate subject must include organization",
            RuleExpression::Exists {
                field: "subject.organization".to_string(),
            },
            Severity::High,
        ));

        policy.enforcement_level = EnforcementLevel::Hard;
        policy.target = PolicyTarget::Operation(OperationType::CertificateIssuance);
        policy
    }

    /// Create a YubiKey provisioning policy
    fn create_yubikey_policy(org: &str) -> Policy {
        let mut policy = Policy::new(
            format!("{} YubiKey Provisioning Policy", org),
            "Requirements for YubiKey configuration"
        );

        // PIN requirements
        policy.add_rule(PolicyRule::new(
            "PIN Configuration Required",
            "YubiKey must have PIN configured",
            RuleExpression::Equal {
                field: "pin_configured".to_string(),
                value: Value::Bool(true),
            },
            Severity::Critical,
        ));

        // PUK requirements
        policy.add_rule(PolicyRule::new(
            "PUK Configuration Required",
            "YubiKey must have PUK configured",
            RuleExpression::Equal {
                field: "puk_configured".to_string(),
                value: Value::Bool(true),
            },
            Severity::Critical,
        ));

        // Touch policy
        policy.add_rule(PolicyRule::new(
            "Touch Policy Required",
            "Touch confirmation required for signing operations",
            RuleExpression::In {
                field: "touch_policy".to_string(),
                values: vec![
                    Value::String("always".to_string()),
                    Value::String("cached".to_string()),
                ],
            },
            Severity::High,
        ));

        // Management key change
        policy.add_rule(PolicyRule::new(
            "Management Key Changed",
            "Default management key must be changed",
            RuleExpression::NotEqual {
                field: "management_key".to_string(),
                value: Value::String("default".to_string()),
            },
            Severity::Critical,
        ));

        // Firmware version
        policy.add_rule(PolicyRule::new(
            "Minimum Firmware Version",
            "YubiKey firmware must be at least version 5.0",
            RuleExpression::GreaterThanOrEqual {
                field: "firmware_version".to_string(),
                value: Value::String("5.0".to_string()),
            },
            Severity::Medium,
        ));

        policy.enforcement_level = EnforcementLevel::Critical;
        policy.target = PolicyTarget::Resource(ResourceType::Key);
        policy
    }

    /// Create a NATS operator key policy
    fn create_nats_operator_policy(org: &str) -> Policy {
        let mut policy = Policy::new(
            format!("{} NATS Operator Key Policy", org),
            "Requirements for NATS operator keys"
        );

        // Algorithm requirement
        policy.add_rule(PolicyRule::new(
            "Ed25519 Required",
            "NATS operator keys must use Ed25519",
            RuleExpression::Equal {
                field: "algorithm".to_string(),
                value: Value::String("Ed25519".to_string()),
            },
            Severity::Critical,
        ));

        // Storage requirement
        policy.add_rule(PolicyRule::new(
            "Offline Storage Required",
            "Operator keys must be stored offline",
            RuleExpression::Equal {
                field: "storage_location".to_string(),
                value: Value::String("offline".to_string()),
            },
            Severity::Critical,
        ));

        // Multi-signature requirement
        policy.add_rule(PolicyRule::new(
            "Multi-Signature Required",
            "Operator modifications require multiple signatures",
            RuleExpression::GreaterThanOrEqual {
                field: "required_signatures".to_string(),
                value: Value::Integer(2),
            },
            Severity::High,
        ));

        // Backup requirement
        policy.add_rule(PolicyRule::new(
            "Backup Required",
            "Operator keys must have secure backup",
            RuleExpression::Equal {
                field: "backup_exists".to_string(),
                value: Value::Bool(true),
            },
            Severity::Critical,
        ));

        policy.enforcement_level = EnforcementLevel::Critical;
        policy.target = PolicyTarget::Composite(vec![
            PolicyTarget::Resource(ResourceType::Key),
            PolicyTarget::Role("nats-operator".to_string()),
        ]);
        policy
    }

    /// Create a root CA policy
    fn create_root_ca_policy(org: &str) -> Policy {
        let mut policy = Policy::new(
            format!("{} Root CA Policy", org),
            "Requirements for Root Certificate Authority"
        );

        // Key size requirement
        policy.add_rule(PolicyRule::new(
            "Root CA Key Size",
            "Root CA must use at least 4096-bit RSA or 384-bit ECDSA",
            RuleExpression::Or(vec![
                RuleExpression::And(vec![
                    RuleExpression::Equal {
                        field: "algorithm".to_string(),
                        value: Value::String("RSA".to_string()),
                    },
                    RuleExpression::GreaterThanOrEqual {
                        field: "key_size".to_string(),
                        value: Value::Integer(4096),
                    },
                ]),
                RuleExpression::And(vec![
                    RuleExpression::Equal {
                        field: "algorithm".to_string(),
                        value: Value::String("ECDSA".to_string()),
                    },
                    RuleExpression::GreaterThanOrEqual {
                        field: "key_size".to_string(),
                        value: Value::Integer(384),
                    },
                ]),
            ]),
            Severity::Critical,
        ));

        // Validity period
        policy.add_rule(PolicyRule::new(
            "Root CA Validity",
            "Root CA must be valid for 10-20 years",
            RuleExpression::And(vec![
                RuleExpression::GreaterThanOrEqual {
                    field: "validity_years".to_string(),
                    value: Value::Integer(10),
                },
                RuleExpression::LessThanOrEqual {
                    field: "validity_years".to_string(),
                    value: Value::Integer(20),
                },
            ]),
            Severity::High,
        ));

        // Storage requirement
        policy.add_rule(PolicyRule::new(
            "Root CA Storage",
            "Root CA private key must be stored offline",
            RuleExpression::Equal {
                field: "storage_location".to_string(),
                value: Value::String("offline".to_string()),
            },
            Severity::Critical,
        ));

        // Approval requirement
        policy.add_rule(PolicyRule::new(
            "Root CA Approval",
            "Root CA generation requires multiple approvals",
            RuleExpression::GreaterThanOrEqual {
                field: "approval_count".to_string(),
                value: Value::Integer(3),
            },
            Severity::Critical,
        ));

        policy.enforcement_level = EnforcementLevel::Critical;
        policy.target = PolicyTarget::Composite(vec![
            PolicyTarget::Operation(OperationType::CertificateIssuance),
            PolicyTarget::Role("root-ca".to_string()),
        ]);
        policy
    }

    /// Evaluate a key generation request against policies
    pub fn evaluate_key_generation(
        &self,
        algorithm: &str,
        key_size: u32,
    ) -> Result<(), Vec<String>> {
        use cim_domain_policy::{PolicyEvaluator, EvaluationContext};

        let context = EvaluationContext::new()
            .with_field("algorithm", algorithm)
            .with_field("key_size", key_size as i64);

        let evaluator = PolicyEvaluator::new();
        let result = evaluator.evaluate(&self.key_generation_policy, &context)
            .map_err(|e| vec![e.to_string()])?;

        if result.is_compliant() {
            Ok(())
        } else {
            let violations: Vec<String> = result.violations()
                .into_iter()
                .map(|v| format!("{}: {}", v.rule_description, v.details))
                .collect();
            Err(violations)
        }
    }

    /// Evaluate a certificate request against policies
    pub fn evaluate_certificate_request(
        &self,
        validity_days: u32,
        has_key_usage: bool,
        subject_org: Option<&str>,
    ) -> Result<(), Vec<String>> {
        use cim_domain_policy::{PolicyEvaluator, EvaluationContext};

        let mut context = EvaluationContext::new()
            .with_field("validity_days", validity_days as i64);

        if has_key_usage {
            context = context.with_field("key_usage", true);
        }

        if let Some(org) = subject_org {
            context = context.with_field("subject.organization", org);
        }

        let evaluator = PolicyEvaluator::new();
        let result = evaluator.evaluate(&self.certificate_issuance_policy, &context)
            .map_err(|e| vec![e.to_string()])?;

        if result.is_compliant() {
            Ok(())
        } else {
            let violations: Vec<String> = result.violations()
                .into_iter()
                .map(|v| format!("{}: {}", v.rule_description, v.details))
                .collect();
            Err(violations)
        }
    }
}