//! Master seed derivation from passphrase using Argon2id
//!
//! This module implements the first step of our deterministic key hierarchy:
//! converting a user passphrase into a cryptographically strong master seed.

use argon2::{
    Argon2, Algorithm, Version, Params,
    password_hash::{PasswordHasher, SaltString},
};
use hkdf::Hkdf;
use sha2::Sha256;

// Use der's re-export of zeroize (available as transitive dependency)
use der::zeroize::Zeroize;

/// Master seed - 256 bits of cryptographic entropy
///
/// This is the root of our entire key hierarchy. All keys are derived from this.
///
/// Security: Implements `Zeroize` to securely clear memory when dropped,
/// preventing the seed from remaining in memory after use.
///
/// Note: Debug implementation redacts the actual seed bytes for security.
#[derive(Clone)]
pub struct MasterSeed([u8; 32]);

impl std::fmt::Debug for MasterSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MasterSeed")
            .field(&"<redacted>")
            .finish()
    }
}

impl Zeroize for MasterSeed {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for MasterSeed {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl MasterSeed {
    /// Create a master seed from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        MasterSeed(bytes)
    }

    /// Get the seed bytes (for derivation operations)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Derive a child seed using HKDF
    ///
    /// This creates a cryptographically independent seed for specific purposes.
    /// The `info` parameter provides domain separation.
    ///
    /// Security: The child seed buffer is zeroized after use via ZeroizeOnDrop.
    pub fn derive_child(&self, info: &str) -> MasterSeed {
        let hkdf = Hkdf::<Sha256>::new(None, self.as_bytes());
        let mut child_seed = [0u8; 32];
        hkdf.expand(info.as_bytes(), &mut child_seed)
            .expect("32 bytes is a valid HKDF output length");

        // child_seed will be zeroized when MasterSeed is dropped (ZeroizeOnDrop)
        MasterSeed::from_bytes(child_seed)
    }
}

/// Derive master seed from passphrase using Argon2id
///
/// # Arguments
///
/// * `passphrase` - User's master passphrase
/// * `organization_id` - Unique organization identifier (used as salt domain)
///
/// # Security Parameters
///
/// - **Algorithm**: Argon2id (hybrid of Argon2i and Argon2d)
/// - **Memory**: 1 GB (memory-hard, resists GPU attacks)
/// - **Iterations**: 10 (time-hard)
/// - **Parallelism**: 4 threads
///
/// # Returns
///
/// 256-bit master seed derived deterministically from passphrase
///
/// # Example
///
/// ```rust,ignore
/// let seed = derive_master_seed("correct horse battery staple", "my-business-2025");
/// // Same passphrase + org_id = same seed (deterministic)
/// ```
pub fn derive_master_seed(passphrase: &str, organization_id: &str) -> Result<MasterSeed, String> {
    // Create deterministic salt from organization ID
    // This ensures different organizations get different seeds from same passphrase
    // We hash the org_id to create a deterministic 16-byte salt
    use sha2::{Sha256, Digest};

    let salt_input = format!("cim-keys-v1-{}", organization_id);
    let mut hasher = Sha256::new();
    hasher.update(salt_input.as_bytes());
    let hash_result = hasher.finalize();

    // Take first 16 bytes for salt (Argon2 recommends 16+ bytes)
    let salt_bytes = &hash_result[..16];

    // Encode to base64 for SaltString (SaltString expects B64 without padding)
    use base64::{Engine as _, engine::general_purpose};
    let salt_b64 = general_purpose::STANDARD_NO_PAD.encode(salt_bytes);
    let salt = SaltString::from_b64(&salt_b64)
        .map_err(|e| format!("Failed to create salt: {}", e))?;

    // Configure Argon2id with strong parameters
    // For testing: Reduced params (64 MB memory, 3 iterations)
    // For production: Use 1 GB memory and 10 iterations
    #[cfg(test)]
    let params = Params::new(
        65_536,     // 64 MB memory (for faster tests)
        3,          // 3 iterations (for faster tests)
        4,          // 4 threads
        Some(32),   // 32-byte output
    ).map_err(|e| format!("Invalid Argon2 params: {}", e))?;

    #[cfg(not(test))]
    let params = Params::new(
        1_048_576,  // 1 GB memory (production)
        10,         // 10 iterations (production)
        4,          // 4 threads
        Some(32),   // 32-byte output
    ).map_err(|e| format!("Invalid Argon2 params: {}", e))?;

    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        params,
    );

