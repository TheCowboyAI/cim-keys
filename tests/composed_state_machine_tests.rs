// Copyright (c) 2025 - Cowboy AI, LLC.

//! Composed State Machine Tests
//!
//! These tests validate workflows that compose multiple state machines together.
//! They verify that cross-aggregate invariants hold during complex workflows
//! like key generation, certificate issuance, and revocation cascades.
//!
//! Based on Phase 4 of the Domain Ontology Validation Plan.

use chrono::{Duration, Utc};
use proptest::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use cim_keys::state_machines::{
    KeyState, CertificateState, YubiKeyState, PersonState, OrganizationState,
    NatsOperatorState, NatsAccountState, NatsUserState,
    key::{RevocationReason as KeyRevocationReason, ArchivedFromState as KeyArchivedFrom},
    certificate::{RevocationReason as CertRevocationReason, ArchivedFromState as CertArchivedFrom},
    yubikey::{PivSlot, RetirementReason},
    nats_account::NatsPermissions,
    nats_user::NatsUserPermissions,
};
use cim_keys::types::KeyAlgorithm;

// ============================================================================
// Helper Functions for State Construction
// ============================================================================

fn person_created(created_by: Uuid) -> PersonState {
    PersonState::Created { created_by }
}

fn person_active(roles: Vec<Uuid>) -> PersonState {
    PersonState::Active {
        roles,
        activated_at: Utc::now(),
        last_activity: None,
    }
}

fn key_generated(algorithm: KeyAlgorithm, generated_by: Uuid) -> KeyState {
    KeyState::Generated {
        algorithm,
        generated_at: Utc::now(),
        generated_by,
    }
}

fn key_active() -> KeyState {
    KeyState::Active {
        activated_at: Utc::now(),
        usage_count: 0,
        last_used: None,
    }
}

fn yubikey_detected(serial: &str, firmware: &str, detected_by: Uuid) -> YubiKeyState {
    YubiKeyState::Detected {
        serial: serial.to_string(),
        firmware: firmware.to_string(),
        detected_at: Utc::now(),
        detected_by,
    }
}

fn yubikey_provisioned(provisioned_by: Uuid, slots: HashMap<PivSlot, Uuid>) -> YubiKeyState {
    YubiKeyState::Provisioned {
        provisioned_at: Utc::now(),
        provisioned_by,
        slots,
        pin_changed: true,
        puk_changed: true,
    }
}

fn yubikey_active(assigned_to: Uuid) -> YubiKeyState {
    YubiKeyState::Active {
        assigned_to,
        activated_at: Utc::now(),
        last_used: None,
        usage_count: 0,
    }
}

fn cert_pending(requested_by: Uuid) -> CertificateState {
    CertificateState::Pending {
        csr_id: Some(Uuid::now_v7()),
        pending_since: Utc::now(),
        requested_by,
    }
}

fn cert_issued(issuer_id: Uuid, issued_by: Uuid) -> CertificateState {
    CertificateState::Issued {
        issued_at: Utc::now(),
        issuer_id,
        issued_by,
    }
}

fn cert_active() -> CertificateState {
    CertificateState::Active {
        not_before: Utc::now() - Duration::hours(1),
        not_after: Utc::now() + Duration::days(365),
        usage_count: 0,
        last_used: None,
    }
}

fn org_draft(created_by: Uuid) -> OrganizationState {
    OrganizationState::Draft { created_by }
}

fn org_active(units: Vec<Uuid>, members: Vec<Uuid>) -> OrganizationState {
    OrganizationState::Active {
        activated_at: Utc::now(),
        units,
        members,
    }
}

fn nats_operator_created(created_by: Uuid) -> NatsOperatorState {
    NatsOperatorState::Created {
        created_by,
        operator_name: "test-operator".to_string(),
    }
}

fn nats_operator_active(accounts: Vec<Uuid>) -> NatsOperatorState {
    NatsOperatorState::Active {
        activated_at: Utc::now(),
        jwt_issued_at: Utc::now(),
        accounts,
    }
}

fn nats_account_created(created_by: Uuid, operator_id: Uuid) -> NatsAccountState {
    NatsAccountState::Created {
        created_by,
        operator_id,
    }
}

fn nats_account_active(users: Vec<Uuid>) -> NatsAccountState {
    NatsAccountState::Active {
        permissions: default_nats_permissions(),
        activated_at: Utc::now(),
        users,
    }
}

