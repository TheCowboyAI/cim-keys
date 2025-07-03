//! Infrastructure Layer 1.2: Key Generation Tests for cim-keys
//! 
//! User Story: As a security system, I need to generate various types of cryptographic keys
//!
//! Test Requirements:
//! - Verify RSA key generation
//! - Verify Ed25519 key generation
//! - Verify ECDSA key generation
//! - Verify key metadata creation
//!
//! Event Sequence:
//! 1. KeyGenerationRequested { key_type, key_size }
//! 2. KeyGenerated { key_id, key_type, fingerprint }
//! 3. KeyMetadataCreated { key_id, metadata }
//! 4. KeyStoredSecurely { key_id, storage_backend }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Request Key Generation]
//!     B --> C[KeyGenerationRequested]
//!     C --> D[Generate Key]
//!     D --> E[KeyGenerated]
//!     E --> F[Create Metadata]
//!     F --> G[KeyMetadataCreated]
//!     G --> H[Store Key]
//!     H --> I[KeyStoredSecurely]
//!     I --> J[Test Success]
//! ```

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Cryptographic key types
#[derive(Debug, Clone, PartialEq)]
pub enum KeyType {
    RSA { bits: u32 },
    Ed25519,
    ECDSA { curve: String },
}

/// Key generation events
#[derive(Debug, Clone, PartialEq)]
pub enum KeyGenerationEvent {
    KeyGenerationRequested { key_type: KeyType, purpose: String },
    KeyGenerated { key_id: String, key_type: KeyType, fingerprint: String },
    KeyMetadataCreated { key_id: String, metadata: HashMap<String, String> },
    KeyStoredSecurely { key_id: String, storage_backend: String },
    KeyGenerationFailed { key_type: KeyType, error: String },
}

/// Mock key generator
pub struct MockKeyGenerator {
    generated_keys: HashMap<String, (KeyType, Vec<u8>)>,
}

impl MockKeyGenerator {
    pub fn new() -> Self {
        Self {
            generated_keys: HashMap::new(),
        }
    }

    pub async fn generate_key(&mut self, key_type: KeyType, purpose: String) -> Result<String, String> {
        // Simulate key generation time
        match &key_type {
            KeyType::RSA { bits } => {
                if *bits < 2048 {
                    return Err("RSA key size must be at least 2048 bits".to_string());
                }
                tokio::time::sleep(Duration::from_millis(50)).await; // RSA is slower
            }
            KeyType::Ed25519 => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            KeyType::ECDSA { curve } => {
                if !["P-256", "P-384", "P-521"].contains(&curve.as_str()) {
                    return Err(format!("Unsupported ECDSA curve: {curve}"));
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }

        // Generate mock key ID
        let key_id = format!("key_{}_{}_{}", 
            match &key_type {
                KeyType::RSA { .. } => "rsa",
                KeyType::Ed25519 => "ed25519",
                KeyType::ECDSA { .. } => "ecdsa",
            },
            purpose.to_lowercase().replace(' ', "_"),
            self.generated_keys.len()
        );

        // Generate mock key data
        let key_data = match &key_type {
            KeyType::RSA { bits } => vec![0x52; (*bits / 8) as usize], // 'R' = 0x52
            KeyType::Ed25519 => vec![0xED; 32],
            KeyType::ECDSA { .. } => vec![0xEC; 64],
        };

        self.generated_keys.insert(key_id.clone(), (key_type, key_data));
        Ok(key_id)
    }

    pub fn calculate_fingerprint(&self, key_id: &str) -> Option<String> {
        self.generated_keys.get(key_id).map(|(_key_type, data)| {
            // Mock fingerprint calculation
            let hash = data.iter().fold(0u64, |acc, &b| {
                acc.wrapping_mul(31).wrapping_add(b as u64)
            });
            format!("{:016X}", hash)
        })
    }

    pub fn get_key_type(&self, key_id: &str) -> Option<KeyType> {
        self.generated_keys.get(key_id).map(|(kt, _)| kt.clone())
    }
}

/// Key metadata manager
pub struct KeyMetadataManager {
    metadata: HashMap<String, HashMap<String, String>>,
}

impl KeyMetadataManager {
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
        }
    }

    pub fn create_metadata(&mut self, key_id: String, key_type: &KeyType, purpose: String) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        
        meta.insert("created_at".to_string(), 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string());
        
        meta.insert("purpose".to_string(), purpose);
        
        meta.insert("algorithm".to_string(), match key_type {
            KeyType::RSA { bits } => format!("RSA-{bits}"),
            KeyType::Ed25519 => "Ed25519".to_string(),
            KeyType::ECDSA { curve } => format!("ECDSA-{curve}"),
        });

        meta.insert("key_id".to_string(), key_id.clone());
        
        self.metadata.insert(key_id, meta.clone());
        meta
    }

    pub fn get_metadata(&self, key_id: &str) -> Option<&HashMap<String, String>> {
        self.metadata.get(key_id)
    }
}

/// Secure key storage
pub struct SecureKeyStorage {
    stored_keys: HashMap<String, (String, Vec<u8>)>, // key_id -> (backend, encrypted_key)
    backend: String,
}

impl SecureKeyStorage {
    pub fn new(backend: String) -> Self {
        Self {
            stored_keys: HashMap::new(),
            backend,
        }
    }