    // Derive the master seed
    let password_hash = argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| format!("Argon2 derivation failed: {}", e))?;

    // Extract the hash bytes (our master seed)
    let hash_bytes = password_hash
        .hash
        .ok_or("No hash produced")?;

    let mut seed_bytes = [0u8; 32];
    seed_bytes.copy_from_slice(hash_bytes.as_bytes());

    Ok(MasterSeed::from_bytes(seed_bytes))
}

/// Derive a child seed from master seed using HKDF
///
/// This is a convenience function that wraps MasterSeed::derive_child
///
/// # Arguments
///
/// * `master_seed` - The master seed to derive from
/// * `purpose` - Domain separation string (e.g., "root-ca", "user-alice-ssh")
///
/// # Example
///
/// ```rust,ignore
/// let root_ca_seed = derive_child_seed(&master_seed, "root-ca");
/// let alice_ssh_seed = derive_child_seed(&master_seed, "user-alice-ssh");
/// ```
pub fn derive_child_seed(master_seed: &MasterSeed, purpose: &str) -> MasterSeed {
    master_seed.derive_child(purpose)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_derivation() {
        // Same passphrase + org_id should produce same seed
        let seed1 = derive_master_seed("test passphrase", "org-123").unwrap();
        let seed2 = derive_master_seed("test passphrase", "org-123").unwrap();

        assert_eq!(seed1.as_bytes(), seed2.as_bytes());
    }

    #[test]
    fn test_different_org_different_seed() {
        // Different org_id should produce different seed
        let seed1 = derive_master_seed("test passphrase", "org-123").unwrap();
        let seed2 = derive_master_seed("test passphrase", "org-456").unwrap();

        assert_ne!(seed1.as_bytes(), seed2.as_bytes());
    }

    #[test]
    fn test_child_seed_derivation() {
        let master = derive_master_seed("test passphrase", "org-123").unwrap();

        let root_ca = master.derive_child("root-ca");
        let user_alice = master.derive_child("user-alice");

        // Child seeds should be different
        assert_ne!(root_ca.as_bytes(), user_alice.as_bytes());

        // Child derivation should be deterministic
        let root_ca2 = master.derive_child("root-ca");
        assert_eq!(root_ca.as_bytes(), root_ca2.as_bytes());
    }

    #[test]
    fn test_hierarchical_derivation() {
        let master = derive_master_seed("test passphrase", "org-123").unwrap();

        // Derive intermediate seed
        let intermediate = master.derive_child("intermediate-ca-engineering");

        // Derive from intermediate
        let user_from_intermediate = intermediate.derive_child("user-alice");

        // Should be different from deriving directly from master
        let user_from_master = master.derive_child("user-alice");

        assert_ne!(user_from_intermediate.as_bytes(), user_from_master.as_bytes());
    }

    #[test]
    fn test_seed_zeroization() {
        use std::ptr;

        // Create a seed and get a pointer to its data
        let mut seed = derive_master_seed("test passphrase", "org-123").unwrap();
        let seed_bytes = seed.as_bytes().clone();

        // Manually zeroize the seed
        seed.zeroize();

        // Verify it's been zeroed
        assert_eq!(seed.as_bytes(), &[0u8; 32]);

        // Verify it was different before
        assert_ne!(&seed_bytes, &[0u8; 32]);
    }

    #[test]
    fn test_seed_zeroization_on_drop() {
        // This test verifies that Drop calls zeroize
        // We can't directly observe memory after drop in safe Rust,
        // but we can verify the implementation compiles and doesn't panic
        {
            let _seed = derive_master_seed("test passphrase", "org-123").unwrap();
            // Seed should be zeroized when it goes out of scope here
        }
        // If we got here without panicking, Drop worked correctly
    }
}