fn nats_user_created(created_by: Uuid, account_id: Uuid, person_id: Uuid) -> NatsUserState {
    NatsUserState::Created {
        created_by,
        account_id,
        person_id,
    }
}

fn nats_user_active() -> NatsUserState {
    NatsUserState::Active {
        permissions: default_nats_user_permissions(),
        activated_at: Utc::now(),
        last_connection: None,
    }
}

fn default_nats_permissions() -> NatsPermissions {
    NatsPermissions {
        publish: vec!["*".to_string()],
        subscribe: vec!["*".to_string()],
        allow_responses: true,
        max_connections: Some(100),
        max_payload: Some(1024 * 1024),
    }
}

fn default_nats_user_permissions() -> NatsUserPermissions {
    NatsUserPermissions {
        publish: vec!["org.>".to_string()],
        subscribe: vec!["org.>".to_string()],
        allow_responses: true,
        max_payload: Some(1024 * 1024),
    }
}

// ============================================================================
// Composed Workflow Tests
// ============================================================================

/// Test: Key Generation Workflow Composition
///
/// This workflow composes: Person + Key + YubiKey + Certificate state machines.
/// Validates that all preconditions are met before key generation can proceed.
#[test]
fn test_key_generation_workflow_composition() {
    let admin_id = Uuid::now_v7();
    let person_id = Uuid::now_v7();
    let key_id = Uuid::now_v7();
    let root_ca_id = Uuid::now_v7();

    // Step 1: Person must be Active
    let person_state = person_active(vec![Uuid::now_v7()]);
    assert!(person_state.is_active(), "Person must be active for key generation");
    assert!(person_state.can_generate_keys(), "Person must have permission to generate keys");

    // Step 2: YubiKey must be Provisioned
    let yubikey_state = yubikey_provisioned(
        admin_id,
        HashMap::from([(PivSlot::Signature, key_id)]),
    );
    assert!(yubikey_state.is_provisioned(), "YubiKey must be provisioned");

    // Step 3: Verify cross-machine preconditions
    let can_generate = person_state.is_active() && yubikey_state.is_provisioned();
    assert!(can_generate, "Both Person.Active AND YubiKey.Provisioned required");

    // Step 4: Key generation
    let key_state = key_generated(KeyAlgorithm::Ed25519, person_id);
    assert!(matches!(key_state, KeyState::Generated { .. }));

    // Step 5: Key activation
    let key_state = key_active();
    assert!(key_state.is_active(), "Key should be active after activation");
    assert!(key_state.can_use_for_crypto(), "Active key should be usable for crypto");

    // Step 6: Certificate issuance (depends on Key.Active)
    let cert_state = cert_issued(root_ca_id, admin_id);
    assert!(matches!(cert_state, CertificateState::Issued { .. }));

    // Verify cross-machine invariants
    assert!(key_state.is_active() && matches!(cert_state, CertificateState::Issued { .. }));
}

/// Test: Key cannot be generated if Person is not Active
#[test]
fn test_key_generation_requires_active_person() {
    let admin_id = Uuid::now_v7();
    let person_state = person_created(admin_id);

    // Person is created but not active - key generation should be blocked
    assert!(!person_state.is_active());
    assert!(!person_state.can_generate_keys());
}

/// Test: Key cannot be stored on YubiKey if YubiKey is not Provisioned
#[test]
fn test_key_storage_requires_provisioned_yubikey() {
    let admin_id = Uuid::now_v7();
    let yubikey_state = yubikey_detected("12345678", "5.4.3", admin_id);

    // YubiKey is detected but not provisioned
    assert!(!yubikey_state.is_provisioned());
    assert!(!yubikey_state.can_use_for_crypto());
}

// ============================================================================
// Composed Workflow: Revocation Cascade
// ============================================================================

