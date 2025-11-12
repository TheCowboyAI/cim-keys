//! NSC adapter for NATS key management
//!
//! This adapter implements the NatsKeyPort using the NSC (NATS Security Client) tool.
//! It can either call NSC via command line or use the nkeys crate directly.

use crate::ports::nats::*;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use serde_json;
use nkeys::KeyPair;

/// NSC adapter for NATS key operations
pub struct NscAdapter {
    /// Path to NSC store directory
    store_dir: PathBuf,

    /// Use command line NSC vs native implementation
    use_cli: bool,
}

impl NscAdapter {
    /// Create a new NSC adapter
    pub fn new<P: AsRef<Path>>(store_dir: P, use_cli: bool) -> Self {
        Self {
            store_dir: store_dir.as_ref().to_path_buf(),
            use_cli,
        }
    }

    /// Execute NSC command
    fn execute_nsc(&self, args: &[&str]) -> Result<String, NatsKeyError> {
        let output = Command::new("nsc")
            .env("NSC_STORE", &self.store_dir)
            .args(args)
            .output()
            .map_err(|e| NatsKeyError::IoError(format!("Failed to execute NSC: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NatsKeyError::GenerationFailed(format!("NSC command failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Generate keys using native Rust implementation with nkeys crate
    fn generate_native_keys(&self, key_type: &str) -> Result<(String, String), NatsKeyError> {
        let kp = match key_type {
            "operator" => KeyPair::new_operator(),
            "account" => KeyPair::new_account(),
            "user" => KeyPair::new_user(),
            "signing" => KeyPair::new_operator(), // Signing keys use operator type
            _ => return Err(NatsKeyError::InvalidConfiguration(
                format!("Unknown key type: {}", key_type)
            )),
        };

        let public_key = kp.public_key();
        let seed = kp.seed()
            .map_err(|e| NatsKeyError::GenerationFailed(format!("Failed to get seed: {}", e)))?;

        Ok((public_key, seed))
    }
}

#[async_trait]
impl NatsKeyPort for NscAdapter {
    async fn generate_operator(&self, name: &str) -> Result<NatsOperatorKeys, NatsKeyError> {
        let id = Uuid::now_v7();

        if self.use_cli {
            // Use NSC CLI
            let _output = self.execute_nsc(&["add", "operator", name])?;
            // Note: In real implementation, we would parse _output to extract actual keys

            // Parse the output to extract keys
            // In real implementation, we'd parse NSC output properly
            let public_key = format!("O{}", id.simple());
            let seed = format!("SO{}", id.simple());

            Ok(NatsOperatorKeys {
                id,
                name: name.to_string(),
                public_key,
                seed,
                jwt: None,
            })
        } else {
            // Use native implementation with nkeys
            let kp = KeyPair::new_operator();
            let public_key = kp.public_key();
            let seed = kp.seed()
                .map_err(|e| NatsKeyError::GenerationFailed(format!("Failed to get operator seed: {}", e)))?;

            // Generate self-signed operator JWT
            let claims = JwtClaims {
                subject: public_key.clone(),
                issuer: public_key.clone(), // Self-signed
                audience: None,
                name: name.to_string(),
                nats: NatsJwtClaims {
                    version: 2,
                    r#type: "operator".to_string(),
                    permissions: None,
                    limits: None,
                },
            };

            let jwt = self.create_jwt(&claims, &seed).await?;

            Ok(NatsOperatorKeys {
                id,
                name: name.to_string(),
                public_key,
                seed,
                jwt: Some(jwt),
            })
        }
    }

    async fn generate_account(&self, operator_id: &str, name: &str) -> Result<NatsAccountKeys, NatsKeyError> {
        let id = Uuid::now_v7();
        let operator_uuid = Uuid::parse_str(operator_id)
            .map_err(|e| NatsKeyError::InvalidConfiguration(format!("Invalid operator ID: {}", e)))?;

        if self.use_cli {
            // Use NSC CLI
            let _output = self.execute_nsc(&["add", "account", name])?;
            // Note: In real implementation, we would parse _output to extract actual keys

            let public_key = format!("A{}", id.simple());
            let seed = format!("SA{}", id.simple());

            Ok(NatsAccountKeys {
                id,
                operator_id: operator_uuid,
                name: name.to_string(),
                public_key,
                seed,
                jwt: None,
                is_system: name == "SYS",
            })
        } else {
            // Use native implementation with nkeys
            let kp = KeyPair::new_account();
            let public_key = kp.public_key();
            let seed = kp.seed()
                .map_err(|e| NatsKeyError::GenerationFailed(format!("Failed to get account seed: {}", e)))?;

            // Note: Account JWT should be signed by operator, not self-signed
            // For now, we'll create it but it needs operator's signing key in real usage
            let claims = JwtClaims {
                subject: public_key.clone(),
                issuer: operator_id.to_string(), // Signed by operator
                audience: None,
                name: name.to_string(),
                nats: NatsJwtClaims {
                    version: 2,
                    r#type: "account".to_string(),
                    permissions: None,
                    limits: Some(NatsLimits {
                        subs: Some(-1), // Unlimited subscriptions
                        payload: Some(-1), // Unlimited payload
                        data: Some(-1), // Unlimited data
                    }),
                },
            };

            // TODO: This should use operator's signing key, not account's seed
            let jwt = self.create_jwt(&claims, &seed).await?;

            Ok(NatsAccountKeys {
                id,
                operator_id: operator_uuid,
                name: name.to_string(),
                public_key,
                seed,
                jwt: Some(jwt),
                is_system: name == "SYS",
            })
        }
    }

    async fn generate_user(&self, account_id: &str, name: &str) -> Result<NatsUserKeys, NatsKeyError> {
        let id = Uuid::now_v7();
        let account_uuid = Uuid::parse_str(account_id)
            .map_err(|e| NatsKeyError::InvalidConfiguration(format!("Invalid account ID: {}", e)))?;

        if self.use_cli {
            // Use NSC CLI
            let _output = self.execute_nsc(&["add", "user", name])?;
            // Note: In real implementation, we would parse _output to extract actual keys

            let public_key = format!("U{}", id.simple());
            let seed = format!("SU{}", id.simple());

            Ok(NatsUserKeys {
                id,
                account_id: account_uuid,
                name: name.to_string(),
                public_key,
                seed,
                jwt: None,
            })
        } else {
            // Use native implementation with nkeys
            let kp = KeyPair::new_user();
            let public_key = kp.public_key();
            let seed = kp.seed()
                .map_err(|e| NatsKeyError::GenerationFailed(format!("Failed to get user seed: {}", e)))?;

            // Note: User JWT should be signed by account, not self-signed
            // For now, we'll create it but it needs account's signing key in real usage
            let claims = JwtClaims {
                subject: public_key.clone(),
                issuer: account_id.to_string(), // Signed by account
                audience: None,
                name: name.to_string(),
                nats: NatsJwtClaims {
                    version: 2,
                    r#type: "user".to_string(),
                    permissions: Some(NatsPermissions {
                        publish: NatsSubjectPermissions {
                            allow: vec!["*".to_string()],
                            deny: vec![],
                        },
                        subscribe: NatsSubjectPermissions {
                            allow: vec!["*".to_string()],
                            deny: vec![],
                        },
                        allow_responses: true,
                        max_payload: None,
                    }),
                    limits: None,
                },
            };

            // TODO: This should use account's signing key, not user's seed
            let jwt = self.create_jwt(&claims, &seed).await?;

            Ok(NatsUserKeys {
                id,
                account_id: account_uuid,
                name: name.to_string(),
                public_key,
                seed,
                jwt: Some(jwt),
            })
        }
    }

    async fn generate_signing_key(&self, entity_id: &str) -> Result<NatsSigningKey, NatsKeyError> {
        let id = Uuid::now_v7();
        let entity_uuid = Uuid::parse_str(entity_id)
            .map_err(|e| NatsKeyError::InvalidConfiguration(format!("Invalid entity ID: {}", e)))?;

        let (public_key, seed) = if self.use_cli {
            // Use NSC to generate signing key
            let _output = self.execute_nsc(&["generate", "nkey", "--operator"])?;
            // Note: In real implementation, we would parse _output to extract actual keys

            // Parse output
            (format!("O{}", id.simple()), format!("SO{}", id.simple()))
        } else {
            self.generate_native_keys("signing")?
        };

        Ok(NatsSigningKey {
            id,
            entity_id: entity_uuid,
            public_key,
            seed,
        })
    }

    async fn create_jwt(&self, claims: &JwtClaims, signing_key: &str) -> Result<String, NatsKeyError> {
        // Validate signing key is provided
        if signing_key.is_empty() {
            return Err(NatsKeyError::InvalidConfiguration(
                "Signing key cannot be empty".to_string()
            ));
        }

        // Parse the seed to get keypair for signing
        let kp = KeyPair::from_seed(signing_key)
            .map_err(|e| NatsKeyError::InvalidConfiguration(format!("Invalid signing key: {}", e)))?;

        // Build JWT header
        let header = serde_json::json!({
            "typ": "JWT",
            "alg": "ed25519-nkey"
        });

        // Build JWT payload with NATS claims
        let payload = serde_json::json!({
            "jti": uuid::Uuid::now_v7().to_string(),
            "iat": chrono::Utc::now().timestamp(),
            "iss": claims.issuer,
            "sub": claims.subject,
            "name": claims.name,
            "nats": claims.nats,
        });

        // Encode header and payload
        let header_encoded = BASE64.encode(serde_json::to_string(&header)
            .map_err(|e| NatsKeyError::GenerationFailed(format!("Failed to encode header: {}", e)))?);
        let payload_encoded = BASE64.encode(serde_json::to_string(&payload)
            .map_err(|e| NatsKeyError::GenerationFailed(format!("Failed to encode payload: {}", e)))?);

        // Create signing input
        let signing_input = format!("{}.{}", header_encoded, payload_encoded);

        // Sign with nkey
        let signature = kp.sign(signing_input.as_bytes())
            .map_err(|e| NatsKeyError::GenerationFailed(format!("Failed to sign JWT: {}", e)))?;
        let signature_encoded = BASE64.encode(&signature);

        // Combine into final JWT
        Ok(format!("{}.{}", signing_input, signature_encoded))
    }

    async fn export_keys(&self, keys: &NatsKeys) -> Result<NatsKeyExport, NatsKeyError> {
        let mut operators = HashMap::new();
        let mut accounts = HashMap::new();
        let mut users = HashMap::new();

        // Export operator
        operators.insert(
            keys.operator.name.clone(),
            OperatorExport {
                name: keys.operator.name.clone(),
                public_key: keys.operator.public_key.clone(),
                jwt_file: format!("{}.jwt", keys.operator.name),
                seed_file: format!("{}.nk", keys.operator.name),
            },
        );

        // Export accounts
        for account in &keys.accounts {
            accounts.insert(
                account.name.clone(),
                AccountExport {
                    name: account.name.clone(),
                    public_key: account.public_key.clone(),
                    jwt_file: format!("{}.jwt", account.name),
                    seed_file: format!("{}.nk", account.name),
                },
            );
        }

        // Export users
        for user in &keys.users {
            users.insert(
                user.name.clone(),
                UserExport {
                    name: user.name.clone(),
                    public_key: user.public_key.clone(),
                    creds_file: format!("{}.creds", user.name),
                },
            );
        }

        // Find system account
        let system_account = keys.accounts
            .iter()
            .find(|a| a.is_system)
            .map(|a| a.public_key.clone())
            .unwrap_or_default();

        Ok(NatsKeyExport {
            nsc_format: NscStoreExport {
                operators,
                accounts,
                users,
            },
            resolver_config: ResolverConfig {
                operator_jwt_path: format!("{}.jwt", keys.operator.name),
                system_account: system_account.clone(),
                resolver_url: None,
            },
            server_config: ServerConfig {
                operator: keys.operator.public_key.clone(),
                system_account,
                jwt_path: format!("{}.jwt", keys.operator.name),
                resolver: ResolverType::Memory,
            },
        })
    }

    async fn validate_key(&self, key: &str) -> Result<bool, NatsKeyError> {
        // Validate key format
        if key.len() < 56 {
            return Ok(false);
        }

        let prefix = &key[0..1];
        match prefix {
            "O" | "A" | "U" | "S" => Ok(true),
            _ => Ok(false),
        }
    }
}

use std::collections::HashMap;

impl NscAdapter {
    /// Export NATS hierarchy to NSC directory structure
    ///
    /// Creates the NSC-compatible directory structure:
    /// ```
    /// $NSC_STORE/stores/<org-name>/
    /// ├── operator.jwt
    /// ├── .nkeys/creds/<org-name>/<org-name>.nk
    /// └── accounts/<account-name>/
    ///     ├── account.jwt
    ///     └── users/<user-name>.jwt
    /// ```
    pub async fn export_to_nsc_store(
        &self,
        keys: &NatsKeys,
        output_dir: &Path,
    ) -> Result<(), NatsKeyError> {
        let org_name = &keys.operator.name;

        // Create base directory structure
        let stores_dir = output_dir.join("stores").join(org_name);
        let nkeys_dir = stores_dir.join(".nkeys").join("creds").join(org_name);
        let accounts_dir = stores_dir.join("accounts");

        fs::create_dir_all(&stores_dir)
            .map_err(|e| NatsKeyError::IoError(format!("Failed to create stores directory: {}", e)))?;
        fs::create_dir_all(&nkeys_dir)
            .map_err(|e| NatsKeyError::IoError(format!("Failed to create nkeys directory: {}", e)))?;
        fs::create_dir_all(&accounts_dir)
            .map_err(|e| NatsKeyError::IoError(format!("Failed to create accounts directory: {}", e)))?;

        // Write operator JWT
        if let Some(jwt) = &keys.operator.jwt {
            let operator_jwt_path = stores_dir.join("operator.jwt");
            fs::write(&operator_jwt_path, jwt)
                .map_err(|e| NatsKeyError::IoError(format!("Failed to write operator JWT: {}", e)))?;
        }

        // Write operator seed (nkey)
        let operator_nk_path = nkeys_dir.join(format!("{}.nk", org_name));
        fs::write(&operator_nk_path, &keys.operator.seed)
            .map_err(|e| NatsKeyError::IoError(format!("Failed to write operator seed: {}", e)))?;

        // Process each account
        for account in &keys.accounts {
            let account_dir = accounts_dir.join(&account.name);
            let account_users_dir = account_dir.join("users");
            let account_nkeys_dir = nkeys_dir.join("accounts").join(&account.name);

            fs::create_dir_all(&account_dir)
                .map_err(|e| NatsKeyError::IoError(format!("Failed to create account directory: {}", e)))?;
            fs::create_dir_all(&account_users_dir)
                .map_err(|e| NatsKeyError::IoError(format!("Failed to create account users directory: {}", e)))?;
            fs::create_dir_all(&account_nkeys_dir)
                .map_err(|e| NatsKeyError::IoError(format!("Failed to create account nkeys directory: {}", e)))?;

            // Write account JWT
            if let Some(jwt) = &account.jwt {
                let account_jwt_path = account_dir.join("account.jwt");
                fs::write(&account_jwt_path, jwt)
                    .map_err(|e| NatsKeyError::IoError(format!("Failed to write account JWT: {}", e)))?;
            }

            // Write account seed
            let account_nk_path = account_nkeys_dir.join(format!("{}.nk", account.name));
            fs::write(&account_nk_path, &account.seed)
                .map_err(|e| NatsKeyError::IoError(format!("Failed to write account seed: {}", e)))?;
        }

        // Process each user
        for user in &keys.users {
            // Find the account this user belongs to
            let account = keys.accounts.iter()
                .find(|a| a.id == user.account_id)
                .ok_or_else(|| NatsKeyError::InvalidConfiguration(
                    format!("Account not found for user {}", user.name)
                ))?;

            let users_dir = accounts_dir.join(&account.name).join("users");
            let user_nkeys_dir = nkeys_dir.join("accounts").join(&account.name).join("users");

            fs::create_dir_all(&users_dir)
                .map_err(|e| NatsKeyError::IoError(format!("Failed to create users directory: {}", e)))?;
            fs::create_dir_all(&user_nkeys_dir)
                .map_err(|e| NatsKeyError::IoError(format!("Failed to create user nkeys directory: {}", e)))?;

            // Write user JWT
            if let Some(jwt) = &user.jwt {
                let user_jwt_path = users_dir.join(format!("{}.jwt", user.name));
                fs::write(&user_jwt_path, jwt)
                    .map_err(|e| NatsKeyError::IoError(format!("Failed to write user JWT: {}", e)))?;
            }

            // Generate and write .creds file (combines JWT + seed)
            if let Some(jwt) = &user.jwt {
                let creds_content = format!(
                    "-----BEGIN NATS USER JWT-----\n{}\n------END NATS USER JWT------\n\n\
                    ************************* IMPORTANT *************************\n\
                    NKEY Seed printed below can be used to sign and prove identity.\n\
                    NKEYs are sensitive and should be treated as secrets.\n\n\
                    -----BEGIN USER NKEY SEED-----\n{}\n------END USER NKEY SEED------\n",
                    jwt, user.seed
                );

                let creds_path = user_nkeys_dir.join(format!("{}.creds", user.name));
                fs::write(&creds_path, creds_content)
                    .map_err(|e| NatsKeyError::IoError(format!("Failed to write user creds: {}", e)))?;
            }
        }

        tracing::info!(
            "Successfully exported NATS hierarchy to NSC store at {}",
            output_dir.display()
        );

        Ok(())
    }
}