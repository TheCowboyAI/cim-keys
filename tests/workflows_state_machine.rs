//! Comprehensive PKI Bootstrap Workflow State Machine Tests
//!
//! Target: 95%+ coverage of src/state_machines/workflows.rs

use chrono::Utc;
use cim_keys::state_machines::workflows::{
    CertificateSubject, PKIBootstrapState, PivSlot, PinPolicy, TouchPolicy, PivAlgorithm,
};
use cim_keys::events::{KeyAlgorithm, KeyPurpose};
use uuid::Uuid;

// Test Helpers
fn test_subject() -> CertificateSubject {
    CertificateSubject {
        common_name: "Test CA".to_string(),
        organization: "Test Org".to_string(),
        organizational_unit: Some("Test Unit".to_string()),
        country: "US".to_string(),
        state: Some("CA".to_string()),
        locality: Some("San Francisco".to_string()),
        email: Some("admin@test.com".to_string()),
    }
}

// State Helpers
fn uninitialized_state() -> PKIBootstrapState {
    PKIBootstrapState::Uninitialized
}

fn root_ca_planned_state() -> PKIBootstrapState {
    PKIBootstrapState::RootCAPlanned {
        subject: test_subject(),
        validity_years: 10,
        yubikey_serial: "12345678".to_string(),
    }
}

fn root_ca_generated_state() -> PKIBootstrapState {
    PKIBootstrapState::RootCAGenerated {
        root_ca_cert_id: Uuid::now_v7(),
        root_ca_key_id: Uuid::now_v7(),
        generated_at: Utc::now(),
    }
}

fn intermediate_ca_planned_state() -> PKIBootstrapState {
    PKIBootstrapState::IntermediateCAPlanned {
        subject: test_subject(),
        validity_years: 5,
        path_len: Some(0),
    }
}

fn intermediate_ca_generated_state() -> PKIBootstrapState {
    PKIBootstrapState::IntermediateCAGenerated {
        intermediate_ca_ids: vec![Uuid::now_v7(), Uuid::now_v7()],
    }
}

fn leaf_certs_generated_state() -> PKIBootstrapState {
    PKIBootstrapState::LeafCertsGenerated {
        leaf_cert_ids: vec![Uuid::now_v7()],
    }
}

fn yubikeys_provisioned_state() -> PKIBootstrapState {
    PKIBootstrapState::YubiKeysProvisioned {
        yubikey_serials: vec!["12345678".to_string(), "87654321".to_string()],
    }
}

fn export_ready_state() -> PKIBootstrapState {
    PKIBootstrapState::ExportReady {
        manifest_id: Uuid::now_v7(),
    }
}

fn bootstrapped_state() -> PKIBootstrapState {
    PKIBootstrapState::Bootstrapped {
        export_location: Uuid::now_v7(),
        export_checksum: "sha256:abc123".to_string(),
        bootstrapped_at: Utc::now(),
    }
}

// Basic State Tests
#[test]
fn test_uninitialized_state() {
    let state = uninitialized_state();
    assert!(matches!(state, PKIBootstrapState::Uninitialized));
}

#[test]
fn test_root_ca_planned_state() {
    let state = root_ca_planned_state();
    assert!(matches!(state, PKIBootstrapState::RootCAPlanned { .. }));
}

#[test]
fn test_root_ca_generated_state() {
    let state = root_ca_generated_state();
    assert!(matches!(state, PKIBootstrapState::RootCAGenerated { .. }));
}

// State Permission Tests - can_plan_root_ca
#[test]
fn test_can_plan_root_ca() {
    assert!(uninitialized_state().can_plan_root_ca());
    assert!(!root_ca_planned_state().can_plan_root_ca());
    assert!(!root_ca_generated_state().can_plan_root_ca());
    assert!(!intermediate_ca_planned_state().can_plan_root_ca());
    assert!(!intermediate_ca_generated_state().can_plan_root_ca());
    assert!(!leaf_certs_generated_state().can_plan_root_ca());
    assert!(!yubikeys_provisioned_state().can_plan_root_ca());
    assert!(!export_ready_state().can_plan_root_ca());
    assert!(!bootstrapped_state().can_plan_root_ca());
}

// State Permission Tests - can_generate_root_ca
#[test]
fn test_can_generate_root_ca() {
    assert!(!uninitialized_state().can_generate_root_ca());
    assert!(root_ca_planned_state().can_generate_root_ca());
    assert!(!root_ca_generated_state().can_generate_root_ca());
    assert!(!intermediate_ca_planned_state().can_generate_root_ca());
    assert!(!intermediate_ca_generated_state().can_generate_root_ca());
    assert!(!leaf_certs_generated_state().can_generate_root_ca());
    assert!(!yubikeys_provisioned_state().can_generate_root_ca());
    assert!(!export_ready_state().can_generate_root_ca());
    assert!(!bootstrapped_state().can_generate_root_ca());
}

