//! Comprehensive Certificate State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/certificate.rs
//!
//! Tests all:
//! - Valid state transition validations
//! - Invalid state transition rejections
//! - State query methods
//! - Time-based validity checks
//! - Revocation reasons
//! - Archived state tracking
//! - Serialization

use chrono::{Duration, Utc};
use cim_keys::state_machines::certificate::{ArchivedFromState, CertificateState, RevocationReason};
use uuid::Uuid;

// ============================================================================
// Test Helpers
// ============================================================================

fn test_person_id() -> Uuid {
    Uuid::now_v7()
}

fn test_ca_id() -> Uuid {
    Uuid::now_v7()
}

fn pending_state() -> CertificateState {
    CertificateState::Pending {
        csr_id: Some(Uuid::now_v7()),
        pending_since: Utc::now(),
        requested_by: test_person_id(),
    }
}

fn issued_state() -> CertificateState {
    CertificateState::Issued {
        issued_at: Utc::now(),
        issuer_id: test_ca_id(),
        issued_by: test_person_id(),
    }
}

fn active_state() -> CertificateState {
    let now = Utc::now();
    CertificateState::Active {
        not_before: now - Duration::days(1),
        not_after: now + Duration::days(365),
        usage_count: 0,
        last_used: None,
    }
}

fn renewal_pending_state() -> CertificateState {
    CertificateState::RenewalPending {
        new_cert_id: Uuid::now_v7(),
        initiated_at: Utc::now(),
        initiated_by: test_person_id(),
    }
}

fn renewed_state() -> CertificateState {
    CertificateState::Renewed {
        new_cert_id: Uuid::now_v7(),
        renewed_at: Utc::now(),
        renewed_by: test_person_id(),
    }
}

fn revoked_state() -> CertificateState {
    CertificateState::Revoked {
        reason: RevocationReason::KeyCompromise,
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        crl_published: true,
        ocsp_updated: true,
    }
}

fn expired_state() -> CertificateState {
    let now = Utc::now();
    CertificateState::Expired {
        expired_at: now,
        not_after: now - Duration::days(1),
    }
}

fn archived_state(from: ArchivedFromState) -> CertificateState {
    CertificateState::Archived {
        archived_at: Utc::now(),
        archived_by: test_person_id(),
        previous_state: from,
    }
}

// ============================================================================
// State Query Tests
// ============================================================================

#[test]
fn test_is_active_for_all_states() {
    assert!(!pending_state().is_active());
    assert!(!issued_state().is_active());
    assert!(active_state().is_active());
    assert!(!renewal_pending_state().is_active());
    assert!(!renewed_state().is_active());
    assert!(!revoked_state().is_active());
    assert!(!expired_state().is_active());
    assert!(!archived_state(ArchivedFromState::Revoked).is_active());
}

#[test]
fn test_can_use_for_crypto_for_all_states() {
    assert!(!pending_state().can_use_for_crypto());
    assert!(!issued_state().can_use_for_crypto());
    assert!(active_state().can_use_for_crypto());
    assert!(!renewal_pending_state().can_use_for_crypto());
    assert!(!renewed_state().can_use_for_crypto());
    assert!(!revoked_state().can_use_for_crypto());
    assert!(!expired_state().can_use_for_crypto());
    assert!(!archived_state(ArchivedFromState::Revoked).can_use_for_crypto());
}

#[test]
fn test_can_be_modified_for_all_states() {
    assert!(pending_state().can_be_modified());
    assert!(issued_state().can_be_modified());
    assert!(active_state().can_be_modified());
    assert!(renewal_pending_state().can_be_modified());
    assert!(renewed_state().can_be_modified());
    assert!(!revoked_state().can_be_modified());
    assert!(expired_state().can_be_modified());
    assert!(!archived_state(ArchivedFromState::Revoked).can_be_modified());
}

#[test]
fn test_is_terminal_for_all_states() {
    assert!(!pending_state().is_terminal());
    assert!(!issued_state().is_terminal());
    assert!(!active_state().is_terminal());
    assert!(!renewal_pending_state().is_terminal());
    assert!(!renewed_state().is_terminal());
    assert!(revoked_state().is_terminal());
    assert!(!expired_state().is_terminal());
    assert!(archived_state(ArchivedFromState::Revoked).is_terminal());
}

