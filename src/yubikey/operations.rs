//! YubiKey cryptographic operations
//!
//! This module provides YubiKey signing, encryption, and key generation operations.

use crate::{KeyError, Result};
use crate::types::*;
use crate::yubikey::YubiKeyManager;
use crate::yubikey::piv::YubiKeyPivManager;
use yubikey::{YubiKey, MgmKey, PinPolicy, TouchPolicy};
use yubikey::piv::{SlotId, AlgorithmId};
use sha2::{Sha256, Sha384, Sha512, Digest};
use rsa::{Pkcs1v15Encrypt, RsaPublicKey, BigUint};
use tracing::{debug, info, warn};

/// YubiKey cryptographic operations
pub struct YubiKeyOperations;

impl YubiKeyOperations {
    /// Generate a new key pair on the YubiKey
    pub fn generate_key_pair(
        manager: &YubiKeyManager,
        serial: &str,
        slot: SlotId,
        algorithm: KeyAlgorithm,
        pin: &SecureString,
        touch_required: bool,
    ) -> Result<Vec<u8>> {
        manager.with_yubikey(serial, |yubikey| {
            // Verify PIN first
            yubikey.verify_pin(pin.expose_secret().as_bytes())
                .map_err(|e| {
                    warn!("PIN verification failed");
                    KeyError::YubiKey(e)
                })?;
            
            // Convert algorithm
            let alg_id = YubiKeyPivManager::to_algorithm_id(&algorithm)?;
            
            // Set policies
            let pin_policy = PinPolicy::Once;
            let touch_policy = if touch_required {
                TouchPolicy::Always
            } else {
                TouchPolicy::Never
            };
            
            // Get default management key (or use a custom one)
            let mgm_key = MgmKey::default();
            
            // Generate the key
            let public_key = YubiKeyPivManager::generate_key(
                yubikey,
                slot,
                alg_id,
                pin_policy,
                touch_policy,
                &mgm_key,
            )?;
            
            info!("Generated {:?} key in slot {:?}", algorithm, slot);
            Ok(public_key)
        })
    }
    
    /// Sign data using a YubiKey slot
    pub fn sign_data(
        manager: &YubiKeyManager,
        serial: &str,
        slot: SlotId,
        data: &[u8],
        algorithm: KeyAlgorithm,
        hash_algorithm: HashAlgorithm,
        pin: &SecureString,
    ) -> Result<Vec<u8>> {
        manager.with_yubikey(serial, |yubikey| {
            // Verify PIN
            yubikey.verify_pin(pin.expose_secret().as_bytes())
                .map_err(|e| {
                    warn!("PIN verification failed");
                    KeyError::YubiKey(e)
                })?;
            
            // Hash the data
            let hashed = Self::hash_data(data, hash_algorithm)?;
            
            // Prepare data for signing based on algorithm
            let sign_data = match algorithm {
                KeyAlgorithm::Rsa(_) => {
                    // For RSA, we need to add PKCS#1 v1.5 padding
                    Self::pkcs1_v15_sign_padding(&hashed, hash_algorithm)?
                }
                KeyAlgorithm::Ecdsa(_) => {
                    // For ECDSA, just use the hash
                    hashed
                }
                _ => return Err(KeyError::UnsupportedAlgorithm(
                    format!("Algorithm {:?} not supported for signing", algorithm)
                ))
            };
            
            // Get algorithm ID
            let alg_id = YubiKeyPivManager::to_algorithm_id(&algorithm)?;
            
            // Sign the data
            let signature = YubiKeyPivManager::sign_data(
                yubikey,
                slot,
                &sign_data,
                alg_id,
            )?;
            
            debug!("Successfully signed data with slot {:?}", slot);
            Ok(signature)
        })
    }
    
    /// Decrypt data using a YubiKey slot (RSA only)
    pub fn decrypt_data(
        manager: &YubiKeyManager,
        serial: &str,
        slot: SlotId,
        encrypted_data: &[u8],
        algorithm: KeyAlgorithm,
        pin: &SecureString,
    ) -> Result<Vec<u8>> {
        // Only RSA supports decryption
        match algorithm {
            KeyAlgorithm::Rsa(_) => {},
            _ => return Err(KeyError::UnsupportedAlgorithm(
                "Only RSA keys can perform decryption".to_string()
            ))
        }
        
        manager.with_yubikey(serial, |yubikey| {
            // Verify PIN
            yubikey.verify_pin(pin.expose_secret().as_bytes())
                .map_err(|e| {
                    warn!("PIN verification failed");
                    KeyError::YubiKey(e)
                })?;
            
            // Get algorithm ID
            let alg_id = YubiKeyPivManager::to_algorithm_id(&algorithm)?;
            
            // Decrypt the data
            let decrypted = YubiKeyPivManager::decrypt_data(
                yubikey,
                slot,
                encrypted_data,
                alg_id,
            )?;
            
            debug!("Successfully decrypted data with slot {:?}", slot);
            Ok(decrypted)
        })
    }
    
