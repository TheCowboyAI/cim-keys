//! Location Aggregate Commands
//!
//! Commands for the Location aggregate root.
//! Currently re-exports from organization.rs - full migration pending.

// Re-export location-related commands from organization module
pub use super::organization::{
    CreateLocation,
    handle_create_location,
};

// TODO: Future refactoring
// - Add UpdateLocation command
// - Add DeactivateLocation command
// - Add GrantAccess, RevokeAccess commands
// - Add StoreAsset, RemoveAsset commands
// - Align with LocationEvents from events/location.rs
