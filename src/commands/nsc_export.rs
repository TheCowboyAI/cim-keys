// NSC Export Module
//
// Exports NATS credentials in NSC (NATS Security) hierarchical format
// for CLAN infrastructure integration.
//
// Directory Structure:
// nsc/stores/{operator}/
// ├── {operator}.jwt                    # Operator JWT
// ├── accounts/
// │   ├── {account}/
// │   │   ├── {account}.jwt             # Account JWT
// │   │   └── users/
// │   │       └── {user}.creds          # User credentials
// └── keys/
//     └── ... (private nkey seeds - SECURE)

use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::domain::{Organization, OrganizationUnit, Person};
use crate::events::DomainEvent;
use crate::value_objects::{NKeyPair, NatsJwt, NKeyType, AccountLimits, Permissions};

// ============================================================================
// NSC Export Command
// ============================================================================

/// Command to export NATS credentials in NSC hierarchical format
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportToNscStore {
    pub output_directory: PathBuf,
    pub operator_name: String,
    pub organization: Organization,
    pub organizational_units: Vec<OrganizationUnit>,
    pub service_people: Vec<Person>,
    pub operator_nkey: NKeyPair,
    pub operator_jwt: NatsJwt,
    pub account_credentials: Vec<NscAccountCredentials>,
    pub user_credentials: Vec<NscUserCredentials>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// NSC Account credentials
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NscAccountCredentials {
    pub account_name: String,
    pub account_id: Uuid,
    pub organizational_unit_id: Uuid,
    pub nkey: NKeyPair,
    pub jwt: NatsJwt,
}

/// NSC User credentials
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NscUserCredentials {
    pub user_name: String,
    pub user_id: Uuid,
    pub person_id: Uuid,
    pub account_name: String,
    pub nkey: NKeyPair,
    pub jwt: NatsJwt,
}

/// Result of NSC export operation
#[derive(Debug, Clone)]
pub struct NscExportCompleted {
    pub nsc_store_path: PathBuf,
    pub operator_jwt_path: PathBuf,
    pub accounts_exported: usize,
    pub users_exported: usize,
    pub total_bytes_written: u64,
    pub events: Vec<DomainEvent>,
}

// ============================================================================
// NSC Directory Structure Generator
// ============================================================================

/// Create NSC directory structure
pub fn create_nsc_directory_structure(
    base_path: &Path,
    operator_name: &str,
) -> Result<PathBuf, String> {
    let nsc_store = base_path.join("nsc").join("stores").join(operator_name);

    // Create main directories
    fs::create_dir_all(&nsc_store)
        .map_err(|e| format!("Failed to create NSC store directory: {}", e))?;

    let accounts_dir = nsc_store.join("accounts");
    fs::create_dir_all(&accounts_dir)
        .map_err(|e| format!("Failed to create accounts directory: {}", e))?;

    let keys_dir = nsc_store.join("keys");
    fs::create_dir_all(&keys_dir)
        .map_err(|e| format!("Failed to create keys directory: {}", e))?;

    // Set secure permissions on keys directory (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o700); // rwx------
        fs::set_permissions(&keys_dir, perms)
            .map_err(|e| format!("Failed to set permissions on keys directory: {}", e))?;
    }

    Ok(nsc_store)
}

/// Create account directory structure
pub fn create_account_directory(
    nsc_store: &Path,
    account_name: &str,
) -> Result<PathBuf, String> {
    let account_dir = nsc_store.join("accounts").join(account_name);
    fs::create_dir_all(&account_dir)
        .map_err(|e| format!("Failed to create account directory {}: {}", account_name, e))?;

    let users_dir = account_dir.join("users");
    fs::create_dir_all(&users_dir)
        .map_err(|e| format!("Failed to create users directory for {}: {}", account_name, e))?;

    Ok(account_dir)
}

// ============================================================================
// JWT Export Functions
// ============================================================================

/// Export operator JWT to root of NSC store
pub fn export_operator_jwt(
    nsc_store: &Path,
    operator_name: &str,
    operator_jwt: &NatsJwt,
) -> Result<u64, String> {
    let jwt_path = nsc_store.join(format!("{}.jwt", operator_name));
    let jwt_content = format!("{}\n", operator_jwt.token());

    fs::write(&jwt_path, &jwt_content)
        .map_err(|e| format!("Failed to write operator JWT: {}", e))?;

    // Set read-only permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o444); // r--r--r--
        fs::set_permissions(&jwt_path, perms)
            .map_err(|e| format!("Failed to set permissions on operator JWT: {}", e))?;
    }

    Ok(jwt_content.len() as u64)
}

/// Export account JWT to account directory
pub fn export_account_jwt(
    nsc_store: &Path,
    account_name: &str,
    account_jwt: &NatsJwt,
) -> Result<u64, String> {
    let account_dir = nsc_store.join("accounts").join(account_name);
    let jwt_path = account_dir.join(format!("{}.jwt", account_name));
    let jwt_content = format!("{}\n", account_jwt.token());

    fs::write(&jwt_path, &jwt_content)
        .map_err(|e| format!("Failed to write account JWT for {}: {}", account_name, e))?;

    // Set read-only permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o444); // r--r--r--
        fs::set_permissions(&jwt_path, perms)
            .map_err(|e| format!("Failed to set permissions on account JWT: {}", e))?;
    }

    Ok(jwt_content.len() as u64)
}

