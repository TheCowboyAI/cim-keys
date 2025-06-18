//! YubiKey hardware token support
//!
//! This module provides integration with YubiKey hardware tokens for secure
//! key storage and cryptographic operations.

use crate::{KeyError, Result};
use crate::types::*;
use crate::traits::*;
use async_trait::async_trait;
use yubikey::{YubiKey, Serial};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tracing::{debug, info, warn};

pub mod operations;
pub mod piv;

// TODO: Re-enable when these modules are implemented
// pub use operations::*;
// pub use piv::*;

/// YubiKey manager for hardware token operations
pub struct YubiKeyManager {
    /// Connected YubiKeys indexed by serial number
    connected_keys: Arc<Mutex<HashMap<String, YubiKey>>>,
    /// PC/SC context
    pcsc_context: Arc<Mutex<Option<pcsc::Context>>>,
}

impl YubiKeyManager {
    /// Create a new YubiKey manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            connected_keys: Arc::new(Mutex::new(HashMap::new())),
            pcsc_context: Arc::new(Mutex::new(None)),
        })
    }

    /// Initialize PC/SC context
    pub fn init_pcsc(&self) -> Result<()> {
        let mut ctx_guard = self.pcsc_context.lock().unwrap();
        if ctx_guard.is_none() {
            let ctx = pcsc::Context::establish(pcsc::Scope::User)
                .map_err(KeyError::PcSc)?;
            *ctx_guard = Some(ctx);
        }
        Ok(())
    }

    /// Get or create PC/SC context
    #[allow(dead_code)]
    fn get_pcsc_context(&self) -> Result<pcsc::Context> {
        self.init_pcsc()?;
        let ctx_guard = self.pcsc_context.lock().unwrap();
        ctx_guard.as_ref()
            .ok_or_else(|| KeyError::Other("PC/SC context not initialized".to_string()))
            .cloned()
    }

    /// Find all connected YubiKeys
    pub fn find_yubikeys(&self) -> Result<Vec<Serial>> {
        let mut readers = yubikey::reader::Context::open()
            .map_err(KeyError::YubiKey)?;
        let mut serials = Vec::new();
        
        // For now, just count readers since serial() method doesn't exist
        // TODO: Implement proper serial number detection
        for (index, _reader) in readers.iter()
            .map_err(KeyError::YubiKey)?
            .enumerate()
        {
            // Use index as a placeholder serial until we find the correct API
            serials.push(Serial::from(index as u32));
        }

        info!("Found {} YubiKey(s)", serials.len());
        Ok(serials)
    }

    /// Connect to a specific YubiKey
    pub fn connect(&self, serial: Serial) -> Result<()> {
        let yubikey = YubiKey::open_by_serial(serial)
            .map_err(KeyError::YubiKey)?;

        info!("Connected to YubiKey {}", serial);
        debug!("YubiKey version: {:?}", yubikey.version());

        let mut keys = self.connected_keys.lock().unwrap();
        keys.insert(serial.to_string(), yubikey);

        Ok(())
    }

    /// Disconnect from a YubiKey
    pub fn disconnect(&self, serial: &str) -> Result<()> {
        let mut keys = self.connected_keys.lock().unwrap();
        if keys.remove(serial).is_some() {
            info!("Disconnected from YubiKey {}", serial);
            Ok(())
        } else {
            Err(KeyError::KeyNotFound(format!("YubiKey {serial} not connected")))
        }
    }

    /// Check if a YubiKey is connected
    pub fn is_connected(&self, serial: &str) -> bool {
        let keys = self.connected_keys.lock().unwrap();
        keys.contains_key(serial)
    }

    /// Execute a closure with a mutable reference to the YubiKey
    pub fn with_yubikey<F, R>(&self, serial: &str, f: F) -> Result<R>
    where
        F: FnOnce(&mut YubiKey) -> Result<R>,
    {
        let mut keys = self.connected_keys.lock().unwrap();
        let yubikey = keys.get_mut(serial)
            .ok_or_else(|| KeyError::KeyNotFound(format!("YubiKey {serial} not connected")))?;
        f(yubikey)
    }

    /// Authenticate to a YubiKey with PIN
    pub fn authenticate(&self, serial: &str, pin: &SecureString) -> Result<()> {
        self.with_yubikey(serial, |yubikey| {
            // Verify PIN
            yubikey.verify_pin(pin.expose_secret().as_bytes())
                .map_err(|e| {
                    warn!("PIN verification failed for YubiKey {}", serial);
                    KeyError::YubiKey(e)
                })?;

            info!("Successfully authenticated to YubiKey {}", serial);
            Ok(())
        })
    }

    /// Get YubiKey information
    pub fn get_info(&self, serial: &str) -> Result<HardwareTokenInfo> {
        self.with_yubikey(serial, |yubikey| {
            let version = yubikey.version();
            let serial_num = yubikey.serial();

            // Get PIV slots info
            let slots = vec![
                "PIV Authentication".to_string(),
                "PIV Signing".to_string(),
                "Key Management".to_string(),
                "Card Authentication".to_string(),
            ];

            // Supported algorithms based on YubiKey version
            let algorithms = vec![
                KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
                KeyAlgorithm::Rsa(RsaKeySize::Rsa4096),
                KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
                KeyAlgorithm::Ecdsa(EcdsaCurve::P384),
            ];

            Ok(HardwareTokenInfo {
                token_type: format!("YubiKey {version}"),
                serial_number: serial_num.to_string(),
                firmware_version: format!("{}.{}.{}", version.major, version.minor, version.patch),
                available_slots: slots,
                supported_algorithms: algorithms,
                pin_retries: None, // Would need to query this
                puk_retries: None, // Would need to query this
            })
        })
    }
}

