//! Secrets file loader for importing organizational configuration
//!
//! This module handles loading secrets from JSON files (cowboyai.json, secrets.json)
//! and converting them into the domain model structures.

use crate::domain::{
    Organization, Person, PersonRole, RoleType,
    RoleScope, Permission, YubiKeyConfig, YubiKeyRole, PivConfig, PivAlgorithm,
    PgpConfig, FidoConfig, SslConfig,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Error type for secrets loading
#[derive(Debug, thiserror::Error)]
pub enum SecretsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Top-level secrets.json structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretsFile {
    pub org: OrgSecrets,
    pub people: Vec<PersonSecrets>,
}

/// Organization section from secrets.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrgSecrets {
    pub name: String,
    #[serde(default)]
    pub id: Option<String>,
    pub login: String,
    pub email: String,
    pub country: String,
    pub region: String,
    pub locality: String,
    #[serde(default)]
    pub certify_pass: Option<String>,
    #[serde(default)]
    pub yubikey: Option<YubiKeySecrets>,
}

/// Person section from secrets.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PersonSecrets {
    pub name: String,
    #[serde(default)]
    pub id: Option<String>,
    pub login: String,
    pub common_name: String,
    pub email: String,
    pub country: String,
    pub region: String,
    pub locality: String,
    pub language: String,
    #[serde(default)]
    pub yubikey: Option<YubiKeySecrets>,
    #[serde(default)]
    pub yubikeys: Vec<String>,  // Just serials
}

/// YubiKey configuration from secrets.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YubiKeySecrets {
    #[serde(default)]
    pub keys: Vec<String>,  // Serial numbers
    pub piv: PivSecrets,
    #[serde(default)]
    pub pgp: Option<PgpSecrets>,
    #[serde(default)]
    pub oauth: Option<OAuthSecrets>,
    #[serde(default)]
    pub fido: Option<FidoSecrets>,
    #[serde(default)]
    pub ssl: Option<SslSecrets>,
}

/// PIV secrets
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PivSecrets {
    pub default_puk: String,
    pub default_pin: String,
    pub puk: String,
    pub pin: String,
    pub mgmt_key: String,
    #[serde(default)]
    pub mgmt_key_old: Option<String>,
    pub piv_alg: String,
}

/// PGP secrets
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PgpSecrets {
    #[serde(default)]
    pub user_pin_old: Option<String>,
    pub user_pin: String,
    #[serde(default)]
    pub admin_pin_old: Option<String>,
    pub admin_pin: String,
    #[serde(default)]
    pub reset_code: Option<String>,
    pub key_type_auth: String,
    pub key_type_sign: String,
    pub key_type_encr: String,
    pub expiration: String,
}

/// OAuth secrets
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthSecrets {
    pub password: String,
}

/// FIDO secrets
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FidoSecrets {
    pub pin: String,
    pub retries: String,
}

/// SSL secrets
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SslSecrets {
    #[serde(default)]
    pub common_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    pub key_type: String,
    pub expiration: String,
}

/// Simplified cowboyai.json structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CowboyAiFile {
    pub organization: OrgInfo,
    pub domain: String,
    pub pin: String,
    pub yubikeys: Vec<YubiKeyInfo>,
    pub users: Vec<String>,
}

/// Organization info from cowboyai.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrgInfo {
    pub name: String,
    pub common_name: String,
    pub country: String,
    pub state: String,
    pub city: String,
    pub email: String,
}

/// YubiKey info from cowboyai.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YubiKeyInfo {
    pub serial: String,
    pub name: String,
    pub owner: String,
    pub role: String,
    pub pin: String,
    pub puk: String,
    pub mgmt_key: String,
}

/// Secrets loader
pub struct SecretsLoader;

impl SecretsLoader {
    /// Load and merge secrets from both JSON files
    pub fn load_from_files(
        secrets_path: impl AsRef<Path>,
        cowboyai_path: impl AsRef<Path>,
    ) -> Result<(Organization, Vec<Person>, Vec<YubiKeyConfig>), SecretsError> {
        // Load secrets.json
        let secrets_content = fs::read_to_string(secrets_path)?;
        let secrets: SecretsFile = serde_json::from_str(&secrets_content)?;

        // Load cowboyai.json
        let cowboyai_content = fs::read_to_string(cowboyai_path)?;
        let cowboyai: CowboyAiFile = serde_json::from_str(&cowboyai_content)?;

        // Convert to domain models
        let organization = Self::build_organization(&secrets.org, &cowboyai)?;
        let people = Self::build_people(&secrets.people, &organization)?;
        let yubikey_configs = Self::build_yubikey_configs(&cowboyai.yubikeys, &secrets)?;

        Ok((organization, people, yubikey_configs))
    }

    /// Build organization from secrets
    fn build_organization(
        org_secrets: &OrgSecrets,
        cowboyai: &CowboyAiFile,
    ) -> Result<Organization, SecretsError> {
        let org_id = if let Some(id_str) = &org_secrets.id {
            Uuid::parse_str(id_str)
                .map_err(|e| SecretsError::Validation(format!("Invalid org ID: {}", e)))?
        } else {
            Uuid::now_v7()
        };

        let mut metadata = HashMap::new();
        metadata.insert("domain".to_string(), cowboyai.domain.clone());
        metadata.insert("state".to_string(), cowboyai.organization.state.clone());
        metadata.insert("city".to_string(), cowboyai.organization.city.clone());

        Ok(Organization {
            id: org_id,
            name: org_secrets.login.clone(),
            display_name: org_secrets.name.clone(),
            description: None,
            parent_id: None,
            units: Vec::new(),  // TODO: Parse organizational units
            created_at: Utc::now(),
            metadata,
        })
    }

