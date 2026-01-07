// Copyright (c) 2025 - Cowboy AI, LLC.

//! Organization Unit Message Definitions
//!
//! This module defines the message types for the Organization Unit bounded context.
//! Handlers are in gui.rs - this module only provides message organization.
//!
//! ## Sub-domains
//!
//! 1. **UI State**: Section visibility
//! 2. **Form Input**: Name, type, parent, NATS account, responsible person
//! 3. **Lifecycle**: Create, created result, remove

use uuid::Uuid;

use crate::domain::{OrganizationUnit, OrganizationUnitType};
use crate::domain::ids::UnitId;

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
    Remove(UnitId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_unit_message_variants() {
        // Verify enum variants compile
        let _ = OrgUnitMessage::ToggleSection;
        let _ = OrgUnitMessage::NameChanged("Test Unit".to_string());
        let _ = OrgUnitMessage::TypeSelected(OrganizationUnitType::Department);
        let _ = OrgUnitMessage::ParentSelected("parent".to_string());
        let _ = OrgUnitMessage::NatsAccountChanged("account".to_string());
        let _ = OrgUnitMessage::ResponsiblePersonSelected(Uuid::nil());
        let _ = OrgUnitMessage::Create;
        let _ = OrgUnitMessage::Remove(UnitId::new());
    }
}
