//! Relationship Aggregate Commands
//!
//! Commands for the Relationship aggregate root.
//! Currently re-exports from organization.rs - full migration pending.

// Re-export relationship-related commands from organization module
pub use super::organization::{
    EstablishRelationship,
    handle_establish_relationship,
    RelationshipType,
};

// TODO: Future refactoring
// - Add ModifyRelationship command
// - Add TerminateRelationship command
// - Add ValidateAccountability command
// - Add TrustEstablishment commands
// - Align with RelationshipEvents from events/relationship.rs