// State Permission Tests - can_plan_intermediate_ca
#[test]
fn test_can_plan_intermediate_ca() {
    assert!(!uninitialized_state().can_plan_intermediate_ca());
    assert!(!root_ca_planned_state().can_plan_intermediate_ca());
    assert!(root_ca_generated_state().can_plan_intermediate_ca());
    assert!(intermediate_ca_planned_state().can_plan_intermediate_ca());
    assert!(!intermediate_ca_generated_state().can_plan_intermediate_ca());
    assert!(!leaf_certs_generated_state().can_plan_intermediate_ca());
    assert!(!yubikeys_provisioned_state().can_plan_intermediate_ca());
    assert!(!export_ready_state().can_plan_intermediate_ca());
    assert!(!bootstrapped_state().can_plan_intermediate_ca());
}

// State Permission Tests - can_generate_intermediate_ca
#[test]
fn test_can_generate_intermediate_ca() {
    assert!(!uninitialized_state().can_generate_intermediate_ca());
    assert!(!root_ca_planned_state().can_generate_intermediate_ca());
    assert!(root_ca_generated_state().can_generate_intermediate_ca());
    assert!(intermediate_ca_planned_state().can_generate_intermediate_ca());
    assert!(!intermediate_ca_generated_state().can_generate_intermediate_ca());
    assert!(!leaf_certs_generated_state().can_generate_intermediate_ca());
    assert!(!yubikeys_provisioned_state().can_generate_intermediate_ca());
    assert!(!export_ready_state().can_generate_intermediate_ca());
    assert!(!bootstrapped_state().can_generate_intermediate_ca());
}

// State Permission Tests - can_generate_leaf_cert
#[test]
fn test_can_generate_leaf_cert() {
    assert!(!uninitialized_state().can_generate_leaf_cert());
    assert!(!root_ca_planned_state().can_generate_leaf_cert());
    assert!(!root_ca_generated_state().can_generate_leaf_cert());
    assert!(!intermediate_ca_planned_state().can_generate_leaf_cert());
    assert!(intermediate_ca_generated_state().can_generate_leaf_cert());
    assert!(leaf_certs_generated_state().can_generate_leaf_cert());
    assert!(!yubikeys_provisioned_state().can_generate_leaf_cert());
    assert!(!export_ready_state().can_generate_leaf_cert());
    assert!(!bootstrapped_state().can_generate_leaf_cert());
}

// State Permission Tests - can_provision_yubikey
#[test]
fn test_can_provision_yubikey() {
    assert!(!uninitialized_state().can_provision_yubikey());
    assert!(!root_ca_planned_state().can_provision_yubikey());
    assert!(!root_ca_generated_state().can_provision_yubikey());
    assert!(!intermediate_ca_planned_state().can_provision_yubikey());
    assert!(!intermediate_ca_generated_state().can_provision_yubikey());
    assert!(leaf_certs_generated_state().can_provision_yubikey());
    assert!(yubikeys_provisioned_state().can_provision_yubikey());
    assert!(!export_ready_state().can_provision_yubikey());
    assert!(!bootstrapped_state().can_provision_yubikey());
}

// State Permission Tests - can_prepare_export
#[test]
fn test_can_prepare_export() {
    assert!(!uninitialized_state().can_prepare_export());
    assert!(!root_ca_planned_state().can_prepare_export());
    assert!(!root_ca_generated_state().can_prepare_export());
    assert!(!intermediate_ca_planned_state().can_prepare_export());
    assert!(!intermediate_ca_generated_state().can_prepare_export());
    assert!(!leaf_certs_generated_state().can_prepare_export());
    assert!(yubikeys_provisioned_state().can_prepare_export());
    assert!(!export_ready_state().can_prepare_export());
    assert!(!bootstrapped_state().can_prepare_export());
}

// State Permission Tests - can_export
#[test]
fn test_can_export() {
    assert!(!uninitialized_state().can_export());
    assert!(!root_ca_planned_state().can_export());
    assert!(!root_ca_generated_state().can_export());
    assert!(!intermediate_ca_planned_state().can_export());
    assert!(!intermediate_ca_generated_state().can_export());
    assert!(!leaf_certs_generated_state().can_export());
    assert!(!yubikeys_provisioned_state().can_export());
    assert!(export_ready_state().can_export());
    assert!(!bootstrapped_state().can_export());
}