    /// Encrypt data using the public key from a certificate
    pub fn encrypt_data(
        manager: &YubiKeyManager,
        serial: &str,
        slot: SlotId,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        manager.with_yubikey(serial, |yubikey| {
            // Read certificate from slot
            let cert_der = YubiKeyPivManager::read_certificate(yubikey, slot)?;
            
            // Parse certificate to extract public key
            let (_, cert) = x509_parser::parse_x509_certificate(&cert_der)
                .map_err(|e| KeyError::X509(format!("Failed to parse certificate: {:?}", e)))?;
            
            // Extract public key
            let public_key = cert.public_key();
            
            // Only RSA encryption is supported
            match public_key.algorithm {
                x509_parser::oid_registry::OID_PKCS1_RSAENCRYPTION => {
                    // Extract RSA public key components
                    let rsa_key = public_key.subject_public_key.data;
                    
                    // Parse RSA public key (this is simplified - real implementation would need proper ASN.1 parsing)
                    // For now, return an error as we'd need additional dependencies
                    Err(KeyError::Other("RSA public key extraction not fully implemented".to_string()))
                }
                _ => Err(KeyError::UnsupportedAlgorithm(
                    "Only RSA keys can perform encryption".to_string()
                ))
            }
        })
    }
    
    /// Get certificate from a slot
    pub fn get_certificate(
        manager: &YubiKeyManager,
        serial: &str,
        slot: SlotId,
    ) -> Result<Vec<u8>> {
        manager.with_yubikey(serial, |yubikey| {
            YubiKeyPivManager::read_certificate(yubikey, slot)
        })
    }
    
    /// Store certificate in a slot
    pub fn store_certificate(
        manager: &YubiKeyManager,
        serial: &str,
        slot: SlotId,
        certificate: &[u8],
        mgm_key: Option<&[u8]>,
    ) -> Result<()> {
        manager.with_yubikey(serial, |yubikey| {
            // Use provided management key or default
            let mgm = if let Some(key_data) = mgm_key {
                MgmKey::from_bytes(key_data)
                    .map_err(|e| KeyError::Other(format!("Invalid management key: {}", e)))?
            } else {
                MgmKey::default()
            };
            
            YubiKeyPivManager::write_certificate(
                yubikey,
                slot,
                certificate,
                &mgm,
            )
        })
    }
    
    /// Get attestation certificate for a slot
    pub fn get_attestation(
        manager: &YubiKeyManager,
        serial: &str,
        slot: SlotId,
    ) -> Result<Vec<u8>> {
        manager.with_yubikey(serial, |yubikey| {
            YubiKeyPivManager::get_attestation(yubikey, slot)
        })
    }
    
    /// Hash data using the specified algorithm
    fn hash_data(data: &[u8], algorithm: HashAlgorithm) -> Result<Vec<u8>> {
        let hash = match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Sha384 => {
                let mut hasher = Sha384::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            _ => return Err(KeyError::UnsupportedAlgorithm(
                format!("Hash algorithm {:?} not supported", algorithm)
            ))
        };
        
        Ok(hash)
    }
    
    /// Add PKCS#1 v1.5 signature padding
    fn pkcs1_v15_sign_padding(hash: &[u8], hash_alg: HashAlgorithm) -> Result<Vec<u8>> {
        // DigestInfo encoding for different hash algorithms
        let digest_info_prefix = match hash_alg {
            HashAlgorithm::Sha256 => vec![
                0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86,
                0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01, 0x05,
                0x00, 0x04, 0x20
            ],
            HashAlgorithm::Sha384 => vec![
                0x30, 0x41, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86,
                0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02, 0x05,
                0x00, 0x04, 0x30
            ],
            HashAlgorithm::Sha512 => vec![
                0x30, 0x51, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86,
                0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03, 0x05,
                0x00, 0x04, 0x40
            ],
            _ => return Err(KeyError::UnsupportedAlgorithm(
                format!("Hash algorithm {:?} not supported for RSA signing", hash_alg)
            ))
        };
        
        let mut padded = digest_info_prefix;
        padded.extend_from_slice(hash);
        
        Ok(padded)
    }
    
