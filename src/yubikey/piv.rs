//! YubiKey PIV (Personal Identity Verification) operations
//!
//! This module provides YubiKey PIV slot management, certificate operations,
//! and key management functionality.

use crate::{KeyError, Result};
use crate::types::*;
use crate::traits::*;
use async_trait::async_trait;
use yubikey::{YubiKey, Serial, MgmKey, PinPolicy, TouchPolicy};
use yubikey::piv::{self, SlotId, AlgorithmId, Key, RetiredSlotId};
use std::convert::TryFrom;
use tracing::{debug, info, warn};
use x509_parser::prelude::*;

/// PIV slot information
#[derive(Debug, Clone)]
pub struct PivSlotInfo {
    /// Slot identifier
    pub slot: SlotId,
    /// Slot name/description
    pub name: String,
    /// Whether the slot contains a key
    pub has_key: bool,
    /// Whether the slot contains a certificate
    pub has_certificate: bool,
    /// Key algorithm if present
    pub algorithm: Option<AlgorithmId>,
    /// Certificate subject if present
    pub certificate_subject: Option<String>,
}

/// YubiKey PIV manager for slot operations
pub struct YubiKeyPivManager;

impl YubiKeyPivManager {
    /// List all PIV slots and their status
    pub fn list_slots(yubikey: &mut YubiKey) -> Result<Vec<PivSlotInfo>> {
        let mut slots = Vec::new();
        
        // Standard PIV slots
        let standard_slots = vec![
            (SlotId::Authentication, "PIV Authentication"),
            (SlotId::Signature, "Digital Signature"),
            (SlotId::KeyManagement, "Key Management"),
            (SlotId::CardAuthentication, "Card Authentication"),
        ];
        
        for (slot_id, name) in standard_slots {
            let info = Self::get_slot_info(yubikey, slot_id, name)?;
            slots.push(info);
        }
        
        // Retired slots (82-95)
        for i in 1..=20 {
            if let Ok(retired_slot) = RetiredSlotId::try_from(i) {
                let slot_id = SlotId::Retired(retired_slot);
                let name = format!("Retired Slot {}", i);
                if let Ok(info) = Self::get_slot_info(yubikey, slot_id, &name) {
                    slots.push(info);
                }
            }
        }
        
        // Attestation slot
        let attestation_info = Self::get_slot_info(
            yubikey, 
            SlotId::Attestation, 
            "Attestation"
        )?;
        slots.push(attestation_info);
        
        Ok(slots)
    }
    
    /// Get information about a specific slot
    fn get_slot_info(yubikey: &mut YubiKey, slot: SlotId, name: &str) -> Result<PivSlotInfo> {
        let mut info = PivSlotInfo {
            slot,
            name: name.to_string(),
            has_key: false,
            has_certificate: false,
            algorithm: None,
            certificate_subject: None,
        };
        
        // Try to read certificate from slot
        match piv::read_certificate(yubikey, slot) {
            Ok(cert_der) => {
                info.has_certificate = true;
                // Parse certificate to get subject
                if let Ok((_, cert)) = X509Certificate::from_der(&cert_der) {
                    info.certificate_subject = Some(cert.subject.to_string());
                }
            }
            Err(_) => {
                // No certificate in slot
            }
        }
        
        // Check for key presence by attempting to get metadata
        // Note: This is a heuristic - there's no direct way to check key presence
        if info.has_certificate {
            // If there's a certificate, assume there's a key
            info.has_key = true;
            // Try to determine algorithm from certificate
            // This is approximate - actual key algorithm may differ
        }
        
        Ok(info)
    }
    
    /// Generate a new key in a PIV slot
    pub fn generate_key(
        yubikey: &mut YubiKey,
        slot: SlotId,
        algorithm: AlgorithmId,
        pin_policy: PinPolicy,
        touch_policy: TouchPolicy,
        mgm_key: &MgmKey,
    ) -> Result<Vec<u8>> {
        info!("Generating {:?} key in slot {:?}", algorithm, slot);
        
        // Generate key
        let public_key = piv::generate(
            yubikey,
            slot,
            algorithm,
            pin_policy,
            touch_policy,
            mgm_key,
        ).map_err(|e| {
            warn!("Failed to generate key in slot {:?}: {}", slot, e);
            KeyError::YubiKey(e)
        })?;
        
        debug!("Successfully generated key in slot {:?}", slot);
        Ok(public_key.to_vec())
    }
    
