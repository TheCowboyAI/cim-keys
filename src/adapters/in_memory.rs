//! In-memory storage adapter for testing
//!
//! This adapter implements the StoragePort trait using an in-memory HashMap.
//! It provides a functor from the Storage category to the Domain category.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: Storage (filesystem, S3, etc.)
//! - **Target Category**: Domain (key management operations)
//! - **Functor**: InMemoryStorageAdapter maps storage operations to domain operations
//! - **Morphisms Preserved**: read/write operations maintain their composition laws

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ports::storage::{StoragePort, StorageError, StorageMetadata};

/// In-memory storage adapter for testing
///
/// This is a **Functor** F: Storage → Domain where:
/// - Objects: Storage paths → Domain data
/// - Morphisms: Storage operations (read/write/delete) → Domain projections
///
/// **Functor Laws:**
/// 1. Identity: F(id) = id - Empty operation maps to empty result
/// 2. Composition: F(g ∘ f) = F(g) ∘ F(f) - write then read preserves data
#[derive(Clone)]
pub struct InMemoryStorageAdapter {
    /// Storage state (path -> data)
    storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,

    /// Directory structure (path -> children)
    directories: Arc<RwLock<HashMap<String, Vec<String>>>>,

    /// Metadata (path -> metadata)
    metadata: Arc<RwLock<HashMap<String, StorageMetadata>>>,
}

impl InMemoryStorageAdapter {
    /// Create a new in-memory storage adapter
    ///
    /// This constructs the functor with an empty storage state.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            directories: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Clear all storage (for test isolation)
    pub fn clear(&self) {
        self.storage.write().unwrap().clear();
        self.directories.write().unwrap().clear();
        self.metadata.write().unwrap().clear();
    }

    /// Get current storage size (for testing/debugging)
    pub fn size(&self) -> usize {
        self.storage.read().unwrap().len()
    }

    fn normalize_path(path: &str) -> String {
        path.trim_start_matches('/').to_string()
    }

    fn parent_path(path: &str) -> Option<String> {
        let normalized = Self::normalize_path(path);
        if normalized.is_empty() {
            return None;
        }

        let parts: Vec<&str> = normalized.split('/').collect();
        if parts.len() == 1 {
            return Some(String::new());
        }

        Some(parts[..parts.len() - 1].join("/"))
    }

    fn update_metadata(&self, path: &str, is_dir: bool, size: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let metadata = StorageMetadata {
            size,
            is_dir,
            is_file: !is_dir,
            modified: now,
            created: now,
        };

        self.metadata.write().unwrap().insert(path.to_string(), metadata);
    }
}