#[test]
fn test_is_renewal_pending_for_all_states() {
    assert!(!pending_state().is_renewal_pending());
    assert!(!issued_state().is_renewal_pending());
    assert!(!active_state().is_renewal_pending());
    assert!(renewal_pending_state().is_renewal_pending());
    assert!(!renewed_state().is_renewal_pending());
    assert!(!revoked_state().is_renewal_pending());
    assert!(!expired_state().is_renewal_pending());
    assert!(!archived_state(ArchivedFromState::Revoked).is_renewal_pending());
}

#[test]
fn test_is_renewed_for_all_states() {
    assert!(!pending_state().is_renewed());
    assert!(!issued_state().is_renewed());
    assert!(!active_state().is_renewed());
    assert!(!renewal_pending_state().is_renewed());
    assert!(renewed_state().is_renewed());
    assert!(!revoked_state().is_renewed());
    assert!(!expired_state().is_renewed());
    assert!(!archived_state(ArchivedFromState::Revoked).is_renewed());
}

#[test]
fn test_is_expired_for_all_states() {
    assert!(!pending_state().is_expired());
    assert!(!issued_state().is_expired());
    assert!(!active_state().is_expired());
    assert!(!renewal_pending_state().is_expired());
    assert!(!renewed_state().is_expired());
    assert!(!revoked_state().is_expired());
    assert!(expired_state().is_expired());
    assert!(!archived_state(ArchivedFromState::Revoked).is_expired());
}

#[test]
fn test_is_revoked_for_all_states() {
    assert!(!pending_state().is_revoked());
    assert!(!issued_state().is_revoked());
    assert!(!active_state().is_revoked());
    assert!(!renewal_pending_state().is_revoked());
    assert!(!renewed_state().is_revoked());
    assert!(revoked_state().is_revoked());
    assert!(!expired_state().is_revoked());
    assert!(!archived_state(ArchivedFromState::Revoked).is_revoked());
}

#[test]
fn test_is_pending_for_all_states() {
    assert!(pending_state().is_pending());
    assert!(!issued_state().is_pending());
    assert!(!active_state().is_pending());
    assert!(!renewal_pending_state().is_pending());
    assert!(!renewed_state().is_pending());
    assert!(!revoked_state().is_pending());
    assert!(!expired_state().is_pending());
    assert!(!archived_state(ArchivedFromState::Revoked).is_pending());
}

// ============================================================================
// Transition Validation Tests
// ============================================================================

#[test]
fn test_can_transition_pending_to_issued() {
    let pending = pending_state();
    let issued = issued_state();
    assert!(pending.can_transition_to(&issued));
}

#[test]
fn test_can_transition_issued_to_active() {
    let issued = issued_state();
    let active = active_state();
    assert!(issued.can_transition_to(&active));
}

#[test]
fn test_can_transition_active_to_renewal_pending() {
    let active = active_state();
    let renewal = renewal_pending_state();
    assert!(active.can_transition_to(&renewal));
}

#[test]
fn test_can_transition_renewal_pending_to_renewed() {
    let renewal = renewal_pending_state();
    let renewed = renewed_state();
    assert!(renewal.can_transition_to(&renewed));
}

#[test]
fn test_can_transition_any_non_terminal_to_revoked() {
    let revoked = revoked_state();

    assert!(pending_state().can_transition_to(&revoked));
    assert!(issued_state().can_transition_to(&revoked));
    assert!(active_state().can_transition_to(&revoked));
    assert!(renewal_pending_state().can_transition_to(&revoked));
    assert!(renewed_state().can_transition_to(&revoked));
    assert!(expired_state().can_transition_to(&revoked));
}

#[test]
fn test_cannot_transition_terminal_to_revoked() {
    let revoked1 = revoked_state();
    let revoked2 = revoked_state();
    let archived = archived_state(ArchivedFromState::Revoked);

    assert!(!revoked1.can_transition_to(&revoked2));
    assert!(!archived.can_transition_to(&revoked2));
}

#[test]
fn test_can_transition_active_to_expired() {
    let active = active_state();
    let expired = expired_state();
    assert!(active.can_transition_to(&expired));
}

#[test]
fn test_can_transition_renewed_to_archived() {
    let renewed = renewed_state();
    let archived = archived_state(ArchivedFromState::Renewed);
    assert!(renewed.can_transition_to(&archived));
}

#[test]
fn test_can_transition_revoked_to_archived() {
    let revoked = revoked_state();
    let archived = archived_state(ArchivedFromState::Revoked);
    assert!(revoked.can_transition_to(&archived));
}

