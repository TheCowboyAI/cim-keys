// Copyright (c) 2025 - Cowboy AI, LLC.

//! # NSC Store Projection
//!
//! Composable projections for domain credentials → NSC (NATS Security CLI) store format.
//!
//! ## Architecture
//!
//! ```text
//! Domain Credentials (NKeys + JWTs)
//!     ↓ via
//! CredentialsToNscStoreProjection (pure)
//!     ↓ produces
//! NscStore (directory structure ready for export)
//!     ↓ via
//! NscStoreToFilesystemProjection (I/O)
//!     ↓ produces
//! NscExportResult
//! ```
//!
//! ## NSC Store Structure
//!
//! ```text
//! ~/.nsc/
//! ├── nsc.json                    # Store configuration
//! └── stores/
//!     └── <operator-name>/
//!         ├── <operator>.jwt      # Operator JWT
//!         └── accounts/
//!             └── <account-name>/
//!                 ├── <account>.jwt  # Account JWT
//!                 └── users/
//!                     └── <user>.jwt  # User JWTs
//! ```

use crate::projection::{Projection, ProjectionError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================================
// DOMAIN TYPES
// ============================================================================

/// NATS operator credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorCredentials {
    /// Operator name
    pub name: String,
    /// Operator JWT
    pub jwt: String,
    /// Operator public key (nkey)
    pub public_key: String,
    /// Signing keys for this operator
    pub signing_keys: Vec<String>,
    /// System account public key (if set)
    pub system_account: Option<String>,
}

/// NATS account credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountCredentials {
    /// Account name
    pub name: String,
    /// Account JWT
    pub jwt: String,
    /// Account public key (nkey)
    pub public_key: String,
    /// Parent operator public key
    pub operator_public_key: String,
    /// Signing keys for this account
    pub signing_keys: Vec<String>,
}

/// NATS user credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    /// User name
    pub name: String,
    /// User JWT
    pub jwt: String,
    /// User public key (nkey)
    pub public_key: String,
    /// User private seed (for .creds file)
    pub seed: Option<String>,
    /// Parent account public key
    pub account_public_key: String,
    /// Associated person ID (if any)
    pub person_id: Option<Uuid>,
}

/// Complete NATS credentials set for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainNatsCredentials {
    /// Organization ID
    pub organization_id: Uuid,
    /// Organization name
    pub organization_name: String,
    /// Operator credentials
    pub operator: OperatorCredentials,
    /// Account credentials (keyed by account name)
    pub accounts: HashMap<String, AccountCredentials>,
    /// User credentials (keyed by account name -> user name)
    pub users: HashMap<String, Vec<UserCredentials>>,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
}

// ============================================================================
// NSC STORE STRUCTURE
// ============================================================================

/// NSC store configuration (nsc.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NscConfig {
    /// Store version
    pub version: u32,
    /// Default operator
    pub default_operator: String,
    /// Default account
    pub default_account: Option<String>,
}

impl Default for NscConfig {
    fn default() -> Self {
        Self {
            version: 2,
            default_operator: String::new(),
            default_account: None,
        }
    }
}

/// A file to be written in the NSC store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NscFile {
    /// Relative path within NSC store
    pub path: PathBuf,
    /// File content
    pub content: String,
    /// File type
    pub file_type: NscFileType,
    /// SHA-256 checksum
    pub checksum: String,
}

impl NscFile {
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>, file_type: NscFileType) -> Self {
        let content = content.into();
        let checksum = compute_checksum(&content);
        Self {
            path: path.into(),
            content,
            file_type,
            checksum,
        }
    }
}

/// Type of NSC file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NscFileType {
    /// NSC configuration file
    Config,
    /// Operator JWT
    OperatorJwt,
    /// Account JWT
    AccountJwt,
    /// User JWT
    UserJwt,
    /// Credentials file (.creds)
    Credentials,
    /// NKey seed file
    Seed,
}

