//! Infrastructure Layer 1.3: Key Lifecycle Tests for cim-keys
//! 
//! User Story: As a security system, I need to manage the complete lifecycle of cryptographic keys
//!
//! Test Requirements:
//! - Verify key rotation
//! - Verify key expiration
//! - Verify key revocation
//! - Verify key archival
//!
//! Event Sequence:
//! 1. KeyRotationScheduled { key_id, rotation_date }
//! 2. NewKeyGenerated { old_key_id, new_key_id }
//! 3. KeyExpired { key_id, expiration_date }
//! 4. KeyRevoked { key_id, reason, revocation_date }
//! 5. KeyArchived { key_id, archive_location }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Schedule Rotation]
//!     B --> C[KeyRotationScheduled]
//!     C --> D[Generate New Key]
//!     D --> E[NewKeyGenerated]
//!     E --> F[Check Expiration]
//!     F --> G{Expired?}
//!     G -->|Yes| H[KeyExpired]
//!     G -->|No| I[Revoke Key]
//!     H --> J[Archive Key]
//!     I --> K[KeyRevoked]
//!     K --> J
//!     J --> L[KeyArchived]
//!     L --> M[Test Success]
//! ```

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Key lifecycle states
#[derive(Debug, Clone, PartialEq)]
pub enum KeyState {
    Active,
    Scheduled,
    Expired,
    Revoked { reason: String },
    Archived,
}

/// Key lifecycle events
#[derive(Debug, Clone, PartialEq)]
pub enum KeyLifecycleEvent {
    KeyRotationScheduled { key_id: String, rotation_date: SystemTime },
    NewKeyGenerated { old_key_id: String, new_key_id: String },
    KeyExpired { key_id: String, expiration_date: SystemTime },
    KeyRevoked { key_id: String, reason: String, revocation_date: SystemTime },
    KeyArchived { key_id: String, archive_location: String },
}

/// Key lifecycle manager
pub struct KeyLifecycleManager {
    keys: HashMap<String, KeyInfo>,
    rotation_schedule: HashMap<String, SystemTime>,
}

#[derive(Debug, Clone)]
struct KeyInfo {
    key_id: String,
    state: KeyState,
    created_at: SystemTime,
    expires_at: Option<SystemTime>,
    metadata: HashMap<String, String>,
}

