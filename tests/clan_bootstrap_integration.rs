//! Integration test for CLAN bootstrap workflow
//!
//! Tests the complete end-to-end workflow:
//! 1. Load clan-bootstrap.json configuration
//! 2. Convert to domain models
//! 3. Generate NATS credentials
//! 4. Export to NSC directory structure

use cim_keys::clan_bootstrap::ClanBootstrapLoader;
use cim_keys::commands::nsc_export::generate_and_export_credentials;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_clan_bootstrap_end_to_end() {
    // Step 1: Load clan-bootstrap.json
    let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json");
    assert!(
        config.is_ok(),
        "Failed to load clan-bootstrap.json: {:?}",
        config.err()
    );

    let config = config.unwrap();

    // Verify configuration structure
    assert_eq!(config.organization.name, "thecowboyai");
    assert_eq!(config.organizational_units.len(), 3, "Expected 3 organizational units");
    assert_eq!(config.service_people.len(), 11, "Expected 11 service people");

    // Verify organizational units
    let unit_names: Vec<&str> = config.organizational_units.iter()
        .map(|u| u.name.as_str())
        .collect();
    assert!(unit_names.contains(&"Core"), "Expected Core unit");
    assert!(unit_names.contains(&"Media"), "Expected Media unit");
    assert!(unit_names.contains(&"Development"), "Expected Development unit");

    // Step 2: Convert to domain models
    let result = ClanBootstrapLoader::to_domain_models(config);
    assert!(
        result.is_ok(),
        "Failed to convert to domain models: {:?}",
        result.err()
    );

    let (org, units, people) = result.unwrap();

    // Verify organization
    assert_eq!(org.name, "thecowboyai");
    assert_eq!(org.display_name, "The Cowboy AI");

    // Verify organizational units
    assert_eq!(units.len(), 3, "Expected 3 organizational units");
    for unit in &units {
        assert!(unit.nats_account_name.is_some(), "Each unit should have NATS account name");
    }

    // Verify people
    assert_eq!(people.len(), 11, "Expected 11 service people");
    for person in &people {
        assert!(person.nats_permissions.is_some(), "Each service person should have NATS permissions");

        let perms = person.nats_permissions.as_ref().unwrap();
        assert!(!perms.publish.is_empty(), "Each person should have publish permissions");
        assert!(!perms.subscribe.is_empty(), "Each person should have subscribe permissions");
    }

    // Verify Core unit services
    let core_unit = units.iter().find(|u| u.name == "Core").unwrap();
    let core_people: Vec<_> = people.iter()
        .filter(|p| p.unit_ids.contains(&core_unit.id))
        .collect();
    assert_eq!(core_people.len(), 5, "Expected 5 people in Core unit");

    // Verify Media unit services
    let media_unit = units.iter().find(|u| u.name == "Media").unwrap();
    let media_people: Vec<_> = people.iter()
        .filter(|p| p.unit_ids.contains(&media_unit.id))
        .collect();
    assert_eq!(media_people.len(), 3, "Expected 3 people in Media unit");

    // Verify Development unit services
    let dev_unit = units.iter().find(|u| u.name == "Development").unwrap();
    let dev_people: Vec<_> = people.iter()
        .filter(|p| p.unit_ids.contains(&dev_unit.id))
        .collect();
    assert_eq!(dev_people.len(), 3, "Expected 3 people in Development unit");
}

#[test]
fn test_clan_bootstrap_nats_permissions() {
    // Load and convert configuration
    let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json")
        .expect("Failed to load clan-bootstrap.json");

    let (_org, _units, people) = ClanBootstrapLoader::to_domain_models(config)
        .expect("Failed to convert to domain models");

    // Verify organization service permissions
    let org_service = people.iter()
        .find(|p| p.name == "organization-service")
        .expect("organization-service not found");

    let perms = org_service.nats_permissions.as_ref()
        .expect("organization-service should have NATS permissions");

    assert!(perms.publish.contains(&"thecowboyai.org.organization.>".to_string()));
    assert!(perms.subscribe.contains(&"thecowboyai.org.organization.>".to_string()));
    assert_eq!(perms.allow_responses, true);
    assert_eq!(perms.max_payload, Some(1048576));

    // Verify person service permissions
    let person_service = people.iter()
        .find(|p| p.name == "person-service")
        .expect("person-service not found");

    let perms = person_service.nats_permissions.as_ref()
        .expect("person-service should have NATS permissions");

    assert!(perms.publish.contains(&"thecowboyai.org.person.>".to_string()));
    assert!(perms.subscribe.contains(&"thecowboyai.org.person.>".to_string()));
    assert!(perms.subscribe.contains(&"thecowboyai.org.location.>".to_string()));
}

#[test]
fn test_clan_bootstrap_with_nsc_export() {
    // This test requires actual NSC export implementation with mock NATS credentials
    // For now, we just verify the domain models are correct

    let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json")
        .expect("Failed to load clan-bootstrap.json");

    let (org, units, people) = ClanBootstrapLoader::to_domain_models(config)
        .expect("Failed to convert to domain models");

    // Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let nsc_store_path = temp_dir.path().join("nsc/stores");
    fs::create_dir_all(&nsc_store_path).expect("Failed to create NSC store directory");

    // Verify we have the right structure for NSC export:
    // - 1 organization (becomes NATS Operator)
    // - 3 organizational units (become NATS Accounts)
    // - 11 service people (become NATS Users)

    assert_eq!(org.name, "thecowboyai");
    assert_eq!(units.len(), 3);
    assert_eq!(people.len(), 11);

    // Verify each unit has a NATS account name
    for unit in &units {
        assert!(
            unit.nats_account_name.is_some(),
            "Unit {} missing NATS account name",
            unit.name
        );
    }

    // Verify each person has NATS permissions
    for person in &people {
        assert!(
            person.nats_permissions.is_some(),
            "Person {} missing NATS permissions",
            person.name
        );
    }

    // NOTE: Actual credential generation is tested in test_complete_credential_generation_and_export
}

