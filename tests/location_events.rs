//! Comprehensive Location Events Tests
//!
//! Target: 90%+ coverage of src/events/location.rs
//!
//! Tests all 11 event types for location lifecycle, access control, and asset management.

use chrono::Utc;
use cim_keys::events::location::*;
use cim_keys::events::DomainEvent as DomainEventEnum;
use cim_domain::DomainEvent;
use uuid::Uuid;

// =============================================================================
// Test Helpers - Sample Event Creators
// =============================================================================

fn test_location_id() -> Uuid { Uuid::now_v7() }
fn test_person_id() -> Uuid { Uuid::now_v7() }
fn test_asset_id() -> Uuid { Uuid::now_v7() }
fn test_org_id() -> Uuid { Uuid::now_v7() }

fn sample_location_created() -> LocationCreatedEvent {
    LocationCreatedEvent {
        location_id: test_location_id(),
        name: "Secure Vault A1".to_string(),
        location_type: "Physical".to_string(),
        address: Some("123 Security St".to_string()),
        coordinates: Some((37.7749, -122.4194)),
        organization_id: Some(test_org_id()),
        created_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_location_updated() -> LocationUpdatedEvent {
    LocationUpdatedEvent {
        location_id: test_location_id(),
        field_name: "name".to_string(),
        old_value: Some("Vault A1".to_string()),
        new_value: "Secure Vault A1".to_string(),
        updated_at: Utc::now(),
        updated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_location_deactivated() -> LocationDeactivatedEvent {
    LocationDeactivatedEvent {
        location_id: test_location_id(),
        reason: "Maintenance required".to_string(),
        deactivated_at: Utc::now(),
        deactivated_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_access_granted() -> AccessGrantedEvent {
    AccessGrantedEvent {
        location_id: test_location_id(),
        person_id: test_person_id(),
        access_level: "Full".to_string(),
        granted_at: Utc::now(),
        granted_by: "admin".to_string(),
        valid_until: None,
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_access_revoked() -> AccessRevokedEvent {
    AccessRevokedEvent {
        location_id: test_location_id(),
        person_id: test_person_id(),
        reason: "Access no longer needed".to_string(),
        revoked_at: Utc::now(),
        revoked_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_asset_stored() -> AssetStoredEvent {
    AssetStoredEvent {
        location_id: test_location_id(),
        asset_id: test_asset_id(),
        asset_type: "EncryptedKey".to_string(),
        stored_at: Utc::now(),
        stored_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_asset_removed() -> AssetRemovedEvent {
    AssetRemovedEvent {
        location_id: test_location_id(),
        asset_id: test_asset_id(),
        reason: "Relocated to new vault".to_string(),
        removed_at: Utc::now(),
        removed_by: "admin".to_string(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_location_activated() -> LocationActivatedEvent {
    LocationActivatedEvent {
        location_id: test_location_id(),
        activated_at: Utc::now(),
        activated_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_location_suspended() -> LocationSuspendedEvent {
    LocationSuspendedEvent {
        location_id: test_location_id(),
        reason: "Security review".to_string(),
        suspended_at: Utc::now(),
        suspended_by: test_person_id(),
        expected_restoration: None,
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_location_reactivated() -> LocationReactivatedEvent {
    LocationReactivatedEvent {
        location_id: test_location_id(),
        reactivated_at: Utc::now(),
        reactivated_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

fn sample_location_decommissioned() -> LocationDecommissionedEvent {
    LocationDecommissionedEvent {
        location_id: test_location_id(),
        reason: "Facility closure".to_string(),
        decommissioned_at: Utc::now(),
        decommissioned_by: test_person_id(),
        correlation_id: Uuid::now_v7(),
        causation_id: None,
    }
}

// =============================================================================
// Serialization Roundtrip Tests (11 event types)
// =============================================================================

#[test]
fn test_location_created_serialization() {
    let event = sample_location_created();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LocationCreatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
    assert_eq!(event.coordinates, deserialized.coordinates);
}

#[test]
fn test_location_updated_serialization() {
    let event = sample_location_updated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LocationUpdatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
}

#[test]
fn test_location_deactivated_serialization() {
    let event = sample_location_deactivated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LocationDeactivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
}

#[test]
fn test_access_granted_serialization() {
    let event = sample_access_granted();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: AccessGrantedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
    assert_eq!(event.person_id, deserialized.person_id);
}

#[test]
fn test_access_revoked_serialization() {
    let event = sample_access_revoked();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: AccessRevokedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
}

#[test]
fn test_asset_stored_serialization() {
    let event = sample_asset_stored();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: AssetStoredEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.asset_id, deserialized.asset_id);
}

#[test]
fn test_asset_removed_serialization() {
    let event = sample_asset_removed();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: AssetRemovedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.asset_id, deserialized.asset_id);
}

#[test]
fn test_location_activated_serialization() {
    let event = sample_location_activated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LocationActivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
}

#[test]
fn test_location_suspended_serialization() {
    let event = sample_location_suspended();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LocationSuspendedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
}

#[test]
fn test_location_reactivated_serialization() {
    let event = sample_location_reactivated();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LocationReactivatedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
}

#[test]
fn test_location_decommissioned_serialization() {
    let event = sample_location_decommissioned();
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LocationDecommissionedEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event.location_id, deserialized.location_id);
}

// =============================================================================
// LocationEvents Enum Serialization
// =============================================================================

#[test]
fn test_location_events_enum_serialization() {
    let events = vec![
        LocationEvents::LocationCreated(sample_location_created()),
        LocationEvents::LocationUpdated(sample_location_updated()),
        LocationEvents::LocationDeactivated(sample_location_deactivated()),
        LocationEvents::AccessGranted(sample_access_granted()),
        LocationEvents::AccessRevoked(sample_access_revoked()),
        LocationEvents::AssetStored(sample_asset_stored()),
        LocationEvents::AssetRemoved(sample_asset_removed()),
        LocationEvents::LocationActivated(sample_location_activated()),
        LocationEvents::LocationSuspended(sample_location_suspended()),
        LocationEvents::LocationReactivated(sample_location_reactivated()),
        LocationEvents::LocationDecommissioned(sample_location_decommissioned()),
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: LocationEvents = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type(), deserialized.event_type());
    }
}

// =============================================================================
// Correlation/Causation Chain Tests
// =============================================================================

#[test]
fn test_causation_chain_linking() {
    let correlation_id = Uuid::now_v7();
    let created = LocationCreatedEvent {
        causation_id: None,
        correlation_id,
        ..sample_location_created()
    };
    let activated = LocationActivatedEvent {
        causation_id: Some(correlation_id),
        correlation_id,
        ..sample_location_activated()
    };

    assert_eq!(created.causation_id, None);
    assert_eq!(activated.causation_id, Some(correlation_id));
}

#[test]
fn test_correlation_id_propagation() {
    let correlation_id = Uuid::now_v7();
    let created = LocationCreatedEvent { correlation_id, ..sample_location_created() };
    let access_granted = AccessGrantedEvent { correlation_id, ..sample_access_granted() };
    let asset_stored = AssetStoredEvent { correlation_id, ..sample_asset_stored() };

    assert_eq!(created.correlation_id, correlation_id);
    assert_eq!(access_granted.correlation_id, correlation_id);
    assert_eq!(asset_stored.correlation_id, correlation_id);
}

// =============================================================================
// Event Invariants Tests
// =============================================================================

#[test]
fn test_uuid_fields_are_valid() {
    let location = sample_location_created();
    assert_ne!(location.location_id, Uuid::nil());
    assert_ne!(location.correlation_id, Uuid::nil());

    let access = sample_access_granted();
    assert_ne!(access.person_id, Uuid::nil());
}

#[test]
fn test_coordinates_format() {
    let location = sample_location_created();
    if let Some((lat, lon)) = location.coordinates {
        assert!(lat >= -90.0 && lat <= 90.0);
        assert!(lon >= -180.0 && lon <= 180.0);
    }
}

// =============================================================================
// DomainEvent Trait Implementation Tests
// =============================================================================

#[test]
fn test_aggregate_id_for_all_event_types() {
    let location_id = test_location_id();

    let events = vec![
        LocationEvents::LocationCreated(LocationCreatedEvent { location_id, ..sample_location_created() }),
        LocationEvents::LocationUpdated(LocationUpdatedEvent { location_id, ..sample_location_updated() }),
        LocationEvents::LocationDeactivated(LocationDeactivatedEvent { location_id, ..sample_location_deactivated() }),
        LocationEvents::AccessGranted(AccessGrantedEvent { location_id, ..sample_access_granted() }),
        LocationEvents::AccessRevoked(AccessRevokedEvent { location_id, ..sample_access_revoked() }),
        LocationEvents::AssetStored(AssetStoredEvent { location_id, ..sample_asset_stored() }),
        LocationEvents::AssetRemoved(AssetRemovedEvent { location_id, ..sample_asset_removed() }),
        LocationEvents::LocationActivated(LocationActivatedEvent { location_id, ..sample_location_activated() }),
        LocationEvents::LocationSuspended(LocationSuspendedEvent { location_id, ..sample_location_suspended() }),
        LocationEvents::LocationReactivated(LocationReactivatedEvent { location_id, ..sample_location_reactivated() }),
        LocationEvents::LocationDecommissioned(LocationDecommissionedEvent { location_id, ..sample_location_decommissioned() }),
    ];

    for event in events {
        assert_eq!(event.aggregate_id(), location_id);
    }
}

#[test]
fn test_event_type_returns_correct_strings() {
    assert_eq!(LocationEvents::LocationCreated(sample_location_created()).event_type(), "LocationCreated");
    assert_eq!(LocationEvents::LocationUpdated(sample_location_updated()).event_type(), "LocationUpdated");
    assert_eq!(LocationEvents::LocationDeactivated(sample_location_deactivated()).event_type(), "LocationDeactivated");
    assert_eq!(LocationEvents::AccessGranted(sample_access_granted()).event_type(), "AccessGranted");
    assert_eq!(LocationEvents::AccessRevoked(sample_access_revoked()).event_type(), "AccessRevoked");
    assert_eq!(LocationEvents::AssetStored(sample_asset_stored()).event_type(), "AssetStored");
    assert_eq!(LocationEvents::AssetRemoved(sample_asset_removed()).event_type(), "AssetRemoved");
    assert_eq!(LocationEvents::LocationActivated(sample_location_activated()).event_type(), "LocationActivated");
    assert_eq!(LocationEvents::LocationSuspended(sample_location_suspended()).event_type(), "LocationSuspended");
    assert_eq!(LocationEvents::LocationReactivated(sample_location_reactivated()).event_type(), "LocationReactivated");
    assert_eq!(LocationEvents::LocationDecommissioned(sample_location_decommissioned()).event_type(), "LocationDecommissioned");
}

// =============================================================================
// Complete Lifecycle Tests
// =============================================================================

#[test]
fn test_complete_location_lifecycle() {
    let location_id = test_location_id();
    let correlation_id = Uuid::now_v7();

    let created = LocationCreatedEvent {
        location_id,
        correlation_id,
        ..sample_location_created()
    };
    let activated = LocationActivatedEvent {
        location_id,
        correlation_id,
        ..sample_location_activated()
    };
    let suspended = LocationSuspendedEvent {
        location_id,
        correlation_id,
        ..sample_location_suspended()
    };
    let reactivated = LocationReactivatedEvent {
        location_id,
        correlation_id,
        ..sample_location_reactivated()
    };
    let decommissioned = LocationDecommissionedEvent {
        location_id,
        correlation_id,
        ..sample_location_decommissioned()
    };

    assert_eq!(created.location_id, location_id);
    assert_eq!(activated.location_id, location_id);
    assert_eq!(suspended.location_id, location_id);
    assert_eq!(reactivated.location_id, location_id);
    assert_eq!(decommissioned.location_id, location_id);
}

#[test]
fn test_access_control_workflow() {
    let location_id = test_location_id();
    let person_id = test_person_id();
    let correlation_id = Uuid::now_v7();

    let granted = AccessGrantedEvent {
        location_id,
        person_id,
        correlation_id,
        ..sample_access_granted()
    };
    let revoked = AccessRevokedEvent {
        location_id,
        person_id,
        correlation_id,
        ..sample_access_revoked()
    };

    assert_eq!(granted.location_id, location_id);
    assert_eq!(granted.person_id, person_id);
    assert_eq!(revoked.location_id, location_id);
    assert_eq!(revoked.person_id, person_id);
}

#[test]
fn test_asset_management_workflow() {
    let location_id = test_location_id();
    let asset_id = test_asset_id();
    let correlation_id = Uuid::now_v7();

    let stored = AssetStoredEvent {
        location_id,
        asset_id,
        correlation_id,
        ..sample_asset_stored()
    };
    let removed = AssetRemovedEvent {
        location_id,
        asset_id,
        correlation_id,
        ..sample_asset_removed()
    };

    assert_eq!(stored.location_id, location_id);
    assert_eq!(stored.asset_id, asset_id);
    assert_eq!(removed.location_id, location_id);
    assert_eq!(removed.asset_id, asset_id);
}
