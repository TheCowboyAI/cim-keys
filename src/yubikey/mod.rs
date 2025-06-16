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
                .map_err(|e| KeyError::PcSc(e))?;
            *ctx_guard = Some(ctx);
        }
        Ok(())
    }

    /// Get or create PC/SC context
    fn get_pcsc_context(&self) -> Result<pcsc::Context> {
        self.init_pcsc()?;
        let ctx_guard = self.pcsc_context.lock().unwrap();
        ctx_guard.as_ref()
            .ok_or_else(|| KeyError::Other("PC/SC context not initialized".to_string()))
            .map(|ctx| ctx.clone())
    }

    /// Find all connected YubiKeys
    pub fn find_yubikeys(&self) -> Result<Vec<Serial>> {
        let mut readers = yubikey::reader::Context::open()
            .map_err(|e| KeyError::YubiKey(e))?;
        let mut serials = Vec::new();
        
        // For now, just count readers since serial() method doesn't exist
        // TODO: Implement proper serial number detection
        for (index, _reader) in readers.iter()
            .map_err(|e| KeyError::YubiKey(e))?
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
            .map_err(|e| KeyError::YubiKey(e))?;

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
            Err(KeyError::KeyNotFound(format!("YubiKey {} not connected", serial)))
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
            .ok_or_else(|| KeyError::KeyNotFound(format!("YubiKey {} not connected", serial)))?;
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
                token_type: format!("YubiKey {}", version),
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