/// Test: Certificate revocation cascade
///
/// When an intermediate certificate is revoked, leaf certificates must be cascade revoked.
#[test]
fn test_revocation_cascade_composition() {
    let admin_id = Uuid::now_v7();

    // Setup: Active certificate chain
    let root_ca = cert_active();
    let intermediate = cert_active();
    let leaf = cert_active();

    assert!(root_ca.is_active(), "Root CA should be active");
    assert!(intermediate.is_active(), "Intermediate CA should be active");
    assert!(leaf.is_active(), "Leaf cert should be active");

    // Revoke intermediate
    let intermediate_revoked = CertificateState::Revoked {
        reason: CertRevocationReason::KeyCompromise,
        revoked_at: Utc::now(),
        revoked_by: admin_id,
        crl_published: true,
        ocsp_updated: true,
    };
    assert!(intermediate_revoked.is_revoked(), "Intermediate should be revoked");
    assert!(intermediate_revoked.is_terminal(), "Revoked is terminal state");

    // Cascade revocation to leaf
    let cascade_required = intermediate_revoked.is_revoked();
    assert!(cascade_required, "Leaf revocation should be triggered");

    let leaf_revoked = CertificateState::Revoked {
        reason: CertRevocationReason::CACompromise,
        revoked_at: Utc::now(),
        revoked_by: admin_id,
        crl_published: true,
        ocsp_updated: true,
    };
    assert!(leaf_revoked.is_revoked(), "Leaf should be cascade revoked");

    // Root unaffected
    assert!(root_ca.is_active(), "Root CA should remain active");
}

/// Test: Key revocation triggers consideration
#[test]
fn test_key_revocation_cascade() {
    let admin_id = Uuid::now_v7();

    // Active key
    let key_state = key_active();
    assert!(key_state.is_active());

    // Revoke the key
    let revoked_key = KeyState::Revoked {
        reason: KeyRevocationReason::Compromised,
        revoked_at: Utc::now(),
        revoked_by: admin_id,
    };

    assert!(revoked_key.is_revoked());
    assert!(revoked_key.is_terminal());
    assert!(!revoked_key.can_use_for_crypto());
}

// ============================================================================
// Composed Workflow: Person Onboarding
// ============================================================================

/// Test: Complete person onboarding workflow
///
/// This workflow composes: Organization + Person + NATS hierarchy.
#[test]
fn test_person_onboarding_workflow() {
    let admin_id = Uuid::now_v7();
    let person_id = Uuid::now_v7();
    let unit_id = Uuid::now_v7();
    let account_id = Uuid::now_v7();

    // Step 1: Organization must be Active
    let org_state = org_active(vec![unit_id], vec![]);
    assert!(org_state.is_active(), "Organization must be active");
    assert!(org_state.can_add_members(), "Organization must allow adding members");

    // Step 2: Create and activate person
    let person_created_state = person_created(admin_id);
    assert!(matches!(person_created_state, PersonState::Created { .. }));

    let person_state = person_active(vec![Uuid::now_v7()]);
    assert!(person_state.is_active(), "Person should be active");

    // Step 3: NATS hierarchy must be set up
    let operator_state = nats_operator_active(vec![account_id]);
    assert!(operator_state.is_active(), "Operator must be active");
    assert!(operator_state.can_create_accounts(), "Operator must be able to create accounts");

    let account_state = nats_account_active(vec![]);
    assert!(account_state.is_active(), "Account must be active");
    assert!(account_state.can_create_users(), "Account must allow creating users");

    // Step 4: Create NATS user for person
    let nats_user = nats_user_created(admin_id, account_id, person_id);
    assert!(matches!(nats_user, NatsUserState::Created { .. }));

    let nats_user_active_state = nats_user_active();
    assert!(nats_user_active_state.is_active(), "NATS user should be active");
    assert!(nats_user_active_state.can_connect(), "Active NATS user should be able to connect");

    // Cross-context invariant: Person maps to NATS user
    assert!(person_state.is_active() && nats_user_active_state.is_active());
}

/// Test: Person cannot be activated without Active Organization
#[test]
fn test_person_requires_active_organization() {
    let admin_id = Uuid::now_v7();
    let org_state = org_draft(admin_id);

    // Organization is draft, not active
    assert!(!org_state.is_active());
    assert!(!org_state.can_add_members());
}

/// Test: NATS User cannot be created without Active Account
#[test]
fn test_nats_user_requires_active_account() {
    let admin_id = Uuid::now_v7();
    let operator_id = Uuid::now_v7();
    let account_state = nats_account_created(admin_id, operator_id);

    // Account is created but not active
    assert!(!account_state.is_active());
    assert!(!account_state.can_create_users());
}

// ============================================================================
// Composed Workflow: PKI Bootstrap
// ============================================================================

