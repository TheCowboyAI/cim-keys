// Copyright (c) 2025 - Cowboy AI, LLC.

//! Location Message Definitions
//!
//! This module defines the message types for the Location bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **Form Input**: Name, type, and address fields
//! 2. **Lifecycle**: Add and remove locations

use uuid::Uuid;

use crate::domain::LocationType;

/// Location Message
///
/// Organized by sub-domain:
/// - Form Input (8 messages)
/// - Lifecycle (2 messages)
#[derive(Debug, Clone)]
pub enum LocationMessage {
    // === Form Input ===
    /// Location name changed
    NameChanged(String),
    /// Location type selected (Physical, Virtual, Hybrid)
    TypeSelected(LocationType),
    /// Street address changed
    StreetChanged(String),
    /// City changed
    CityChanged(String),
    /// State/Region changed
    RegionChanged(String),
    /// Country changed
    CountryChanged(String),
    /// Postal code changed
    PostalChanged(String),
    /// URL changed (for virtual/hybrid locations)
    UrlChanged(String),

    // === Lifecycle ===
    /// Add a new location
    AddLocation,
    /// Remove an existing location
    RemoveLocation(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_message_variants() {
        let _ = LocationMessage::NameChanged("HQ".to_string());
        let _ = LocationMessage::TypeSelected(LocationType::Physical);
        let _ = LocationMessage::StreetChanged("123 Main St".to_string());
        let _ = LocationMessage::CityChanged("Austin".to_string());
        let _ = LocationMessage::RegionChanged("TX".to_string());
        let _ = LocationMessage::CountryChanged("USA".to_string());
        let _ = LocationMessage::PostalChanged("78701".to_string());
        let _ = LocationMessage::UrlChanged("https://example.com".to_string());
        let _ = LocationMessage::AddLocation;
        let _ = LocationMessage::RemoveLocation(Uuid::nil());
    }
}
