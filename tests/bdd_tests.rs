// Copyright (c) 2025 - Cowboy AI, LLC.
//! BDD Test Suite for cim-keys
//!
//! This test module provides BDD-style tests that correspond to the
//! Gherkin feature files in doc/qa/features/*.feature
//!
//! Feature files covered:
//! - domain_bootstrap.feature
//! - person_management.feature
//! - key_generation.feature
//! - yubikey_provisioning.feature (scenarios only)
//! - nats_security_bootstrap.feature (scenarios only)
//! - export_manifest.feature
//!
//! Run with: cargo test --test bdd_tests --features gui

mod bdd;

// Re-export step definition tests
pub use bdd::domain_bootstrap_steps;
pub use bdd::person_management_steps;
pub use bdd::key_generation_steps;
pub use bdd::export_manifest_steps;

/// Summary of all BDD scenarios
///
/// This function provides a summary of all feature files and their scenarios.
/// Run with `cargo test bdd_summary -- --nocapture` to see the full output.
#[test]
fn bdd_summary() {
    eprintln!("\n╔══════════════════════════════════════════════════════════════╗");
    eprintln!("║           CIM-KEYS BDD SPECIFICATION SUMMARY                 ║");
    eprintln!("╠══════════════════════════════════════════════════════════════╣");
    eprintln!("║                                                              ║");
    eprintln!("║  Feature Files:                                              ║");
    eprintln!("║  ├── domain_bootstrap.feature      (15 scenarios)            ║");
    eprintln!("║  ├── person_management.feature     (18 scenarios)            ║");
    eprintln!("║  ├── key_generation.feature        (20 scenarios)            ║");
    eprintln!("║  ├── yubikey_provisioning.feature  (22 scenarios)            ║");
    eprintln!("║  ├── nats_security_bootstrap.feature (19 scenarios)          ║");
    eprintln!("║  └── export_manifest.feature       (18 scenarios)            ║");
    eprintln!("║                                                              ║");
    eprintln!("║  Total Scenarios: 112                                        ║");
    eprintln!("║  Implemented Step Definitions: 4 modules                     ║");
    eprintln!("║  Executable Tests: 15                                        ║");
    eprintln!("║                                                              ║");
    eprintln!("╚══════════════════════════════════════════════════════════════╝\n");
}

/// Feature: Domain Bootstrap
/// Tests for initializing CIM domains from configuration
#[cfg(test)]
mod domain_bootstrap_feature {
    use super::bdd::domain_bootstrap_steps::tests::*;

    // Tests are defined in domain_bootstrap_steps.rs
    // They are automatically discovered by the test runner
}

/// Feature: Person Management
/// Tests for managing people within organizations
#[cfg(test)]
mod person_management_feature {
    use super::bdd::person_management_steps::tests::*;

    // Tests are defined in person_management_steps.rs
}

/// Feature: Key Generation
/// Tests for cryptographic key generation
#[cfg(test)]
mod key_generation_feature {
    use super::bdd::key_generation_steps::tests::*;

    // Tests are defined in key_generation_steps.rs
}

/// Feature: Export Manifest
/// Tests for domain export and manifest generation
#[cfg(test)]
mod export_manifest_feature {
    use super::bdd::export_manifest_steps::tests::*;

    // Tests are defined in export_manifest_steps.rs
}

/// Integration test: Complete domain workflow
/// This test exercises multiple features in sequence
#[tokio::test]
async fn integration_complete_domain_workflow() {
    use bdd::domain_bootstrap_steps::*;
    use bdd::person_management_steps::*;
    use bdd::key_generation_steps::*;
    use bdd::export_manifest_steps::*;

    eprintln!("\n══════════════════════════════════════════════════════════════");
    eprintln!("  INTEGRATION: Complete Domain Workflow");
    eprintln!("══════════════════════════════════════════════════════════════\n");

    // Phase 1: Bootstrap Domain
    eprintln!("  Phase 1: Domain Bootstrap");
    let mut ctx = given_clean_cim_environment();

    let create_org = given_bootstrap_config_with_organization(&mut ctx, "IntegrationOrg");
    let command = cim_keys::commands::KeyCommand::CreateOrganization(create_org);
    let result = when_execute_bootstrap(&mut ctx, command).await;
    assert!(result.is_ok(), "Organization creation should succeed");
    eprintln!("    ✓ Organization created");

    // Phase 2: Add People
    eprintln!("  Phase 2: Person Management");
    given_units_exist(&mut ctx, &[("Engineering", "Department")]);

    let create_person = when_create_person(
        &mut ctx,
        "Integration User",
        "user@integration.com",
        "Developer",
    );
    let command = cim_keys::commands::KeyCommand::CreatePerson(create_person);
    let result = when_execute_bootstrap(&mut ctx, command).await;
    assert!(result.is_ok(), "Person creation should succeed");
    eprintln!("    ✓ Person created");

    // Phase 3: Generate Keys
    eprintln!("  Phase 3: Key Generation");
    let root_key = when_generate_root_ca(&mut ctx, "IntegrationOrg");
    assert!(!root_key.is_nil());
    eprintln!("    ✓ Root CA generated");

    let personal_key = when_generate_personal_key(
        &mut ctx,
        "Integration User",
        "authentication",
    );
    assert!(!personal_key.is_nil());
    eprintln!("    ✓ Personal key generated");

    // Phase 4: Export Domain
    eprintln!("  Phase 4: Export Manifest");
    let output_path = given_encrypted_partition_mounted(&ctx);
    let exported = when_export_domain(&mut ctx, &output_path);
    assert!(exported, "Export should succeed");
    eprintln!("    ✓ Domain exported");

    // Verify export
    assert!(then_manifest_created(&output_path));
    assert!(then_directory_structure_exists(&output_path));
    assert!(then_verification_passes(&output_path));
    eprintln!("    ✓ Export verified");

    eprintln!("\n══════════════════════════════════════════════════════════════");
    eprintln!("  ✓ INTEGRATION TEST PASSED");
    eprintln!("══════════════════════════════════════════════════════════════\n");
}