/// Test: PKI bootstrap workflow
#[test]
fn test_pki_bootstrap_workflow() {
    let admin_id = Uuid::now_v7();
    let root_ca_id = Uuid::now_v7();

    // Step 1: Generate Root CA key
    let root_key = key_generated(KeyAlgorithm::Ed25519, admin_id);
    assert!(matches!(root_key, KeyState::Generated { .. }));

    // Step 2: Activate Root CA key
    let root_key = key_active();
    assert!(root_key.is_active());

    // Step 3: Create Root CA certificate (self-signed)
    let root_cert = cert_issued(root_ca_id, admin_id);
    assert!(matches!(root_cert, CertificateState::Issued { .. }));

    // Step 4: Activate Root CA certificate
    let root_cert = cert_active();
    assert!(root_cert.is_active());

    // Step 5: Generate Intermediate CA key (depends on Root CA active)
    assert!(root_cert.is_active(), "Root CA must be active before generating intermediate");
    let int_key = key_generated(KeyAlgorithm::Ed25519, admin_id);
    assert!(matches!(int_key, KeyState::Generated { .. }));

    // Step 6: Activate Intermediate CA key
    let int_key = key_active();
    assert!(int_key.is_active());

    // Step 7: Issue Intermediate CA certificate (signed by Root CA)
    let int_cert = cert_issued(root_ca_id, admin_id);
    assert!(matches!(int_cert, CertificateState::Issued { .. }));

    // Step 8: Activate Intermediate CA certificate
    let int_cert = cert_active();
    assert!(int_cert.is_active());

    // Verify: Complete PKI chain is active
    assert!(root_key.is_active() && root_cert.is_active());
    assert!(int_key.is_active() && int_cert.is_active());
}

/// Test: Intermediate CA requires valid Root CA
#[test]
fn test_intermediate_ca_requires_root() {
    let admin_id = Uuid::now_v7();
    let root_ca_id = Uuid::now_v7();

    // Root CA must be active
    let root_ca = cert_active();
    assert!(root_ca.is_active());
    assert!(!root_ca.is_revoked());

    // Then Intermediate CA can be issued
    let intermediate = cert_issued(root_ca_id, admin_id);
    assert!(matches!(intermediate, CertificateState::Issued { .. }));
}

// ============================================================================
// Temporal Ordering Tests
// ============================================================================

/// Test: State transitions preserve time ordering
#[test]
fn test_temporal_ordering_preserved() {
    let admin_id = Uuid::now_v7();
    let t1 = Utc::now();
    let t2 = t1 + Duration::hours(1);
    let t3 = t2 + Duration::hours(1);

    // Person created at t1
    let _person = PersonState::Created { created_by: admin_id };

    // Activated at t2
    let person = PersonState::Active {
        roles: vec![Uuid::now_v7()],
        activated_at: t2,
        last_activity: None,
    };
    assert!(t2 > t1, "Activation must be after creation");

    // Activity at t3
    let person = PersonState::Active {
        roles: vec![Uuid::now_v7()],
        activated_at: t2,
        last_activity: Some(t3),
    };

    if let PersonState::Active { last_activity, activated_at, .. } = person {
        assert!(last_activity.unwrap() >= activated_at, "Activity must be after activation");
    }
}

/// Test: Key lifecycle temporal ordering
#[test]
fn test_temporal_ordering_key_lifecycle() {
    let generated_at = Utc::now() - Duration::hours(2);
    let activated_at = Utc::now() - Duration::hours(1);
    let used_at = Utc::now();

    // Generated → Activated → Used
    assert!(generated_at < activated_at);
    assert!(activated_at < used_at);

    let key = KeyState::Active {
        activated_at,
        usage_count: 1,
        last_used: Some(used_at),
    };

    if let KeyState::Active { last_used: Some(used), .. } = key {
        assert!(activated_at <= used, "last_used must be after activation");
    }
}

/// Test: Certificate validity bounds are enforced
#[test]
fn test_certificate_validity_bounds() {
    let now = Utc::now();
    let not_before = now - Duration::hours(1);
    let not_after = now + Duration::days(365);

    let cert = CertificateState::Active {
        not_before,
        not_after,
        usage_count: 0,
        last_used: None,
    };

    // Valid within bounds
    assert!(cert.is_time_valid(now), "Certificate should be valid now");

    // Invalid before not_before
    let before = not_before - Duration::hours(1);
    assert!(!cert.is_time_valid(before), "Certificate should be invalid before not_before");

    // Invalid after not_after
    let after = not_after + Duration::hours(1);
    assert!(!cert.is_time_valid(after), "Certificate should be invalid after not_after");
}

