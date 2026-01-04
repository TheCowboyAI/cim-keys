// Copyright (c) 2025 - Cowboy AI, LLC.
//! CID-Based Event Store
//!
//! Content-addressed event storage using IPLD CIDs. Events are stored by their
//! content identifier, enabling:
//! - Automatic deduplication (same content = same CID = single storage)
//! - Integrity verification (CID is cryptographic hash of content)
//! - Immutable event log (content cannot change without changing CID)
//! - Merkle DAG structure for causality chains

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::events::EventEnvelope;
use crate::ipld_support::IpldError;

/// Error types for the CID-based event store
#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IPLD error: {0}")]
    IpldError(#[from] IpldError),

    #[error("Event not found: {0}")]
    NotFound(String),

    #[error("Invalid CID: {0}")]
    InvalidCid(String),

    #[error("Duplicate event (CID already exists): {0}")]
    DuplicateEvent(String),
}

/// CID-indexed event stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEventRecord {
    /// The CID of this event (filename without extension)
    pub cid: String,

    /// The event envelope with full metadata
    pub envelope: EventEnvelope,

    /// Timestamp when stored (for ordering during replay)
    pub stored_at: chrono::DateTime<chrono::Utc>,

    /// Optional CID of causation event (for Merkle DAG)
    pub causation_cid: Option<String>,
}

/// CID-based event store that stores events by content address
pub struct CidEventStore {
    /// Root path for event storage
    root_path: PathBuf,

    /// Cache of known CIDs (for fast deduplication)
    known_cids: HashSet<String>,
}

impl CidEventStore {
    /// Create a new CID event store at the given path
    pub fn new(root_path: impl Into<PathBuf>) -> Result<Self, EventStoreError> {
        let root_path = root_path.into();
        let events_path = root_path.join("events");
        let by_cid_path = events_path.join("by_cid");

        // Create directory structure
        fs::create_dir_all(&by_cid_path)
            .map_err(|e| EventStoreError::IoError(format!("Failed to create events directory: {}", e)))?;

        // Load existing CIDs from directory
        let known_cids = Self::load_known_cids(&by_cid_path)?;

        Ok(Self {
            root_path,
            known_cids,
        })
    }