impl KeyLifecycleManager {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            rotation_schedule: HashMap::new(),
        }
    }

    pub fn register_key(&mut self, key_id: String, validity_days: Option<u64>) {
        let created_at = SystemTime::now();
        let expires_at = validity_days.map(|days| 
            created_at + Duration::from_secs(days * 24 * 60 * 60)
        );

        let key_info = KeyInfo {
            key_id: key_id.clone(),
            state: KeyState::Active,
            created_at,
            expires_at,
            metadata: HashMap::new(),
        };

        self.keys.insert(key_id, key_info);
    }

    pub fn schedule_rotation(&mut self, key_id: String, rotation_date: SystemTime) -> Result<(), String> {
        if !self.keys.contains_key(&key_id) {
            return Err("Key not found".to_string());
        }

        if let Some(info) = self.keys.get_mut(&key_id) {
            info.state = KeyState::Scheduled;
        }

        self.rotation_schedule.insert(key_id, rotation_date);
        Ok(())
    }

    pub fn rotate_key(&mut self, old_key_id: String) -> Result<String, String> {
        if !self.keys.contains_key(&old_key_id) {
            return Err("Key not found".to_string());
        }

        // Generate new key ID
        let new_key_id = format!("{}_rotated_{}", old_key_id, 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        // Register new key with same validity period as old key
        let old_info = self.keys.get(&old_key_id).unwrap();
        let validity_days = old_info.expires_at.map(|exp| {
            exp.duration_since(old_info.created_at)
                .unwrap_or_default()
                .as_secs() / (24 * 60 * 60)
        });

        self.register_key(new_key_id.clone(), validity_days);

        // Mark old key as scheduled for revocation
        if let Some(info) = self.keys.get_mut(&old_key_id) {
            info.state = KeyState::Scheduled;
        }

        Ok(new_key_id)
    }

    pub fn check_expiration(&mut self, key_id: &str) -> Result<bool, String> {
        let info = self.keys.get_mut(key_id)
            .ok_or_else(|| "Key not found".to_string())?;

        if let Some(expires_at) = info.expires_at {
            if SystemTime::now() > expires_at {
                info.state = KeyState::Expired;
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn revoke_key(&mut self, key_id: String, reason: String) -> Result<(), String> {
        let info = self.keys.get_mut(&key_id)
            .ok_or_else(|| "Key not found".to_string())?;

        info.state = KeyState::Revoked { reason };
        Ok(())
    }

    pub fn get_key_state(&self, key_id: &str) -> Option<KeyState> {
        self.keys.get(key_id).map(|info| info.state.clone())
    }
}

/// Key archival service
pub struct KeyArchivalService {
    archived_keys: HashMap<String, ArchivedKey>,
}

#[derive(Debug, Clone)]
struct ArchivedKey {
    key_id: String,
    archived_at: SystemTime,
    archive_location: String,
    metadata: HashMap<String, String>,
}

impl KeyArchivalService {
    pub fn new() -> Self {
        Self {
            archived_keys: HashMap::new(),
        }
    }

    pub async fn archive_key(&mut self, key_id: String, _key_data: Vec<u8>) -> Result<String, String> {
        // Simulate archival process
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Generate archive location
        let archive_location = format!("archive://keys/{}/{}", 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() / (365 * 24 * 60 * 60), // Year
            key_id
        );

        let archived_key = ArchivedKey {
            key_id: key_id.clone(),
            archived_at: SystemTime::now(),
            archive_location: archive_location.clone(),
            metadata: HashMap::new(),
        };

        self.archived_keys.insert(key_id, archived_key);
        Ok(archive_location)
    }

    pub fn is_archived(&self, key_id: &str) -> bool {
        self.archived_keys.contains_key(key_id)
    }

    pub fn get_archive_location(&self, key_id: &str) -> Option<String> {
        self.archived_keys.get(key_id)
            .map(|ak| ak.archive_location.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_registration() {
        // Arrange
        let mut manager = KeyLifecycleManager::new();
        let key_id = "test_key_123".to_string();

        // Act
        manager.register_key(key_id.clone(), Some(365)); // 1 year validity

        // Assert
        assert_eq!(manager.get_key_state(&key_id), Some(KeyState::Active));
    }

    #[test]
    fn test_key_rotation_scheduling() {
        // Arrange
        let mut manager = KeyLifecycleManager::new();
        let key_id = "rotate_key_456".to_string();
        manager.register_key(key_id.clone(), Some(90));

        let rotation_date = SystemTime::now() + Duration::from_secs(30 * 24 * 60 * 60); // 30 days

        // Act
        let result = manager.schedule_rotation(key_id.clone(), rotation_date);

        // Assert
        assert!(result.is_ok());
        assert_eq!(manager.get_key_state(&key_id), Some(KeyState::Scheduled));
    }

    #[test]
    fn test_key_rotation() {
        // Arrange
        let mut manager = KeyLifecycleManager::new();
        let old_key_id = "old_key_789".to_string();
        manager.register_key(old_key_id.clone(), Some(180));

        // Act
        let new_key_id = manager.rotate_key(old_key_id.clone()).unwrap();

        // Assert
        assert!(new_key_id.contains("_rotated_"));
        assert_eq!(manager.get_key_state(&new_key_id), Some(KeyState::Active));
        assert_eq!(manager.get_key_state(&old_key_id), Some(KeyState::Scheduled));
    }

    #[test]
    fn test_key_expiration() {
        // Arrange
        let mut manager = KeyLifecycleManager::new();
        let key_id = "expire_key_111".to_string();
        
        // Register key with already expired date
        manager.keys.insert(key_id.clone(), KeyInfo {
            key_id: key_id.clone(),
            state: KeyState::Active,
            created_at: SystemTime::now() - Duration::from_secs(2 * 24 * 60 * 60),
            expires_at: Some(SystemTime::now() - Duration::from_secs(24 * 60 * 60)),
            metadata: HashMap::new(),
        });

        // Act
        let is_expired = manager.check_expiration(&key_id).unwrap();

        // Assert
        assert!(is_expired);
        assert_eq!(manager.get_key_state(&key_id), Some(KeyState::Expired));
    }

    #[test]
    fn test_key_revocation() {
        // Arrange
        let mut manager = KeyLifecycleManager::new();
        let key_id = "revoke_key_222".to_string();
        manager.register_key(key_id.clone(), Some(365));

        // Act
        let result = manager.revoke_key(key_id.clone(), "Compromised".to_string());

        // Assert
        assert!(result.is_ok());
        assert_eq!(
            manager.get_key_state(&key_id), 
            Some(KeyState::Revoked { reason: "Compromised".to_string() })
        );
    }

    #[tokio::test]
    async fn test_key_archival() {
        // Arrange
        let mut archival = KeyArchivalService::new();
        let key_id = "archive_key_333".to_string();
        let key_data = vec![0x42; 32];

        // Act
        let archive_location = archival.archive_key(key_id.clone(), key_data).await.unwrap();

        // Assert
        assert!(archive_location.starts_with("archive://keys/"));
        assert!(archival.is_archived(&key_id));
        assert_eq!(archival.get_archive_location(&key_id), Some(archive_location));
    }

    #[test]
    fn test_nonexistent_key_operations() {
        // Arrange
        let mut manager = KeyLifecycleManager::new();
        let key_id = "nonexistent_key".to_string();

        // Act & Assert
        assert!(manager.schedule_rotation(key_id.clone(), SystemTime::now()).is_err());
        assert!(manager.rotate_key(key_id.clone()).is_err());
        assert!(manager.check_expiration(&key_id).is_err());
        assert!(manager.revoke_key(key_id.clone(), "Test".to_string()).is_err());
    }

    #[tokio::test]
    async fn test_full_key_lifecycle() {
        // Arrange
        let mut manager = KeyLifecycleManager::new();
        let mut archival = KeyArchivalService::new();
        let key_id = "lifecycle_key_444".to_string();

        // Act
        // 1. Register key
        manager.register_key(key_id.clone(), Some(30)); // 30 days validity

        // 2. Schedule rotation
        let rotation_date = SystemTime::now() + Duration::from_secs(20 * 24 * 60 * 60);
        manager.schedule_rotation(key_id.clone(), rotation_date).unwrap();

        // 3. Rotate key
        let new_key_id = manager.rotate_key(key_id.clone()).unwrap();

        // 4. Revoke old key
        manager.revoke_key(key_id.clone(), "Rotated".to_string()).unwrap();

        // 5. Archive old key
        let archive_location = archival.archive_key(key_id.clone(), vec![0x00; 32]).await.unwrap();

        // Assert
        assert_eq!(manager.get_key_state(&new_key_id), Some(KeyState::Active));
        assert_eq!(
            manager.get_key_state(&key_id), 
            Some(KeyState::Revoked { reason: "Rotated".to_string() })
        );
        assert!(archival.is_archived(&key_id));
        assert!(archive_location.contains(&key_id));
    }
} 