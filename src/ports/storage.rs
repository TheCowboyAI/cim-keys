//! Storage port for encrypted key material persistence
//!
//! This defines the interface for storage operations that our domain needs.
//! The actual implementation (filesystem, in-memory, etc.) is separate from this interface.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Port for storage operations
///
/// This is the interface that our domain uses for persisting projections.
/// The actual implementation could be filesystem, S3, in-memory, or any other storage backend.
#[async_trait]
pub trait StoragePort: Send + Sync {
    /// Write data to storage
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), StorageError>;

    /// Read data from storage
    async fn read(&self, path: &str) -> Result<Vec<u8>, StorageError>;

    /// Check if path exists
    async fn exists(&self, path: &str) -> Result<bool, StorageError>;

    /// Delete a file
    async fn delete(&self, path: &str) -> Result<(), StorageError>;

    /// List files in a directory
    async fn list_dir(&self, path: &str) -> Result<Vec<String>, StorageError>;

    /// Create a directory
    async fn create_dir(&self, path: &str) -> Result<(), StorageError>;

    /// Create a directory and all parent directories
    async fn create_dir_all(&self, path: &str) -> Result<(), StorageError>;

    /// Remove a directory and all its contents
    async fn remove_dir_all(&self, path: &str) -> Result<(), StorageError>;

    /// Get metadata about a path
    async fn metadata(&self, path: &str) -> Result<StorageMetadata, StorageError>;

    /// Sync data to ensure durability
    async fn sync(&self, path: &str) -> Result<(), StorageError>;
}

/// Metadata about a stored item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    /// Size in bytes
    pub size: u64,

    /// Is this a directory?
    pub is_dir: bool,

    /// Is this a file?
    pub is_file: bool,

    /// Last modified timestamp (Unix epoch seconds)
    pub modified: u64,

    /// Created timestamp (Unix epoch seconds)
    pub created: u64,
}

/// Storage-specific errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Path not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Not a directory: {0}")]
    NotADirectory(String),

    #[error("Not a file: {0}")]
    NotAFile(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Storage backend error: {0}")]
    BackendError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

/// Configuration for storage backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Root path for storage
    pub root_path: PathBuf,

    /// Enable encryption at rest
    pub encryption_enabled: bool,

    /// Sync mode (for durability)
    pub sync_mode: SyncMode,

    /// Maximum file size in bytes
    pub max_file_size: Option<u64>,
}

/// Sync mode for storage operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncMode {
    /// No fsync (fastest, least durable)
    None,

    /// Sync on every write (slowest, most durable)
    Always,

    /// Sync periodically
    Periodic,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("/mnt/encrypted/cim-keys"),
            encryption_enabled: true,
            sync_mode: SyncMode::Always,
            max_file_size: Some(100 * 1024 * 1024), // 100 MB
        }
    }
}
