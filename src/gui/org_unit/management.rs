// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Unit Management Bounded Context
//!
//! This module implements the Organization Unit domain with:
//! - Message enum for all unit operations
//! - State struct for unit-related fields
//! - Update function for message handling
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Form Input**: Name, type, parent, NATS account, responsible person
//! 3. **Lifecycle**: Create, created result, remove

use iced::Task;
use uuid::Uuid;

use crate::domain::{OrganizationUnit, OrganizationUnitType};

/// Organization Unit Message
///
/// Organized by sub-domain:
/// - UI State (1 message)
/// - Form Input (5 messages)
/// - Lifecycle (3 messages)
#[derive(Debug, Clone)]
pub enum OrgUnitMessage {
    // === UI State ===
    /// Toggle organization unit section visibility
    ToggleSection,

    // === Form Input ===
    /// Unit name changed
    NameChanged(String),
    /// Unit type selected (Division, Department, Team, etc.)
    TypeSelected(OrganizationUnitType),
    /// Parent unit selected (for hierarchy)
    ParentSelected(String),
    /// NATS account name changed
    NatsAccountChanged(String),
    /// Responsible person selected
    ResponsiblePersonSelected(Uuid),

    // === Lifecycle ===
    /// Create a new organization unit
    Create,
    /// Unit creation result
    Created(Result<OrganizationUnit, String>),
    /// Remove an organization unit
    Remove(Uuid),
}

/// Organization Unit State
///
/// Contains all state related to organization unit management.
#[derive(Debug, Clone, Default)]
pub struct OrgUnitState {
    // === UI State ===
    /// Whether the organization unit section is collapsed
    pub section_collapsed: bool,

    // === Form Input ===
    /// Name of the new unit
    pub new_name: String,
    /// Type of the new unit
    pub new_type: Option<OrganizationUnitType>,
    /// Parent unit name (for nesting)
    pub new_parent: Option<String>,
    /// Optional NATS account name for the unit
    pub new_nats_account: String,
    /// Optional responsible person for the unit
    pub new_responsible_person: Option<Uuid>,

    // === Created Units ===
    /// Units created in this session
    pub created_units: Vec<OrganizationUnit>,
}

impl OrgUnitState {
    /// Create a new OrgUnitState with sensible defaults
    pub fn new() -> Self {
        Self {
            section_collapsed: true,
            new_name: String::new(),
            new_type: None,
            new_parent: None,
            new_nats_account: String::new(),
            new_responsible_person: None,
            created_units: Vec::new(),
        }
    }

    /// Check if the form has minimum required fields
    pub fn is_form_valid(&self) -> bool {
        !self.new_name.is_empty() && self.new_type.is_some()
    }

    /// Get validation error message if form is invalid
    pub fn validation_error(&self) -> Option<String> {
        if self.new_name.is_empty() {
            return Some("Unit name is required".to_string());
        }
        if self.new_type.is_none() {
            return Some("Please select a unit type".to_string());
        }
        None
    }

    /// Clear the form fields after successful creation
    pub fn clear_form(&mut self) {
        self.new_name.clear();
        self.new_type = None;
        self.new_parent = None;
        self.new_nats_account.clear();
        self.new_responsible_person = None;
    }

    /// Get count of created units
    pub fn unit_count(&self) -> usize {
        self.created_units.len()
    }

    /// Find a unit by ID
    pub fn find_unit(&self, id: Uuid) -> Option<&OrganizationUnit> {
        self.created_units
            .iter()
            .find(|u| u.id.as_uuid() == id)
    }

    /// Find a unit by name
    pub fn find_unit_by_name(&self, name: &str) -> Option<&OrganizationUnit> {
        self.created_units.iter().find(|u| u.name == name)
    }

    /// Get unit names for parent selection dropdown
    pub fn unit_names(&self) -> Vec<&str> {
        self.created_units.iter().map(|u| u.name.as_str()).collect()
    }

    /// Remove a unit by ID, returns the removed unit's name if found
    pub fn remove_unit(&mut self, id: Uuid) -> Option<String> {
        if let Some(pos) = self.created_units.iter().position(|u| u.id.as_uuid() == id) {
            let removed = self.created_units.remove(pos);
            Some(removed.name)
        } else {
            None
        }
    }