#[test]
fn test_complete_credential_generation_and_export() {
    // Step 1: Load clan-bootstrap.json
    let config = ClanBootstrapLoader::load_from_file("examples/clan-bootstrap.json")
        .expect("Failed to load clan-bootstrap.json");

    // Step 2: Convert to domain models
    let (org, units, people) = ClanBootstrapLoader::to_domain_models(config)
        .expect("Failed to convert to domain models");

    // Step 3: Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Step 4: Generate all credentials and export to NSC structure
    let result = generate_and_export_credentials(
        temp_dir.path(),
        org.clone(),
        units.clone(),
        people.clone(),
    );

    assert!(
        result.is_ok(),
        "Failed to generate and export credentials: {:?}",
        result.err()
    );

    let export_result = result.unwrap();

    // Verify export statistics
    assert_eq!(export_result.accounts_exported, 3, "Expected 3 accounts (one per organizational unit)");
    assert_eq!(export_result.users_exported, 11, "Expected 11 users (one per service person)");
    assert!(export_result.total_bytes_written > 0, "Expected some bytes to be written");

    // Verify NSC directory structure
    let nsc_store = temp_dir.path().join("nsc/stores").join(&org.name);
    assert!(nsc_store.exists(), "NSC store directory should exist");

    // Verify operator JWT
    let operator_jwt_path = nsc_store.join(format!("{}.jwt", org.name));
    assert!(operator_jwt_path.exists(), "Operator JWT should exist");

    // Verify operator private key
    let operator_key_path = nsc_store.join("keys").join(format!("operator-{}.nk", org.name));
    assert!(operator_key_path.exists(), "Operator private key should exist");

    // Verify each account has JWT and private key
    for unit in &units {
        let account_name = unit.nats_account_name.as_ref().unwrap();

        // Verify account JWT
        let account_jwt_path = nsc_store
            .join("accounts")
            .join(account_name)
            .join(format!("{}.jwt", account_name));
        assert!(
            account_jwt_path.exists(),
            "Account JWT for {} should exist",
            account_name
        );

        // Verify account private key
        let account_key_path = nsc_store
            .join("keys")
            .join(format!("account-{}.nk", account_name));
        assert!(
            account_key_path.exists(),
            "Account private key for {} should exist",
            account_name
        );

        // Verify users directory exists
        let users_dir = nsc_store
            .join("accounts")
            .join(account_name)
            .join("users");
        assert!(
            users_dir.exists(),
            "Users directory for {} should exist",
            account_name
        );
    }

    // Verify each user has .creds file
    for person in &people {
        // Find which account this person belongs to
        let unit_id = person.unit_ids.first().unwrap();
        let unit = units.iter().find(|u| &u.id == unit_id).unwrap();
        let account_name = unit.nats_account_name.as_ref().unwrap();

        let user_creds_path = nsc_store
            .join("accounts")
            .join(account_name)
            .join("users")
            .join(format!("{}.creds", person.name));

        assert!(
            user_creds_path.exists(),
            "User credentials for {} should exist",
            person.name
        );

        // Verify .creds file format
        let creds_content = fs::read_to_string(&user_creds_path)
            .expect("Failed to read user credentials file");

        assert!(creds_content.contains("-----BEGIN NATS USER JWT-----"));
        assert!(creds_content.contains("------END NATS USER JWT------"));
        assert!(creds_content.contains("-----BEGIN USER NKEY SEED-----"));
        assert!(creds_content.contains("------END USER NKEY SEED------"));
        assert!(creds_content.contains("IMPORTANT"));

        // Verify the seed starts with SU (User seed)
        assert!(creds_content.contains("SU"), "User seed should start with SU");
    }

    // Verify file permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Keys directory should be 0700 (rwx------)
        let keys_dir = nsc_store.join("keys");
        let keys_perms = fs::metadata(&keys_dir).unwrap().permissions();
        assert_eq!(keys_perms.mode() & 0o777, 0o700, "Keys directory should be 0700");

        // Private keys should be 0400 (r--------)
        let operator_key_path = nsc_store.join("keys").join(format!("operator-{}.nk", org.name));
        let key_perms = fs::metadata(&operator_key_path).unwrap().permissions();
        assert_eq!(key_perms.mode() & 0o777, 0o400, "Private keys should be 0400");

        // User credentials should be 0400 (r--------)
        let first_person = &people[0];
        let first_unit = units.iter().find(|u| u.id == first_person.unit_ids[0]).unwrap();
        let first_account_name = first_unit.nats_account_name.as_ref().unwrap();
        let creds_path = nsc_store
            .join("accounts")
            .join(first_account_name)
            .join("users")
            .join(format!("{}.creds", first_person.name));
        let creds_perms = fs::metadata(&creds_path).unwrap().permissions();
        assert_eq!(creds_perms.mode() & 0o777, 0o400, "User credentials should be 0400");
    }

    println!("\n✅ Successfully generated and exported {} accounts and {} users",
             export_result.accounts_exported,
             export_result.users_exported);
    println!("   NSC store location: {}", nsc_store.display());
    println!("   Total bytes written: {}", export_result.total_bytes_written);
}