    /// Build people from secrets
    fn build_people(
        people_secrets: &[PersonSecrets],
        organization: &Organization,
    ) -> Result<Vec<Person>, SecretsError> {
        let mut people = Vec::new();

        for person_secret in people_secrets {
            let person_id = if let Some(id_str) = &person_secret.id {
                Uuid::parse_str(id_str)
                    .map_err(|e| SecretsError::Validation(format!("Invalid person ID: {}", e)))?
            } else {
                Uuid::now_v7()
            };

            let person = Person {
                id: person_id,
                name: person_secret.name.clone(),
                email: person_secret.email.clone(),
                roles: vec![PersonRole {
                    role_type: RoleType::Developer,  // Default role
                    scope: RoleScope::Organization,
                    permissions: vec![Permission::CreateKeys, Permission::SignCertificates],
                }],
                organization_id: organization.id,
                unit_ids: Vec::new(),
                created_at: Utc::now(),
                active: true,
            };

            people.push(person);
        }

        Ok(people)
    }

    /// Build YubiKey configurations
    fn build_yubikey_configs(
        yubikey_infos: &[YubiKeyInfo],
        secrets: &SecretsFile,
    ) -> Result<Vec<YubiKeyConfig>, SecretsError> {
        let mut configs = Vec::new();

        for yubikey_info in yubikey_infos {
            // Find corresponding detailed config from secrets
            let detailed_config = Self::find_yubikey_config(&yubikey_info.serial, secrets);

            let role = match yubikey_info.role.as_str() {
                "root_ca" => YubiKeyRole::RootCA,
                "backup" => YubiKeyRole::Backup,
                "user" => YubiKeyRole::User,
                "service" => YubiKeyRole::Service,
                _ => YubiKeyRole::User,
            };

            let piv = if let Some(config) = &detailed_config {
                PivConfig {
                    default_pin: config.piv.default_pin.clone(),
                    default_puk: config.piv.default_puk.clone(),
                    pin: config.piv.pin.clone(),
                    puk: config.piv.puk.clone(),
                    mgmt_key: config.piv.mgmt_key.clone(),
                    mgmt_key_old: config.piv.mgmt_key_old.clone(),
                    piv_alg: Self::parse_piv_alg(&config.piv.piv_alg),
                }
            } else {
                // Use values from cowboyai.json
                PivConfig {
                    default_pin: "123456".to_string(),
                    default_puk: "12345678".to_string(),
                    pin: yubikey_info.pin.clone(),
                    puk: yubikey_info.puk.clone(),
                    mgmt_key: yubikey_info.mgmt_key.clone(),
                    mgmt_key_old: None,
                    piv_alg: PivAlgorithm::Aes256,
                }
            };

            let pgp = detailed_config.as_ref().and_then(|c| c.pgp.as_ref()).map(|pgp_secrets| {
                PgpConfig {
                    user_pin: pgp_secrets.user_pin.clone(),
                    user_pin_old: pgp_secrets.user_pin_old.clone(),
                    admin_pin: pgp_secrets.admin_pin.clone(),
                    admin_pin_old: pgp_secrets.admin_pin_old.clone(),
                    reset_code: pgp_secrets.reset_code.clone(),
                    key_type_auth: pgp_secrets.key_type_auth.clone(),
                    key_type_sign: pgp_secrets.key_type_sign.clone(),
                    key_type_encr: pgp_secrets.key_type_encr.clone(),
                    expiration: pgp_secrets.expiration.clone(),
                }
            });

            let fido = detailed_config.as_ref().and_then(|c| c.fido.as_ref()).map(|fido_secrets| {
                FidoConfig {
                    pin: fido_secrets.pin.clone(),
                    retries: fido_secrets.retries.parse().unwrap_or(9),
                }
            });

            let ssl = detailed_config.as_ref().and_then(|c| c.ssl.as_ref()).map(|ssl_secrets| {
                SslConfig {
                    common_name: ssl_secrets.common_name.clone(),
                    email: ssl_secrets.email.clone(),
                    key_type: ssl_secrets.key_type.clone(),
                    expiration: ssl_secrets.expiration.clone(),
                }
            });

            configs.push(YubiKeyConfig {
                serial: yubikey_info.serial.clone(),
                name: yubikey_info.name.clone(),
                owner_email: yubikey_info.owner.clone(),
                role,
                piv,
                pgp,
                fido,
                ssl,
            });
        }

        Ok(configs)
    }

    /// Find YubiKey config in secrets by serial number
    fn find_yubikey_config(serial: &str, secrets: &SecretsFile) -> Option<YubiKeySecrets> {
        // Check org yubikey
        if let Some(org_yk) = &secrets.org.yubikey {
            if org_yk.keys.contains(&serial.to_string()) {
                return Some(org_yk.clone());
            }
        }

        // Check people yubikeys
        for person in &secrets.people {
            if let Some(person_yk) = &person.yubikey {
                if person_yk.keys.contains(&serial.to_string()) {
                    return Some(person_yk.clone());
                }
            }
        }

        None
    }

    /// Parse PIV algorithm string
    fn parse_piv_alg(alg: &str) -> PivAlgorithm {
        match alg.to_lowercase().as_str() {
            "aes128" => PivAlgorithm::Aes128,
            "aes192" => PivAlgorithm::Aes192,
            "aes256" => PivAlgorithm::Aes256,
            "tdes" | "3des" => PivAlgorithm::TripleDes,
            _ => PivAlgorithm::Aes256,  // Default
        }
    }
}
