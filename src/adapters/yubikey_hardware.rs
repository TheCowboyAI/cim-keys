//! YubiKey Hardware adapter using direct PC/SC access
//!
//! This adapter implements the YubiKeyPort trait using the yubikey crate
//! for direct hardware access via PC/SC (smart card interface).
//!
//! Requires: yubikey-support feature enabled
//!
//! NOTE: This is a simplified initial implementation focusing on basic operations.
//! Advanced PIV operations will be added incrementally.

#[cfg(feature = "yubikey-support")]
use async_trait::async_trait;
#[cfg(feature = "yubikey-support")]
use yubikey::YubiKey;

#[cfg(feature = "yubikey-support")]
use crate::ports::yubikey::{
    KeyAlgorithm, PivSlot, PublicKey, SecureString, Signature, YubiKeyDevice, YubiKeyError,
    YubiKeyPort,
};

/// YubiKey adapter using direct hardware access via PC/SC
///
/// This provides a thin wrapper around the yubikey crate for basic operations.
/// More advanced PIV operations are delegated to the CLI adapter until fully implemented.
#[cfg(feature = "yubikey-support")]
#[derive(Clone, Default)]
pub struct YubiKeyHardwareAdapter;

#[cfg(feature = "yubikey-support")]
impl YubiKeyHardwareAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "yubikey-support")]
#[async_trait]
impl YubiKeyPort for YubiKeyHardwareAdapter {
    async fn list_devices(&self) -> Result<Vec<YubiKeyDevice>, YubiKeyError> {
        // Use blocking call in async context (yubikey crate is synchronous)
        let result = tokio::task::spawn_blocking(|| {
            // Try to open default YubiKey
            match YubiKey::open() {
                Ok(mut yk) => {
                    let serial = yk.serial();
                    let version = yk.version();

                    let device = YubiKeyDevice {
                        serial: format!("{}", serial),
                        version: format!("{}.{}.{}", version.major, version.minor, version.patch),
                        model: "YubiKey (PC/SC)".to_string(),
                        piv_enabled: true,
                    };

                    Ok(vec![device])
                }
                Err(e) => Err(YubiKeyError::HardwareError(format!("Failed to open YubiKey: {}", e))),
            }
        }).await
        .map_err(|e| YubiKeyError::HardwareError(format!("Task join error: {}", e)))??;

        Ok(result)
    }

    async fn generate_key_in_slot(
        &self,
        _serial: &str,
        _slot: PivSlot,
        _algorithm: KeyAlgorithm,
        _pin: &SecureString,
    ) -> Result<PublicKey, YubiKeyError> {
        // TODO: Implement PIV key generation using yubikey crate
        // For now, delegate to CLI adapter which has working implementation
        Err(YubiKeyError::NotSupported(
            "PIV key generation not yet implemented in hardware adapter - use CLI adapter".to_string()
        ))
    }

    async fn import_certificate(
        &self,
        _serial: &str,
        _slot: PivSlot,
        _certificate: &[u8],
        _pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("Certificate import not yet implemented - use CLI adapter".to_string()))
    }

    async fn sign_with_slot(
        &self,
        _serial: &str,
        _slot: PivSlot,
        _data: &[u8],
        _pin: &SecureString,
    ) -> Result<Signature, YubiKeyError> {
        Err(YubiKeyError::NotSupported("Signing not yet implemented - use CLI adapter".to_string()))
    }

    async fn verify_pin(&self, _serial: &str, _pin: &SecureString) -> Result<bool, YubiKeyError> {
        Err(YubiKeyError::NotSupported("PIN verification not yet implemented - use CLI adapter".to_string()))
    }

    async fn change_pin(
        &self,
        _serial: &str,
        _old_pin: &SecureString,
        _new_pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("PIN change not yet implemented - use CLI adapter".to_string()))
    }

    async fn change_management_key(
        &self,
        _serial: &str,
        _current_key: &[u8],
        _new_key: &[u8],
    ) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("Management key change not yet implemented - use CLI adapter".to_string()))
    }

    async fn reset_piv(&self, _serial: &str) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("PIV reset not yet implemented - use CLI adapter".to_string()))
    }

    async fn get_attestation(
        &self,
        _serial: &str,
        _slot: PivSlot,
    ) -> Result<Vec<u8>, YubiKeyError> {
        Err(YubiKeyError::NotSupported("Attestation not yet implemented - use CLI adapter".to_string()))
    }

    async fn set_chuid(
        &self,
        _serial: &str,
        _chuid: &[u8],
        _pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("CHUID setting not yet implemented - use CLI adapter".to_string()))
    }

    async fn set_ccc(
        &self,
        _serial: &str,
        _ccc: &[u8],
        _pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("CCC setting not yet implemented - use CLI adapter".to_string()))
    }
}

// Stub implementation when yubikey-support feature is not enabled
#[cfg(not(feature = "yubikey-support"))]
#[derive(Clone, Default)]
pub struct YubiKeyHardwareAdapter;

#[cfg(not(feature = "yubikey-support"))]
impl YubiKeyHardwareAdapter {
    pub fn new() -> Self {
        Self
    }
}
