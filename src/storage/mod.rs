//! Secure key storage
//!
//! This module provides secure storage for cryptographic keys and certificates.

use crate::{KeyError, Result};
use crate::types::*;
use crate::traits::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// File-based key storage
pub struct FileKeyStorage {
    /// Base directory for key storage
    base_dir: PathBuf,
    /// In-memory index of stored keys
    index: Arc<RwLock<HashMap<KeyId, StorageEntry>>>,
}

/// Storage entry metadata
#[derive(Clone)]
struct StorageEntry {
    key_id: KeyId,
    location: KeyLocation,
    metadata: KeyMetadata,
    encrypted: bool,
}

impl FileKeyStorage {
    /// Create a new file-based key storage
    pub async fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_dir).await
            .map_err(|e| KeyError::Io(e))?;

        // Set restrictive permissions (owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&base_dir).await
                .map_err(|e| KeyError::Io(e))?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&base_dir, perms).await
                .map_err(|e| KeyError::Io(e))?;
        }

        let storage = Self {
            base_dir,
            index: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing index
        storage.load_index().await?;

        Ok(storage)
    }

    /// Load the key index from disk
    async fn load_index(&self) -> Result<()> {
        let index_path = self.base_dir.join("index.json");

        if !index_path.exists() {
            return Ok(());
        }

        let data = fs::read(&index_path).await
            .map_err(|e| KeyError::Io(e))?;

        let entries: HashMap<KeyId, StorageEntry> = serde_json::from_slice(&data)
            .map_err(|e| KeyError::Serialization(e))?;

        let mut index = self.index.write().unwrap();
        *index = entries;

        info!("Loaded {} keys from storage index", index.len());
        Ok(())
    }

    /// Save the key index to disk
    async fn save_index(&self) -> Result<()> {
        let index_path = self.base_dir.join("index.json");

        let index = self.index.read().unwrap();
        let data = serde_json::to_vec_pretty(&*index)
            .map_err(|e| KeyError::Serialization(e))?;

        fs::write(&index_path, data).await
            .map_err(|e| KeyError::Io(e))?;

        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&index_path).await
                .map_err(|e| KeyError::Io(e))?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&index_path, perms).await
                .map_err(|e| KeyError::Io(e))?;
        }

        Ok(())
    }

    /// Get the file path for a key
    fn get_key_path(&self, key_id: &KeyId) -> PathBuf {
        self.base_dir.join(format!("{}.key", key_id))
    }
}

#[async_trait]
impl KeyStorage for FileKeyStorage {
    async fn store_key(
        &self,
        key_id: &KeyId,
        key_data: &[u8],
        metadata: KeyMetadata,
        location: KeyLocation,
    ) -> Result<()> {
        // For file storage, we only handle File location type
        match &location {
            KeyLocation::File(path) => {
                // Store the key data
                let key_path = if path.is_absolute() {
                    path.clone()
                } else {
                    self.get_key_path(key_id)
                };

                // Ensure parent directory exists
                if let Some(parent) = key_path.parent() {
                    fs::create_dir_all(parent).await
                        .map_err(|e| KeyError::Io(e))?;
                }

                // Write key data
                fs::write(&key_path, key_data).await
                    .map_err(|e| KeyError::Io(e))?;

                // Set restrictive permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = fs::metadata(&key_path).await
                        .map_err(|e| KeyError::Io(e))?;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o600);
                    fs::set_permissions(&key_path, perms).await
                        .map_err(|e| KeyError::Io(e))?;
                }

                // Update index
                let entry = StorageEntry {
                    key_id: *key_id,
                    location: KeyLocation::File(key_path),
                    metadata,
                    encrypted: false, // TODO: Add encryption support
                };

                let mut index = self.index.write().unwrap();
                index.insert(*key_id, entry);
                drop(index);

                // Save index
                self.save_index().await?;

                info!("Stored key {} to file storage", key_id);
                Ok(())
            }
            _ => Err(KeyError::Storage(
                "FileKeyStorage only supports File location type".to_string()
            )),
        }
    }

    async fn retrieve_key(
        &self,
        key_id: &KeyId,
    ) -> Result<(Vec<u8>, KeyMetadata)> {
        let index = self.index.read().unwrap();
        let entry = index.get(key_id)
            .ok_or_else(|| KeyError::KeyNotFound(key_id.to_string()))?
            .clone();
        drop(index);

        match &entry.location {
            KeyLocation::File(path) => {
                let key_data = fs::read(path).await
                    .map_err(|e| KeyError::Io(e))?;

                Ok((key_data, entry.metadata))
            }
            _ => Err(KeyError::Storage(
                "Unexpected location type in FileKeyStorage".to_string()
            )),
        }
    }

    async fn update_metadata(
        &self,
        key_id: &KeyId,
        metadata: KeyMetadata,
    ) -> Result<()> {
        let mut index = self.index.write().unwrap();

        if let Some(entry) = index.get_mut(key_id) {
            entry.metadata = metadata;
            drop(index);

            // Save updated index
            self.save_index().await?;

            info!("Updated metadata for key {}", key_id);
            Ok(())
        } else {
            Err(KeyError::KeyNotFound(key_id.to_string()))
        }
    }

    async fn key_exists(&self, key_id: &KeyId) -> Result<bool> {
        let index = self.index.read().unwrap();
        Ok(index.contains_key(key_id))
    }
}