    /// Verify a signature using the public key from a certificate
    pub fn verify_signature(
        manager: &YubiKeyManager,
        serial: &str,
        slot: SlotId,
        data: &[u8],
        signature: &[u8],
        hash_algorithm: HashAlgorithm,
    ) -> Result<bool> {
        manager.with_yubikey(serial, |yubikey| {
            // Read certificate from slot
            let cert_der = YubiKeyPivManager::read_certificate(yubikey, slot)?;
            
            // Parse certificate
            let (_, cert) = x509_parser::parse_x509_certificate(&cert_der)
                .map_err(|e| KeyError::X509(format!("Failed to parse certificate: {:?}", e)))?;
            
            // Hash the data
            let hashed = Self::hash_data(data, hash_algorithm)?;
            
            // Verify based on algorithm
            match cert.public_key().algorithm {
                x509_parser::oid_registry::OID_PKCS1_RSAENCRYPTION => {
                    // RSA signature verification would require parsing the public key
                    // and using an RSA library - for now, return an error
                    Err(KeyError::Other("RSA signature verification not fully implemented".to_string()))
                }
                x509_parser::oid_registry::OID_EC_PUBLIC_KEY => {
                    // ECDSA signature verification would require parsing the public key
                    // and using an ECDSA library - for now, return an error
                    Err(KeyError::Other("ECDSA signature verification not fully implemented".to_string()))
                }
                _ => Err(KeyError::UnsupportedAlgorithm(
                    "Unsupported public key algorithm".to_string()
                ))
            }
        })
    }
}

/// Helper to convert slot name to SlotId
pub fn slot_from_name(name: &str) -> Result<SlotId> {
    match name.to_lowercase().as_str() {
        "authentication" | "9a" => Ok(SlotId::Authentication),
        "signature" | "9c" => Ok(SlotId::Signature),
        "key-management" | "key_management" | "9d" => Ok(SlotId::KeyManagement),
        "card-authentication" | "card_authentication" | "9e" => Ok(SlotId::CardAuthentication),
        "attestation" | "f9" => Ok(SlotId::Attestation),
        _ => {
            // Try to parse as retired slot number
            if let Some(num_str) = name.strip_prefix("retired-").or_else(|| name.strip_prefix("retired_")) {
                if let Ok(num) = num_str.parse::<u8>() {
                    if (1..=20).contains(&num) {
                        return Ok(SlotId::Retired(
                            yubikey::piv::RetiredSlotId::try_from(num)
                                .map_err(|_| KeyError::Other(format!("Invalid retired slot number: {}", num)))?
                        ));
                    }
                }
            }
            Err(KeyError::Other(format!("Unknown slot name: {}", name)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_slot_from_name() {
        // Test standard slots
        assert!(matches!(slot_from_name("authentication"), Ok(SlotId::Authentication)));
        assert!(matches!(slot_from_name("SIGNATURE"), Ok(SlotId::Signature)));
        assert!(matches!(slot_from_name("key-management"), Ok(SlotId::KeyManagement)));
        assert!(matches!(slot_from_name("9a"), Ok(SlotId::Authentication)));
        
        // Test retired slots
        assert!(matches!(slot_from_name("retired-1"), Ok(SlotId::Retired(_))));
        assert!(matches!(slot_from_name("retired_5"), Ok(SlotId::Retired(_))));
        
        // Test invalid names
        assert!(slot_from_name("invalid").is_err());
        assert!(slot_from_name("retired-25").is_err());
    }
    
    #[test]
    fn test_hash_data() {
        let data = b"Hello, YubiKey!";
        
        // Test SHA-256
        let hash = YubiKeyOperations::hash_data(data, HashAlgorithm::Sha256).unwrap();
        assert_eq!(hash.len(), 32);
        
        // Test SHA-384
        let hash = YubiKeyOperations::hash_data(data, HashAlgorithm::Sha384).unwrap();
        assert_eq!(hash.len(), 48);
        
        // Test SHA-512
        let hash = YubiKeyOperations::hash_data(data, HashAlgorithm::Sha512).unwrap();
        assert_eq!(hash.len(), 64);
    }
    
    #[test]
    fn test_pkcs1_padding() {
        let hash = vec![0u8; 32]; // 32-byte hash for SHA-256
        
        let padded = YubiKeyOperations::pkcs1_v15_sign_padding(&hash, HashAlgorithm::Sha256).unwrap();
        
        // Check that padding was added
        assert!(padded.len() > hash.len());
        
        // Check DigestInfo structure is present
        assert_eq!(&padded[0..2], &[0x30, 0x31]); // SEQUENCE, length 49
    }
}