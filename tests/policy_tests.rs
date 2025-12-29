//! Unit tests for claims-based policy system
//!
//! Tests cover:
//! - Policy creation and condition evaluation
//! - Claims composition (additive union)
//! - Priority-based policy sorting
//! - Role assignment and fulfillment
//! - Complex multi-policy scenarios

use cim_keys::domain::*;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_policy_claim_composition() {
    // Given: Two policies with different claims
    let policy_a = Policy {
        id: Uuid::now_v7(),
        name: "Developer Access".to_string(),
        description: "Basic developer permissions".to_string(),
        claims: vec![
            PolicyClaim::CanAccessDevelopment,
            PolicyClaim::CanSignCode,
        ],
        conditions: vec![],
        priority: 100,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let policy_b = Policy {
        id: Uuid::now_v7(),
        name: "Production Access".to_string(),
        description: "Production environment access".to_string(),
        claims: vec![
            PolicyClaim::CanAccessProduction,
            PolicyClaim::CanSignCode, // Overlapping claim
        ],
        conditions: vec![],
        priority: 200,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    // When: Policies are bound to a person
    let person_id = Uuid::now_v7();
    let bindings = vec![
        PolicyBinding {
            id: Uuid::now_v7(),
            policy_id: policy_a.id,
            entity_id: person_id,
            entity_type: PolicyEntityType::Person,
            bound_at: Utc::now(),
            bound_by: Uuid::now_v7(),
            active: true,
        },
        PolicyBinding {
            id: Uuid::now_v7(),
            policy_id: policy_b.id,
            entity_id: person_id,
            entity_type: PolicyEntityType::Person,
            bound_at: Utc::now(),
            bound_by: Uuid::now_v7(),
            active: true,
        },
    ];

    let context = PolicyEvaluationContext {
        person_id,
        person_clearance: SecurityClearance::Internal,
        person_units: vec![],
        person_roles: vec![],
        employment_start_date: Utc::now(),
        completed_training: vec![],
        current_time: Utc::now(),
        current_location: None,
        source_ip: None,
        mfa_verified: false,
        yubikey_present: false,
        witnesses: vec![],
    };

    // Then: Claims should be unioned (3 unique claims, not 4)
    let evaluation = evaluate_policies(
        &[policy_a, policy_b],
        &bindings,
        person_id,
        PolicyEntityType::Person,
        &context,
    );

    assert_eq!(evaluation.active_policies.len(), 2);
    assert_eq!(evaluation.granted_claims.len(), 3); // Union, not 4
    assert!(evaluation.granted_claims.contains(&PolicyClaim::CanAccessDevelopment));
    assert!(evaluation.granted_claims.contains(&PolicyClaim::CanAccessProduction));
    assert!(evaluation.granted_claims.contains(&PolicyClaim::CanSignCode));
}

#[test]
fn test_policy_priority_sorting() {
    // Given: Three policies with different priorities
    let low_priority = Policy {
        id: Uuid::now_v7(),
        name: "Low Priority".to_string(),
        description: "Low priority policy".to_string(),
        claims: vec![PolicyClaim::CanAccessDevelopment],
        conditions: vec![],
        priority: 10,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let high_priority = Policy {
        id: Uuid::now_v7(),
        name: "High Priority".to_string(),
        description: "High priority policy".to_string(),
        claims: vec![PolicyClaim::CanAccessProduction],
        conditions: vec![],
        priority: 1000,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let medium_priority = Policy {
        id: Uuid::now_v7(),
        name: "Medium Priority".to_string(),
        description: "Medium priority policy".to_string(),
        claims: vec![PolicyClaim::CanAccessStaging],
        conditions: vec![],
        priority: 500,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let person_id = Uuid::now_v7();
    let bindings = vec![
        PolicyBinding {
            id: Uuid::now_v7(),
            policy_id: low_priority.id,
            entity_id: person_id,
            entity_type: PolicyEntityType::Person,
            bound_at: Utc::now(),
            bound_by: Uuid::now_v7(),
            active: true,
        },
        PolicyBinding {
            id: Uuid::now_v7(),
            policy_id: high_priority.id,
            entity_id: person_id,
            entity_type: PolicyEntityType::Person,
            bound_at: Utc::now(),
            bound_by: Uuid::now_v7(),
            active: true,
        },
        PolicyBinding {
            id: Uuid::now_v7(),
            policy_id: medium_priority.id,
            entity_id: person_id,
            entity_type: PolicyEntityType::Person,
            bound_at: Utc::now(),
            bound_by: Uuid::now_v7(),
            active: true,
        },
    ];

    let context = PolicyEvaluationContext {
        person_id,
        person_clearance: SecurityClearance::Internal,
        person_units: vec![],
        person_roles: vec![],
        employment_start_date: Utc::now(),
        completed_training: vec![],
        current_time: Utc::now(),
        current_location: None,
        source_ip: None,
        mfa_verified: false,
        yubikey_present: false,
        witnesses: vec![],
    };

    // When: Policies are evaluated
    let evaluation = evaluate_policies(
        &[low_priority.clone(), high_priority.clone(), medium_priority.clone()],
        &bindings,
        person_id,
        PolicyEntityType::Person,
        &context,
    );

    // Then: All policies should be active (sorted by priority internally)
    assert_eq!(evaluation.active_policies.len(), 3);
    // Note: active_policies are sorted by priority (high to low)
    assert_eq!(evaluation.active_policies[0], high_priority.id);
    assert_eq!(evaluation.active_policies[1], medium_priority.id);
    assert_eq!(evaluation.active_policies[2], low_priority.id);
}

#[test]
fn test_policy_condition_minimum_clearance() {
    // Given: Policy requiring Secret clearance
    let secret_policy = Policy {
        id: Uuid::now_v7(),
        name: "Secret Operations".to_string(),
        description: "Requires Secret clearance".to_string(),
        claims: vec![PolicyClaim::CanAccessProduction],
        conditions: vec![PolicyCondition::MinimumSecurityClearance(SecurityClearance::Secret)],
        priority: 100,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let person_id = Uuid::now_v7();
    let binding = PolicyBinding {
        id: Uuid::now_v7(),
        policy_id: secret_policy.id,
        entity_id: person_id,
        entity_type: PolicyEntityType::Person,
        bound_at: Utc::now(),
        bound_by: Uuid::now_v7(),
        active: true,
    };

    // When: Person has insufficient clearance (Internal < Secret)
    let context_low_clearance = PolicyEvaluationContext {
        person_id,
        person_clearance: SecurityClearance::Internal,
        person_units: vec![],
        person_roles: vec![],
        employment_start_date: Utc::now(),
        completed_training: vec![],
        current_time: Utc::now(),
        current_location: None,
        source_ip: None,
        mfa_verified: false,
        yubikey_present: false,
        witnesses: vec![],
    };

    let evaluation_fail = evaluate_policies(
        &[secret_policy.clone()],
        &[binding.clone()],
        person_id,
        PolicyEntityType::Person,
        &context_low_clearance,
    );

    // Then: Policy should be inactive
    assert_eq!(evaluation_fail.active_policies.len(), 0);
    assert_eq!(evaluation_fail.inactive_policies.len(), 1);
    assert_eq!(evaluation_fail.granted_claims.len(), 0);

    // When: Person has sufficient clearance (TopSecret > Secret)
    let context_high_clearance = PolicyEvaluationContext {
        person_clearance: SecurityClearance::TopSecret,
        ..context_low_clearance
    };

    let evaluation_pass = evaluate_policies(
        &[secret_policy.clone()],
        &[binding],
        person_id,
        PolicyEntityType::Person,
        &context_high_clearance,
    );

    // Then: Policy should be active
    assert_eq!(evaluation_pass.active_policies.len(), 1);
    assert_eq!(evaluation_pass.inactive_policies.len(), 0);
    assert_eq!(evaluation_pass.granted_claims.len(), 1);
    assert!(evaluation_pass.granted_claims.contains(&PolicyClaim::CanAccessProduction));
}

#[test]
fn test_policy_condition_mfa_required() {
    // Given: Policy requiring MFA
    let mfa_policy = Policy {
        id: Uuid::now_v7(),
        name: "MFA Required".to_string(),
        description: "Must have MFA verified".to_string(),
        claims: vec![PolicyClaim::CanModifyInfrastructure],
        conditions: vec![PolicyCondition::MFAEnabled(true)],
        priority: 100,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let person_id = Uuid::now_v7();
    let binding = PolicyBinding {
        id: Uuid::now_v7(),
        policy_id: mfa_policy.id,
        entity_id: person_id,
        entity_type: PolicyEntityType::Person,
        bound_at: Utc::now(),
        bound_by: Uuid::now_v7(),
        active: true,
    };

    // When: MFA is not verified
    let context_no_mfa = PolicyEvaluationContext {
        person_id,
        person_clearance: SecurityClearance::Internal,
        person_units: vec![],
        person_roles: vec![],
        employment_start_date: Utc::now(),
        completed_training: vec![],
        current_time: Utc::now(),
        current_location: None,
        source_ip: None,
        mfa_verified: false,
        yubikey_present: false,
        witnesses: vec![],
    };

    let evaluation_fail = evaluate_policies(
        &[mfa_policy.clone()],
        &[binding.clone()],
        person_id,
        PolicyEntityType::Person,
        &context_no_mfa,
    );

    // Then: Policy should be inactive
    assert_eq!(evaluation_fail.active_policies.len(), 0);
    assert_eq!(evaluation_fail.granted_claims.len(), 0);

    // When: MFA is verified
    let context_with_mfa = PolicyEvaluationContext {
        mfa_verified: true,
        ..context_no_mfa
    };

    let evaluation_pass = evaluate_policies(
        &[mfa_policy],
        &[binding],
        person_id,
        PolicyEntityType::Person,
        &context_with_mfa,
    );

    // Then: Policy should be active
    assert_eq!(evaluation_pass.active_policies.len(), 1);
    assert_eq!(evaluation_pass.granted_claims.len(), 1);
}

#[test]
fn test_policy_condition_witness_required() {
    // Given: Policy requiring 2 witnesses with Secret clearance
    let witness_policy = Policy {
        id: Uuid::now_v7(),
        name: "Dual Control".to_string(),
        description: "Requires 2 witnesses with Secret clearance".to_string(),
        claims: vec![PolicyClaim::CanOverrideSecurityControls],
        conditions: vec![PolicyCondition::RequiresWitness {
            count: 2,
            witness_clearance: Some(SecurityClearance::Secret),
        }],
        priority: 100,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let person_id = Uuid::now_v7();
    let binding = PolicyBinding {
        id: Uuid::now_v7(),
        policy_id: witness_policy.id,
        entity_id: person_id,
        entity_type: PolicyEntityType::Person,
        bound_at: Utc::now(),
        bound_by: Uuid::now_v7(),
        active: true,
    };

    // When: Insufficient witnesses (only 1)
    let context_one_witness = PolicyEvaluationContext {
        person_id,
        person_clearance: SecurityClearance::TopSecret,
        person_units: vec![],
        person_roles: vec![],
        employment_start_date: Utc::now(),
        completed_training: vec![],
        current_time: Utc::now(),
        current_location: None,
        source_ip: None,
        mfa_verified: true,
        yubikey_present: true,
        witnesses: vec![WitnessInfo {
            person_id: Uuid::now_v7(),
            clearance: SecurityClearance::TopSecret,
        }],
    };

    let evaluation_fail = evaluate_policies(
        &[witness_policy.clone()],
        &[binding.clone()],
        person_id,
        PolicyEntityType::Person,
        &context_one_witness,
    );

    // Then: Policy should be inactive (need 2 witnesses)
    assert_eq!(evaluation_fail.active_policies.len(), 0);

    // When: Sufficient witnesses but insufficient clearance
    let context_low_clearance_witnesses = PolicyEvaluationContext {
        witnesses: vec![
            WitnessInfo {
                person_id: Uuid::now_v7(),
                clearance: SecurityClearance::Internal,  // Too low!
            },
            WitnessInfo {
                person_id: Uuid::now_v7(),
                clearance: SecurityClearance::Confidential,  // Too low!
            },
        ],
        ..context_one_witness.clone()
    };

    let evaluation_fail_clearance = evaluate_policies(
        &[witness_policy.clone()],
        &[binding.clone()],
        person_id,
        PolicyEntityType::Person,
        &context_low_clearance_witnesses,
    );

    // Then: Policy should be inactive (witnesses lack clearance)
    assert_eq!(evaluation_fail_clearance.active_policies.len(), 0);

    // When: Sufficient witnesses with sufficient clearance
    let context_good_witnesses = PolicyEvaluationContext {
        witnesses: vec![
            WitnessInfo {
                person_id: Uuid::now_v7(),
                clearance: SecurityClearance::Secret,
            },
            WitnessInfo {
                person_id: Uuid::now_v7(),
                clearance: SecurityClearance::TopSecret,
            },
        ],
        ..context_one_witness
    };

    let evaluation_pass = evaluate_policies(
        &[witness_policy],
        &[binding],
        person_id,
        PolicyEntityType::Person,
        &context_good_witnesses,
    );

    // Then: Policy should be active
    assert_eq!(evaluation_pass.active_policies.len(), 1);
    assert_eq!(evaluation_pass.granted_claims.len(), 1);
    assert!(evaluation_pass.granted_claims.contains(&PolicyClaim::CanOverrideSecurityControls));
}

#[test]
fn test_role_fulfillment() {
    // Given: A role requiring specific policies
    let dev_policy = Policy {
        id: Uuid::now_v7(),
        name: "Developer Policy".to_string(),
        description: "Basic developer access".to_string(),
        claims: vec![
            PolicyClaim::CanAccessDevelopment,
            PolicyClaim::CanSignCode,
        ],
        conditions: vec![],
        priority: 100,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let senior_dev_role = Role {
        id: Uuid::now_v7(),
        name: "Senior Developer".to_string(),
        description: "Senior development role".to_string(),
        organization_id: Uuid::now_v7(),
        unit_id: None,
        required_policies: vec![dev_policy.id],
        responsibilities: vec!["Code review".to_string(), "Mentoring".to_string()],
        created_by: Uuid::now_v7(),
        active: true,
    };

    let person_id = Uuid::now_v7();
    let binding = PolicyBinding {
        id: Uuid::now_v7(),
        policy_id: dev_policy.id,
        entity_id: person_id,
        entity_type: PolicyEntityType::Person,
        bound_at: Utc::now(),
        bound_by: Uuid::now_v7(),
        active: true,
    };

    let context = PolicyEvaluationContext {
        person_id,
        person_clearance: SecurityClearance::Internal,
        person_units: vec![],
        person_roles: vec![],
        employment_start_date: Utc::now(),
        completed_training: vec![],
        current_time: Utc::now(),
        current_location: None,
        source_ip: None,
        mfa_verified: false,
        yubikey_present: false,
        witnesses: vec![],
    };

    // When: Person has the required policy active
    let evaluation = evaluate_policies(
        &[dev_policy.clone()],
        &[binding],
        person_id,
        PolicyEntityType::Person,
        &context,
    );

    // Then: Person can fulfill the role
    assert!(senior_dev_role.can_person_fulfill(
        person_id,
        &evaluation,
        &[dev_policy],
    ));
}

#[test]
fn test_complex_multi_policy_scenario() {
    // Given: Complex scenario with multiple policies and conditions
    let base_access = Policy {
        id: Uuid::now_v7(),
        name: "Base Access".to_string(),
        description: "Basic access for all employees".to_string(),
        claims: vec![
            PolicyClaim::CanAccessDevelopment,
            PolicyClaim::CanViewAuditLogs,
        ],
        conditions: vec![],
        priority: 10,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let staging_access = Policy {
        id: Uuid::now_v7(),
        name: "Staging Access".to_string(),
        description: "Access to staging environment".to_string(),
        claims: vec![
            PolicyClaim::CanAccessStaging,
            PolicyClaim::CanDeployServices,
        ],
        conditions: vec![
            PolicyCondition::MinimumSecurityClearance(SecurityClearance::Confidential),
            PolicyCondition::MFAEnabled(true),
        ],
        priority: 50,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let production_access = Policy {
        id: Uuid::now_v7(),
        name: "Production Access".to_string(),
        description: "Access to production environment".to_string(),
        claims: vec![
            PolicyClaim::CanAccessProduction,
            PolicyClaim::CanDeployServices,
            PolicyClaim::CanModifyInfrastructure,
        ],
        conditions: vec![
            PolicyCondition::MinimumSecurityClearance(SecurityClearance::Secret),
            PolicyCondition::MFAEnabled(true),
            PolicyCondition::YubiKeyRequired(true),
        ],
        priority: 100,
        enabled: true,
        created_by: Uuid::now_v7(),
        metadata: std::collections::HashMap::new(),
    };

    let person_id = Uuid::now_v7();
    let bindings = vec![
        PolicyBinding {
            id: Uuid::now_v7(),
            policy_id: base_access.id,
            entity_id: person_id,
            entity_type: PolicyEntityType::Person,
            bound_at: Utc::now(),
            bound_by: Uuid::now_v7(),
            active: true,
        },
        PolicyBinding {
            id: Uuid::now_v7(),
            policy_id: staging_access.id,
            entity_id: person_id,
            entity_type: PolicyEntityType::Person,
            bound_at: Utc::now(),
            bound_by: Uuid::now_v7(),
            active: true,
        },
        PolicyBinding {
            id: Uuid::now_v7(),
            policy_id: production_access.id,
            entity_id: person_id,
            entity_type: PolicyEntityType::Person,
            bound_at: Utc::now(),
            bound_by: Uuid::now_v7(),
            active: true,
        },
    ];

    // Scenario 1: Junior developer (Confidential clearance, MFA but no YubiKey)
    let junior_context = PolicyEvaluationContext {
        person_id,
        person_clearance: SecurityClearance::Confidential,
        person_units: vec![],
        person_roles: vec![],
        employment_start_date: Utc::now(),
        completed_training: vec![],
        current_time: Utc::now(),
        current_location: None,
        source_ip: None,
        mfa_verified: true,
        yubikey_present: false,  // No YubiKey!
        witnesses: vec![],
    };

    let junior_evaluation = evaluate_policies(
        &[base_access.clone(), staging_access.clone(), production_access.clone()],
        &bindings,
        person_id,
        PolicyEntityType::Person,
        &junior_context,
    );

    // Then: Should have base + staging access, but NOT production
    assert_eq!(junior_evaluation.active_policies.len(), 2);
    assert!(junior_evaluation.active_policies.contains(&base_access.id));
    assert!(junior_evaluation.active_policies.contains(&staging_access.id));
    assert!(!junior_evaluation.active_policies.contains(&production_access.id));

    assert!(junior_evaluation.granted_claims.contains(&PolicyClaim::CanAccessDevelopment));
    assert!(junior_evaluation.granted_claims.contains(&PolicyClaim::CanAccessStaging));
    assert!(!junior_evaluation.granted_claims.contains(&PolicyClaim::CanAccessProduction));

    // Scenario 2: Senior engineer (Secret clearance, MFA, YubiKey)
    let senior_context = PolicyEvaluationContext {
        person_clearance: SecurityClearance::Secret,
        mfa_verified: true,
        yubikey_present: true,
        ..junior_context
    };

    let senior_evaluation = evaluate_policies(
        &[base_access, staging_access, production_access],
        &bindings,
        person_id,
        PolicyEntityType::Person,
        &senior_context,
    );

    // Then: Should have ALL access
    assert_eq!(senior_evaluation.active_policies.len(), 3);
    assert!(senior_evaluation.granted_claims.contains(&PolicyClaim::CanAccessDevelopment));
    assert!(senior_evaluation.granted_claims.contains(&PolicyClaim::CanAccessStaging));
    assert!(senior_evaluation.granted_claims.contains(&PolicyClaim::CanAccessProduction));
    assert!(senior_evaluation.granted_claims.contains(&PolicyClaim::CanModifyInfrastructure));
}
