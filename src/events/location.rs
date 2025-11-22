//! Location Aggregate Events
//!
//! Events related to the Location aggregate root.
//! A Location represents physical or virtual storage locations for cryptographic assets.

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Events for the Location aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum LocationEvents {
    /// A new location was created
    LocationCreated(LocationCreatedEvent),

    /// Location information was updated
    LocationUpdated(LocationUpdatedEvent),

    /// Location was deactivated
    LocationDeactivated(LocationDeactivatedEvent),

    /// Access to location was granted
    AccessGranted(AccessGrantedEvent),

    /// Access to location was revoked
    AccessRevoked(AccessRevokedEvent),

    /// Asset was stored at location
    AssetStored(AssetStoredEvent),

    /// Asset was removed from location
    AssetRemoved(AssetRemovedEvent),

    // Lifecycle State Transitions (Phase 12)
    /// Location activated
    LocationActivated(LocationActivatedEvent),

    /// Location suspended
    LocationSuspended(LocationSuspendedEvent),

    /// Location reactivated
    LocationReactivated(LocationReactivatedEvent),

    /// Location decommissioned (terminal)
    LocationDecommissioned(LocationDecommissionedEvent),
}

/// A new location was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCreatedEvent {
    pub location_id: Uuid,
    pub name: String,
    pub location_type: String,
    pub address: Option<String>,
    pub coordinates: Option<(f64, f64)>,
    pub organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location information was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationUpdatedEvent {
    pub location_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location was deactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationDeactivatedEvent {
    pub location_id: Uuid,
    pub reason: String,
    pub deactivated_at: DateTime<Utc>,
    pub deactivated_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Access to location was granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessGrantedEvent {
    pub location_id: Uuid,
    pub person_id: Uuid,
    pub access_level: String,
    pub granted_at: DateTime<Utc>,
    pub granted_by: String,
    pub valid_until: Option<DateTime<Utc>>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Access to location was revoked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRevokedEvent {
    pub location_id: Uuid,
    pub person_id: Uuid,
    pub reason: String,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Asset was stored at location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStoredEvent {
    pub location_id: Uuid,
    pub asset_id: Uuid,
    pub asset_type: String,
    pub stored_at: DateTime<Utc>,
    pub stored_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Asset was removed from location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRemovedEvent {
    pub location_id: Uuid,
    pub asset_id: Uuid,
    pub reason: String,
    pub removed_at: DateTime<Utc>,
    pub removed_by: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

// ============================================================================
// Location Lifecycle State Transitions (Phase 12)
// ============================================================================

/// Location activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationActivatedEvent {
    pub location_id: Uuid,
    pub activated_at: DateTime<Utc>,
    pub activated_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location suspended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSuspendedEvent {
    pub location_id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Uuid,
    pub expected_restoration: Option<DateTime<Utc>>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location reactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReactivatedEvent {
    pub location_id: Uuid,
    pub reactivated_at: DateTime<Utc>,
    pub reactivated_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

/// Location decommissioned (terminal state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationDecommissionedEvent {
    pub location_id: Uuid,
    pub reason: String,
    pub decommissioned_at: DateTime<Utc>,
    pub decommissioned_by: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
}

impl DomainEvent for LocationEvents {
    fn aggregate_id(&self) -> Uuid {
        match self {
            LocationEvents::LocationCreated(e) => e.location_id,
            LocationEvents::LocationUpdated(e) => e.location_id,
            LocationEvents::LocationDeactivated(e) => e.location_id,
            LocationEvents::AccessGranted(e) => e.location_id,
            LocationEvents::AccessRevoked(e) => e.location_id,
            LocationEvents::AssetStored(e) => e.location_id,
            LocationEvents::AssetRemoved(e) => e.location_id,
            LocationEvents::LocationActivated(e) => e.location_id,
            LocationEvents::LocationSuspended(e) => e.location_id,
            LocationEvents::LocationReactivated(e) => e.location_id,
            LocationEvents::LocationDecommissioned(e) => e.location_id,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            LocationEvents::LocationCreated(_) => "LocationCreated",
            LocationEvents::LocationUpdated(_) => "LocationUpdated",
            LocationEvents::LocationDeactivated(_) => "LocationDeactivated",
            LocationEvents::AccessGranted(_) => "AccessGranted",
            LocationEvents::AccessRevoked(_) => "AccessRevoked",
            LocationEvents::AssetStored(_) => "AssetStored",
            LocationEvents::AssetRemoved(_) => "AssetRemoved",
            LocationEvents::LocationActivated(_) => "LocationActivated",
            LocationEvents::LocationSuspended(_) => "LocationSuspended",
            LocationEvents::LocationReactivated(_) => "LocationReactivated",
            LocationEvents::LocationDecommissioned(_) => "LocationDecommissioned",
        }
    }
}
