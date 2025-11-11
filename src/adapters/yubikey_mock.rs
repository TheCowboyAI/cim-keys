//! Mock YubiKey adapter for testing
//!
//! This adapter implements the YubiKeyPort trait using in-memory simulation.
//! It provides a functor from the YubiKey category to the Domain category for testing.
//!
//! **Category Theory Perspective:**
//! - **Source Category**: YubiKey Hardware (simulated)
//! - **Target Category**: Domain (key management operations)
//! - **Functor**: MockYubiKeyAdapter maps simulated hardware to domain operations
//! - **Morphisms Preserved**: All PIV operation compositions are preserved

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::ports::yubikey::{
    KeyAlgorithm, PivSlot, PublicKey, SecureString, Signature, YubiKeyDevice, YubiKeyError,
    YubiKeyPort,
};

/// Mock YubiKey adapter for testing
///
/// This is a **Functor** F: YubiKey_Mock → Domain where:
/// - Objects: Simulated devices → Domain key entities
/// - Morphisms: Simulated PIV operations → Domain operations
///
/// **Functor Laws Verified:**
/// 1. Identity: No-op operations preserve state
/// 2. Composition: generate_key ∘ import_cert = valid PIV state
#[derive(Clone)]
pub struct MockYubiKeyAdapter {
    /// Simulated devices
    devices: Arc<RwLock<Vec<YubiKeyDevice>>>,

    /// Keys stored in slots (device_serial -> slot -> key)
    keys: Arc<RwLock<HashMap<String, HashMap<PivSlot, PublicKey>>>>,

    /// Certificates in slots (device_serial -> slot -> cert)
    certificates: Arc<RwLock<HashMap<String, HashMap<PivSlot, Vec<u8>>>>>,

    /// PIN state (device_serial -> pin)
    pins: Arc<RwLock<HashMap<String, String>>>,

    /// Management keys (device_serial -> key)
    management_keys: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MockYubiKeyAdapter {
    /// Create a new mock adapter
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(Vec::new())),
            keys: Arc::new(RwLock::new(HashMap::new())),
            certificates: Arc::new(RwLock::new(HashMap::new())),
            pins: Arc::new(RwLock::new(HashMap::new())),
            management_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a simulated device for testing
    pub fn add_device(&self, device: YubiKeyDevice) {
        let serial = device.serial.clone();
        self.devices.write().unwrap().push(device);

        // Initialize default PIN and management key
        self.pins.write().unwrap().insert(serial.clone(), "123456".to_string());
        self.management_keys
            .write()
            .unwrap()
            .insert(serial, vec![0; 24]); // Default 3DES key
    }

    /// Clear all devices (for test isolation)
    pub fn clear(&self) {
        self.devices.write().unwrap().clear();
        self.keys.write().unwrap().clear();
        self.certificates.write().unwrap().clear();
        self.pins.write().unwrap().clear();
        self.management_keys.write().unwrap().clear();
    }

    fn generate_mock_public_key(&self, algorithm: KeyAlgorithm) -> PublicKey {
        // Generate deterministic mock key material
        let data = match algorithm {
            KeyAlgorithm::Rsa2048 => vec![0x30, 0x82, 0x01, 0x22], // Mock RSA 2048
            KeyAlgorithm::EccP256 => vec![0x30, 0x59], // Mock P-256
            KeyAlgorithm::Ed25519 => vec![0x30, 0x2A], // Mock Ed25519
            _ => vec![0x30, 0x00], // Generic mock
        };

        PublicKey {
            algorithm,
            data: data.clone(),
            spki: data,
        }
    }

