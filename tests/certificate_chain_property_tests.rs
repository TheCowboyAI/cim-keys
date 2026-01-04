// Copyright (c) 2025 - Cowboy AI, LLC.

//! Property-Based Tests for Certificate Chain Verification
//!
//! These tests use proptest to verify invariants and laws of the
//! certificate chain verification system.

use chrono::{DateTime, Duration, Utc};
use proptest::prelude::*;
use cim_keys::value_objects::{
    Certificate, CertificateChain, CertificateSubject, CertificateVerificationError,
    PublicKey, PublicKeyFormat, Signature, SignatureAlgorithm, TrustLevel, TrustPath,
    Validity, ValueObjectError,
};
use cim_keys::events::KeyAlgorithm;

// ============================================================================
// Arbitrary Generators
// ============================================================================

/// Generate a random common name
fn arb_common_name() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9-]{2,20}(\\.[a-z]{2,6})?")
        .unwrap()
        .prop_map(|s| if s.is_empty() { "default".to_string() } else { s })
}

/// Generate a valid validity period
fn arb_validity() -> impl Strategy<Value = Validity> {
    (
        // Days ago for not_before (0 to 365)
        0i64..365,
        // Days ahead for not_after (1 to 730)
        1i64..730,
    ).prop_map(|(before_days, after_days)| {
        let now = Utc::now();
        Validity {
            not_before: now - Duration::days(before_days),
            not_after: now + Duration::days(after_days),
        }
    })
}

/// Generate an expired validity period
fn arb_expired_validity() -> impl Strategy<Value = Validity> {
    (
        // Days ago for not_before (30 to 365)
        30i64..365,
        // Days ago for not_after (1 to 29)
        1i64..29,
    ).prop_map(|(before_days, after_days)| {
        let now = Utc::now();
        Validity {
            not_before: now - Duration::days(before_days),
            not_after: now - Duration::days(after_days),
        }
    })
}

/// Generate a not-yet-valid validity period
fn arb_future_validity() -> impl Strategy<Value = Validity> {
    (
        // Days ahead for not_before (1 to 30)
        1i64..30,
        // Days ahead for not_after (31 to 365)
        31i64..365,
    ).prop_map(|(before_days, after_days)| {
        let now = Utc::now();
        Validity {
            not_before: now + Duration::days(before_days),
            not_after: now + Duration::days(after_days),
        }
    })
}

/// Generate a certificate subject
fn arb_certificate_subject() -> impl Strategy<Value = CertificateSubject> {
    arb_common_name().prop_map(|cn| CertificateSubject {
        common_name: cn,
        organization: Some("Test Organization".to_string()),
        organizational_unit: None,
        country: Some("US".to_string()),
        state: None,
        locality: None,
        email: None,
    })
}

/// Generate a mock public key
fn arb_public_key() -> impl Strategy<Value = PublicKey> {
    prop::collection::vec(any::<u8>(), 32..33).prop_map(|data| PublicKey {
        algorithm: KeyAlgorithm::Ed25519,
        data,
        format: PublicKeyFormat::Der,
    })
}

/// Generate a mock signature
fn arb_signature() -> impl Strategy<Value = Signature> {
    prop::collection::vec(any::<u8>(), 64..65).prop_map(|data| Signature {
        algorithm: SignatureAlgorithm::Ed25519,
        data,
    })
}