// Description Tests
#[test]
fn test_description_for_all_states() {
    assert_eq!(uninitialized_state().description(), "PKI infrastructure not initialized");
    assert_eq!(root_ca_planned_state().description(), "Root CA planned, awaiting generation");
    assert_eq!(root_ca_generated_state().description(), "Root CA generated (offline ceremony complete)");
    assert_eq!(intermediate_ca_planned_state().description(), "Intermediate CA planned");
    assert_eq!(intermediate_ca_generated_state().description(), "Intermediate CA(s) generated");
    assert_eq!(leaf_certs_generated_state().description(), "Leaf certificates generated");
    assert_eq!(yubikeys_provisioned_state().description(), "YubiKeys provisioned");
    assert_eq!(export_ready_state().description(), "Export manifest ready");
    assert_eq!(bootstrapped_state().description(), "Bootstrap complete");
}

// CertificateSubject Tests
#[test]
fn test_certificate_subject_structure() {
    let subject = test_subject();
    assert_eq!(subject.common_name, "Test CA");
    assert_eq!(subject.organization, "Test Org");
    assert_eq!(subject.country, "US");
    assert_eq!(subject.organizational_unit, Some("Test Unit".to_string()));
    assert_eq!(subject.state, Some("CA".to_string()));
    assert_eq!(subject.locality, Some("San Francisco".to_string()));
    assert_eq!(subject.email, Some("admin@test.com".to_string()));
}

#[test]
fn test_certificate_subject_minimal() {
    let subject = CertificateSubject {
        common_name: "Minimal CA".to_string(),
        organization: "Test".to_string(),
        organizational_unit: None,
        country: "US".to_string(),
        state: None,
        locality: None,
        email: None,
    };
    assert_eq!(subject.common_name, "Minimal CA");
    assert_eq!(subject.organizational_unit, None);
}

// PivSlot Tests
#[test]
fn test_piv_slot_hex() {
    assert_eq!(PivSlot::Authentication.hex(), "9a");
    assert_eq!(PivSlot::Signature.hex(), "9c");
    assert_eq!(PivSlot::KeyManagement.hex(), "9d");
    assert_eq!(PivSlot::CardAuth.hex(), "9e");
    assert_eq!(PivSlot::Retired(0).hex(), "82");
    assert_eq!(PivSlot::Retired(1).hex(), "83");
    assert_eq!(PivSlot::Retired(19).hex(), "95"); // Max retired slot
}

// Workflow Lifecycle Test
#[test]
fn test_complete_pki_bootstrap_workflow() {
    // Start uninitialized
    let state = uninitialized_state();
    assert!(state.can_plan_root_ca());

    // Plan root CA
    let state = root_ca_planned_state();
    assert!(state.can_generate_root_ca());

    // Generate root CA
    let state = root_ca_generated_state();
    assert!(state.can_plan_intermediate_ca());
    assert!(state.can_generate_intermediate_ca());

    // Plan intermediate CA
    let state = intermediate_ca_planned_state();
    assert!(state.can_plan_intermediate_ca());
    assert!(state.can_generate_intermediate_ca());

    // Generate intermediate CA
    let state = intermediate_ca_generated_state();
    assert!(state.can_generate_leaf_cert());

    // Generate leaf certs
    let state = leaf_certs_generated_state();
    assert!(state.can_provision_yubikey());

    // Provision YubiKeys
    let state = yubikeys_provisioned_state();
    assert!(state.can_prepare_export());

    // Prepare export
    let state = export_ready_state();
    assert!(state.can_export());

    // Complete bootstrap
    let state = bootstrapped_state();
    assert!(matches!(state, PKIBootstrapState::Bootstrapped { .. }));
}

