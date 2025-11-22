//! NATS User Aggregate Commands
//!
//! Commands for the NATS User aggregate root.
//! Currently re-exports from nats_identity.rs - full migration pending.

// Re-export NATS user commands from nats_identity module
pub use super::nats_identity::{
    CreateNatsUser,
    NatsUserCreated,
    BootstrapNatsInfrastructure,
    NatsInfrastructureBootstrapped,
};

// TODO: Future refactoring
// - Add UpdateNatsUser command
// - Add SetUserPermissions command
// - Add SuspendNatsUser command
// - Add ReactivateNatsUser command
// - Add CreateServiceAccount command
// - Add CreateAgent command
// - Align with NatsUserEvents from events/nats_user.rs