/// In-memory key storage (for testing and temporary keys)
pub struct MemoryKeyStorage {
    storage: Arc<RwLock<HashMap<KeyId, (Vec<u8>, KeyMetadata)>>>,
}

impl MemoryKeyStorage {
    /// Create a new in-memory key storage
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl KeyStorage for MemoryKeyStorage {
    async fn store_key(
        &self,
        key_id: &KeyId,
        key_data: &[u8],
        metadata: KeyMetadata,
        location: KeyLocation,
    ) -> Result<()> {
        if !matches!(location, KeyLocation::Memory) {
            return Err(KeyError::Storage(
                "MemoryKeyStorage only supports Memory location type".to_string()
            ));
        }

        let mut storage = self.storage.write().unwrap();
        storage.insert(*key_id, (key_data.to_vec(), metadata));

        debug!("Stored key {} in memory", key_id);
        Ok(())
    }

    async fn retrieve_key(
        &self,
        key_id: &KeyId,
    ) -> Result<(Vec<u8>, KeyMetadata)> {
        let storage = self.storage.read().unwrap();
        storage.get(key_id)
            .cloned()
            .ok_or_else(|| KeyError::KeyNotFound(key_id.to_string()))
    }

    async fn update_metadata(
        &self,
        key_id: &KeyId,
        metadata: KeyMetadata,
    ) -> Result<()> {
        let mut storage = self.storage.write().unwrap();

        if let Some((_, stored_metadata)) = storage.get_mut(key_id) {
            *stored_metadata = metadata;
            Ok(())
        } else {
            Err(KeyError::KeyNotFound(key_id.to_string()))
        }
    }

    async fn key_exists(&self, key_id: &KeyId) -> Result<bool> {
        let storage = self.storage.read().unwrap();
        Ok(storage.contains_key(key_id))
    }
}

impl Default for MemoryKeyStorage {
    fn default() -> Self {
        Self::new()
    }
}

// Make StorageEntry serializable
impl serde::Serialize for StorageEntry {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("StorageEntry", 4)?;
        state.serialize_field("key_id", &self.key_id)?;
        state.serialize_field("location", &self.location)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.serialize_field("encrypted", &self.encrypted)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for StorageEntry {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct StorageEntryData {
            key_id: KeyId,
            location: KeyLocation,
            metadata: KeyMetadata,
            encrypted: bool,
        }

        let data = StorageEntryData::deserialize(deserializer)?;
        Ok(StorageEntry {
            key_id: data.key_id,
            location: data.location,
            metadata: data.metadata,
            encrypted: data.encrypted,
        })
    }
}