    /// Import a key into a PIV slot
    pub fn import_key(
        yubikey: &mut YubiKey,
        slot: SlotId,
        key_data: Key,
        pin_policy: PinPolicy,
        touch_policy: TouchPolicy,
        mgm_key: &MgmKey,
    ) -> Result<()> {
        info!("Importing key to slot {:?}", slot);
        
        piv::import_key(
            yubikey,
            slot,
            key_data,
            pin_policy,
            touch_policy,
            mgm_key,
        ).map_err(|e| {
            warn!("Failed to import key to slot {:?}: {}", slot, e);
            KeyError::YubiKey(e)
        })?;
        
        debug!("Successfully imported key to slot {:?}", slot);
        Ok(())
    }
    
    /// Write a certificate to a PIV slot
    pub fn write_certificate(
        yubikey: &mut YubiKey,
        slot: SlotId,
        certificate: &[u8],
        mgm_key: &MgmKey,
    ) -> Result<()> {
        info!("Writing certificate to slot {:?}", slot);
        
        piv::write_certificate(
            yubikey,
            slot,
            certificate,
            mgm_key,
        ).map_err(|e| {
            warn!("Failed to write certificate to slot {:?}: {}", slot, e);
            KeyError::YubiKey(e)
        })?;
        
        debug!("Successfully wrote certificate to slot {:?}", slot);
        Ok(())
    }
    
    /// Read a certificate from a PIV slot
    pub fn read_certificate(
        yubikey: &mut YubiKey,
        slot: SlotId,
    ) -> Result<Vec<u8>> {
        debug!("Reading certificate from slot {:?}", slot);
        
        piv::read_certificate(yubikey, slot)
            .map_err(|e| {
                warn!("Failed to read certificate from slot {:?}: {}", slot, e);
                KeyError::YubiKey(e)
            })
    }
    
    /// Delete a certificate from a PIV slot
    pub fn delete_certificate(
        yubikey: &mut YubiKey,
        slot: SlotId,
        mgm_key: &MgmKey,
    ) -> Result<()> {
        info!("Deleting certificate from slot {:?}", slot);
        
        // Write empty certificate data
        piv::write_certificate(
            yubikey,
            slot,
            &[],
            mgm_key,
        ).map_err(|e| {
            warn!("Failed to delete certificate from slot {:?}: {}", slot, e);
            KeyError::YubiKey(e)
        })?;
        
        debug!("Successfully deleted certificate from slot {:?}", slot);
        Ok(())
    }
    
    /// Perform a signing operation using a PIV slot
    pub fn sign_data(
        yubikey: &mut YubiKey,
        slot: SlotId,
        data: &[u8],
        algorithm: AlgorithmId,
    ) -> Result<Vec<u8>> {
        debug!("Signing data with slot {:?} using {:?}", slot, algorithm);
        
        // For RSA, data should be pre-hashed and padded
        // For ECDSA, data should be pre-hashed
        let signature = piv::sign_data(
            yubikey,
            slot,
            data,
            algorithm,
        ).map_err(|e| {
            warn!("Failed to sign with slot {:?}: {}", slot, e);
            KeyError::YubiKey(e)
        })?;
        
        Ok(signature)
    }
    
    /// Decrypt data using a PIV slot
    pub fn decrypt_data(
        yubikey: &mut YubiKey,
        slot: SlotId,
        encrypted_data: &[u8],
        algorithm: AlgorithmId,
    ) -> Result<Vec<u8>> {
        debug!("Decrypting data with slot {:?}", slot);
        
        // Only RSA keys can decrypt
        match algorithm {
            AlgorithmId::Rsa1024 | AlgorithmId::Rsa2048 => {
                let decrypted = piv::decrypt_data(
                    yubikey,
                    slot,
                    encrypted_data,
                    algorithm,
                ).map_err(|e| {
                    warn!("Failed to decrypt with slot {:?}: {}", slot, e);
                    KeyError::YubiKey(e)
                })?;
                
                Ok(decrypted)
            }
            _ => Err(KeyError::UnsupportedAlgorithm(
                "Only RSA keys can perform decryption".to_string()
            ))
        }
    }
    
    /// Change the management key
    pub fn change_management_key(
        yubikey: &mut YubiKey,
        current_key: &MgmKey,
        new_key: &MgmKey,
    ) -> Result<()> {
        info!("Changing YubiKey management key");
        
        piv::change_mgm_key(yubikey, current_key, new_key)
            .map_err(|e| {
                warn!("Failed to change management key: {}", e);
                KeyError::YubiKey(e)
            })?;
        
        info!("Successfully changed management key");
        Ok(())
    }
    
    /// Set the number of PIN retries
    pub fn set_pin_retries(
        yubikey: &mut YubiKey,
        pin_retries: u8,
        puk_retries: u8,
        mgm_key: &MgmKey,
    ) -> Result<()> {
        info!("Setting PIN retries to {} and PUK retries to {}", pin_retries, puk_retries);
        
        piv::set_pin_retries(yubikey, pin_retries, puk_retries, mgm_key)
            .map_err(|e| {
                warn!("Failed to set PIN retries: {}", e);
                KeyError::YubiKey(e)
            })?;
        
        Ok(())
    }
    
