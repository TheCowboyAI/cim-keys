//! Backend integration test for GUI workflow
//!
//! Tests the backend workflow that the GUI uses for CLAN bootstrap credential generation,
//! without requiring the full GUI to be running.

use cim_keys::clan_bootstrap::ClanBootstrapLoader;
use cim_keys::commands::nsc_export::generate_and_export_credentials;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_gui_backend_workflow() {
    // This simulates the workflow that happens when a user:
    // 1. Clicks "Load CLAN Bootstrap"
    // 2. Clicks "Generate Credentials"

    println!("\n🧪 Testing GUI backend workflow...\n");

    // Step 1: Load CLAN bootstrap (simulates Message::LoadClanBootstrap)
    println!("📋 Step 1: Loading clan-bootstrap.json...");
    let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json")
        .expect("Failed to load clan-bootstrap.json");

    let (org, units, people) = ClanBootstrapLoader::to_domain_models(config)
        .expect("Failed to convert to domain models");

    println!("   ✅ Loaded: {} ({} units, {} services)", org.name, units.len(), people.len());
    assert_eq!(org.name, "thecowboyai");
    assert_eq!(units.len(), 3);
    assert_eq!(people.len(), 11);

    // Step 2: Generate credentials (simulates Message::GenerateClanCredentials)
    println!("\n🔐 Step 2: Generating NATS credentials...");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let result = generate_and_export_credentials(
        temp_dir.path(),
        org.clone(),
        units.clone(),
        people.clone(),
    );

    assert!(
        result.is_ok(),
        "Failed to generate credentials: {:?}",
        result.err()
    );

    let export_result = result.unwrap();

    println!("   ✅ Generated {} accounts and {} users",
             export_result.accounts_exported,
             export_result.users_exported);
    println!("   📂 NSC store: {}", export_result.nsc_store_path.display());
    println!("   💾 Total bytes: {}", export_result.total_bytes_written);

    // Step 3: Verify the results (what the GUI would display)
    assert_eq!(export_result.accounts_exported, 3, "Expected 3 accounts");
    assert_eq!(export_result.users_exported, 11, "Expected 11 users");
    assert!(export_result.total_bytes_written > 0, "Expected bytes written");

    // Step 4: Verify NSC directory structure exists
    let nsc_store = &export_result.nsc_store_path;
    assert!(nsc_store.exists(), "NSC store should exist");

    println!("\n🔍 Step 3: Verifying NSC structure...");

    // Verify operator JWT
    let operator_jwt = nsc_store.join(format!("{}.jwt", org.name));
    assert!(operator_jwt.exists(), "Operator JWT should exist");
    println!("   ✅ Operator JWT: {}", operator_jwt.display());

    // Verify accounts
    for unit in &units {
        let account_name = unit.nats_account_name.as_ref().unwrap();
        let account_jwt = nsc_store
            .join("accounts")
            .join(account_name)
            .join(format!("{}.jwt", account_name));
        assert!(account_jwt.exists(), "Account JWT for {} should exist", account_name);
        println!("   ✅ Account JWT: {}", account_name);
    }

    // Verify users
    for person in &people {
        let unit_id = person.unit_ids.first().unwrap();
        let unit = units.iter().find(|u| &u.id == unit_id).unwrap();
        let account_name = unit.nats_account_name.as_ref().unwrap();

        let user_creds = nsc_store
            .join("accounts")
            .join(account_name)
            .join("users")
            .join(format!("{}.creds", person.name));

        assert!(user_creds.exists(), "User creds for {} should exist", person.name);

        // Verify creds file format
        let creds_content = fs::read_to_string(&user_creds)
            .expect("Failed to read user credentials");
        assert!(creds_content.contains("-----BEGIN NATS USER JWT-----"));
        assert!(creds_content.contains("-----BEGIN USER NKEY SEED-----"));
    }

    println!("   ✅ All {} user credentials verified", people.len());

    // Step 5: Simulate what GUI would display to user
    println!("\n📊 GUI would display:");
    println!("   Status: ✅ Credentials generated successfully");
    println!("   Accounts: {}", export_result.accounts_exported);
    println!("   Users: {}", export_result.users_exported);
    println!("   NSC Store: {}", export_result.nsc_store_path.display());
    println!("   Bytes Written: {}", export_result.total_bytes_written);

    println!("\n✅ GUI backend workflow test PASSED\n");
}

#[test]
fn test_gui_error_handling() {
    // Test what happens when user tries to load non-existent file
    println!("\n🧪 Testing GUI error handling...\n");

    println!("📋 Testing: Load non-existent file...");
    let result = ClanBootstrapLoader::load_from_file("non-existent.json");
    assert!(result.is_err(), "Should fail to load non-existent file");
    println!("   ✅ Correctly returns error: {:?}", result.err().unwrap());

    // Test what happens when credentials are generated to invalid path
    println!("\n🔐 Testing: Generate to invalid path...");
    let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json")
        .expect("Failed to load clan-bootstrap.json");
    let (org, units, people) = ClanBootstrapLoader::to_domain_models(config)
        .expect("Failed to convert to domain models");

    // Try to write to /invalid/path (should fail)
    let result = generate_and_export_credentials(
        std::path::Path::new("/invalid/path/that/does/not/exist"),
        org,
        units,
        people,
    );

    assert!(result.is_err(), "Should fail with invalid path");
    println!("   ✅ Correctly returns error: {}", result.err().unwrap());

    println!("\n✅ GUI error handling test PASSED\n");
}

#[test]
fn test_gui_state_transitions() {
    // Test the state transitions that happen in the GUI
    println!("\n🧪 Testing GUI state transitions...\n");

    // Simulate GUI state
    struct GuiState {
        clan_bootstrap_loaded: bool,
        clan_credentials_generated: bool,
        clan_accounts_exported: usize,
        clan_users_exported: usize,
    }

    let mut state = GuiState {
        clan_bootstrap_loaded: false,
        clan_credentials_generated: false,
        clan_accounts_exported: 0,
        clan_users_exported: 0,
    };

    println!("📊 Initial state: loaded={}, generated={}",
             state.clan_bootstrap_loaded,
             state.clan_credentials_generated);

    // State transition 1: Load bootstrap
    println!("\n📋 State transition: LoadClanBootstrap -> ClanBootstrapLoaded");
    let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json")
        .expect("Failed to load");
    let (org, units, people) = ClanBootstrapLoader::to_domain_models(config)
        .expect("Failed to convert");

    state.clan_bootstrap_loaded = true;
    println!("   ✅ State: loaded={}, generated={}",
             state.clan_bootstrap_loaded,
             state.clan_credentials_generated);

    // State transition 2: Generate credentials
    println!("\n🔐 State transition: GenerateClanCredentials -> ClanCredentialsGenerated");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let result = generate_and_export_credentials(temp_dir.path(), org, units, people)
        .expect("Failed to generate");

    state.clan_credentials_generated = true;
    state.clan_accounts_exported = result.accounts_exported;
    state.clan_users_exported = result.users_exported;

    println!("   ✅ State: loaded={}, generated={}, accounts={}, users={}",
             state.clan_bootstrap_loaded,
             state.clan_credentials_generated,
             state.clan_accounts_exported,
             state.clan_users_exported);

    // Verify final state
    assert!(state.clan_bootstrap_loaded);
    assert!(state.clan_credentials_generated);
    assert_eq!(state.clan_accounts_exported, 3);
    assert_eq!(state.clan_users_exported, 11);

    println!("\n✅ GUI state transitions test PASSED\n");
}