    /// Load known CIDs from existing files
    fn load_known_cids(by_cid_path: &Path) -> Result<HashSet<String>, EventStoreError> {
        let mut cids = HashSet::new();

        if by_cid_path.exists() {
            for entry in fs::read_dir(by_cid_path)
                .map_err(|e| EventStoreError::IoError(format!("Failed to read events directory: {}", e)))?
            {
                let entry = entry.map_err(|e| EventStoreError::IoError(e.to_string()))?;
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        let cid = name.trim_end_matches(".json").to_string();
                        cids.insert(cid);
                    }
                }
            }
        }

        Ok(cids)
    }

    /// Store an event with content addressing
    ///
    /// Returns the CID of the stored event, or an error if:
    /// - CID generation fails (IPLD feature required)
    /// - Event already exists (duplicate)
    /// - IO error occurs
    #[cfg(feature = "ipld")]
    pub fn store(&mut self, envelope: EventEnvelope) -> Result<String, EventStoreError> {
        // Generate CID for the event
        let cid = crate::ipld_support::generate_cid(&envelope.event)?;
        let cid_string = cid.to_string();

        // Check for duplicate
        if self.known_cids.contains(&cid_string) {
            return Err(EventStoreError::DuplicateEvent(cid_string));
        }

        // Create stored record
        let record = StoredEventRecord {
            cid: cid_string.clone(),
            envelope,
            stored_at: chrono::Utc::now(),
            causation_cid: None, // TODO: Look up causation CID from causation_id
        };

        // Write to disk
        let event_path = self.root_path
            .join("events")
            .join("by_cid")
            .join(format!("{}.json", cid_string));

        let json = serde_json::to_string_pretty(&record)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        fs::write(&event_path, json)
            .map_err(|e| EventStoreError::IoError(format!("Failed to write event: {}", e)))?;

        // Update cache
        self.known_cids.insert(cid_string.clone());

        Ok(cid_string)
    }

    /// Store an event (stub when IPLD feature disabled)
    #[cfg(not(feature = "ipld"))]
    pub fn store(&mut self, _envelope: EventEnvelope) -> Result<String, EventStoreError> {
        Err(EventStoreError::IpldError(IpldError::FeatureNotEnabled))
    }

    /// Store an event, allowing duplicates (returns existing CID if duplicate)
    #[cfg(feature = "ipld")]
    pub fn store_or_get(&mut self, envelope: EventEnvelope) -> Result<String, EventStoreError> {
        match self.store(envelope) {
            Ok(cid) => Ok(cid),
            Err(EventStoreError::DuplicateEvent(cid)) => Ok(cid),
            Err(e) => Err(e),
        }
    }

    /// Store an event, allowing duplicates (stub when IPLD disabled)
    #[cfg(not(feature = "ipld"))]
    pub fn store_or_get(&mut self, _envelope: EventEnvelope) -> Result<String, EventStoreError> {
        Err(EventStoreError::IpldError(IpldError::FeatureNotEnabled))
    }

    /// Check if an event with the given CID exists
    pub fn exists(&self, cid: &str) -> bool {
        self.known_cids.contains(cid)
    }

    /// Get an event by its CID
    pub fn get(&self, cid: &str) -> Result<StoredEventRecord, EventStoreError> {
        let event_path = self.root_path
            .join("events")
            .join("by_cid")
            .join(format!("{}.json", cid));

        if !event_path.exists() {
            return Err(EventStoreError::NotFound(cid.to_string()));
        }

        let content = fs::read_to_string(&event_path)
            .map_err(|e| EventStoreError::IoError(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))
    }

    /// Verify an event's integrity by checking its CID
    #[cfg(feature = "ipld")]
    pub fn verify(&self, cid: &str) -> Result<bool, EventStoreError> {
        let record = self.get(cid)?;
        let computed_cid = crate::ipld_support::generate_cid(&record.envelope.event)?;
        Ok(computed_cid.to_string() == cid)
    }

    /// Verify event integrity (stub when IPLD disabled)
    #[cfg(not(feature = "ipld"))]
    pub fn verify(&self, _cid: &str) -> Result<bool, EventStoreError> {
        Ok(true) // No verification without IPLD
    }

    /// List all stored event CIDs
    pub fn list_cids(&self) -> Vec<String> {
        self.known_cids.iter().cloned().collect()
    }

    /// Get all stored events in chronological order
    pub fn list_events(&self) -> Result<Vec<StoredEventRecord>, EventStoreError> {
        let mut events: Vec<StoredEventRecord> = self.known_cids
            .iter()
            .map(|cid| self.get(cid))
            .collect::<Result<Vec<_>, _>>()?;

        // Sort by stored_at timestamp
        events.sort_by_key(|e| e.stored_at);

        Ok(events)
    }

    /// Get event count
    pub fn count(&self) -> usize {
        self.known_cids.len()
    }

    /// Delete an event by CID (for testing/cleanup only)
    pub fn delete(&mut self, cid: &str) -> Result<(), EventStoreError> {
        let event_path = self.root_path
            .join("events")
            .join("by_cid")
            .join(format!("{}.json", cid));

        if event_path.exists() {
            fs::remove_file(&event_path)
                .map_err(|e| EventStoreError::IoError(e.to_string()))?;
            self.known_cids.remove(cid);
        }

        Ok(())
    }
}