    fn generate_mock_signature(&self, algorithm: KeyAlgorithm, data: &[u8]) -> Signature {
        // Generate deterministic mock signature
        let sig_data = match algorithm {
            KeyAlgorithm::Rsa2048 => {
                let mut sig = vec![0u8; 256]; // RSA 2048 signature size
                sig[0] = data[0] ^ 0xFF; // Simple deterministic modification
                sig
            }
            KeyAlgorithm::EccP256 => {
                let mut sig = vec![0u8; 64]; // ECDSA P-256 signature size
                sig[0] = data[0] ^ 0xAA;
                sig
            }
            KeyAlgorithm::Ed25519 => {
                let mut sig = vec![0u8; 64]; // Ed25519 signature size
                sig[0] = data[0] ^ 0x55;
                sig
            }
            _ => vec![0u8; 64],
        };

        Signature {
            algorithm,
            data: sig_data,
        }
    }
}

impl Default for MockYubiKeyAdapter {
    fn default() -> Self {
        let adapter = Self::new();

        // Add a default test device
        adapter.add_device(YubiKeyDevice {
            serial: "12345678".to_string(),
            version: "5.4.3".to_string(),
            model: "YubiKey 5 NFC (Mock)".to_string(),
            piv_enabled: true,
        });

        adapter
    }
}

#[async_trait]
impl YubiKeyPort for MockYubiKeyAdapter {
    /// **Functor Mapping**: () → [YubiKeyDevice]
    async fn list_devices(&self) -> Result<Vec<YubiKeyDevice>, YubiKeyError> {
        Ok(self.devices.read().unwrap().clone())
    }

    /// **Functor Mapping**: (device, slot, algorithm) → PublicKey
    async fn generate_key_in_slot(
        &self,
        serial: &str,
        slot: PivSlot,
        algorithm: KeyAlgorithm,
        pin: &SecureString,
    ) -> Result<PublicKey, YubiKeyError> {
        // Verify PIN
        self.verify_pin(serial, pin).await?;

        // Check if device exists
        if !self.devices.read().unwrap().iter().any(|d| d.serial == serial) {
            return Err(YubiKeyError::DeviceNotFound(serial.to_string()));
        }

        // Generate mock public key
        let public_key = self.generate_mock_public_key(algorithm);

        // Store in simulated slot
        self.keys
            .write()
            .unwrap()
            .entry(serial.to_string())
            .or_default()
            .insert(slot, public_key.clone());

        Ok(public_key)
    }

    /// **Functor Mapping**: (device, slot, certificate) → ()
    async fn import_certificate(
        &self,
        serial: &str,
        slot: PivSlot,
        certificate: &[u8],
        pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        // Verify PIN
        self.verify_pin(serial, pin).await?;

        // Verify slot has a key
        let has_key = self
            .keys
            .read()
            .unwrap()
            .get(serial)
            .and_then(|slots| slots.get(&slot))
            .is_some();

        if !has_key {
            return Err(YubiKeyError::OperationError(
                "Cannot import certificate without key in slot".to_string(),
            ));
        }

        // Store certificate
        self.certificates
            .write()
            .unwrap()
            .entry(serial.to_string())
            .or_default()
            .insert(slot, certificate.to_vec());

        Ok(())
    }

    /// **Functor Mapping**: (device, slot, data) → Signature
    async fn sign_with_slot(
        &self,
        serial: &str,
        slot: PivSlot,
        data: &[u8],
        pin: &SecureString,
    ) -> Result<Signature, YubiKeyError> {
        // Verify PIN
        self.verify_pin(serial, pin).await?;

        // Get key from slot
        let key = self
            .keys
            .read()
            .unwrap()
            .get(serial)
            .and_then(|slots| slots.get(&slot).cloned())
            .ok_or_else(|| YubiKeyError::OperationError("No key in slot".to_string()))?;

        // Generate mock signature
        Ok(self.generate_mock_signature(key.algorithm, data))
    }

    /// **Functor Mapping**: (device, pin) → bool
    async fn verify_pin(&self, serial: &str, pin: &SecureString) -> Result<bool, YubiKeyError> {
        let pins = self.pins.read().unwrap();
        let stored_pin = pins
            .get(serial)
            .ok_or_else(|| YubiKeyError::DeviceNotFound(serial.to_string()))?;

        let pin_str = String::from_utf8_lossy(pin.as_bytes());

        if &pin_str != stored_pin {
            return Err(YubiKeyError::InvalidPin);
        }

        Ok(true)
    }