#[async_trait]
impl HardwareTokenManager for YubiKeyManager {
    async fn list_tokens(&self) -> Result<Vec<HardwareTokenInfo>> {
        let serials = self.find_yubikeys()?;
        let mut tokens = Vec::new();

        for serial in serials {
            // Try to connect and get info
            if self.connect(serial).is_ok() {
                if let Ok(info) = self.get_info(&serial.to_string()) {
                    tokens.push(info);
                }
                // Disconnect after getting info
                let _ = self.disconnect(&serial.to_string());
            }
        }

        Ok(tokens)
    }

    async fn connect_token(&self, serial: &str) -> Result<()> {
        let serial_num = Serial::from(
            serial.parse::<u32>()
                .map_err(|_| KeyError::Other("Invalid serial number".to_string()))?
        );
        self.connect(serial_num)
    }

    async fn disconnect_token(&self, serial: &str) -> Result<()> {
        self.disconnect(serial)
    }

    async fn authenticate_token(
        &self,
        serial: &str,
        pin: SecureString,
    ) -> Result<()> {
        self.authenticate(serial, &pin)
    }

    async fn change_pin(
        &self,
        serial: &str,
        old_pin: SecureString,
        new_pin: SecureString,
    ) -> Result<()> {
        self.with_yubikey(serial, |yubikey| {
            // Change PIN
            yubikey.change_pin(
                old_pin.expose_secret().as_bytes(),
                new_pin.expose_secret().as_bytes(),
            ).map_err(|e| {
                warn!("Failed to change PIN for YubiKey {}", serial);
                KeyError::YubiKey(e)
            })?;

            info!("Successfully changed PIN for YubiKey {}", serial);
            Ok(())
        })
    }

    async fn reset_token(
        &self,
        serial: &str,
        puk: SecureString,
    ) -> Result<()> {
        self.with_yubikey(serial, |yubikey| {
            // Reset PIN using PUK
            yubikey.unblock_pin(
                puk.expose_secret().as_bytes(),
                b"123456", // Default PIN
            ).map_err(|e| {
                warn!("Failed to reset YubiKey {} with PUK", serial);
                KeyError::YubiKey(e)
            })?;

            info!("Successfully reset YubiKey {} to default PIN", serial);
            Ok(())
        })
    }
}

impl Default for YubiKeyManager {
    fn default() -> Self {
        Self::new().expect("Failed to create YubiKey manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yubikey_manager_creation() {
        // This should succeed even without pcscd running
        let manager = YubiKeyManager::new();
        assert!(manager.is_ok(), "Should create YubiKey manager");
    }

    #[test]
    fn test_pcsc_context_initialization() {
        let manager = YubiKeyManager::new().unwrap();
        
        // Try to initialize PC/SC context
        // This might fail if pcscd is not running, which is expected
        let result = manager.init_pcsc();
        
        // We don't assert success here because it depends on system state
        // Just verify it doesn't panic
        match result {
            Ok(_) => println!("PC/SC context initialized successfully"),
            Err(e) => println!("PC/SC initialization failed (expected if pcscd not running): {:?}", e),
        }
    }

    #[test]
    fn test_find_yubikeys_without_hardware() {
        let manager = YubiKeyManager::new().unwrap();
        
        // This should not panic even without YubiKeys connected
        let result = manager.find_yubikeys();
        
        match result {
            Ok(serials) => {
                println!("Found {} YubiKey(s)", serials.len());
                // It's fine if no YubiKeys are found
            }
            Err(e) => {
                println!("YubiKey search failed (expected without hardware): {:?}", e);
                // This is expected without pcscd or YubiKeys
            }
        }
    }

    #[test]
    fn test_is_connected_empty() {
        let manager = YubiKeyManager::new().unwrap();
        
        // Should return false for any serial when nothing is connected
        assert!(!manager.is_connected("12345678"));
        assert!(!manager.is_connected("nonexistent"));
    }

    #[test]
    fn test_disconnect_nonexistent() {
        let manager = YubiKeyManager::new().unwrap();
        
        // Should return error when trying to disconnect non-connected device
        let result = manager.disconnect("12345678");
        assert!(result.is_err());
        
        match result {
            Err(KeyError::KeyNotFound(_)) => {
                // Expected error type
            }
            _ => panic!("Expected KeyNotFound error"),
        }
    }

    #[test]
    fn test_hardware_token_info_format() {
        // Test that HardwareTokenInfo can be created and serialized
        let info = HardwareTokenInfo {
            token_type: "YubiKey 5C".to_string(),
            serial_number: "12345678".to_string(),
            firmware_version: "5.2.7".to_string(),
            available_slots: vec![
                "PIV Authentication".to_string(),
                "PIV Signing".to_string(),
            ],
            supported_algorithms: vec![
                KeyAlgorithm::Rsa(RsaKeySize::Rsa2048),
                KeyAlgorithm::Ecdsa(EcdsaCurve::P256),
            ],
            pin_retries: Some(3),
            puk_retries: Some(3),
        };
        
        // Should be able to serialize to JSON
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("YubiKey 5C"));
        
        // And deserialize back
        let deserialized: HardwareTokenInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.serial_number, "12345678");
    }
}