/// Complete NSC store ready for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NscStore {
    /// Store root directory name
    pub store_name: String,
    /// Organization ID this store belongs to
    pub organization_id: Uuid,
    /// Files to write
    pub files: Vec<NscFile>,
    /// Directories to create
    pub directories: Vec<PathBuf>,
    /// Store metadata
    pub metadata: NscStoreMetadata,
}

/// Metadata about the NSC store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NscStoreMetadata {
    pub store_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub operator_name: String,
    pub operator_public_key: String,
    pub account_count: usize,
    pub user_count: usize,
    pub source: String,
}

impl NscStore {
    pub fn new(store_name: impl Into<String>, organization_id: Uuid) -> Self {
        Self {
            store_name: store_name.into(),
            organization_id,
            files: Vec::new(),
            directories: Vec::new(),
            metadata: NscStoreMetadata {
                store_id: Uuid::now_v7(),
                created_at: Utc::now(),
                operator_name: String::new(),
                operator_public_key: String::new(),
                account_count: 0,
                user_count: 0,
                source: "cim-keys".to_string(),
            },
        }
    }

    pub fn add_directory(&mut self, path: PathBuf) {
        if !self.directories.contains(&path) {
            self.directories.push(path);
        }
    }

    pub fn add_file(&mut self, file: NscFile) {
        self.files.push(file);
    }

    pub fn total_files(&self) -> usize {
        self.files.len()
    }
}

/// Compute SHA-256 checksum
fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

// ============================================================================
// PROJECTIONS
// ============================================================================

/// Projection: DomainNatsCredentials → NscStore
///
/// Transforms domain NATS credentials into NSC store structure.
pub struct CredentialsToNscStoreProjection {
    /// Include seed files (private keys)
    include_seeds: bool,
    /// Include .creds files (combined JWT + seed)
    include_creds: bool,
}

impl Default for CredentialsToNscStoreProjection {
    fn default() -> Self {
        Self {
            include_seeds: false,
            include_creds: true,
        }
    }
}