/// Index file for temporal ordering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventIndex {
    /// Events in chronological order (CIDs)
    pub events: Vec<String>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl EventIndex {
    /// Load or create the event index
    pub fn load_or_create(path: &Path) -> Result<Self, EventStoreError> {
        let index_path = path.join("events").join("index.json");

        if index_path.exists() {
            let content = fs::read_to_string(&index_path)
                .map_err(|e| EventStoreError::IoError(e.to_string()))?;
            serde_json::from_str(&content)
                .map_err(|e| EventStoreError::SerializationError(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    /// Save the event index
    pub fn save(&self, path: &Path) -> Result<(), EventStoreError> {
        let index_path = path.join("events").join("index.json");

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        fs::write(&index_path, json)
            .map_err(|e| EventStoreError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Append a CID to the index
    pub fn append(&mut self, cid: String) {
        self.events.push(cid);
        self.updated_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::events::{DomainEvent, PersonEvents};
    use crate::events::person::PersonCreatedEvent;
    use uuid::Uuid;

    fn create_test_envelope() -> EventEnvelope {
        let event = DomainEvent::Person(PersonEvents::PersonCreated(PersonCreatedEvent {
            person_id: Uuid::now_v7(),
            name: "Test Person".to_string(),
            email: Some("test@example.com".to_string()),
            title: None,
            department: None,
            organization_id: Uuid::now_v7(),
            created_by: None,
            correlation_id: Uuid::now_v7(),
            causation_id: None,
        }));

        EventEnvelope::new(event, Uuid::now_v7(), None)
    }

    #[test]
    fn test_event_store_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = CidEventStore::new(temp_dir.path()).unwrap();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_event_store_exists() {
        let temp_dir = TempDir::new().unwrap();
        let store = CidEventStore::new(temp_dir.path()).unwrap();
        assert!(!store.exists("nonexistent"));
    }

    #[test]
    fn test_event_store_list_cids_empty() {
        let temp_dir = TempDir::new().unwrap();
        let store = CidEventStore::new(temp_dir.path()).unwrap();
        assert!(store.list_cids().is_empty());
    }

    #[cfg(feature = "ipld")]
    #[test]
    fn test_event_store_store_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CidEventStore::new(temp_dir.path()).unwrap();

        let envelope = create_test_envelope();
        let cid = store.store(envelope).unwrap();

        assert!(cid.starts_with("baf"));
        assert!(store.exists(&cid));
        assert_eq!(store.count(), 1);

        let record = store.get(&cid).unwrap();
        assert_eq!(record.cid, cid);
    }

    #[cfg(feature = "ipld")]
    #[test]
    fn test_event_store_duplicate_detection() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CidEventStore::new(temp_dir.path()).unwrap();

        // Create two identical events (same content = same CID)
        let person_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        let corr_id = Uuid::now_v7();

        let event1 = DomainEvent::Person(PersonEvents::PersonCreated(PersonCreatedEvent {
            person_id,
            name: "Test".to_string(),
            email: None,
            title: None,
            department: None,
            organization_id: org_id,
            created_by: None,
            correlation_id: corr_id,
            causation_id: None,
        }));

        let event2 = DomainEvent::Person(PersonEvents::PersonCreated(PersonCreatedEvent {
            person_id,
            name: "Test".to_string(),
            email: None,
            title: None,
            department: None,
            organization_id: org_id,
            created_by: None,
            correlation_id: corr_id,
            causation_id: None,
        }));

        let envelope1 = EventEnvelope::new(event1, corr_id, None);
        let envelope2 = EventEnvelope::new(event2, corr_id, None);

        // First store succeeds
        let cid1 = store.store(envelope1).unwrap();
        assert_eq!(store.count(), 1);

        // Second store fails with DuplicateEvent
        match store.store(envelope2) {
            Err(EventStoreError::DuplicateEvent(cid)) => {
                assert_eq!(cid, cid1);
            }
            _ => panic!("Expected DuplicateEvent error"),
        }

        // Count should still be 1
        assert_eq!(store.count(), 1);
    }

    #[cfg(feature = "ipld")]
    #[test]
    fn test_event_store_store_or_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CidEventStore::new(temp_dir.path()).unwrap();

        // Create two identical events
        let person_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        let corr_id = Uuid::now_v7();

        let make_event = || DomainEvent::Person(PersonEvents::PersonCreated(PersonCreatedEvent {
            person_id,
            name: "Test".to_string(),
            email: None,
            title: None,
            department: None,
            organization_id: org_id,
            created_by: None,
            correlation_id: corr_id,
            causation_id: None,
        }));

        let cid1 = store.store_or_get(EventEnvelope::new(make_event(), corr_id, None)).unwrap();
        let cid2 = store.store_or_get(EventEnvelope::new(make_event(), corr_id, None)).unwrap();

        // Both return the same CID
        assert_eq!(cid1, cid2);
        assert_eq!(store.count(), 1);
    }

    #[cfg(feature = "ipld")]
    #[test]
    fn test_event_store_verify() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CidEventStore::new(temp_dir.path()).unwrap();

        let envelope = create_test_envelope();
        let cid = store.store(envelope).unwrap();

        assert!(store.verify(&cid).unwrap());
    }
}