/// Generate a mock certificate with given subject and issuer
fn arb_certificate_with_names(
    subject_cn: String,
    issuer_cn: String,
    validity: Validity,
) -> Certificate {
    Certificate {
        serial_number: format!("SN-{}", uuid::Uuid::now_v7()),
        subject: CertificateSubject {
            common_name: subject_cn,
            organization: Some("Test Org".to_string()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        },
        issuer: CertificateSubject {
            common_name: issuer_cn,
            organization: Some("Test Org".to_string()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            email: None,
        },
        public_key: PublicKey {
            algorithm: KeyAlgorithm::Ed25519,
            data: vec![0u8; 32],
            format: PublicKeyFormat::Der,
        },
        validity,
        signature: Signature {
            algorithm: SignatureAlgorithm::Ed25519,
            data: vec![0u8; 64],
        },
        der: vec![0u8; 100],
        pem: String::new(),
    }
}

// ============================================================================
// Property Tests: Temporal Validity
// ============================================================================

proptest! {
    /// Property: A certificate with not_before in the past and not_after in the future
    /// should always be temporally valid at the current time.
    #[test]
    fn prop_valid_certificate_passes_temporal_check(
        before_days in 1i64..365,
        after_days in 1i64..365,
    ) {
        let now = Utc::now();
        let cert = arb_certificate_with_names(
            "test.example.com".to_string(),
            "CA".to_string(),
            Validity {
                not_before: now - Duration::days(before_days),
                not_after: now + Duration::days(after_days),
            },
        );

        let result = cert.verify_temporal_validity(now);
        prop_assert!(result.is_ok(), "Valid certificate should pass temporal check");
    }

    /// Property: A certificate that has expired should always fail temporal validation
    /// with an Expired error.
    #[test]
    fn prop_expired_certificate_fails_temporal_check(
        expired_days in 1i64..365,
    ) {
        let now = Utc::now();
        let cert = arb_certificate_with_names(
            "expired.example.com".to_string(),
            "CA".to_string(),
            Validity {
                not_before: now - Duration::days(expired_days + 30),
                not_after: now - Duration::days(expired_days),
            },
        );

        let result = cert.verify_temporal_validity(now);
        prop_assert!(result.is_err(), "Expired certificate should fail temporal check");
        match result {
            Err(CertificateVerificationError::Expired { .. }) => (),
            Err(e) => prop_assert!(false, "Expected Expired error, got: {:?}", e),
            _ => unreachable!(),
        }
    }

    /// Property: A certificate that is not yet valid should always fail temporal
    /// validation with a NotYetValid error.
    #[test]
    fn prop_future_certificate_fails_temporal_check(
        future_days in 1i64..365,
    ) {
        let now = Utc::now();
        let cert = arb_certificate_with_names(
            "future.example.com".to_string(),
            "CA".to_string(),
            Validity {
                not_before: now + Duration::days(future_days),
                not_after: now + Duration::days(future_days + 365),
            },
        );

        let result = cert.verify_temporal_validity(now);
        prop_assert!(result.is_err(), "Future certificate should fail temporal check");
        match result {
            Err(CertificateVerificationError::NotYetValid { .. }) => (),
            Err(e) => prop_assert!(false, "Expected NotYetValid error, got: {:?}", e),
            _ => unreachable!(),
        }
    }

    /// Property: Temporal validity check is monotonic - if valid at time T,
    /// also valid at T - epsilon (within the not_before bound).
    #[test]
    fn prop_temporal_validity_is_monotonic_backward(
        valid_days in 10i64..100,
        check_offset in 1i64..9,
    ) {
        let now = Utc::now();
        let cert = arb_certificate_with_names(
            "test.example.com".to_string(),
            "CA".to_string(),
            Validity {
                not_before: now - Duration::days(valid_days),
                not_after: now + Duration::days(365),
            },
        );

        // If valid at now, should also be valid at (now - check_offset) if within bounds
        let t1 = now;
        let t2 = now - Duration::days(check_offset);

        let result_t1 = cert.verify_temporal_validity(t1);
        let result_t2 = cert.verify_temporal_validity(t2);

        // If t2 is still after not_before, both should succeed
        if t2 >= cert.validity.not_before {
            prop_assert!(result_t1.is_ok() && result_t2.is_ok());
        }
    }
}

// ============================================================================
// Property Tests: Issuer Chain
// ============================================================================

proptest! {
    /// Property: Issuer verification is reflexive - a self-signed certificate's
    /// issuer should match itself.
    #[test]
    fn prop_self_signed_issuer_matches_self(
        cn in arb_common_name(),
    ) {
        let now = Utc::now();
        let cert = arb_certificate_with_names(
            cn.clone(),
            cn.clone(),
            Validity {
                not_before: now - Duration::days(30),
                not_after: now + Duration::days(365),
            },
        );

        // A self-signed cert should match itself as issuer
        let result = cert.verify_issuer_matches(&cert);
        prop_assert!(result.is_ok(), "Self-signed cert should match itself as issuer");
    }

    /// Property: Issuer verification should fail when CNs don't match.
    #[test]
    fn prop_mismatched_issuer_fails(
        subject_cn in arb_common_name(),
        issuer_cn in arb_common_name(),
        wrong_ca_cn in arb_common_name(),
    ) {
        prop_assume!(issuer_cn != wrong_ca_cn);

        let now = Utc::now();
        let cert = arb_certificate_with_names(
            subject_cn,
            issuer_cn.clone(),
            Validity {
                not_before: now - Duration::days(30),
                not_after: now + Duration::days(365),
            },
        );

        let wrong_ca = arb_certificate_with_names(
            wrong_ca_cn.clone(),
            wrong_ca_cn,
            Validity {
                not_before: now - Duration::days(365),
                not_after: now + Duration::days(365),
            },
        );

        let result = cert.verify_issuer_matches(&wrong_ca);
        prop_assert!(result.is_err(), "Mismatched issuer should fail");
    }
}

// ============================================================================
// Property Tests: Trust Path
// ============================================================================

proptest! {
    /// Property: Trust path length equals number of add_link calls.
    #[test]
    fn prop_trust_path_length_correct(
        num_links in 1usize..10,
    ) {
        let mut path = TrustPath::new();
        prop_assert!(path.is_empty());

        for i in 0..num_links {
            let fp = format!("cert-{}", i);
            let issuer_fp = if i == 0 { None } else { Some(format!("cert-{}", i - 1)) };
            path.add_link(fp, issuer_fp, TrustLevel::Complete);
        }

        prop_assert_eq!(path.len(), num_links);
    }

    /// Property: Empty trust path has length 0 and is_empty true.
    #[test]
    fn prop_empty_trust_path_invariants(_: ()) {
        let path = TrustPath::new();
        prop_assert!(path.is_empty());
        prop_assert_eq!(path.len(), 0);
    }
}

// ============================================================================
// Property Tests: Certificate Chain Structure
// ============================================================================

proptest! {
    /// Property: Chain depth = 2 + number of intermediates
    #[test]
    fn prop_chain_depth_formula(
        num_intermediates in 0usize..5,
    ) {
        let now = Utc::now();
        let validity = Validity {
            not_before: now - Duration::days(30),
            not_after: now + Duration::days(365),
        };

        let root = arb_certificate_with_names(
            "Root CA".to_string(),
            "Root CA".to_string(),
            validity.clone(),
        );

        let mut prev_cn = "Root CA".to_string();
        let mut intermediates = Vec::new();

        for i in 0..num_intermediates {
            let int_cn = format!("Intermediate CA {}", i);
            intermediates.push(arb_certificate_with_names(
                int_cn.clone(),
                prev_cn.clone(),
                validity.clone(),
            ));
            prev_cn = int_cn;
        }

        let leaf = arb_certificate_with_names(
            "leaf.example.com".to_string(),
            prev_cn,
            validity,
        );

        let chain = CertificateChain::new(leaf, intermediates, root);
        prop_assert_eq!(chain.depth(), 2 + num_intermediates);
    }

    /// Property: all_certificates returns leaf first, root last
    #[test]
    fn prop_all_certificates_order(
        num_intermediates in 0usize..3,
    ) {
        let now = Utc::now();
        let validity = Validity {
            not_before: now - Duration::days(30),
            not_after: now + Duration::days(365),
        };

        let root = arb_certificate_with_names(
            "Root CA".to_string(),
            "Root CA".to_string(),
            validity.clone(),
        );

        let mut prev_cn = "Root CA".to_string();
        let mut intermediates = Vec::new();

        for i in 0..num_intermediates {
            let int_cn = format!("Int-{}", i);
            intermediates.push(arb_certificate_with_names(
                int_cn.clone(),
                prev_cn.clone(),
                validity.clone(),
            ));
            prev_cn = int_cn;
        }

        let leaf = arb_certificate_with_names(
            "Leaf".to_string(),
            prev_cn,
            validity,
        );

        let chain = CertificateChain::new(leaf, intermediates, root);
        let all = chain.all_certificates();

        // First should be leaf
        prop_assert_eq!(&all[0].subject.common_name, "Leaf");
        // Last should be root
        prop_assert_eq!(&all[all.len() - 1].subject.common_name, "Root CA");
    }
}

// ============================================================================
// Property Tests: Fingerprint Determinism
// ============================================================================

proptest! {
    /// Property: Certificate fingerprint is deterministic (same input -> same output)
    #[test]
    fn prop_fingerprint_deterministic(
        der_data in prop::collection::vec(any::<u8>(), 50..200),
    ) {
        let cert = Certificate {
            serial_number: "SN-1".to_string(),
            subject: CertificateSubject {
                common_name: "test".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            issuer: CertificateSubject {
                common_name: "test".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            public_key: PublicKey {
                algorithm: KeyAlgorithm::Ed25519,
                data: vec![0u8; 32],
                format: PublicKeyFormat::Der,
            },
            validity: Validity {
                not_before: Utc::now() - Duration::days(1),
                not_after: Utc::now() + Duration::days(1),
            },
            signature: Signature {
                algorithm: SignatureAlgorithm::Ed25519,
                data: vec![0u8; 64],
            },
            der: der_data,
            pem: String::new(),
        };

        let fp1 = cert.fingerprint();
        let fp2 = cert.fingerprint();
        prop_assert_eq!(fp1, fp2, "Fingerprint should be deterministic");
    }

    /// Property: Fingerprint is exactly 64 hex characters (SHA-256)
    #[test]
    fn prop_fingerprint_length(
        der_data in prop::collection::vec(any::<u8>(), 1..500),
    ) {
        let cert = Certificate {
            serial_number: "SN-1".to_string(),
            subject: CertificateSubject {
                common_name: "test".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            issuer: CertificateSubject {
                common_name: "test".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            public_key: PublicKey {
                algorithm: KeyAlgorithm::Ed25519,
                data: vec![0u8; 32],
                format: PublicKeyFormat::Der,
            },
            validity: Validity {
                not_before: Utc::now() - Duration::days(1),
                not_after: Utc::now() + Duration::days(1),
            },
            signature: Signature {
                algorithm: SignatureAlgorithm::Ed25519,
                data: vec![0u8; 64],
            },
            der: der_data,
            pem: String::new(),
        };

        let fp = cert.fingerprint();
        prop_assert_eq!(fp.len(), 64, "SHA-256 fingerprint should be 64 hex chars");
    }

    /// Property: Different DER data produces different fingerprints (high probability)
    #[test]
    fn prop_different_der_different_fingerprint(
        der1 in prop::collection::vec(any::<u8>(), 50..100),
        der2 in prop::collection::vec(any::<u8>(), 50..100),
    ) {
        prop_assume!(der1 != der2);

        let make_cert = |der: Vec<u8>| Certificate {
            serial_number: "SN-1".to_string(),
            subject: CertificateSubject {
                common_name: "test".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            issuer: CertificateSubject {
                common_name: "test".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            public_key: PublicKey {
                algorithm: KeyAlgorithm::Ed25519,
                data: vec![0u8; 32],
                format: PublicKeyFormat::Der,
            },
            validity: Validity {
                not_before: Utc::now() - Duration::days(1),
                not_after: Utc::now() + Duration::days(1),
            },
            signature: Signature {
                algorithm: SignatureAlgorithm::Ed25519,
                data: vec![0u8; 64],
            },
            der,
            pem: String::new(),
        };

        let cert1 = make_cert(der1);
        let cert2 = make_cert(der2);

        prop_assert_ne!(cert1.fingerprint(), cert2.fingerprint(),
            "Different DER should produce different fingerprints");
    }
}

// ============================================================================
// Property Tests: Validity Period
// ============================================================================

proptest! {
    /// Property: Validity duration in days is non-negative
    #[test]
    fn prop_validity_duration_non_negative(
        before_days in 0i64..365,
        duration_days in 1i64..3650,
    ) {
        let now = Utc::now();
        let validity = Validity {
            not_before: now - Duration::days(before_days),
            not_after: now - Duration::days(before_days) + Duration::days(duration_days),
        };

        prop_assert!(validity.duration_days() >= 0, "Duration should be non-negative");
        prop_assert_eq!(validity.duration_days(), duration_days);
    }

    /// Property: is_valid_at is consistent with not_before and not_after bounds
    #[test]
    fn prop_is_valid_at_respects_bounds(
        before_days in 1i64..100,
        after_days in 1i64..100,
        check_offset in -150i64..150,
    ) {
        let now = Utc::now();
        let validity = Validity {
            not_before: now - Duration::days(before_days),
            not_after: now + Duration::days(after_days),
        };

        let check_time = now + Duration::days(check_offset);
        let is_valid = validity.is_valid_at(check_time);

        let expected_valid = check_time >= validity.not_before && check_time <= validity.not_after;
        prop_assert_eq!(is_valid, expected_valid);
    }
}