/// Export user credentials to account/users/ directory
pub fn export_user_credentials(
    nsc_store: &Path,
    account_name: &str,
    user_name: &str,
    user_nkey: &NKeyPair,
    user_jwt: &NatsJwt,
) -> Result<u64, String> {
    let users_dir = nsc_store.join("accounts").join(account_name).join("users");
    let creds_path = users_dir.join(format!("{}.creds", user_name));

    // Create credentials file in NATS standard format
    let credentials_content = format!(
        "-----BEGIN NATS USER JWT-----\n{}\n------END NATS USER JWT------\n\n\
         ************************* IMPORTANT *************************\n\
         NKEY Seed printed below can be used to sign and prove identity.\n\
         NKEYs are sensitive and should be treated as secrets.\n\n\
         -----BEGIN USER NKEY SEED-----\n{}\n------END USER NKEY SEED------\n\n\
         *************************************************************\n",
        user_jwt.token(),
        user_nkey.seed_string()
    );

    fs::write(&creds_path, &credentials_content)
        .map_err(|e| format!("Failed to write user credentials for {}: {}", user_name, e))?;

    // Set restrictive permissions (Unix only) - credentials are VERY sensitive
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o400); // r--------
        fs::set_permissions(&creds_path, perms)
            .map_err(|e| format!("Failed to set permissions on user credentials: {}", e))?;
    }

    Ok(credentials_content.len() as u64)
}

/// Export private nkey to secure keys/ directory
pub fn export_private_nkey(
    nsc_store: &Path,
    key_id: &str,
    nkey: &NKeyPair,
) -> Result<u64, String> {
    let keys_dir = nsc_store.join("keys");
    let key_path = keys_dir.join(format!("{}.nk", key_id));
    let key_content = format!("{}\n", nkey.seed_string());

    fs::write(&key_path, &key_content)
        .map_err(|e| format!("Failed to write private nkey {}: {}", key_id, e))?;

    // Set highly restrictive permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o400); // r--------
        fs::set_permissions(&key_path, perms)
            .map_err(|e| format!("Failed to set permissions on private nkey: {}", e))?;
    }

    Ok(key_content.len() as u64)
}

// ============================================================================
// Complete Credential Generation Workflow
// ============================================================================

/// Generate all NATS credentials from domain models and export to NSC store
///
/// This is the main entry point for the complete workflow:
/// 1. Generate operator nkey and JWT from Organization
/// 2. Generate account nkeys and JWTs from OrganizationUnits
/// 3. Generate user nkeys and JWTs from People (service accounts)
/// 4. Export all credentials to NSC directory structure
pub fn generate_and_export_credentials(
    output_directory: &Path,
    organization: Organization,
    organizational_units: Vec<OrganizationUnit>,
    service_people: Vec<Person>,
) -> Result<NscExportCompleted, String> {
    let correlation_id = Uuid::now_v7();

    // Step 1: Generate operator credentials
    let operator_name = organization.name.clone();
    let operator_nkey = NKeyPair::generate(
        NKeyType::Operator,
        Some(operator_name.clone()),
    )?;

    let operator_jwt = NatsJwt::generate_operator(
        &operator_nkey,
        organization.display_name.clone(),
        vec![], // No additional signing keys for now
    )?;

    // Step 2: Generate account credentials for each organizational unit
    let mut account_credentials = Vec::new();

    for unit in &organizational_units {
        let account_name = unit.nats_account_name
            .as_ref()
            .ok_or_else(|| format!("OrganizationUnit {} missing NATS account name", unit.name))?
            .clone();

        let account_nkey = NKeyPair::generate(
            NKeyType::Account,
            Some(account_name.clone()),
        )?;

        let account_jwt = NatsJwt::generate_account(
            &account_nkey,
            &operator_nkey,
            account_name.clone(),
            vec![], // No additional signing keys
            Some(AccountLimits::default()),
            None, // No expiration
        )?;

        account_credentials.push(NscAccountCredentials {
            account_name,
            account_id: Uuid::now_v7(),
            organizational_unit_id: unit.id.as_uuid(),
            nkey: account_nkey,
            jwt: account_jwt,
        });
    }

    // Step 3: Generate user credentials for each service person
    let mut user_credentials = Vec::new();

    for person in &service_people {
        // Find the account this person belongs to
        let unit_id = person.unit_ids.first()
            .ok_or_else(|| format!("Person {} has no organizational unit", person.name))?;

        let account = account_credentials.iter()
            .find(|acc| acc.organizational_unit_id == unit_id.as_uuid())
            .ok_or_else(|| format!("No account found for unit {:?}", unit_id))?;

        let user_nkey = NKeyPair::generate(
            NKeyType::User,
            Some(person.name.clone()),
        )?;

        // Convert domain permissions to NATS permissions
        let permissions = person.nats_permissions.as_ref()
            .map(|perms| Permissions {
                pub_allow: Some(perms.publish.clone()),
                pub_deny: None,
                sub_allow: Some(perms.subscribe.clone()),
                sub_deny: None,
            });

        let user_jwt = NatsJwt::generate_user(
            &user_nkey,
            &account.nkey,
            person.name.clone(),
            permissions,
            None, // No custom limits
            None, // No expiration
        )?;

        user_credentials.push(NscUserCredentials {
            user_name: person.name.clone(),
            user_id: Uuid::now_v7(),
            person_id: person.id.as_uuid(),
            account_name: account.account_name.clone(),
            nkey: user_nkey,
            jwt: user_jwt,
        });
    }

    // Step 4: Create and execute export command
    // A4: Use correlation_id as causation for the nested command (root command)
    let cmd = ExportToNscStore {
        output_directory: output_directory.to_path_buf(),
        operator_name: operator_name.clone(),
        organization,
        organizational_units,
        service_people,
        operator_nkey,
        operator_jwt,
        account_credentials,
        user_credentials,
        correlation_id,
        causation_id: Some(correlation_id), // A4: Self-reference for root command
    };

    handle_export_to_nsc_store(cmd)
}