    pub async fn store_key(&mut self, key_id: String, key_data: Vec<u8>) -> Result<(), String> {
        // Simulate encryption and storage
        tokio::time::sleep(Duration::from_millis(5)).await;
        
        // Mock encryption
        let encrypted = key_data.iter().map(|b| b.wrapping_add(1)).collect();
        
        self.stored_keys.insert(key_id, (self.backend.clone(), encrypted));
        Ok(())
    }

    pub fn is_key_stored(&self, key_id: &str) -> bool {
        self.stored_keys.contains_key(key_id)
    }

    pub fn get_storage_backend(&self) -> &str {
        &self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rsa_key_generation() {
        // Arrange
        let mut generator = MockKeyGenerator::new();
        let key_type = KeyType::RSA { bits: 4096 };
        let purpose = "TLS Certificate".to_string();

        // Act
        let key_id = generator.generate_key(key_type.clone(), purpose.clone()).await.unwrap();
        let fingerprint = generator.calculate_fingerprint(&key_id).unwrap();

        // Assert
        assert!(key_id.starts_with("key_rsa_"));
        assert_eq!(fingerprint.len(), 16); // 16 hex chars
        assert_eq!(generator.get_key_type(&key_id).unwrap(), key_type);
    }

    #[tokio::test]
    async fn test_ed25519_key_generation() {
        // Arrange
        let mut generator = MockKeyGenerator::new();
        let key_type = KeyType::Ed25519;
        let purpose = "SSH Authentication".to_string();

        // Act
        let key_id = generator.generate_key(key_type.clone(), purpose).await.unwrap();
        let fingerprint = generator.calculate_fingerprint(&key_id).unwrap();

        // Assert
        assert!(key_id.starts_with("key_ed25519_"));
        assert_eq!(fingerprint.len(), 16);
        assert_eq!(generator.get_key_type(&key_id).unwrap(), key_type);
    }

    #[tokio::test]
    async fn test_ecdsa_key_generation() {
        // Arrange
        let mut generator = MockKeyGenerator::new();
        let key_type = KeyType::ECDSA { curve: "P-256".to_string() };
        let purpose = "Code Signing".to_string();

        // Act
        let key_id = generator.generate_key(key_type.clone(), purpose).await.unwrap();
        let fingerprint = generator.calculate_fingerprint(&key_id).unwrap();

        // Assert
        assert!(key_id.starts_with("key_ecdsa_"));
        assert_eq!(fingerprint.len(), 16);
        assert_eq!(generator.get_key_type(&key_id).unwrap(), key_type);
    }

    #[tokio::test]
    async fn test_key_metadata_creation() {
        // Arrange
        let mut metadata_mgr = KeyMetadataManager::new();
        let key_id = "key_rsa_test_0".to_string();
        let key_type = KeyType::RSA { bits: 2048 };
        let purpose = "Test Key".to_string();

        // Act
        let metadata = metadata_mgr.create_metadata(key_id.clone(), &key_type, purpose.clone());

        // Assert
        assert_eq!(metadata.get("purpose").unwrap(), &purpose);
        assert_eq!(metadata.get("algorithm").unwrap(), "RSA-2048");
        assert_eq!(metadata.get("key_id").unwrap(), &key_id);
        assert!(metadata.contains_key("created_at"));
    }

    #[tokio::test]
    async fn test_secure_key_storage() {
        // Arrange
        let mut storage = SecureKeyStorage::new("FileSystem".to_string());
        let key_id = "key_test_123".to_string();
        let key_data = vec![0x42; 32];

        // Act
        storage.store_key(key_id.clone(), key_data).await.unwrap();

        // Assert
        assert!(storage.is_key_stored(&key_id));
        assert_eq!(storage.get_storage_backend(), "FileSystem");
    }

    #[tokio::test]
    async fn test_invalid_rsa_key_size() {
        // Arrange
        let mut generator = MockKeyGenerator::new();
        let key_type = KeyType::RSA { bits: 1024 }; // Too small

        // Act
        let result = generator.generate_key(key_type, "Test".to_string()).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 2048 bits"));
    }

    #[tokio::test]
    async fn test_invalid_ecdsa_curve() {
        // Arrange
        let mut generator = MockKeyGenerator::new();
        let key_type = KeyType::ECDSA { curve: "InvalidCurve".to_string() };

        // Act
        let result = generator.generate_key(key_type, "Test".to_string()).await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported ECDSA curve"));
    }

    #[tokio::test]
    async fn test_full_key_generation_flow() {
        // Arrange
        let mut generator = MockKeyGenerator::new();
        let mut metadata_mgr = KeyMetadataManager::new();
        let mut storage = SecureKeyStorage::new("YubiKey".to_string());
        
        let key_type = KeyType::Ed25519;
        let purpose = "Email Encryption".to_string();

        // Act
        // 1. Generate key
        let key_id = generator.generate_key(key_type.clone(), purpose.clone()).await.unwrap();
        
        // 2. Get fingerprint
        let fingerprint = generator.calculate_fingerprint(&key_id).unwrap();
        
        // 3. Create metadata
        let metadata = metadata_mgr.create_metadata(key_id.clone(), &key_type, purpose);
        
        // 4. Store key
        let key_data = vec![0xED; 32]; // Mock key data
        storage.store_key(key_id.clone(), key_data).await.unwrap();

        // Assert
        assert!(key_id.starts_with("key_ed25519_"));
        assert_eq!(fingerprint.len(), 16);
        assert_eq!(metadata.get("algorithm").unwrap(), "Ed25519");
        assert!(storage.is_key_stored(&key_id));
        assert_eq!(storage.get_storage_backend(), "YubiKey");
    }
} 