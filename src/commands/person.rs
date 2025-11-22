//! Person Aggregate Commands
//!
//! Commands for the Person aggregate root.
//! Currently re-exports from organization.rs - full migration pending.

// Re-export person-related commands from organization module
pub use super::organization::{
    CreatePerson,
    handle_create_person,
};

// TODO: Future refactoring
// - Add UpdatePerson command
// - Add DeactivatePerson command
// - Add AssignRole, RemoveRole commands
// - Add SSH/GPG key generation for person
// - Align with PersonEvents from events/person.rs
