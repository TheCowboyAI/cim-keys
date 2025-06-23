//! Infrastructure Layer 1.2: Event Stream Tests for cim-keys
//! 
//! User Story: As a security system, I need to persist key events with CID chains for integrity
//!
//! Test Requirements:
//! - Verify key event persistence with CID calculation
//! - Verify CID chain integrity for key operations
//! - Verify key event replay from store
//! - Verify key snapshot creation and restoration
//!
//! Event Sequence:
//! 1. KeyEventStoreInitialized
//! 2. KeyEventPersisted { event_id, cid, previous_cid }
//! 3. CIDChainValidated { start_cid, end_cid, length }
//! 4. KeyEventsReplayed { count, key_id }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Initialize Store]
//!     B --> C[KeyEventStoreInitialized]
//!     C --> D[Generate Key Event]
//!     D --> E[KeyEventPersisted]
//!     E --> F[Validate CID Chain]
//!     F --> G[CIDChainValidated]
//!     G --> H[Replay Events]
//!     H --> I[KeyEventsReplayed]
//!     I --> J[Test Success]
//! ```

use std::collections::HashMap;
use std::time::SystemTime;
use ring::digest::{Context, SHA256};
use base64::{engine::general_purpose::STANDARD, Engine};

/// CID representation for testing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cid(String);

impl Cid {
    pub fn new(data: &[u8]) -> Self {
        let mut context = Context::new(&SHA256);
        context.update(data);
        let digest = context.finish();
        Self(format!("Qm{}", STANDARD.encode(digest.as_ref())))
    }
}

/// Key domain events
#[derive(Debug, Clone, PartialEq)]
pub enum KeyDomainEvent {
    KeyGenerated {
        key_id: String,
        algorithm: String,
        purpose: String,
        timestamp: SystemTime,
    },
    KeyRotated {
        old_key_id: String,
        new_key_id: String,
        reason: String,
        timestamp: SystemTime,
    },
    KeyRevoked {
        key_id: String,
        reason: String,
        timestamp: SystemTime,
    },
    KeyArchived {
        key_id: String,
        archive_location: String,
        timestamp: SystemTime,
    },
}

/// Event store events for testing
#[derive(Debug, Clone, PartialEq)]
pub enum KeyEventStoreEvent {
    KeyEventStoreInitialized,
    KeyEventPersisted {
        event_id: String,
        cid: Cid,
        previous_cid: Option<Cid>,
    },
    CIDChainValidated {
        start_cid: Cid,
        end_cid: Cid,
        length: usize,
    },
    KeyEventsReplayed {
        count: usize,
        key_id: String,
    },
    SnapshotCreated {
        snapshot_cid: Cid,
        event_count: usize,
    },
    SnapshotRestored {
        snapshot_cid: Cid,
        restored_count: usize,
    },
}

/// Event with CID chain
#[derive(Debug, Clone)]
pub struct ChainedKeyEvent {
    pub event_id: String,
    pub event: KeyDomainEvent,
    pub cid: Cid,
    pub previous_cid: Option<Cid>,
    pub sequence: u64,
}

/// Mock event store for key events
pub struct MockKeyEventStore {
    events: Vec<ChainedKeyEvent>,
    snapshots: HashMap<Cid, Vec<ChainedKeyEvent>>,
}