// Serialization Tests
#[test]
fn test_serde_roundtrip_all_states() {
    let states = vec![
        uninitialized_state(),
        root_ca_planned_state(),
        root_ca_generated_state(),
        intermediate_ca_planned_state(),
        intermediate_ca_generated_state(),
        leaf_certs_generated_state(),
        yubikeys_provisioned_state(),
        export_ready_state(),
        bootstrapped_state(),
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: PKIBootstrapState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}

// Edge Cases
#[test]
fn test_multiple_intermediate_cas() {
    let state = PKIBootstrapState::IntermediateCAGenerated {
        intermediate_ca_ids: vec![Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7()],
    };
    assert!(state.can_generate_leaf_cert());
}

#[test]
fn test_multiple_leaf_certs() {
    let state = PKIBootstrapState::LeafCertsGenerated {
        leaf_cert_ids: vec![Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7()],
    };
    assert!(state.can_provision_yubikey());
}

#[test]
fn test_multiple_yubikeys() {
    let serials = vec![
        "11111111".to_string(),
        "22222222".to_string(),
        "33333333".to_string(),
        "44444444".to_string(),
        "55555555".to_string(),
    ];
    let state = PKIBootstrapState::YubiKeysProvisioned {
        yubikey_serials: serials.clone(),
    };
    assert!(state.can_prepare_export());
}

// ============================================================================
// YubiKey Provisioning Workflow Tests
// ============================================================================

use cim_keys::state_machines::workflows::{YubiKeyProvisioningState, ExportWorkflowState, ArtifactType};

// YubiKey State Helpers
fn yubikey_detected() -> YubiKeyProvisioningState {
    YubiKeyProvisioningState::Detected {
        serial: "12345678".to_string(),
        firmware_version: "5.4.3".to_string(),
    }
}

fn yubikey_authenticated() -> YubiKeyProvisioningState {
    YubiKeyProvisioningState::Authenticated {
        pin_retries_remaining: 3,
    }
}

fn yubikey_pin_changed() -> YubiKeyProvisioningState {
    YubiKeyProvisioningState::PINChanged {
        new_pin_hash: "sha256:newhash".to_string(),
    }
}

fn yubikey_management_key_rotated() -> YubiKeyProvisioningState {
    YubiKeyProvisioningState::ManagementKeyRotated {
        algorithm: PivAlgorithm::EcdsaP256,
    }
}

fn yubikey_slot_planned() -> YubiKeyProvisioningState {
    use cim_keys::state_machines::workflows::SlotPlan;
    use std::collections::HashMap;
    let mut slot_plan = HashMap::new();
    slot_plan.insert(PivSlot::Authentication, SlotPlan {
        purpose: KeyPurpose::Authentication,
        algorithm: KeyAlgorithm::Ed25519,
        pin_policy: PinPolicy::Once,
        touch_policy: TouchPolicy::Never,
    });
    YubiKeyProvisioningState::SlotPlanned { slot_plan }
}

fn yubikey_keys_generated() -> YubiKeyProvisioningState {
    use std::collections::HashMap;
    let mut slot_keys = HashMap::new();
    slot_keys.insert(PivSlot::Authentication, vec![1, 2, 3, 4]);
    YubiKeyProvisioningState::KeysGenerated { slot_keys }
}

fn yubikey_certs_imported() -> YubiKeyProvisioningState {
    use std::collections::HashMap;
    let mut slot_certs = HashMap::new();
    slot_certs.insert(PivSlot::Authentication, Uuid::now_v7());
    YubiKeyProvisioningState::CertificatesImported { slot_certs }
}

fn yubikey_attested() -> YubiKeyProvisioningState {
    YubiKeyProvisioningState::Attested {
        attestation_chain_verified: true,
        attestation_cert_ids: vec![Uuid::now_v7()],
    }
}

fn yubikey_sealed() -> YubiKeyProvisioningState {
    YubiKeyProvisioningState::Sealed {
        sealed_at: Utc::now(),
        final_config_hash: "sha256:abc123".to_string(),
    }
}

#[test]
fn test_yubikey_can_authenticate() {
    assert!(yubikey_detected().can_authenticate());
    assert!(!yubikey_authenticated().can_authenticate());
    assert!(!yubikey_sealed().can_authenticate());
}

#[test]
fn test_yubikey_can_change_pin() {
    assert!(!yubikey_detected().can_change_pin());
    assert!(yubikey_authenticated().can_change_pin());
}

#[test]
fn test_yubikey_is_sealed() {
    assert!(!yubikey_detected().is_sealed());
    assert!(!yubikey_authenticated().is_sealed());
    assert!(yubikey_sealed().is_sealed());
}

#[test]
fn test_yubikey_descriptions() {
    assert_eq!(yubikey_detected().description(), "YubiKey detected");
    assert_eq!(yubikey_authenticated().description(), "Authenticated with PIN");
    assert_eq!(yubikey_pin_changed().description(), "PIN changed");
    assert_eq!(yubikey_management_key_rotated().description(), "Management key rotated");
    assert_eq!(yubikey_slot_planned().description(), "Slot allocation planned");
    assert_eq!(yubikey_keys_generated().description(), "Keys generated in slots");
    assert_eq!(yubikey_certs_imported().description(), "Certificates imported");
    assert_eq!(yubikey_attested().description(), "Keys attested");
    assert_eq!(yubikey_sealed().description(), "Configuration sealed");
}

#[test]
fn test_yubikey_can_rotate_management_key() {
    assert!(!yubikey_detected().can_rotate_management_key());
    assert!(!yubikey_authenticated().can_rotate_management_key());
    assert!(yubikey_pin_changed().can_rotate_management_key());
    assert!(!yubikey_sealed().can_rotate_management_key());
}

#[test]
fn test_yubikey_can_plan_slots() {
    assert!(!yubikey_detected().can_plan_slots());
    assert!(!yubikey_pin_changed().can_plan_slots());
    assert!(yubikey_management_key_rotated().can_plan_slots());
    assert!(!yubikey_sealed().can_plan_slots());
}

#[test]
fn test_yubikey_can_generate_keys() {
    assert!(!yubikey_detected().can_generate_keys());
    assert!(!yubikey_management_key_rotated().can_generate_keys());
    assert!(yubikey_slot_planned().can_generate_keys());
    assert!(!yubikey_sealed().can_generate_keys());
}

#[test]
fn test_yubikey_can_import_certs() {
    assert!(!yubikey_detected().can_import_certs());
    assert!(!yubikey_slot_planned().can_import_certs());
    assert!(yubikey_keys_generated().can_import_certs());
    assert!(!yubikey_sealed().can_import_certs());
}

#[test]
fn test_yubikey_can_attest() {
    assert!(!yubikey_detected().can_attest());
    assert!(!yubikey_keys_generated().can_attest());
    assert!(yubikey_certs_imported().can_attest());
    assert!(!yubikey_sealed().can_attest());
}

#[test]
fn test_yubikey_can_seal() {
    assert!(!yubikey_detected().can_seal());
    assert!(!yubikey_certs_imported().can_seal());
    assert!(yubikey_attested().can_seal());
    assert!(!yubikey_sealed().can_seal());
}

// ============================================================================
// Export Workflow Tests
// ============================================================================

fn export_planning() -> ExportWorkflowState {
    ExportWorkflowState::Planning {
        artifacts_to_export: vec![ArtifactType::RootCACertificate],
        target_location: Uuid::now_v7(),
    }
}

// Skip Generating state for now due to type complexity - covered by other workflow tests

fn export_encrypting() -> ExportWorkflowState {
    ExportWorkflowState::Encrypting {
        encryption_key_id: Uuid::now_v7(),
        progress_percent: 50,
    }
}

fn export_writing() -> ExportWorkflowState {
    ExportWorkflowState::Writing {
        bytes_written: 1024,
        total_bytes: 2048,
    }
}

fn export_verifying() -> ExportWorkflowState {
    use std::collections::HashMap;
    let mut checksums = HashMap::new();
    checksums.insert("manifest.json".to_string(), "sha256:abc123".to_string());
    ExportWorkflowState::Verifying { checksums }
}

fn export_completed() -> ExportWorkflowState {
    ExportWorkflowState::Completed {
        manifest_checksum: "sha256:def456".to_string(),
        exported_at: Utc::now(),
    }
}

fn export_failed() -> ExportWorkflowState {
    ExportWorkflowState::Failed {
        error: "Disk full".to_string(),
        failed_at: Utc::now(),
    }
}

#[test]
fn test_export_can_generate() {
    assert!(export_planning().can_generate());
    assert!(!export_completed().can_generate());
}

#[test]
fn test_export_is_complete() {
    assert!(!export_planning().is_complete());
    assert!(export_completed().is_complete());
}

#[test]
fn test_export_has_failed() {
    assert!(!export_planning().has_failed());
    assert!(!export_completed().has_failed());
    assert!(export_failed().has_failed());
}

#[test]
fn test_export_descriptions() {
    assert_eq!(export_planning().description(), "Planning export");
    assert_eq!(export_encrypting().description(), "Encrypting sensitive data");
    assert_eq!(export_writing().description(), "Writing to target location");
    assert_eq!(export_verifying().description(), "Verifying checksums");
    assert_eq!(export_completed().description(), "Export complete");
    assert_eq!(export_failed().description(), "Export failed");
}

#[test]
fn test_export_can_write() {
    assert!(!export_planning().can_write());
    assert!(export_encrypting().can_write());
    assert!(!export_completed().can_write());
}

#[test]
fn test_export_can_verify() {
    assert!(!export_planning().can_verify());
    assert!(!export_encrypting().can_verify());
    assert!(export_writing().can_verify());
    assert!(!export_completed().can_verify());
}