// ============================================================================
// Main Export Handler
// ============================================================================

/// Handle ExportToNscStore command
///
/// Exports NATS credentials in NSC hierarchical format for CLAN integration.
///
/// Emits:
/// - NatsOperatorExportedEvent
/// - NatsAccountExportedEvent (for each account)
/// - NatsUserExportedEvent (for each user)
///
/// Directory structure created:
/// nsc/stores/{operator}/
/// ├── {operator}.jwt
/// ├── accounts/{account}/{account}.jwt
/// ├── accounts/{account}/users/{user}.creds
/// └── keys/{key-id}.nk
pub fn handle_export_to_nsc_store(
    cmd: ExportToNscStore,
) -> Result<NscExportCompleted, String> {
    let events = Vec::new();
    let mut total_bytes = 0u64;

    // Step 1: Create NSC directory structure
    let nsc_store = create_nsc_directory_structure(&cmd.output_directory, &cmd.operator_name)?;

    // Step 2: Export operator JWT
    let operator_jwt_path = nsc_store.join(format!("{}.jwt", cmd.operator_name));
    total_bytes += export_operator_jwt(&nsc_store, &cmd.operator_name, &cmd.operator_jwt)?;

    // Step 3: Export operator private nkey
    total_bytes += export_private_nkey(&nsc_store, &format!("operator-{}", cmd.operator_name), &cmd.operator_nkey)?;

    // TODO: Emit NatsOperatorExportedEvent
    // events.push(DomainEvent::NatsOperatorExported { ... });

    // Step 4: Export account JWTs and private nkeys
    for account in &cmd.account_credentials {
        // Create account directory
        create_account_directory(&nsc_store, &account.account_name)?;

        // Export account JWT
        total_bytes += export_account_jwt(&nsc_store, &account.account_name, &account.jwt)?;

        // Export account private nkey
        total_bytes += export_private_nkey(&nsc_store, &format!("account-{}", account.account_name), &account.nkey)?;

        // TODO: Emit NatsAccountExportedEvent
        // events.push(DomainEvent::NatsAccountExported { ... });
    }

    // Step 5: Export user credentials
    for user in &cmd.user_credentials {
        total_bytes += export_user_credentials(
            &nsc_store,
            &user.account_name,
            &user.user_name,
            &user.nkey,
            &user.jwt,
        )?;

        // TODO: Emit NatsUserExportedEvent
        // events.push(DomainEvent::NatsUserExported { ... });
    }

    Ok(NscExportCompleted {
        nsc_store_path: nsc_store.clone(),
        operator_jwt_path,
        accounts_exported: cmd.account_credentials.len(),
        users_exported: cmd.user_credentials.len(),
        total_bytes_written: total_bytes,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_nsc_directory_structure() {
        let temp_dir = std::env::temp_dir().join(format!("nsc-test-{}", Uuid::now_v7()));
        let result = create_nsc_directory_structure(&temp_dir, "thecowboyai");
        assert!(result.is_ok());

        let nsc_store = result.unwrap();
        assert!(nsc_store.exists());
        assert!(nsc_store.join("accounts").exists());
        assert!(nsc_store.join("keys").exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_account_directory() {
        let temp_dir = std::env::temp_dir().join(format!("nsc-test-{}", Uuid::now_v7()));
        let nsc_store = create_nsc_directory_structure(&temp_dir, "thecowboyai").unwrap();

        let result = create_account_directory(&nsc_store, "core");
        assert!(result.is_ok());

        let account_dir = result.unwrap();
        assert!(account_dir.exists());
        assert!(account_dir.join("users").exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