impl MockKeyEventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            snapshots: HashMap::new(),
        }
    }

    pub fn append_event(
        &mut self,
        event: KeyDomainEvent,
    ) -> Result<(String, Cid, Option<Cid>), String> {
        let event_id = format!("evt_{}", uuid::Uuid::new_v4());
        let previous_cid = self.events.last().map(|e| e.cid.clone());
        
        // Calculate CID including previous CID
        let event_data = format!("{:?}{:?}", event, previous_cid);
        let cid = Cid::new(event_data.as_bytes());
        
        let sequence = self.events.len() as u64;
        
        let chained_event = ChainedKeyEvent {
            event_id: event_id.clone(),
            event,
            cid: cid.clone(),
            previous_cid: previous_cid.clone(),
            sequence,
        };
        
        self.events.push(chained_event);
        
        Ok((event_id, cid, previous_cid))
    }

    pub fn validate_chain(&self) -> Result<(Cid, Cid, usize), String> {
        if self.events.is_empty() {
            return Err("No events to validate".to_string());
        }

        // Validate each event's CID chain
        for i in 1..self.events.len() {
            let current = &self.events[i];
            let previous = &self.events[i - 1];
            
            if current.previous_cid.as_ref() != Some(&previous.cid) {
                return Err(format!(
                    "Chain broken at sequence {}: expected {:?}, got {:?}",
                    i, previous.cid, current.previous_cid
                ));
            }
        }

        let start_cid = self.events.first().unwrap().cid.clone();
        let end_cid = self.events.last().unwrap().cid.clone();
        let length = self.events.len();

        Ok((start_cid, end_cid, length))
    }

    pub fn replay_events(&self, key_id: &str) -> Vec<ChainedKeyEvent> {
        self.events
            .iter()
            .filter(|e| match &e.event {
                KeyDomainEvent::KeyGenerated { key_id: id, .. } => id == key_id,
                KeyDomainEvent::KeyRotated { old_key_id, new_key_id, .. } => {
                    old_key_id == key_id || new_key_id == key_id
                }
                KeyDomainEvent::KeyRevoked { key_id: id, .. } => id == key_id,
                KeyDomainEvent::KeyArchived { key_id: id, .. } => id == key_id,
            })
            .cloned()
            .collect()
    }

    pub fn create_snapshot(&mut self) -> Result<Cid, String> {
        if self.events.is_empty() {
            return Err("No events to snapshot".to_string());
        }

        let snapshot_data = format!("{:?}", self.events);
        let snapshot_cid = Cid::new(snapshot_data.as_bytes());
        
        self.snapshots.insert(snapshot_cid.clone(), self.events.clone());
        
        Ok(snapshot_cid)
    }

    pub fn restore_from_snapshot(&mut self, snapshot_cid: &Cid) -> Result<usize, String> {
        match self.snapshots.get(snapshot_cid) {
            Some(events) => {
                self.events = events.clone();
                Ok(events.len())
            }
            None => Err("Snapshot not found".to_string()),
        }
    }
}

/// Event stream validator for key event store testing
pub struct KeyEventStreamValidator {
    expected_events: Vec<KeyEventStoreEvent>,
    captured_events: Vec<KeyEventStoreEvent>,
}