impl CredentialsToNscStoreProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seeds(mut self, include: bool) -> Self {
        self.include_seeds = include;
        self
    }

    pub fn with_creds(mut self, include: bool) -> Self {
        self.include_creds = include;
        self
    }

    /// Build the directory structure
    fn build_directories(&self, credentials: &DomainNatsCredentials) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        let operator_name = sanitize_name(&credentials.operator.name);

        // Base directories
        dirs.push(PathBuf::from("stores"));
        dirs.push(PathBuf::from(format!("stores/{}", operator_name)));
        dirs.push(PathBuf::from(format!("stores/{}/accounts", operator_name)));

        // Account directories
        for account_name in credentials.accounts.keys() {
            let safe_name = sanitize_name(account_name);
            dirs.push(PathBuf::from(format!(
                "stores/{}/accounts/{}",
                operator_name, safe_name
            )));
            dirs.push(PathBuf::from(format!(
                "stores/{}/accounts/{}/users",
                operator_name, safe_name
            )));
        }

        dirs
    }

    /// Build the NSC config file
    fn build_config(&self, credentials: &DomainNatsCredentials) -> NscFile {
        let config = NscConfig {
            version: 2,
            default_operator: credentials.operator.name.clone(),
            default_account: credentials.accounts.keys().next().cloned(),
        };

        NscFile::new(
            "nsc.json",
            serde_json::to_string_pretty(&config).unwrap_or_default(),
            NscFileType::Config,
        )
    }

    /// Build operator JWT file
    fn build_operator_jwt(&self, credentials: &DomainNatsCredentials) -> NscFile {
        let operator_name = sanitize_name(&credentials.operator.name);
        NscFile::new(
            format!("stores/{}/{}.jwt", operator_name, operator_name),
            &credentials.operator.jwt,
            NscFileType::OperatorJwt,
        )
    }

    /// Build account JWT files
    fn build_account_jwts(&self, credentials: &DomainNatsCredentials) -> Vec<NscFile> {
        let operator_name = sanitize_name(&credentials.operator.name);
        let mut files = Vec::new();

        for (account_name, account) in &credentials.accounts {
            let safe_name = sanitize_name(account_name);
            files.push(NscFile::new(
                format!("stores/{}/accounts/{}/{}.jwt", operator_name, safe_name, safe_name),
                &account.jwt,
                NscFileType::AccountJwt,
            ));
        }

        files
    }

    /// Build user JWT and credential files
    fn build_user_files(&self, credentials: &DomainNatsCredentials) -> Vec<NscFile> {
        let operator_name = sanitize_name(&credentials.operator.name);
        let mut files = Vec::new();

        for (account_name, users) in &credentials.users {
            let safe_account = sanitize_name(account_name);

            for user in users {
                let safe_user = sanitize_name(&user.name);
                let user_path = format!(
                    "stores/{}/accounts/{}/users/{}",
                    operator_name, safe_account, safe_user
                );

                // User JWT
                files.push(NscFile::new(
                    format!("{}.jwt", user_path),
                    &user.jwt,
                    NscFileType::UserJwt,
                ));

                // Credentials file (JWT + seed)
                if self.include_creds {
                    if let Some(ref seed) = user.seed {
                        let creds_content = format!(
                            "-----BEGIN NATS USER JWT-----\n{}\n------END NATS USER JWT------\n\n\
                             ************************* IMPORTANT *************************\n\
                             NKEY Seed printed below can be used to sign and prove identity.\n\
                             NKEYs are sensitive and should be treated as secrets.\n\n\
                             -----BEGIN USER NKEY SEED-----\n{}\n------END USER NKEY SEED------\n\n\
                             *************************************************************\n",
                            user.jwt, seed
                        );
                        files.push(NscFile::new(
                            format!("{}.creds", user_path),
                            creds_content,
                            NscFileType::Credentials,
                        ));
                    }
                }

                // Seed file (private key)
                if self.include_seeds {
                    if let Some(ref seed) = user.seed {
                        files.push(NscFile::new(
                            format!("{}.nk", user_path),
                            seed,
                            NscFileType::Seed,
                        ));
                    }
                }
            }
        }

        files
    }
}

impl Projection<DomainNatsCredentials, NscStore, ProjectionError> for CredentialsToNscStoreProjection {
    fn project(&self, credentials: DomainNatsCredentials) -> Result<NscStore, ProjectionError> {
        let operator_name = sanitize_name(&credentials.operator.name);
        let mut store = NscStore::new(&operator_name, credentials.organization_id);

        // Build directories
        for dir in self.build_directories(&credentials) {
            store.add_directory(dir);
        }

        // Build files
        store.add_file(self.build_config(&credentials));
        store.add_file(self.build_operator_jwt(&credentials));

        for file in self.build_account_jwts(&credentials) {
            store.add_file(file);
        }

        for file in self.build_user_files(&credentials) {
            store.add_file(file);
        }

        // Update metadata
        store.metadata.operator_name = credentials.operator.name.clone();
        store.metadata.operator_public_key = credentials.operator.public_key.clone();
        store.metadata.account_count = credentials.accounts.len();
        store.metadata.user_count = credentials.users.values().map(|v| v.len()).sum();

        Ok(store)
    }

    fn name(&self) -> &'static str {
        "CredentialsToNscStore"
    }
}

/// Sanitize a name for use in file paths
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

// ============================================================================
// EXPORT RESULT
// ============================================================================

/// Result of exporting NSC store to filesystem
#[derive(Debug, Clone)]
pub struct NscExportResult {
    /// Base path where store was exported
    pub base_path: PathBuf,
    /// Number of directories created
    pub directories_created: usize,
    /// Number of files written
    pub files_written: usize,
    /// Total bytes written
    pub bytes_written: usize,
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    /// Errors encountered (non-fatal)
    pub errors: Vec<String>,
}