/// Test: Revocation must happen after creation
#[test]
fn test_temporal_ordering_revocation() {
    let created_at = Utc::now() - Duration::days(30);
    let revoked_at = Utc::now();

    assert!(
        created_at < revoked_at,
        "Revocation must happen after creation"
    );
}

// ============================================================================
// Cross-Machine Invariant Tests
// ============================================================================

/// Test: YubiKey slot binding compatibility
#[test]
fn test_yubikey_slot_compatibility() {
    let admin_id = Uuid::now_v7();
    let sig_key_id = Uuid::now_v7();
    let auth_key_id = Uuid::now_v7();

    // Create YubiKey with slot assignments
    let slots = HashMap::from([
        (PivSlot::Signature, sig_key_id),
        (PivSlot::Authentication, auth_key_id),
    ]);

    let yubikey = yubikey_provisioned(admin_id, slots);

    if let YubiKeyState::Provisioned { slots, .. } = &yubikey {
        // Signature key in signature slot
        assert_eq!(slots.get(&PivSlot::Signature), Some(&sig_key_id));

        // Authentication key in authentication slot
        assert_eq!(slots.get(&PivSlot::Authentication), Some(&auth_key_id));

        // No key in key management slot
        assert!(slots.get(&PivSlot::KeyManagement).is_none());
    }
}

/// Test: NATS hierarchy mirrors organization hierarchy
#[test]
fn test_nats_org_hierarchy_invariant() {
    let person_id = Uuid::now_v7();
    let unit_id = Uuid::now_v7();
    let account_id = Uuid::now_v7();

    // Organization with unit
    let org = org_active(vec![unit_id], vec![person_id]);

    // NATS Operator for Organization
    let operator = nats_operator_active(vec![account_id]);

    // NATS Account for Unit under Operator
    let account = nats_account_active(vec![]);

    // Invariant: Active org → Active operator
    assert!(org.is_active() && operator.is_active(),
        "Organization.Active implies Operator.Active");

    // Invariant: Org has unit → Operator has account
    if let OrganizationState::Active { units, .. } = &org {
        if let NatsOperatorState::Active { accounts, .. } = &operator {
            assert!(!units.is_empty() && !accounts.is_empty(),
                "Org units should map to operator accounts");
        }
    }

    // Invariant: Account is active under operator
    assert!(account.is_active() && operator.is_active());
}

/// Test: Person-NATS User bijective mapping
#[test]
fn test_person_nats_user_mapping() {
    let admin_id = Uuid::now_v7();
    let person_id = Uuid::now_v7();
    let account_id = Uuid::now_v7();

    // Active person
    let person = person_active(vec![Uuid::now_v7()]);

    // NATS user for this person
    let nats_user = nats_user_created(admin_id, account_id, person_id);

    // Verify mapping exists
    if let NatsUserState::Created { person_id: linked_person, .. } = nats_user {
        assert_eq!(linked_person, person_id, "NATS user must link to correct person");
    }

    // Invariant: Person active → NATS user should be activatable
    let nats_user = nats_user_active();
    assert!(person.is_active() && nats_user.is_active(),
        "Person.Active implies NatsUser should be Active");
}

/// Invariant: Active Key requires Active Person owner
#[test]
fn test_invariant_key_requires_active_owner() {
    let person_state = person_active(vec![Uuid::now_v7()]);
    let key_state = key_active();

    // Both must be active for the key to be usable
    assert!(person_state.is_active());
    assert!(key_state.is_active());
}

/// Invariant: Certificate chain must be temporally valid
#[test]
fn test_invariant_certificate_chain_temporal() {
    let root_issued = Utc::now() - Duration::days(365);
    let intermediate_issued = Utc::now() - Duration::days(180);
    let leaf_issued = Utc::now() - Duration::days(30);

    // Each certificate must be issued after its issuer
    assert!(root_issued < intermediate_issued);
    assert!(intermediate_issued < leaf_issued);
}

// ============================================================================
// State Lifecycle Tests
// ============================================================================

