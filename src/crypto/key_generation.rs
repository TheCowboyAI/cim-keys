//! Deterministic key generation from seeds

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use super::seed_derivation::MasterSeed;

/// A cryptographic keypair
#[derive(Clone)]
pub struct KeyPair {
    /// Private/signing key
    pub private_key: SigningKey,
    /// Public/verifying key
    pub public_key: VerifyingKey,
}

impl KeyPair {
    /// Generate a keypair from a seed
    ///
    /// This is deterministic - same seed always produces same keypair
    pub fn from_seed(seed: &MasterSeed) -> Self {
        let signing_key = SigningKey::from_bytes(seed.as_bytes());
        let verifying_key = signing_key.verifying_key();

        KeyPair {
            private_key: signing_key,
            public_key: verifying_key,
        }
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.private_key.sign(message)
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public_key.to_bytes()
    }

    /// Get private key bytes (use with caution!)
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.private_key.to_bytes()
    }
}

/// Generate a keypair from seed
///
/// This is a convenience wrapper around KeyPair::from_seed
///
/// # Example
///
/// ```rust,ignore
/// let seed = derive_master_seed("passphrase", "org-id")?;
/// let root_ca_seed = seed.derive_child("root-ca");
/// let keypair = generate_keypair_from_seed(&root_ca_seed);
/// ```
pub fn generate_keypair_from_seed(seed: &MasterSeed) -> KeyPair {
    KeyPair::from_seed(seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::seed_derivation::derive_master_seed;

    #[test]
    fn test_deterministic_keypair_generation() {
        let seed = derive_master_seed("test passphrase", "org-123").unwrap();

        let keypair1 = generate_keypair_from_seed(&seed);
        let keypair2 = generate_keypair_from_seed(&seed);

        // Same seed = same keypair
        assert_eq!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
        assert_eq!(keypair1.private_key_bytes(), keypair2.private_key_bytes());
    }

    #[test]
    fn test_different_seeds_different_keypairs() {
        let seed1 = derive_master_seed("test passphrase", "org-123").unwrap();
        let seed2 = derive_master_seed("test passphrase", "org-456").unwrap();

        let keypair1 = generate_keypair_from_seed(&seed1);
        let keypair2 = generate_keypair_from_seed(&seed2);

        // Different seeds = different keypairs
        assert_ne!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
    }

    #[test]
    fn test_sign_and_verify() {
        let seed = derive_master_seed("test passphrase", "org-123").unwrap();
        let keypair = generate_keypair_from_seed(&seed);

        let message = b"Hello, CIM!";
        let signature = keypair.sign(message);

        // Verify signature
        use ed25519_dalek::Verifier;
        assert!(keypair.public_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_child_seed_keypairs() {
        let master = derive_master_seed("test passphrase", "org-123").unwrap();

        let root_ca_keypair = generate_keypair_from_seed(&master.derive_child("root-ca"));
        let user_alice_keypair = generate_keypair_from_seed(&master.derive_child("user-alice"));

        // Different purposes = different keypairs
        assert_ne!(root_ca_keypair.public_key_bytes(), user_alice_keypair.public_key_bytes());
    }
}