impl Default for InMemoryStorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StoragePort for InMemoryStorageAdapter {
    /// **Functor Mapping**: write: (path, data) → ()
    ///
    /// Preserves composition: writing data then reading it returns the same data
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), StorageError> {
        let normalized = Self::normalize_path(path);

        // Ensure parent directory exists
        if let Some(parent) = Self::parent_path(&normalized) {
            if !parent.is_empty() {
                let dirs = self.directories.read().unwrap();
                if !dirs.contains_key(&parent) {
                    return Err(StorageError::NotADirectory(parent));
                }
            }
        }

        self.storage.write().unwrap().insert(normalized.clone(), data.to_vec());
        self.update_metadata(&normalized, false, data.len() as u64);

        Ok(())
    }

    /// **Functor Mapping**: read: path → data
    ///
    /// Preserves identity: read(write(path, data)) = data
    async fn read(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        let normalized = Self::normalize_path(path);

        self.storage
            .read()
            .unwrap()
            .get(&normalized)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
    }

    /// **Functor Mapping**: exists: path → bool
    async fn exists(&self, path: &str) -> Result<bool, StorageError> {
        let normalized = Self::normalize_path(path);
        Ok(self.storage.read().unwrap().contains_key(&normalized)
            || self.directories.read().unwrap().contains_key(&normalized))
    }

    /// **Functor Mapping**: delete: path → ()
    ///
    /// Preserves composition: delete ∘ write = id (no-op)
    async fn delete(&self, path: &str) -> Result<(), StorageError> {
        let normalized = Self::normalize_path(path);

        self.storage
            .write()
            .unwrap()
            .remove(&normalized)
            .ok_or_else(|| StorageError::NotFound(path.to_string()))?;

        self.metadata.write().unwrap().remove(&normalized);

        Ok(())
    }

    /// **Functor Mapping**: list_dir: path → [paths]
    async fn list_dir(&self, path: &str) -> Result<Vec<String>, StorageError> {
        let normalized = Self::normalize_path(path);

        self.directories
            .read()
            .unwrap()
            .get(&normalized)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
    }

    /// **Functor Mapping**: create_dir: path → ()
    async fn create_dir(&self, path: &str) -> Result<(), StorageError> {
        let normalized = Self::normalize_path(path);

        if self.directories.read().unwrap().contains_key(&normalized) {
            return Err(StorageError::AlreadyExists(path.to_string()));
        }

        // Ensure parent exists
        if let Some(parent) = Self::parent_path(&normalized) {
            if !parent.is_empty() {
                let dirs = self.directories.read().unwrap();
                if !dirs.contains_key(&parent) {
                    return Err(StorageError::NotFound(parent));
                }
            }
        }

        self.directories.write().unwrap().insert(normalized.clone(), Vec::new());
        self.update_metadata(&normalized, true, 0);

        // Update parent directory's children
        if let Some(parent) = Self::parent_path(&normalized) {
            if let Some(children) = self.directories.write().unwrap().get_mut(&parent) {
                children.push(normalized.clone());
            }
        }

        Ok(())
    }

    /// **Functor Mapping**: create_dir_all: path → ()
    ///
    /// Creates entire path hierarchy
    async fn create_dir_all(&self, path: &str) -> Result<(), StorageError> {
        let normalized = Self::normalize_path(path);
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();

        let mut current = String::new();
        for part in parts {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(part);

            if !self.directories.read().unwrap().contains_key(&current) {
                self.directories.write().unwrap().insert(current.clone(), Vec::new());
                self.update_metadata(&current, true, 0);
            }
        }

        Ok(())
    }

    /// **Functor Mapping**: remove_dir_all: path → ()
    async fn remove_dir_all(&self, path: &str) -> Result<(), StorageError> {
        let normalized = Self::normalize_path(path);

        // Get all children recursively
        fn collect_children(
            path: &str,
            dirs: &HashMap<String, Vec<String>>,
            storage: &HashMap<String, Vec<u8>>,
        ) -> Vec<String> {
            let mut result = vec![path.to_string()];

            if let Some(children) = dirs.get(path) {
                for child in children {
                    result.extend(collect_children(child, dirs, storage));
                }
            }

            result
        }

        let dirs_guard = self.directories.read().unwrap();
        let storage_guard = self.storage.read().unwrap();
        let to_remove = collect_children(&normalized, &dirs_guard, &storage_guard);
        drop(dirs_guard);
        drop(storage_guard);

        // Remove all collected paths
        for p in to_remove {
            self.directories.write().unwrap().remove(&p);
            self.storage.write().unwrap().remove(&p);
            self.metadata.write().unwrap().remove(&p);
        }

        Ok(())
    }

    /// **Functor Mapping**: metadata: path → StorageMetadata
    async fn metadata(&self, path: &str) -> Result<StorageMetadata, StorageError> {
        let normalized = Self::normalize_path(path);

        self.metadata
            .read()
            .unwrap()
            .get(&normalized)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
    }

    /// **Functor Mapping**: sync: path → ()
    ///
    /// In-memory storage is always synced (no-op)
    async fn sync(&self, _path: &str) -> Result<(), StorageError> {
        // In-memory storage is always synced
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_functor_identity_law() {
        // F(id) = id
        let adapter = InMemoryStorageAdapter::new();

        // Identity operation: write then read
        let path = "test.txt";
        let data = b"hello world";

        adapter.write(path, data).await.unwrap();
        let result = adapter.read(path).await.unwrap();

        assert_eq!(data, result.as_slice());
    }

    #[tokio::test]
    async fn test_functor_composition_law() {
        // F(g ∘ f) = F(g) ∘ F(f)
        let adapter = InMemoryStorageAdapter::new();

        // Composition: write -> write -> read should preserve the last write
        let path = "test.txt";
        let data1 = b"first";
        let data2 = b"second";

        adapter.write(path, data1).await.unwrap();
        adapter.write(path, data2).await.unwrap();
        let result = adapter.read(path).await.unwrap();

        assert_eq!(data2, result.as_slice());
    }

    #[tokio::test]
    async fn test_directory_operations() {
        let adapter = InMemoryStorageAdapter::new();

        adapter.create_dir_all("a/b/c").await.unwrap();

        let exists = adapter.exists("a/b/c").await.unwrap();
        assert!(exists);

        adapter.remove_dir_all("a").await.unwrap();

        let exists = adapter.exists("a").await.unwrap();
        assert!(!exists);
    }
}