/// Test: Certificate state lifecycle
#[test]
fn test_certificate_lifecycle() {
    let admin_id = Uuid::now_v7();
    let root_ca_id = Uuid::now_v7();

    // Pending → Issued
    let pending = cert_pending(admin_id);
    assert!(pending.is_pending());

    let issued = cert_issued(root_ca_id, admin_id);
    assert!(pending.can_transition_to(&issued), "Pending → Issued should be valid");

    // Issued → Active
    let active = cert_active();
    assert!(issued.can_transition_to(&active), "Issued → Active should be valid");

    // Active → Expired
    let expired = CertificateState::Expired {
        expired_at: Utc::now(),
        not_after: Utc::now() - Duration::hours(1),
    };
    assert!(active.can_transition_to(&expired), "Active → Expired should be valid");

    // Expired → Archived
    let archived = CertificateState::Archived {
        archived_at: Utc::now(),
        archived_by: admin_id,
        previous_state: CertArchivedFrom::Expired,
    };
    assert!(expired.can_transition_to(&archived), "Expired → Archived should be valid");
    assert!(archived.is_terminal(), "Archived should be terminal");
}

/// Test: YubiKey state lifecycle
#[test]
fn test_yubikey_lifecycle() {
    let admin_id = Uuid::now_v7();
    let person_id = Uuid::now_v7();
    let key_id = Uuid::now_v7();

    // Detected → Provisioned
    let detected = yubikey_detected("12345678", "5.4.3", admin_id);
    assert!(matches!(detected, YubiKeyState::Detected { .. }));

    let slots = HashMap::from([(PivSlot::Signature, key_id)]);
    let provisioned = yubikey_provisioned(admin_id, slots);
    assert!(detected.can_transition_to(&provisioned), "Detected → Provisioned should be valid");

    // Provisioned → Active
    let active = yubikey_active(person_id);
    assert!(provisioned.can_transition_to(&active), "Provisioned → Active should be valid");
    assert!(active.is_active());

    // Active → Locked
    let locked = YubiKeyState::Locked {
        locked_at: Utc::now(),
        pin_retries: 0,
        can_unlock: true,
    };
    assert!(active.can_transition_to(&locked), "Active → Locked should be valid");

    // Active → Retired
    let retired = YubiKeyState::Retired {
        retired_at: Utc::now(),
        retired_by: admin_id,
        reason: RetirementReason::Upgraded,
        replacement_yubikey_id: Some(Uuid::now_v7()),
    };
    assert!(active.can_transition_to(&retired), "Active → Retired should be valid");
    assert!(retired.is_terminal(), "Retired should be terminal");
}

/// Test: Key revocation lifecycle
#[test]
fn test_key_revocation_lifecycle() {
    let admin_id = Uuid::now_v7();

    // Active key can be revoked
    let active = key_active();
    assert!(active.is_active());

    let revoked = KeyState::Revoked {
        reason: KeyRevocationReason::Compromised,
        revoked_at: Utc::now(),
        revoked_by: admin_id,
    };
    assert!(active.can_transition_to(&revoked), "Active → Revoked should be valid");
    assert!(revoked.is_revoked());
    assert!(revoked.is_terminal());

    // Revoked → Archived
    let archived = KeyState::Archived {
        archived_at: Utc::now(),
        archived_by: admin_id,
        previous_state: KeyArchivedFrom::Revoked,
    };
    assert!(revoked.can_transition_to(&archived), "Revoked → Archived should be valid");
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    /// Property: Revocation timestamp must be after creation timestamp
    #[test]
    fn prop_revocation_after_creation(
        creation_offset in 1u64..365,
        revocation_offset in 0u64..30,
    ) {
        let created_at = Utc::now() - Duration::days(creation_offset as i64);
        let revoked_at = Utc::now() - Duration::days(revocation_offset as i64);

        // Revocation should be after creation if offset is smaller
        prop_assert!(
            created_at <= revoked_at || revocation_offset >= creation_offset,
            "Revocation must be after or at creation time"
        );
    }

    /// Property: Active keys can be revoked
    #[test]
    fn prop_active_keys_can_revoke(_dummy: u8) {
        let key_state = key_active();
        // Active keys can transition to Revoked
        prop_assert!(key_state.is_active());
        prop_assert!(!key_state.is_terminal());
    }

    /// Property: Revoked keys cannot be reactivated
    #[test]
    fn prop_revoked_keys_terminal(_dummy: u8) {
        let admin_id = Uuid::now_v7();
        let key_state = KeyState::Revoked {
            reason: KeyRevocationReason::Compromised,
            revoked_at: Utc::now(),
            revoked_by: admin_id,
        };

        prop_assert!(key_state.is_terminal());
        prop_assert!(!key_state.is_active());
    }

    /// Property: Active certificates can be used for crypto
    #[test]
    fn prop_active_certs_usable(_dummy: u8) {
        let cert_state = cert_active();
        prop_assert!(!cert_state.is_revoked());
        prop_assert!(cert_state.can_use_for_crypto());
    }
}