#[test]
fn test_can_transition_expired_to_archived() {
    let expired = expired_state();
    let archived = archived_state(ArchivedFromState::Expired);
    assert!(expired.can_transition_to(&archived));
}

#[test]
fn test_cannot_transition_pending_to_active() {
    let pending = pending_state();
    let active = active_state();
    assert!(!pending.can_transition_to(&active));
}

#[test]
fn test_cannot_transition_pending_to_renewal() {
    let pending = pending_state();
    let renewal = renewal_pending_state();
    assert!(!pending.can_transition_to(&renewal));
}

#[test]
fn test_cannot_transition_pending_to_renewed() {
    let pending = pending_state();
    let renewed = renewed_state();
    assert!(!pending.can_transition_to(&renewed));
}

#[test]
fn test_cannot_transition_pending_to_expired() {
    let pending = pending_state();
    let expired = expired_state();
    assert!(!pending.can_transition_to(&expired));
}

#[test]
fn test_cannot_transition_issued_to_renewal() {
    let issued = issued_state();
    let renewal = renewal_pending_state();
    assert!(!issued.can_transition_to(&renewal));
}

#[test]
fn test_cannot_transition_from_archived() {
    let archived = archived_state(ArchivedFromState::Revoked);

    assert!(!archived.can_transition_to(&pending_state()));
    assert!(!archived.can_transition_to(&issued_state()));
    assert!(!archived.can_transition_to(&active_state()));
    assert!(!archived.can_transition_to(&renewal_pending_state()));
    assert!(!archived.can_transition_to(&renewed_state()));
    assert!(!archived.can_transition_to(&revoked_state()));
    assert!(!archived.can_transition_to(&expired_state()));
    assert!(!archived.can_transition_to(&archived_state(ArchivedFromState::Expired)));
}

// ============================================================================
// Metadata Tests
// ============================================================================

#[test]
fn test_description_for_all_states() {
    assert_eq!(
        pending_state().description(),
        "Pending (awaiting CA signature)"
    );
    assert_eq!(
        issued_state().description(),
        "Issued (not yet valid)"
    );
    assert_eq!(
        active_state().description(),
        "Active (valid for use)"
    );
    assert_eq!(
        renewal_pending_state().description(),
        "Renewal Pending (new cert being issued)"
    );
    assert_eq!(
        renewed_state().description(),
        "Renewed (superseded by new certificate)"
    );
    assert_eq!(
        revoked_state().description(),
        "Revoked (TERMINAL - check CRL/OCSP)"
    );
    assert_eq!(
        expired_state().description(),
        "Expired (validity period ended)"
    );
    assert_eq!(
        archived_state(ArchivedFromState::Revoked).description(),
        "Archived (TERMINAL - long-term storage)"
    );
}

// ============================================================================
// Time Validation Tests
// ============================================================================

#[test]
fn test_is_time_valid_for_active_cert_in_validity_period() {
    let now = Utc::now();
    let cert = CertificateState::Active {
        not_before: now - Duration::days(1),
        not_after: now + Duration::days(365),
        usage_count: 0,
        last_used: None,
    };

    assert!(cert.is_time_valid(now));
}

#[test]
fn test_is_time_valid_at_not_before_boundary() {
    let not_before = Utc::now();
    let cert = CertificateState::Active {
        not_before,
        not_after: not_before + Duration::days(365),
        usage_count: 0,
        last_used: None,
    };

    assert!(cert.is_time_valid(not_before));
}

#[test]
fn test_is_time_valid_at_not_after_boundary() {
    let not_after = Utc::now();
    let cert = CertificateState::Active {
        not_before: not_after - Duration::days(365),
        not_after,
        usage_count: 0,
        last_used: None,
    };

    assert!(cert.is_time_valid(not_after));
}

#[test]
fn test_is_time_invalid_before_not_before() {
    let not_before = Utc::now() + Duration::days(1);
    let cert = CertificateState::Active {
        not_before,
        not_after: not_before + Duration::days(365),
        usage_count: 0,
        last_used: None,
    };

    let check_time = not_before - Duration::seconds(1);
    assert!(!cert.is_time_valid(check_time));
}

#[test]
fn test_is_time_invalid_after_not_after() {
    let not_after = Utc::now() - Duration::days(1);
    let cert = CertificateState::Active {
        not_before: not_after - Duration::days(365),
        not_after,
        usage_count: 0,
        last_used: None,
    };

    let check_time = not_after + Duration::seconds(1);
    assert!(!cert.is_time_valid(check_time));
}

