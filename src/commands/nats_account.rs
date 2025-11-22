//! NATS Account Aggregate Commands
//!
//! Commands for the NATS Account aggregate root.
//! Currently re-exports from nats_identity.rs - full migration pending.

// Re-export NATS account commands from nats_identity module
pub use super::nats_identity::{
    CreateNatsAccount,
    NatsAccountCreated,
};

// TODO: Future refactoring
// - Add UpdateNatsAccount command
// - Add SetNatsPermissions command
// - Add SuspendNatsAccount command
// - Add ReactivateNatsAccount command
// - Align with NatsAccountEvents from events/nats_account.rs