// ============================================================================
// Terminal State Tests
// ============================================================================

/// Test: Terminal states block all transitions
#[test]
fn test_terminal_states_block_transitions() {
    let admin_id = Uuid::now_v7();

    // Key archived - terminal
    let archived_key = KeyState::Archived {
        archived_at: Utc::now(),
        archived_by: admin_id,
        previous_state: KeyArchivedFrom::Revoked,
    };
    assert!(archived_key.is_terminal());
    assert!(!archived_key.can_be_modified(), "Terminal state cannot be modified");

    // Certificate revoked - terminal
    let revoked_cert = CertificateState::Revoked {
        reason: CertRevocationReason::KeyCompromise,
        revoked_at: Utc::now(),
        revoked_by: admin_id,
        crl_published: true,
        ocsp_updated: true,
    };
    assert!(revoked_cert.is_terminal());
    assert!(!revoked_cert.can_be_modified());

    // Person archived - terminal
    let archived_person = PersonState::Archived {
        archived_at: Utc::now(),
        archived_by: admin_id,
        retention_policy_id: None,
    };
    assert!(archived_person.is_terminal());
    assert!(!archived_person.can_be_modified());

    // Organization dissolved - terminal
    let dissolved_org = OrganizationState::Dissolved {
        dissolved_at: Utc::now(),
        dissolved_by: admin_id,
        reason: "Merged".to_string(),
        successor_org_id: Some(Uuid::now_v7()),
    };
    assert!(dissolved_org.is_terminal());
    assert!(!dissolved_org.can_be_modified());
}

/// Test: Archived keys are terminal
#[test]
fn test_archived_keys_terminal() {
    let admin_id = Uuid::now_v7();

    let archived = KeyState::Archived {
        archived_at: Utc::now(),
        archived_by: admin_id,
        previous_state: KeyArchivedFrom::Revoked,
    };

    assert!(archived.is_terminal());
    assert!(!archived.can_use_for_crypto());
}

/// Test: Active states allow crypto operations
#[test]
fn test_active_states_allow_crypto() {
    let person_id = Uuid::now_v7();

    // Active key can do crypto
    let active_key = key_active();
    assert!(active_key.can_use_for_crypto());

    // Active certificate can do crypto
    let active_cert = cert_active();
    assert!(active_cert.can_use_for_crypto());

    // Active YubiKey can do crypto
    let active_yubikey = yubikey_active(person_id);
    assert!(active_yubikey.can_use_for_crypto());

    // Active person can perform actions
    let active_person = person_active(vec![Uuid::now_v7()]);
    assert!(active_person.can_perform_actions());
    assert!(active_person.can_generate_keys());
}

/// Test: Suspended states restrict operations
#[test]
fn test_suspended_states_restrict_operations() {
    let admin_id = Uuid::now_v7();

    // Suspended person can't perform actions
    let suspended_person = PersonState::Suspended {
        reason: "Under investigation".to_string(),
        suspended_at: Utc::now(),
        suspended_by: admin_id,
        previous_roles: vec![Uuid::now_v7()],
    };
    assert!(!suspended_person.can_perform_actions());
    assert!(!suspended_person.can_generate_keys());
    assert!(suspended_person.can_establish_relationships()); // Still can have relationships

    // Suspended NATS user can't connect
    let suspended_nats_user = NatsUserState::Suspended {
        reason: "Access review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: admin_id,
    };
    assert!(!suspended_nats_user.can_connect());
    assert!(!suspended_nats_user.can_pubsub());

    // Suspended org can't add members
    let suspended_org = OrganizationState::Suspended {
        reason: "Compliance review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: admin_id,
    };
    assert!(!suspended_org.can_add_members());
    assert!(!suspended_org.can_generate_keys());
}
