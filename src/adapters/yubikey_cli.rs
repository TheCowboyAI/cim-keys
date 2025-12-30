//! YubiKey CLI adapter using ykman
//!
//! This adapter implements the YubiKeyPort trait using the ykman CLI tool.

use async_trait::async_trait;
use std::process::Command;

use crate::ports::yubikey::{
    KeyAlgorithm, PivSlot, PublicKey, SecureString, Signature, YubiKeyDevice, YubiKeyError,
    YubiKeyPort,
};

/// YubiKey adapter using ykman CLI
#[derive(Clone, Default)]
pub struct YubiKeyCliAdapter;

impl YubiKeyCliAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl YubiKeyPort for YubiKeyCliAdapter {
    async fn list_devices(&self) -> Result<Vec<YubiKeyDevice>, YubiKeyError> {
        let output = Command::new("ykman")
            .arg("list")
            .output()
            .map_err(|e| YubiKeyError::HardwareError(format!("Failed to run ykman: {}", e)))?;

        if !output.status.success() {
            return Err(YubiKeyError::NoDeviceFound);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        // Parse output: "YubiKey 5C NFC (5.2.7) [OTP+FIDO+CCID] Serial: 15905511"
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Extract serial number
            let serial = if let Some(serial_part) = line.split("Serial: ").nth(1) {
                serial_part.trim().to_string()
            } else {
                continue;
            };

            // Extract model
            let model = if let Some(model_part) = line.split('(').next() {
                model_part.trim().to_string()
            } else {
                "Unknown".to_string()
            };

            // Extract version
            let version = if let Some(version_part) = line.split('(').nth(1) {
                if let Some(v) = version_part.split(')').next() {
                    v.trim().to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            // Check if PIV is enabled (CCID in capabilities)
            let piv_enabled = line.contains("CCID");

            devices.push(YubiKeyDevice {
                serial,
                version,
                model,
                piv_enabled,
            });
        }

        if devices.is_empty() {
            Err(YubiKeyError::NoDeviceFound)
        } else {
            Ok(devices)
        }
    }

    async fn generate_key_in_slot(
        &self,
        serial: &str,
        slot: PivSlot,
        algorithm: KeyAlgorithm,
        management_key: &SecureString,
    ) -> Result<PublicKey, YubiKeyError> {
        let slot_id = match slot {
            PivSlot::Authentication => "9a",
            PivSlot::Signature => "9c",
            PivSlot::KeyManagement => "9d",
            PivSlot::CardAuth => "9e",
            PivSlot::Retired(n) => return Err(YubiKeyError::NotSupported(format!("Retired slot {} not yet implemented", n))),
        };

        let alg = match algorithm {
            KeyAlgorithm::Rsa2048 => "RSA2048",
            KeyAlgorithm::EccP256 => "ECCP256",
            KeyAlgorithm::EccP384 => "ECCP384",
            _ => return Err(YubiKeyError::InvalidAlgorithm(format!("{:?} not supported via CLI", algorithm))),
        };

        let mgmt_key_str = String::from_utf8_lossy(management_key.as_bytes());

        let output = Command::new("ykman")
            .args(["--device", serial])
            .args(["piv", "keys", "generate"])
            .args(["--management-key", &mgmt_key_str])
            .args(["--algorithm", alg])
            .args(["--pin-policy", "ONCE"])
            .args(["--touch-policy", "CACHED"])
            .arg(slot_id)
            .arg("-")  // Output to stdout
            .output()
            .map_err(|e| YubiKeyError::OperationError(format!("Failed to generate key: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(YubiKeyError::OperationError(format!("Key generation failed: {}", stderr)));
        }

        // Return the public key data
        Ok(PublicKey {
            algorithm,
            data: output.stdout.clone(),
            spki: output.stdout,
        })
    }

    async fn import_certificate(
        &self,
        _serial: &str,
        _slot: PivSlot,
        _certificate: &[u8],
        _pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        // TODO: Implement certificate import
        Err(YubiKeyError::NotSupported("Certificate import not yet implemented".to_string()))
    }

    async fn sign_with_slot(
        &self,
        _serial: &str,
        _slot: PivSlot,
        _data: &[u8],
        _pin: &SecureString,
    ) -> Result<Signature, YubiKeyError> {
        // TODO: Implement signing
        Err(YubiKeyError::NotSupported("Signing not yet implemented".to_string()))
    }

    async fn verify_pin(&self, serial: &str, pin: &SecureString) -> Result<bool, YubiKeyError> {
        let pin_str = String::from_utf8_lossy(pin.as_bytes());

        let output = Command::new("ykman")
            .args(["--device", serial])
            .args(["piv", "info"])
            .env("YKMAN_PIN", pin_str.as_ref())
            .output()
            .map_err(|e| YubiKeyError::OperationError(format!("Failed to verify PIN: {}", e)))?;

        Ok(output.status.success())
    }

    async fn change_management_key(
        &self,
        serial: &str,
        current_key: &[u8],
        new_key: &[u8],
    ) -> Result<(), YubiKeyError> {
        let current_key_hex = hex::encode(current_key);
        let new_key_hex = hex::encode(new_key);

        let output = Command::new("ykman")
            .args(["--device", serial])
            .args(["piv", "access", "change-management-key"])
            .args(["--management-key", &current_key_hex])
            .args(["--new-management-key", &new_key_hex])
            .output()
            .map_err(|e| YubiKeyError::OperationError(format!("Failed to change management key: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(YubiKeyError::OperationError(format!("Management key change failed: {}", stderr)));
        }

        Ok(())
    }

    async fn reset_piv(&self, _serial: &str) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("PIV reset not yet implemented".to_string()))
    }

    async fn get_attestation(
        &self,
        _serial: &str,
        _slot: PivSlot,
    ) -> Result<Vec<u8>, YubiKeyError> {
        Err(YubiKeyError::NotSupported("Attestation not yet implemented".to_string()))
    }

    async fn set_chuid(
        &self,
        _serial: &str,
        _chuid: &[u8],
        _pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("CHUID setting not yet implemented".to_string()))
    }

    async fn set_ccc(
        &self,
        _serial: &str,
        _ccc: &[u8],
        _pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        Err(YubiKeyError::NotSupported("CCC setting not yet implemented".to_string()))
    }
}
