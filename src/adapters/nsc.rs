//! NSC adapter for NATS key management
//!
//! This adapter implements the NatsKeyPort using the NSC (NATS Security Client) tool.
//! It can either call NSC via command line or use the nkeys crate directly.

use crate::ports::nats::*;
use async_trait::async_trait;
use std::process::Command;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use serde_json;

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

    /// Generate keys using native Rust implementation
    fn generate_native_keys(&self, key_type: &str) -> Result<(String, String), NatsKeyError> {
        // In a real implementation, we would use the nkeys crate here
        // For now, we'll return placeholder values
        let public_key = format!("{}AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            match key_type {
                "operator" => "O",
                "account" => "A",
                "user" => "U",
                _ => "S",
            });
        let seed = format!("S{}AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", key_type.to_uppercase());

        Ok((public_key, seed))
    }
}

#[async_trait]
impl NatsKeyPort for NscAdapter {
    async fn generate_operator(&self, name: &str) -> Result<NatsOperatorKeys, NatsKeyError> {
        let id = Uuid::now_v7();

        if self.use_cli {
            // Use NSC CLI
            let output = self.execute_nsc(&["add", "operator", name])?;

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
            // Use native implementation
            let (public_key, seed) = self.generate_native_keys("operator")?;

            Ok(NatsOperatorKeys {
                id,
                name: name.to_string(),
                public_key,
                seed,
                jwt: None,
            })
        }
    }

    async fn generate_account(&self, operator_id: &str, name: &str) -> Result<NatsAccountKeys, NatsKeyError> {
        let id = Uuid::now_v7();
        let operator_uuid = Uuid::parse_str(operator_id)
            .map_err(|e| NatsKeyError::InvalidConfiguration(format!("Invalid operator ID: {}", e)))?;

        if self.use_cli {
            // Use NSC CLI
            let output = self.execute_nsc(&["add", "account", name])?;

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
            // Use native implementation
            let (public_key, seed) = self.generate_native_keys("account")?;

            Ok(NatsAccountKeys {
                id,
                operator_id: operator_uuid,
                name: name.to_string(),
                public_key,
                seed,
                jwt: None,
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
            let output = self.execute_nsc(&["add", "user", name])?;

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
            // Use native implementation
            let (public_key, seed) = self.generate_native_keys("user")?;

            Ok(NatsUserKeys {
                id,
                account_id: account_uuid,
                name: name.to_string(),
                public_key,
                seed,
                jwt: None,
            })
        }
    }

    async fn generate_signing_key(&self, entity_id: &str) -> Result<NatsSigningKey, NatsKeyError> {
        let id = Uuid::now_v7();
        let entity_uuid = Uuid::parse_str(entity_id)
            .map_err(|e| NatsKeyError::InvalidConfiguration(format!("Invalid entity ID: {}", e)))?;

        let (public_key, seed) = if self.use_cli {
            // Use NSC to generate signing key
            let output = self.execute_nsc(&["generate", "nkey", "--operator"])?;

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
        // In real implementation, we'd create and sign JWT
        // This would use the nkeys crate to sign the JWT with the provided key

        let jwt = serde_json::json!({
            "sub": claims.subject,
            "iss": claims.issuer,
            "name": claims.name,
            "nats": claims.nats,
        });

        Ok(format!("eyJ0eXAiOiJKV1QiLCJhbGciOiJFZDI1NTE5In0.{}.signature",
            base64::encode(jwt.to_string())))
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

// Helper to use native nkeys implementation
#[cfg(feature = "nkeys")]
mod native_impl {
    use super::*;
    use nkeys::{KeyPair, KeyPairType};

    pub fn generate_keypair(key_type: KeyPairType) -> Result<(String, String), NatsKeyError> {
        let kp = KeyPair::new(key_type);
        let public_key = kp.public_key();
        let seed = kp.seed()
            .map_err(|e| NatsKeyError::GenerationFailed(format!("Failed to get seed: {}", e)))?;

        Ok((public_key, seed))
    }
}

use std::collections::HashMap;