impl Default for NscExportResult {
    fn default() -> Self {
        Self {
            base_path: PathBuf::new(),
            directories_created: 0,
            files_written: 0,
            bytes_written: 0,
            exported_at: Utc::now(),
            errors: Vec::new(),
        }
    }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/// Create a credentials-to-NSC-store projection
pub fn credentials_to_nscstore() -> CredentialsToNscStoreProjection {
    CredentialsToNscStoreProjection::new()
}

/// Create a projection that includes seed files
pub fn credentials_to_nscstore_with_seeds() -> CredentialsToNscStoreProjection {
    CredentialsToNscStoreProjection::new()
        .with_seeds(true)
        .with_creds(true)
}

/// Create an NscStore directly from operator credentials
pub fn operator_to_nscstore(
    organization_id: Uuid,
    operator: OperatorCredentials,
    accounts: HashMap<String, AccountCredentials>,
    users: HashMap<String, Vec<UserCredentials>>,
) -> Result<NscStore, ProjectionError> {
    let credentials = DomainNatsCredentials {
        organization_id,
        organization_name: operator.name.clone(),
        operator,
        accounts,
        users,
        generated_at: Utc::now(),
    };

    credentials_to_nscstore().project(credentials)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_credentials() -> DomainNatsCredentials {
        let mut accounts = HashMap::new();
        accounts.insert("engineering".to_string(), AccountCredentials {
            name: "engineering".to_string(),
            jwt: "eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.test_account_jwt".to_string(),
            public_key: "ACDEMOACCOUNTPUBLICKEY12345".to_string(),
            operator_public_key: "ODEMOOPERATORPUBLICKEY12345".to_string(),
            signing_keys: vec![],
        });

        let mut users = HashMap::new();
        users.insert("engineering".to_string(), vec![
            UserCredentials {
                name: "alice".to_string(),
                jwt: "eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.test_user_jwt".to_string(),
                public_key: "UDEMOUSERPUBLICKEY12345".to_string(),
                seed: Some("SUACDEMOTEST12345".to_string()),
                account_public_key: "ACDEMOACCOUNTPUBLICKEY12345".to_string(),
                person_id: Some(Uuid::now_v7()),
            },
        ]);

        DomainNatsCredentials {
            organization_id: Uuid::now_v7(),
            organization_name: "CowboyAI".to_string(),
            operator: OperatorCredentials {
                name: "cowboyai".to_string(),
                jwt: "eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.test_operator_jwt".to_string(),
                public_key: "ODEMOOPERATORPUBLICKEY12345".to_string(),
                signing_keys: vec![],
                system_account: None,
            },
            accounts,
            users,
            generated_at: Utc::now(),
        }
    }

    #[test]
    fn test_credentials_to_nscstore_projection() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore();

        let result = projection.project(credentials);
        assert!(result.is_ok());

        let store = result.unwrap();
        assert_eq!(store.store_name, "cowboyai");
        assert!(!store.files.is_empty());
        assert!(!store.directories.is_empty());
    }

    #[test]
    fn test_directory_structure() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore();
        let store = projection.project(credentials).unwrap();