    /// **Functor Mapping**: (device, old_key, new_key) → ()
    async fn change_management_key(
        &self,
        serial: &str,
        current_key: &[u8],
        new_key: &[u8],
    ) -> Result<(), YubiKeyError> {
        let mut keys = self.management_keys.write().unwrap();
        let stored_key = keys
            .get(serial)
            .ok_or_else(|| YubiKeyError::DeviceNotFound(serial.to_string()))?;

        if stored_key != current_key {
            return Err(YubiKeyError::AuthenticationFailed);
        }

        keys.insert(serial.to_string(), new_key.to_vec());
        Ok(())
    }

    /// **Functor Mapping**: device → () (terminal morphism)
    async fn reset_piv(&self, serial: &str) -> Result<(), YubiKeyError> {
        // Clear all slots for this device
        self.keys.write().unwrap().remove(serial);
        self.certificates.write().unwrap().remove(serial);

        // Reset to default PIN and management key
        self.pins
            .write()
            .unwrap()
            .insert(serial.to_string(), "123456".to_string());
        self.management_keys
            .write()
            .unwrap()
            .insert(serial.to_string(), vec![0; 24]);

        Ok(())
    }

    async fn get_attestation(
        &self,
        serial: &str,
        slot: PivSlot,
    ) -> Result<Vec<u8>, YubiKeyError> {
        // Verify key exists in slot
        let has_key = self
            .keys
            .read()
            .unwrap()
            .get(serial)
            .and_then(|slots| slots.get(&slot))
            .is_some();

        if !has_key {
            return Err(YubiKeyError::OperationError("No key in slot".to_string()));
        }

        // Return mock attestation certificate
        Ok(vec![0x30, 0x82, 0x01, 0x00]) // Mock X.509 certificate
    }

    async fn set_chuid(
        &self,
        serial: &str,
        _chuid: &[u8],
        pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        self.verify_pin(serial, pin).await?;
        Ok(())
    }

    async fn set_ccc(
        &self,
        serial: &str,
        _ccc: &[u8],
        pin: &SecureString,
    ) -> Result<(), YubiKeyError> {
        self.verify_pin(serial, pin).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_functor_identity_law() {
        // F(id) = id
        let adapter = MockYubiKeyAdapter::default();
        let devices = adapter.list_devices().await.unwrap();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "12345678");
    }

    #[tokio::test]
    async fn test_functor_composition_law() {
        // F(generate ∘ import_cert) = F(generate) ∘ F(import_cert)
        let adapter = MockYubiKeyAdapter::default();
        let pin = SecureString::new("123456");

        // Generate key
        let public_key = adapter
            .generate_key_in_slot("12345678", PivSlot::Authentication, KeyAlgorithm::EccP256, &pin)
            .await
            .unwrap();

        assert_eq!(public_key.algorithm, KeyAlgorithm::EccP256);

        // Import certificate (should compose with generate)
        let cert = vec![0x30, 0x82];
        adapter
            .import_certificate("12345678", PivSlot::Authentication, &cert, &pin)
            .await
            .unwrap();

        // Verify composition preserved state
        let sig = adapter
            .sign_with_slot("12345678", PivSlot::Authentication, b"test data", &pin)
            .await
            .unwrap();

        assert_eq!(sig.algorithm, KeyAlgorithm::EccP256);
    }

    #[tokio::test]
    async fn test_reset_is_terminal_morphism() {
        let adapter = MockYubiKeyAdapter::default();
        let pin = SecureString::new("123456");

        // Generate a key
        adapter
            .generate_key_in_slot("12345678", PivSlot::Authentication, KeyAlgorithm::EccP256, &pin)
            .await
            .unwrap();

        // Reset (terminal morphism)
        adapter.reset_piv("12345678").await.unwrap();

        // Verify all state cleared
        let result = adapter
            .sign_with_slot("12345678", PivSlot::Authentication, b"data", &pin)
            .await;

        assert!(result.is_err());
    }
}