impl KeyEventStreamValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
        }
    }

    pub fn expect_sequence(mut self, events: Vec<KeyEventStoreEvent>) -> Self {
        self.expected_events = events;
        self
    }

    pub fn capture_event(&mut self, event: KeyEventStoreEvent) {
        self.captured_events.push(event);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.captured_events.len() != self.expected_events.len() {
            return Err(format!(
                "Event count mismatch: expected {}, got {}",
                self.expected_events.len(),
                self.captured_events.len()
            ));
        }

        for (i, (expected, actual)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected != actual {
                return Err(format!(
                    "Event mismatch at position {}: expected {:?}, got {:?}",
                    i, expected, actual
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_store_initialization() {
        // Arrange
        let mut validator = KeyEventStreamValidator::new()
            .expect_sequence(vec![
                KeyEventStoreEvent::KeyEventStoreInitialized,
            ]);

        // Act
        let store = MockKeyEventStore::new();
        validator.capture_event(KeyEventStoreEvent::KeyEventStoreInitialized);

        // Assert
        assert!(validator.validate().is_ok());
        assert_eq!(store.events.len(), 0);
    }

    #[test]
    fn test_key_event_persistence_with_cid() {
        // Arrange
        let mut store = MockKeyEventStore::new();
        let mut validator = KeyEventStreamValidator::new();

        // Act
        let event = KeyDomainEvent::KeyGenerated {
            key_id: "test-key-123".to_string(),
            algorithm: "RSA-2048".to_string(),
            purpose: "signing".to_string(),
            timestamp: SystemTime::now(),
        };

        let (event_id, cid, previous_cid) = store.append_event(event).unwrap();

        // Assert
        assert!(previous_cid.is_none()); // First event has no previous
        assert!(!event_id.is_empty());
        
        validator.capture_event(KeyEventStoreEvent::KeyEventPersisted {
            event_id,
            cid,
            previous_cid,
        });
    }

    #[test]
    fn test_cid_chain_integrity() {
        // Arrange
        let mut store = MockKeyEventStore::new();
        let mut validator = KeyEventStreamValidator::new();

        // Act - Add multiple events
        let event1 = KeyDomainEvent::KeyGenerated {
            key_id: "key-1".to_string(),
            algorithm: "Ed25519".to_string(),
            purpose: "authentication".to_string(),
            timestamp: SystemTime::now(),
        };

        let event2 = KeyDomainEvent::KeyRotated {
            old_key_id: "key-1".to_string(),
            new_key_id: "key-2".to_string(),
            reason: "scheduled rotation".to_string(),
            timestamp: SystemTime::now(),
        };

        let event3 = KeyDomainEvent::KeyRevoked {
            key_id: "key-1".to_string(),
            reason: "superseded".to_string(),
            timestamp: SystemTime::now(),
        };

        store.append_event(event1).unwrap();
        store.append_event(event2).unwrap();
        store.append_event(event3).unwrap();

        // Validate chain
        let (start_cid, end_cid, length) = store.validate_chain().unwrap();

        // Assert
        assert_eq!(length, 3);
        assert_ne!(start_cid, end_cid);
        
        validator.capture_event(KeyEventStoreEvent::CIDChainValidated {
            start_cid,
            end_cid,
            length,
        });
    }

    #[test]
    fn test_key_event_replay() {
        // Arrange
        let mut store = MockKeyEventStore::new();
        let mut validator = KeyEventStreamValidator::new();
        let key_id = "replay-test-key";

        // Add events for different keys
        store.append_event(KeyDomainEvent::KeyGenerated {
            key_id: key_id.to_string(),
            algorithm: "RSA-4096".to_string(),
            purpose: "encryption".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        store.append_event(KeyDomainEvent::KeyGenerated {
            key_id: "other-key".to_string(),
            algorithm: "ECDSA-P256".to_string(),
            purpose: "signing".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        store.append_event(KeyDomainEvent::KeyRotated {
            old_key_id: key_id.to_string(),
            new_key_id: "replay-test-key-v2".to_string(),
            reason: "annual rotation".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        // Act
        let replayed = store.replay_events(key_id);

        // Assert
        assert_eq!(replayed.len(), 2); // Only events for the specific key
        
        validator.capture_event(KeyEventStoreEvent::KeyEventsReplayed {
            count: replayed.len(),
            key_id: key_id.to_string(),
        });
    }

    #[test]
    fn test_snapshot_creation_and_restoration() {
        // Arrange
        let mut store = MockKeyEventStore::new();
        let mut validator = KeyEventStreamValidator::new();

        // Add some events
        for i in 0..5 {
            store.append_event(KeyDomainEvent::KeyGenerated {
                key_id: format!("snapshot-key-{}", i),
                algorithm: "AES-256".to_string(),
                purpose: "encryption".to_string(),
                timestamp: SystemTime::now(),
            }).unwrap();
        }

        // Act - Create snapshot
        let snapshot_cid = store.create_snapshot().unwrap();
        
        validator.capture_event(KeyEventStoreEvent::SnapshotCreated {
            snapshot_cid: snapshot_cid.clone(),
            event_count: 5,
        });

        // Clear events and restore
        store.events.clear();
        let restored_count = store.restore_from_snapshot(&snapshot_cid).unwrap();

        // Assert
        assert_eq!(restored_count, 5);
        assert_eq!(store.events.len(), 5);
        
        validator.capture_event(KeyEventStoreEvent::SnapshotRestored {
            snapshot_cid,
            restored_count,
        });
    }

    #[test]
    fn test_broken_chain_detection() {
        // Arrange
        let mut store = MockKeyEventStore::new();

        // Add valid events
        store.append_event(KeyDomainEvent::KeyGenerated {
            key_id: "chain-test-1".to_string(),
            algorithm: "RSA-2048".to_string(),
            purpose: "signing".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        store.append_event(KeyDomainEvent::KeyGenerated {
            key_id: "chain-test-2".to_string(),
            algorithm: "RSA-2048".to_string(),
            purpose: "signing".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        // Manually break the chain
        if let Some(event) = store.events.get_mut(1) {
            event.previous_cid = Some(Cid::new(b"broken"));
        }

        // Act
        let result = store.validate_chain();

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain broken"));
    }

    #[test]
    fn test_key_archival_event() {
        // Arrange
        let mut store = MockKeyEventStore::new();

        // Act
        let event = KeyDomainEvent::KeyArchived {
            key_id: "archive-test-key".to_string(),
            archive_location: "cold-storage/2024/keys/".to_string(),
            timestamp: SystemTime::now(),
        };

        let (event_id, cid, _) = store.append_event(event.clone()).unwrap();

        // Assert
        assert_eq!(store.events.len(), 1);
        assert_eq!(store.events[0].event, event);
        assert_eq!(store.events[0].event_id, event_id);
        assert_eq!(store.events[0].cid, cid);
    }
} 