        // Check expected directories exist
        let dirs: Vec<String> = store.directories.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        assert!(dirs.contains(&"stores".to_string()));
        assert!(dirs.contains(&"stores/cowboyai".to_string()));
        assert!(dirs.contains(&"stores/cowboyai/accounts".to_string()));
        assert!(dirs.contains(&"stores/cowboyai/accounts/engineering".to_string()));
        assert!(dirs.contains(&"stores/cowboyai/accounts/engineering/users".to_string()));
    }

    #[test]
    fn test_operator_jwt_file() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore();
        let store = projection.project(credentials).unwrap();

        let operator_jwt = store.files.iter()
            .find(|f| f.file_type == NscFileType::OperatorJwt);

        assert!(operator_jwt.is_some());
        let jwt_file = operator_jwt.unwrap();
        assert!(jwt_file.path.to_string_lossy().contains("cowboyai.jwt"));
        assert!(jwt_file.content.contains("eyJ"));
    }

    #[test]
    fn test_account_jwt_file() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore();
        let store = projection.project(credentials).unwrap();

        let account_jwts: Vec<_> = store.files.iter()
            .filter(|f| f.file_type == NscFileType::AccountJwt)
            .collect();

        assert_eq!(account_jwts.len(), 1);
        assert!(account_jwts[0].path.to_string_lossy().contains("engineering.jwt"));
    }

    #[test]
    fn test_user_jwt_file() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore();
        let store = projection.project(credentials).unwrap();

        let user_jwts: Vec<_> = store.files.iter()
            .filter(|f| f.file_type == NscFileType::UserJwt)
            .collect();

        assert_eq!(user_jwts.len(), 1);
        assert!(user_jwts[0].path.to_string_lossy().contains("alice.jwt"));
    }

    #[test]
    fn test_credentials_file_included() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore(); // includes creds by default

        let store = projection.project(credentials).unwrap();

        let creds_files: Vec<_> = store.files.iter()
            .filter(|f| f.file_type == NscFileType::Credentials)
            .collect();

        assert_eq!(creds_files.len(), 1);
        assert!(creds_files[0].content.contains("BEGIN NATS USER JWT"));
        assert!(creds_files[0].content.contains("BEGIN USER NKEY SEED"));
    }

    #[test]
    fn test_seeds_excluded_by_default() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore();
        let store = projection.project(credentials).unwrap();

        let seed_files: Vec<_> = store.files.iter()
            .filter(|f| f.file_type == NscFileType::Seed)
            .collect();

        assert!(seed_files.is_empty());
    }

    #[test]
    fn test_seeds_included_when_enabled() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore_with_seeds();
        let store = projection.project(credentials).unwrap();

        let seed_files: Vec<_> = store.files.iter()
            .filter(|f| f.file_type == NscFileType::Seed)
            .collect();

        assert_eq!(seed_files.len(), 1);
    }

    #[test]
    fn test_metadata() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore();
        let store = projection.project(credentials).unwrap();

        assert_eq!(store.metadata.operator_name, "cowboyai");
        assert_eq!(store.metadata.account_count, 1);
        assert_eq!(store.metadata.user_count, 1);
        assert_eq!(store.metadata.source, "cim-keys");
    }

    #[test]
    fn test_name_sanitization() {
        assert_eq!(sanitize_name("CowboyAI"), "cowboyai");
        assert_eq!(sanitize_name("My Account"), "my_account");
        assert_eq!(sanitize_name("test-account"), "test-account");
        assert_eq!(sanitize_name("test_user"), "test_user");
    }

    #[test]
    fn test_checksum_computed() {
        let credentials = sample_credentials();
        let projection = credentials_to_nscstore();
        let store = projection.project(credentials).unwrap();

        // All files should have non-empty checksums
        for file in &store.files {
            assert!(!file.checksum.is_empty());
            assert_eq!(file.checksum.len(), 64); // SHA-256 hex is 64 chars
        }
    }

    #[test]
    fn test_factory_function() {
        let mut accounts = HashMap::new();
        accounts.insert("test".to_string(), AccountCredentials {
            name: "test".to_string(),
            jwt: "test_jwt".to_string(),
            public_key: "ACTEST".to_string(),
            operator_public_key: "OPTEST".to_string(),
            signing_keys: vec![],
        });

        let result = operator_to_nscstore(
            Uuid::now_v7(),
            OperatorCredentials {
                name: "test-operator".to_string(),
                jwt: "test_jwt".to_string(),
                public_key: "OPTEST".to_string(),
                signing_keys: vec![],
                system_account: None,
            },
            accounts,
            HashMap::new(),
        );

        assert!(result.is_ok());
    }
}