#[test]
fn test_is_time_invalid_for_pending_cert() {
    let pending = pending_state();
    assert!(!pending.is_time_valid(Utc::now()));
}

#[test]
fn test_is_time_invalid_for_issued_cert() {
    let issued = issued_state();
    assert!(!issued.is_time_valid(Utc::now()));
}

#[test]
fn test_is_time_invalid_for_revoked_cert() {
    let revoked = revoked_state();
    assert!(!revoked.is_time_valid(Utc::now()));
}

#[test]
fn test_is_time_invalid_for_expired_cert() {
    let expired = expired_state();
    assert!(!expired.is_time_valid(Utc::now()));
}

#[test]
fn test_is_time_invalid_for_archived_cert() {
    let archived = archived_state(ArchivedFromState::Expired);
    assert!(!archived.is_time_valid(Utc::now()));
}

// ============================================================================
// Revocation Reason Tests
// ============================================================================

#[test]
fn test_all_revocation_reasons() {
    let reasons = vec![
        RevocationReason::Unspecified,
        RevocationReason::KeyCompromise,
        RevocationReason::CACompromise,
        RevocationReason::AffiliationChanged,
        RevocationReason::Superseded,
        RevocationReason::CessationOfOperation,
        RevocationReason::CertificateHold,
        RevocationReason::RemoveFromCRL,
        RevocationReason::PrivilegeWithdrawn,
        RevocationReason::AACompromise,
    ];

    for reason in reasons {
        let cert = CertificateState::Revoked {
            reason: reason.clone(),
            revoked_at: Utc::now(),
            revoked_by: test_person_id(),
            crl_published: true,
            ocsp_updated: true,
        };

        assert!(cert.is_revoked());
        assert!(cert.is_terminal());
    }
}

#[test]
fn test_revocation_with_crl_not_published() {
    let cert = CertificateState::Revoked {
        reason: RevocationReason::KeyCompromise,
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        crl_published: false,
        ocsp_updated: true,
    };

    match cert {
        CertificateState::Revoked { crl_published, .. } => {
            assert!(!crl_published);
        }
        _ => panic!("Expected Revoked state"),
    }
}

#[test]
fn test_revocation_with_ocsp_not_updated() {
    let cert = CertificateState::Revoked {
        reason: RevocationReason::KeyCompromise,
        revoked_at: Utc::now(),
        revoked_by: test_person_id(),
        crl_published: true,
        ocsp_updated: false,
    };

    match cert {
        CertificateState::Revoked { ocsp_updated, .. } => {
            assert!(!ocsp_updated);
        }
        _ => panic!("Expected Revoked state"),
    }
}

// ============================================================================
// Archived State Tests
// ============================================================================

#[test]
fn test_archived_from_renewed() {
    let cert = archived_state(ArchivedFromState::Renewed);

    match cert {
        CertificateState::Archived { previous_state, .. } => {
            assert_eq!(previous_state, ArchivedFromState::Renewed);
        }
        _ => panic!("Expected Archived state"),
    }
}

#[test]
fn test_archived_from_revoked() {
    let cert = archived_state(ArchivedFromState::Revoked);

    match cert {
        CertificateState::Archived { previous_state, .. } => {
            assert_eq!(previous_state, ArchivedFromState::Revoked);
        }
        _ => panic!("Expected Archived state"),
    }
}

