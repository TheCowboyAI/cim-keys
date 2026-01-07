// Copyright (c) 2025 - Cowboy AI, LLC.

//! Location Management Bounded Context
//!
//! This module implements the Location domain with:
//! - Message enum for all location operations
//! - State struct for location-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **Form Input**: Name, type, and address fields
//! 2. **Lifecycle**: Add and remove locations
//! 3. **Loaded Data**: Locations loaded from projection

use iced::Task;
use uuid::Uuid;

use crate::domain::LocationType;
use crate::projections::LocationEntry;

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

/// Location State
///
/// Contains all state related to location management.
#[derive(Debug, Clone, Default)]
pub struct LocationState {
    // === Form Input ===
    /// Name of the new location
    pub new_location_name: String,
    /// Type of location (Physical, Virtual, Hybrid)
    pub new_location_type: Option<LocationType>,
    /// Street address for physical locations
    pub new_location_street: String,
    /// City
    pub new_location_city: String,
    /// State or region
    pub new_location_region: String,
    /// Country
    pub new_location_country: String,
    /// Postal/ZIP code
    pub new_location_postal: String,
    /// URL for virtual/hybrid locations
    pub new_location_url: String,

    // === Loaded Data ===
    /// Locations loaded from projection/manifest
    pub loaded_locations: Vec<LocationEntry>,
}

impl LocationState {
    /// Create a new LocationState with sensible defaults
    pub fn new() -> Self {
        Self {
            new_location_name: String::new(),
            new_location_type: None,
            new_location_street: String::new(),
            new_location_city: String::new(),
            new_location_region: String::new(),
            new_location_country: String::new(),
            new_location_postal: String::new(),
            new_location_url: String::new(),
            loaded_locations: Vec::new(),
        }
    }

    /// Check if form has minimum required fields for a physical location
    pub fn is_physical_location_valid(&self) -> bool {
        !self.new_location_name.is_empty()
            && !self.new_location_street.is_empty()
            && !self.new_location_city.is_empty()
            && !self.new_location_region.is_empty()
            && !self.new_location_country.is_empty()
            && !self.new_location_postal.is_empty()
    }

    /// Check if form has minimum required fields for a virtual location
    pub fn is_virtual_location_valid(&self) -> bool {
        !self.new_location_name.is_empty() && !self.new_location_url.is_empty()
    }

    /// Check if the form is valid based on location type
    pub fn is_form_valid(&self) -> bool {
        match self.new_location_type {
            Some(LocationType::Physical) => self.is_physical_location_valid(),
            Some(LocationType::Virtual) => self.is_virtual_location_valid(),
            Some(LocationType::Logical) => {
                // Logical locations only need a name
                !self.new_location_name.is_empty()
            }
            Some(LocationType::Hybrid) => {
                self.is_physical_location_valid() || self.is_virtual_location_valid()
            }
            None => !self.new_location_name.is_empty(),
        }
    }

    /// Get count of loaded locations
    pub fn location_count(&self) -> usize {
        self.loaded_locations.len()
    }

    /// Find a location by ID
    pub fn find_location(&self, id: Uuid) -> Option<&LocationEntry> {
        self.loaded_locations.iter().find(|loc| loc.location_id == id)
    }

    /// Clear the form fields after successful addition
    pub fn clear_form(&mut self) {
        self.new_location_name.clear();
        self.new_location_type = None;
        self.new_location_street.clear();
        self.new_location_city.clear();
        self.new_location_region.clear();
        self.new_location_country.clear();
        self.new_location_postal.clear();
        self.new_location_url.clear();
    }

