//! Configuration Module
//!
//! Centralized configuration for cim-keys, including NATS streaming,
//! storage paths, and operational modes.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// NATS streaming configuration
    pub nats: NatsConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Operational mode
    pub mode: OperationalMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            nats: NatsConfig::default(),
            storage: StorageConfig::default(),
            mode: OperationalMode::Offline,
        }
    }
}

/// NATS streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    /// Enable NATS event publishing
    pub enabled: bool,

    /// NATS server URL
    pub url: String,

    /// JetStream stream name for events
    pub stream_name: String,

    /// Object store bucket for IPLD payloads
    pub object_store_bucket: String,

    /// Source identifier for this publisher
    pub source_id: String,

    /// Subject prefix for events
    pub subject_prefix: String,

    /// TLS configuration (optional)
    pub tls: Option<TlsConfig>,

    /// Credentials file path (optional)
    pub credentials_file: Option<PathBuf>,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Default to offline mode
            url: "nats://localhost:4222".to_string(),
            stream_name: "CIM_GRAPH_EVENTS".to_string(),
            object_store_bucket: "cim-graph-payloads".to_string(),
            source_id: format!("cim-keys-v{}", env!("CARGO_PKG_VERSION")),
            subject_prefix: "cim.graph".to_string(),
            tls: None,
            credentials_file: None,
        }
    }
}

/// TLS configuration for NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to CA certificate
    pub ca_cert: PathBuf,

    /// Path to client certificate (optional)
    pub client_cert: Option<PathBuf>,

    /// Path to client key (optional)
    pub client_key: Option<PathBuf>,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Output directory for offline events
    pub offline_events_dir: PathBuf,

    /// Output directory for generated keys
    pub keys_output_dir: PathBuf,

    /// Enable automatic backup
    pub enable_backup: bool,

    /// Backup directory
    pub backup_dir: Option<PathBuf>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            offline_events_dir: PathBuf::from("./offline-events"),
            keys_output_dir: PathBuf::from("./keys-output"),
            enable_backup: false,
            backup_dir: None,
        }
    }
}

/// Operational mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalMode {
    /// Offline mode: Events logged locally, not published to NATS
    Offline,

    /// Online mode: Events published to NATS in real-time
    Online,

    /// Hybrid mode: Events logged locally and queued for later batch upload
    Hybrid,
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate NATS configuration
        if self.nats.enabled {
            if self.nats.url.is_empty() {
                return Err(ConfigError::InvalidConfig(
                    "NATS URL cannot be empty when enabled".to_string(),
                ));
            }

            if self.nats.stream_name.is_empty() {
                return Err(ConfigError::InvalidConfig(
                    "Stream name cannot be empty".to_string(),
                ));
            }

            // Validate credentials file exists if specified
            if let Some(creds) = &self.nats.credentials_file {
                if !creds.exists() {
                    return Err(ConfigError::InvalidConfig(
                        format!("Credentials file not found: {}", creds.display()),
                    ));
                }
            }

            // Validate TLS files if specified
            if let Some(tls) = &self.nats.tls {
                if !tls.ca_cert.exists() {
                    return Err(ConfigError::InvalidConfig(
                        format!("CA certificate not found: {}", tls.ca_cert.display()),
                    ));
                }
            }
        }

        // Validate storage paths
        if self.storage.enable_backup {
            if self.storage.backup_dir.is_none() {
                return Err(ConfigError::InvalidConfig(
                    "Backup directory must be specified when backup is enabled".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Create example configuration file
    pub fn create_example(path: &PathBuf) -> Result<(), ConfigError> {
        let example = Config {
            nats: NatsConfig {
                enabled: false,
                url: "nats://leaf-node-1.local:4222".to_string(),
                stream_name: "CIM_GRAPH_EVENTS".to_string(),
                object_store_bucket: "cim-graph-payloads".to_string(),
                source_id: "cim-keys-v0.8.0".to_string(),
                subject_prefix: "cim.graph".to_string(),
                tls: Some(TlsConfig {
                    ca_cert: PathBuf::from("/path/to/ca-cert.pem"),
                    client_cert: Some(PathBuf::from("/path/to/client-cert.pem")),
                    client_key: Some(PathBuf::from("/path/to/client-key.pem")),
                }),
                credentials_file: Some(PathBuf::from("/path/to/nats.creds")),
            },
            storage: StorageConfig {
                offline_events_dir: PathBuf::from("/mnt/encrypted/cim-keys/events"),
                keys_output_dir: PathBuf::from("/mnt/encrypted/cim-keys/keys"),
                enable_backup: true,
                backup_dir: Some(PathBuf::from("/backup/cim-keys")),
            },
            mode: OperationalMode::Hybrid,
        };

        example.save(path)?;
        Ok(())
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialize error: {0}")]
    SerializeError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.nats.enabled);
        assert_eq!(config.mode, OperationalMode::Offline);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Default offline config should validate
        assert!(config.validate().is_ok());

        // Enabled NATS with empty URL should fail
        config.nats.enabled = true;
        config.nats.url = String::new();
        assert!(config.validate().is_err());

        // Fix URL
        config.nats.url = "nats://localhost:4222".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("enabled"));
        assert!(toml_str.contains("stream_name"));
    }
}