    /// Get units by type
    pub fn units_by_type(&self, unit_type: OrganizationUnitType) -> Vec<&OrganizationUnit> {
        self.created_units
            .iter()
            .filter(|u| u.unit_type == unit_type)
            .collect()
    }

    /// Get root units (no parent)
    pub fn root_units(&self) -> Vec<&OrganizationUnit> {
        self.created_units
            .iter()
            .filter(|u| u.parent_unit_id.is_none())
            .collect()
    }
}

/// Root message type for delegation
pub type Message = crate::gui::Message;

/// Update organization unit state based on message
///
/// This function handles organization unit domain messages. Note that Create
/// requires additional context (building the unit with optional fields) and
/// will be delegated to the main update function.
pub fn update(state: &mut OrgUnitState, message: OrgUnitMessage) -> Task<Message> {
    use OrgUnitMessage::*;

    match message {
        // === UI State ===
        ToggleSection => {
            state.section_collapsed = !state.section_collapsed;
            Task::none()
        }

        // === Form Input ===
        NameChanged(name) => {
            state.new_name = name;
            Task::none()
        }

        TypeSelected(unit_type) => {
            state.new_type = Some(unit_type);
            Task::none()
        }

        ParentSelected(parent) => {
            state.new_parent = if parent.is_empty() {
                None
            } else {
                Some(parent)
            };
            Task::none()
        }

        NatsAccountChanged(account) => {
            state.new_nats_account = account;
            Task::none()
        }

        ResponsiblePersonSelected(person_id) => {
            state.new_responsible_person = Some(person_id);
            Task::none()
        }

        // === Lifecycle (delegated to main for unit building) ===
        Create => {
            // Actual creation requires building the unit with optional fields
            // Delegated to main update function
            Task::none()
        }

        Created(_result) => {
            // Result handling requires status/error messaging
            // Delegated to main update function
            Task::none()
        }

        Remove(unit_id) => {
            state.remove_unit(unit_id);
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_unit_state_default() {
        let state = OrgUnitState::default();
        assert!(state.new_name.is_empty());
        assert!(state.new_type.is_none());
        assert!(state.new_parent.is_none());
        assert!(state.new_nats_account.is_empty());
        assert!(state.new_responsible_person.is_none());
        assert!(state.created_units.is_empty());
        assert!(!state.section_collapsed);
    }

    #[test]
    fn test_org_unit_state_new() {
        let state = OrgUnitState::new();
        assert!(state.new_name.is_empty());
        assert!(state.new_type.is_none());
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_toggle_section() {
        let mut state = OrgUnitState::new();
        assert!(state.section_collapsed);

        let _ = update(&mut state, OrgUnitMessage::ToggleSection);
        assert!(!state.section_collapsed);

        let _ = update(&mut state, OrgUnitMessage::ToggleSection);
        assert!(state.section_collapsed);
    }

    #[test]
    fn test_name_changed() {
        let mut state = OrgUnitState::new();
        let _ = update(
            &mut state,
            OrgUnitMessage::NameChanged("Engineering".to_string()),
        );
        assert_eq!(state.new_name, "Engineering");
    }

    #[test]
    fn test_type_selected() {
        let mut state = OrgUnitState::new();
        let _ = update(
            &mut state,
            OrgUnitMessage::TypeSelected(OrganizationUnitType::Department),
        );
        assert_eq!(state.new_type, Some(OrganizationUnitType::Department));
    }

    #[test]
    fn test_parent_selected() {
        let mut state = OrgUnitState::new();

        let _ = update(
            &mut state,
            OrgUnitMessage::ParentSelected("Engineering".to_string()),
        );
        assert_eq!(state.new_parent, Some("Engineering".to_string()));

        // Empty string clears parent
        let _ = update(&mut state, OrgUnitMessage::ParentSelected(String::new()));
        assert_eq!(state.new_parent, None);
    }

    #[test]
    fn test_nats_account_changed() {
        let mut state = OrgUnitState::new();
        let _ = update(
            &mut state,
            OrgUnitMessage::NatsAccountChanged("ENGINEERING".to_string()),
        );
        assert_eq!(state.new_nats_account, "ENGINEERING");
    }

    #[test]
    fn test_responsible_person_selected() {
        let mut state = OrgUnitState::new();
        let person_id = Uuid::now_v7();

        let _ = update(
            &mut state,
            OrgUnitMessage::ResponsiblePersonSelected(person_id),
        );
        assert_eq!(state.new_responsible_person, Some(person_id));
    }

    #[test]
    fn test_is_form_valid() {
        let mut state = OrgUnitState::new();
        assert!(!state.is_form_valid());

        state.new_name = "Engineering".to_string();
        assert!(!state.is_form_valid());

        state.new_type = Some(OrganizationUnitType::Department);
        assert!(state.is_form_valid());
    }

    #[test]
    fn test_validation_error_name_required() {
        let state = OrgUnitState::new();
        assert_eq!(
            state.validation_error(),
            Some("Unit name is required".to_string())
        );
    }

    #[test]
    fn test_validation_error_type_required() {
        let mut state = OrgUnitState::new();
        state.new_name = "Engineering".to_string();

        assert_eq!(
            state.validation_error(),
            Some("Please select a unit type".to_string())
        );
    }

    #[test]
    fn test_clear_form() {
        let mut state = OrgUnitState::new();
        state.new_name = "Engineering".to_string();
        state.new_type = Some(OrganizationUnitType::Department);
        state.new_parent = Some("Company".to_string());
        state.new_nats_account = "ENGINEERING".to_string();
        state.new_responsible_person = Some(Uuid::now_v7());

        state.clear_form();

        assert!(state.new_name.is_empty());
        assert!(state.new_type.is_none());
        assert!(state.new_parent.is_none());
        assert!(state.new_nats_account.is_empty());
        assert!(state.new_responsible_person.is_none());
    }

    #[test]
    fn test_unit_count() {
        let mut state = OrgUnitState::new();
        assert_eq!(state.unit_count(), 0);

        state
            .created_units
            .push(OrganizationUnit::new("Eng".to_string(), OrganizationUnitType::Department));
        assert_eq!(state.unit_count(), 1);
    }

    #[test]
    fn test_find_unit_by_name() {
        let mut state = OrgUnitState::new();
        let unit = OrganizationUnit::new("Engineering".to_string(), OrganizationUnitType::Department);
        state.created_units.push(unit);

        assert!(state.find_unit_by_name("Engineering").is_some());
        assert!(state.find_unit_by_name("Marketing").is_none());
    }

    #[test]
    fn test_unit_names() {
        let mut state = OrgUnitState::new();
        state
            .created_units
            .push(OrganizationUnit::new("Engineering".to_string(), OrganizationUnitType::Department));
        state
            .created_units
            .push(OrganizationUnit::new("Marketing".to_string(), OrganizationUnitType::Department));

        let names = state.unit_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Engineering"));
        assert!(names.contains(&"Marketing"));
    }

    #[test]
    fn test_remove_unit() {
        let mut state = OrgUnitState::new();
        let unit = OrganizationUnit::new("Engineering".to_string(), OrganizationUnitType::Department);
        let unit_id = unit.id.as_uuid();
        state.created_units.push(unit);

        assert_eq!(state.unit_count(), 1);

        let _ = update(&mut state, OrgUnitMessage::Remove(unit_id));
        assert_eq!(state.unit_count(), 0);
    }

    #[test]
    fn test_units_by_type() {
        let mut state = OrgUnitState::new();
        state
            .created_units
            .push(OrganizationUnit::new("Engineering".to_string(), OrganizationUnitType::Department));
        state
            .created_units
            .push(OrganizationUnit::new("Backend".to_string(), OrganizationUnitType::Team));
        state
            .created_units
            .push(OrganizationUnit::new("Frontend".to_string(), OrganizationUnitType::Team));

        let departments = state.units_by_type(OrganizationUnitType::Department);
        assert_eq!(departments.len(), 1);

        let teams = state.units_by_type(OrganizationUnitType::Team);
        assert_eq!(teams.len(), 2);
    }

    #[test]
    fn test_root_units() {
        let mut state = OrgUnitState::new();

        let parent = OrganizationUnit::new("Engineering".to_string(), OrganizationUnitType::Department);
        let parent_id = parent.id;
        state.created_units.push(parent);

        let child = OrganizationUnit::new("Backend".to_string(), OrganizationUnitType::Team)
            .with_parent(parent_id);
        state.created_units.push(child);

        let roots = state.root_units();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "Engineering");
    }
}