    /// Get validation error message if form is invalid
    pub fn validation_error(&self) -> Option<String> {
        if self.new_location_name.is_empty() {
            return Some("Location name is required".to_string());
        }

        match self.new_location_type {
            Some(LocationType::Physical) | None => {
                if self.new_location_street.is_empty() {
                    return Some("Street address is required".to_string());
                }
                if self.new_location_city.is_empty() {
                    return Some("City is required".to_string());
                }
                if self.new_location_region.is_empty() {
                    return Some("State/Region is required".to_string());
                }
                if self.new_location_country.is_empty() {
                    return Some("Country is required".to_string());
                }
                if self.new_location_postal.is_empty() {
                    return Some("Postal code is required".to_string());
                }
            }
            Some(LocationType::Virtual) => {
                if self.new_location_url.is_empty() {
                    return Some("URL is required for virtual locations".to_string());
                }
            }
            Some(LocationType::Logical) => {
                // Logical locations only need a name (already validated above)
            }
            Some(LocationType::Hybrid) => {
                // Hybrid needs at least a URL or full address
                let has_url = !self.new_location_url.is_empty();
                let has_address = !self.new_location_street.is_empty()
                    && !self.new_location_city.is_empty()
                    && !self.new_location_region.is_empty()
                    && !self.new_location_country.is_empty()
                    && !self.new_location_postal.is_empty();

                if !has_url && !has_address {
                    return Some(
                        "Hybrid locations require either a URL or physical address".to_string(),
                    );
                }
            }
        }

        None
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update location state based on message
///
/// This function handles location domain messages. Note that AddLocation
/// and RemoveLocation require access to the projection and will be
/// delegated to the main update function.
pub fn update(state: &mut LocationState, message: LocationMessage) -> Task<Message> {
    use LocationMessage::*;

    match message {
        // === Form Input ===
        NameChanged(value) => {
            state.new_location_name = value;
            Task::none()
        }

        TypeSelected(location_type) => {
            state.new_location_type = Some(location_type);
            Task::none()
        }

        StreetChanged(value) => {
            state.new_location_street = value;
            Task::none()
        }

        CityChanged(value) => {
            state.new_location_city = value;
            Task::none()
        }

        RegionChanged(value) => {
            state.new_location_region = value;
            Task::none()
        }

        CountryChanged(value) => {
            state.new_location_country = value;
            Task::none()
        }

        PostalChanged(value) => {
            state.new_location_postal = value;
            Task::none()
        }

        UrlChanged(value) => {
            state.new_location_url = value;
            Task::none()
        }

        // === Lifecycle (delegated to main for projection access) ===
        AddLocation => {
            // Actual persistence requires projection - delegated to main
            Task::none()
        }

        RemoveLocation(_location_id) => {
            // Actual removal requires projection - delegated to main
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_state_default() {
        let state = LocationState::default();
        assert!(state.new_location_name.is_empty());
        assert!(state.new_location_type.is_none());
        assert!(state.new_location_street.is_empty());
        assert!(state.new_location_city.is_empty());
        assert!(state.new_location_region.is_empty());
        assert!(state.new_location_country.is_empty());
        assert!(state.new_location_postal.is_empty());
        assert!(state.new_location_url.is_empty());
        assert!(state.loaded_locations.is_empty());
    }

    #[test]
    fn test_location_state_new() {
        let state = LocationState::new();
        assert!(state.new_location_name.is_empty());
        assert!(state.new_location_type.is_none());
        assert_eq!(state.location_count(), 0);
    }

    #[test]
    fn test_name_changed() {
        let mut state = LocationState::new();
        let _ = update(&mut state, LocationMessage::NameChanged("HQ".to_string()));
        assert_eq!(state.new_location_name, "HQ");
    }

    #[test]
    fn test_type_selected() {
        let mut state = LocationState::new();
        let _ = update(
            &mut state,
            LocationMessage::TypeSelected(LocationType::Physical),
        );
        assert_eq!(state.new_location_type, Some(LocationType::Physical));
    }

    #[test]
    fn test_address_fields() {
        let mut state = LocationState::new();

        let _ = update(
            &mut state,
            LocationMessage::StreetChanged("123 Main St".to_string()),
        );
        assert_eq!(state.new_location_street, "123 Main St");

        let _ = update(
            &mut state,
            LocationMessage::CityChanged("Austin".to_string()),
        );
        assert_eq!(state.new_location_city, "Austin");

        let _ = update(
            &mut state,
            LocationMessage::RegionChanged("Texas".to_string()),
        );
        assert_eq!(state.new_location_region, "Texas");

        let _ = update(&mut state, LocationMessage::CountryChanged("USA".to_string()));
        assert_eq!(state.new_location_country, "USA");

        let _ = update(
            &mut state,
            LocationMessage::PostalChanged("78701".to_string()),
        );
        assert_eq!(state.new_location_postal, "78701");
    }

    #[test]
    fn test_url_changed() {
        let mut state = LocationState::new();
        let _ = update(
            &mut state,
            LocationMessage::UrlChanged("https://hq.example.com".to_string()),
        );
        assert_eq!(state.new_location_url, "https://hq.example.com");
    }

    #[test]
    fn test_is_physical_location_valid() {
        let mut state = LocationState::new();
        assert!(!state.is_physical_location_valid());

        state.new_location_name = "HQ".to_string();
        state.new_location_street = "123 Main St".to_string();
        state.new_location_city = "Austin".to_string();
        state.new_location_region = "Texas".to_string();
        state.new_location_country = "USA".to_string();
        state.new_location_postal = "78701".to_string();

        assert!(state.is_physical_location_valid());
    }

    #[test]
    fn test_is_virtual_location_valid() {
        let mut state = LocationState::new();
        assert!(!state.is_virtual_location_valid());

        state.new_location_name = "Cloud".to_string();
        assert!(!state.is_virtual_location_valid());

        state.new_location_url = "https://cloud.example.com".to_string();
        assert!(state.is_virtual_location_valid());
    }

    #[test]
    fn test_is_form_valid_physical() {
        let mut state = LocationState::new();
        state.new_location_type = Some(LocationType::Physical);
        assert!(!state.is_form_valid());

        state.new_location_name = "HQ".to_string();
        state.new_location_street = "123 Main St".to_string();
        state.new_location_city = "Austin".to_string();
        state.new_location_region = "Texas".to_string();
        state.new_location_country = "USA".to_string();
        state.new_location_postal = "78701".to_string();

        assert!(state.is_form_valid());
    }

    #[test]
    fn test_is_form_valid_virtual() {
        let mut state = LocationState::new();
        state.new_location_type = Some(LocationType::Virtual);
        assert!(!state.is_form_valid());

        state.new_location_name = "Cloud".to_string();
        state.new_location_url = "https://cloud.example.com".to_string();

        assert!(state.is_form_valid());
    }

    #[test]
    fn test_validation_error_name_required() {
        let state = LocationState::new();
        assert_eq!(
            state.validation_error(),
            Some("Location name is required".to_string())
        );
    }

    #[test]
    fn test_validation_error_physical_fields() {
        let mut state = LocationState::new();
        state.new_location_name = "HQ".to_string();
        state.new_location_type = Some(LocationType::Physical);

        assert_eq!(
            state.validation_error(),
            Some("Street address is required".to_string())
        );

        state.new_location_street = "123 Main St".to_string();
        assert_eq!(
            state.validation_error(),
            Some("City is required".to_string())
        );
    }

    #[test]
    fn test_validation_error_virtual_url() {
        let mut state = LocationState::new();
        state.new_location_name = "Cloud".to_string();
        state.new_location_type = Some(LocationType::Virtual);

        assert_eq!(
            state.validation_error(),
            Some("URL is required for virtual locations".to_string())
        );
    }

    #[test]
    fn test_clear_form() {
        let mut state = LocationState::new();
        state.new_location_name = "HQ".to_string();
        state.new_location_type = Some(LocationType::Physical);
        state.new_location_street = "123 Main St".to_string();
        state.new_location_city = "Austin".to_string();
        state.new_location_region = "Texas".to_string();
        state.new_location_country = "USA".to_string();
        state.new_location_postal = "78701".to_string();
        state.new_location_url = "https://hq.example.com".to_string();

        state.clear_form();

        assert!(state.new_location_name.is_empty());
        assert!(state.new_location_type.is_none());
        assert!(state.new_location_street.is_empty());
        assert!(state.new_location_city.is_empty());
        assert!(state.new_location_region.is_empty());
        assert!(state.new_location_country.is_empty());
        assert!(state.new_location_postal.is_empty());
        assert!(state.new_location_url.is_empty());
    }

    #[test]
    fn test_location_count() {
        let mut state = LocationState::new();
        assert_eq!(state.location_count(), 0);

        state.loaded_locations.push(LocationEntry {
            location_id: Uuid::now_v7(),
            name: "HQ".to_string(),
            location_type: "Physical".to_string(),
            organization_id: Uuid::now_v7(),
            street: Some("123 Main St".to_string()),
            city: Some("Austin".to_string()),
            region: Some("Texas".to_string()),
            country: Some("USA".to_string()),
            postal_code: Some("78701".to_string()),
            virtual_url: None,
            state: None,
        });

        assert_eq!(state.location_count(), 1);
    }
}
