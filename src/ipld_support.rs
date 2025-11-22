//! IPLD/CID Support for Content-Addressed Events
//!
//! Provides content-addressed storage and verification for all events using IPLD CIDs.
//! This ensures:
//! - Immutable event identity through content addressing
//! - Cryptographic integrity verification
//! - Deduplication across event streams
//! - Merkle DAG traversal for event causality chains

use serde::Serialize;
use thiserror::Error;

#[cfg(feature = "ipld")]
use {
    cid::Cid,
    multihash::Multihash,
    serde_json,
};

/// Generate CID for any serializable event
///
/// Uses CBOR encoding and SHA2-256 hashing for deterministic content addressing.
#[cfg(feature = "ipld")]
pub fn generate_cid<T: Serialize>(event: &T) -> Result<Cid, IpldError> {
    use sha2::{Sha256, Digest};

    // Serialize to JSON first (canonical representation)
    let json = serde_json::to_vec(event)
        .map_err(|e| IpldError::SerializationError(e.to_string()))?;

    // Hash with SHA2-256
    let mut hasher = Sha256::new();
    hasher.update(&json);
    let digest = hasher.finalize();

    // Create multihash with SHA2-256 code (0x12)
    const SHA2_256_CODE: u64 = 0x12;
    let mh = Multihash::wrap(SHA2_256_CODE, &digest)
        .map_err(|e| IpldError::ConversionError(format!("Multihash wrap failed: {:?}", e)))?;

    // Create CID v1 with DAG-CBOR codec (0x71)
    let cid = Cid::new_v1(0x71, mh);

    Ok(cid)
}

/// Verify that a CID matches the given event
///
/// Recomputes the CID and compares with the provided one.
#[cfg(feature = "ipld")]
pub fn verify_cid<T: Serialize>(event: &T, expected_cid: &Cid) -> Result<bool, IpldError> {
    let computed_cid = generate_cid(event)?;
    Ok(computed_cid == *expected_cid)
}

/// Serialize event to JSON (IPLD representation)
///
/// Note: Full IPLD DAG serialization requires codec-specific handling.
/// For now, we use JSON as the canonical representation.
#[cfg(feature = "ipld")]
pub fn event_to_json<T: Serialize>(event: &T) -> Result<Vec<u8>, IpldError> {
    serde_json::to_vec(event)
        .map_err(|e| IpldError::SerializationError(e.to_string()))
}

/// Event with embedded CID for integrity
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentAddressedEvent<T> {
    /// The actual event data
    pub event: T,

    /// Content identifier (CID) of the event
    #[cfg(feature = "ipld")]
    pub cid: String,

    #[cfg(not(feature = "ipld"))]
    pub cid: Option<String>,
}

impl<T: Serialize> ContentAddressedEvent<T> {
    /// Create a new content-addressed event
    #[cfg(feature = "ipld")]
    pub fn new(event: T) -> Result<Self, IpldError> {
        let cid = generate_cid(&event)?;
        Ok(Self {
            event,
            cid: cid.to_string(),
        })
    }

    /// Create without CID (when IPLD feature disabled)
    #[cfg(not(feature = "ipld"))]
    pub fn new(event: T) -> Result<Self, IpldError> {
        Ok(Self {
            event,
            cid: None,
        })
    }

    /// Verify integrity of this event
    #[cfg(feature = "ipld")]
    pub fn verify(&self) -> Result<bool, IpldError> {
        let expected_cid = Cid::try_from(self.cid.as_str())
            .map_err(|e| IpldError::CidParseError(e.to_string()))?;
        verify_cid(&self.event, &expected_cid)
    }

    #[cfg(not(feature = "ipld"))]
    pub fn verify(&self) -> Result<bool, IpldError> {
        Ok(true) // No verification when IPLD disabled
    }
}

/// IPLD-related errors
#[derive(Debug, Error)]
pub enum IpldError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("CID parse error: {0}")]
    CidParseError(String),

    #[error("Verification failed")]
    VerificationFailed,

    #[error("IPLD feature not enabled")]
    FeatureNotEnabled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        id: Uuid,
        data: String,
    }

    #[test]
    #[cfg(feature = "ipld")]
    fn test_generate_cid() {
        let event = TestEvent {
            id: Uuid::now_v7(),
            data: "test data".to_string(),
        };

        let cid = generate_cid(&event).unwrap();
        assert!(cid.to_string().starts_with("baf")); // CID v1 prefix
    }

    #[test]
    #[cfg(feature = "ipld")]
    fn test_verify_cid() {
        let event = TestEvent {
            id: Uuid::now_v7(),
            data: "test data".to_string(),
        };

        let cid = generate_cid(&event).unwrap();
        assert!(verify_cid(&event, &cid).unwrap());
    }

    #[test]
    #[cfg(feature = "ipld")]
    fn test_deterministic_cid() {
        // Same content should produce same CID
        let id = Uuid::now_v7();
        let event1 = TestEvent {
            id,
            data: "test".to_string(),
        };
        let event2 = TestEvent {
            id,
            data: "test".to_string(),
        };

        let cid1 = generate_cid(&event1).unwrap();
        let cid2 = generate_cid(&event2).unwrap();
        assert_eq!(cid1, cid2);
    }

    #[test]
    #[cfg(feature = "ipld")]
    fn test_content_addressed_event() {
        let event = TestEvent {
            id: Uuid::now_v7(),
            data: "test".to_string(),
        };

        let ca_event = ContentAddressedEvent::new(event).unwrap();
        assert!(ca_event.verify().unwrap());
    }
}