    /// Get the attestation certificate for a slot
    pub fn get_attestation(
        yubikey: &mut YubiKey,
        slot: SlotId,
    ) -> Result<Vec<u8>> {
        debug!("Getting attestation for slot {:?}", slot);
        
        let attestation = piv::fetch_attestation(yubikey, slot)
            .map_err(|e| {
                warn!("Failed to get attestation for slot {:?}: {}", slot, e);
                KeyError::YubiKey(e)
            })?;
        
        Ok(attestation)
    }
    
    /// Convert key algorithm to PIV algorithm ID
    pub fn to_algorithm_id(algorithm: &KeyAlgorithm) -> Result<AlgorithmId> {
        match algorithm {
            KeyAlgorithm::Rsa(RsaKeySize::Rsa1024) => Ok(AlgorithmId::Rsa1024),
            KeyAlgorithm::Rsa(RsaKeySize::Rsa2048) => Ok(AlgorithmId::Rsa2048),
            KeyAlgorithm::Ecdsa(EcdsaCurve::P256) => Ok(AlgorithmId::EccP256),
            KeyAlgorithm::Ecdsa(EcdsaCurve::P384) => Ok(AlgorithmId::EccP384),
            _ => Err(KeyError::UnsupportedAlgorithm(
                format!("Algorithm {:?} not supported by YubiKey PIV", algorithm)
            ))
        }
    }
    
    /// Convert PIV algorithm ID to key algorithm
    pub fn from_algorithm_id(alg_id: AlgorithmId) -> KeyAlgorithm {
        match alg_id {
            AlgorithmId::Rsa1024 => KeyAlgorithm::Rsa(RsaKeySize::Rsa1024),
            AlgorithmId::Rsa2048 => KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
            AlgorithmId::EccP256 => KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
            AlgorithmId::EccP384 => KeyAlgorithm::Ecdsa(EcdsaCurve::P384),
        }
    }
}

/// PIV slot manager trait implementation
#[async_trait]
impl HardwareTokenSlotManager for YubiKeyPivManager {
    async fn list_slots(&self, _token_serial: &str) -> Result<Vec<SlotInfo>> {
        // This would need access to the YubiKey instance
        // For now, return a placeholder
        Err(KeyError::Other("Slot listing requires YubiKey instance".to_string()))
    }
    
    async fn generate_key_in_slot(
        &self,
        _token_serial: &str,
        _slot_id: &str,
        _algorithm: KeyAlgorithm,
        _pin: SecureString,
    ) -> Result<KeyId> {
        // This would need access to the YubiKey instance
        Err(KeyError::Other("Key generation requires YubiKey instance".to_string()))
    }
    
    async fn import_key_to_slot(
        &self,
        _token_serial: &str,
        _slot_id: &str,
        _key_data: &[u8],
        _pin: SecureString,
    ) -> Result<()> {
        // This would need access to the YubiKey instance
        Err(KeyError::Other("Key import requires YubiKey instance".to_string()))
    }
    
    async fn delete_key_from_slot(
        &self,
        _token_serial: &str,
        _slot_id: &str,
        _pin: SecureString,
    ) -> Result<()> {
        // This would need access to the YubiKey instance
        Err(KeyError::Other("Key deletion requires YubiKey instance".to_string()))
    }
    
    async fn get_slot_info(
        &self,
        _token_serial: &str,
        _slot_id: &str,
    ) -> Result<SlotInfo> {
        // This would need access to the YubiKey instance
        Err(KeyError::Other("Slot info requires YubiKey instance".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_algorithm_conversion() {
        // Test conversion to PIV algorithm ID
        assert!(matches!(
            YubiKeyPivManager::to_algorithm_id(&KeyAlgorithm::Rsa(RsaKeySize::Rsa2048)),
            Ok(AlgorithmId::Rsa2048)
        ));
        
        assert!(matches!(
            YubiKeyPivManager::to_algorithm_id(&KeyAlgorithm::Ecdsa(EcdsaCurve::P256)),
            Ok(AlgorithmId::EccP256)
        ));
        
        // Test unsupported algorithm
        assert!(YubiKeyPivManager::to_algorithm_id(&KeyAlgorithm::Ed25519).is_err());
        
        // Test conversion from PIV algorithm ID
        assert!(matches!(
            YubiKeyPivManager::from_algorithm_id(AlgorithmId::Rsa2048),
            KeyAlgorithm::Rsa(RsaKeySize::Rsa2048)
        ));
    }
}