#[test]
fn test_archived_from_expired() {
    let cert = archived_state(ArchivedFromState::Expired);

    match cert {
        CertificateState::Archived { previous_state, .. } => {
            assert_eq!(previous_state, ArchivedFromState::Expired);
        }
        _ => panic!("Expected Archived state"),
    }
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_serde_roundtrip_pending_state() {
    let pending = pending_state();
    let json = serde_json::to_string(&pending).unwrap();
    let deserialized: CertificateState = serde_json::from_str(&json).unwrap();
    assert_eq!(pending, deserialized);
}

#[test]
fn test_serde_roundtrip_issued_state() {
    let issued = issued_state();
    let json = serde_json::to_string(&issued).unwrap();
    let deserialized: CertificateState = serde_json::from_str(&json).unwrap();
    assert_eq!(issued, deserialized);
}

#[test]
fn test_serde_roundtrip_active_state() {
    let active = active_state();
    let json = serde_json::to_string(&active).unwrap();
    let deserialized: CertificateState = serde_json::from_str(&json).unwrap();
    assert_eq!(active, deserialized);
}

#[test]
fn test_serde_roundtrip_renewal_pending_state() {
    let renewal = renewal_pending_state();
    let json = serde_json::to_string(&renewal).unwrap();
    let deserialized: CertificateState = serde_json::from_str(&json).unwrap();
    assert_eq!(renewal, deserialized);
}

#[test]
fn test_serde_roundtrip_renewed_state() {
    let renewed = renewed_state();
    let json = serde_json::to_string(&renewed).unwrap();
    let deserialized: CertificateState = serde_json::from_str(&json).unwrap();
    assert_eq!(renewed, deserialized);
}

#[test]
fn test_serde_roundtrip_revoked_state() {
    let revoked = revoked_state();
    let json = serde_json::to_string(&revoked).unwrap();
    let deserialized: CertificateState = serde_json::from_str(&json).unwrap();
    assert_eq!(revoked, deserialized);
}

#[test]
fn test_serde_roundtrip_expired_state() {
    let expired = expired_state();
    let json = serde_json::to_string(&expired).unwrap();
    let deserialized: CertificateState = serde_json::from_str(&json).unwrap();
    assert_eq!(expired, deserialized);
}

#[test]
fn test_serde_roundtrip_archived_state() {
    let archived = archived_state(ArchivedFromState::Revoked);
    let json = serde_json::to_string(&archived).unwrap();
    let deserialized: CertificateState = serde_json::from_str(&json).unwrap();
    assert_eq!(archived, deserialized);
}

#[test]
fn test_serde_all_revocation_reasons() {
    let reasons = vec![
        RevocationReason::Unspecified,
        RevocationReason::KeyCompromise,
        RevocationReason::CACompromise,
        RevocationReason::AffiliationChanged,
        RevocationReason::Superseded,
        RevocationReason::CessationOfOperation,
        RevocationReason::CertificateHold,
        RevocationReason::RemoveFromCRL,
        RevocationReason::PrivilegeWithdrawn,
        RevocationReason::AACompromise,
    ];

    for reason in reasons {
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: RevocationReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }
}

#[test]
fn test_serde_all_archived_from_states() {
    let states = vec![
        ArchivedFromState::Renewed,
        ArchivedFromState::Revoked,
        ArchivedFromState::Expired,
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ArchivedFromState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_pending_without_csr_id() {
    let cert = CertificateState::Pending {
        csr_id: None,
        pending_since: Utc::now(),
        requested_by: test_person_id(),
    };

    assert!(cert.is_pending());
    match cert {
        CertificateState::Pending { csr_id, .. } => {
            assert_eq!(csr_id, None);
        }
        _ => panic!("Expected Pending state"),
    }
}

#[test]
fn test_active_with_usage_tracking() {
    let now = Utc::now();
    let last_used = now - Duration::hours(1);

    let cert = CertificateState::Active {
        not_before: now - Duration::days(1),
        not_after: now + Duration::days(365),
        usage_count: 42,
        last_used: Some(last_used),
    };

    assert!(cert.is_active());
    match cert {
        CertificateState::Active { usage_count, last_used: used, .. } => {
            assert_eq!(usage_count, 42);
            assert_eq!(used, Some(last_used));
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_active_never_used() {
    let cert = active_state();

    match cert {
        CertificateState::Active { usage_count, last_used, .. } => {
            assert_eq!(usage_count, 0);
            assert_eq!(last_used, None);
        }
        _ => panic!("Expected Active state"),
    }
}

#[test]
fn test_very_short_validity_period() {
    let now = Utc::now();
    let cert = CertificateState::Active {
        not_before: now,
        not_after: now + Duration::seconds(60), // 1 minute validity
        usage_count: 0,
        last_used: None,
    };

    assert!(cert.is_time_valid(now));
    assert!(cert.is_time_valid(now + Duration::seconds(30)));
    assert!(!cert.is_time_valid(now + Duration::seconds(61)));
}

#[test]
fn test_very_long_validity_period() {
    let now = Utc::now();
    let cert = CertificateState::Active {
        not_before: now,
        not_after: now + Duration::days(365 * 10), // 10 years
        usage_count: 0,
        last_used: None,
    };

    assert!(cert.is_time_valid(now));
    assert!(cert.is_time_valid(now + Duration::days(365 * 5))); // Halfway
}
