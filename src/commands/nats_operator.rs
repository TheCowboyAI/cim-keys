//! NATS Operator Aggregate Commands
//!
//! Commands for the NATS Operator aggregate root.
//! Currently re-exports from nats_identity.rs - full migration pending.

// Re-export NATS operator commands from nats_identity module
pub use super::nats_identity::{
    CreateNatsOperator,
    NatsOperatorCreated,
};

// TODO: Future refactoring
// - Add UpdateNatsOperator command
// - Add GenerateNatsSigningKey command
// - Add ExportNatsConfig command
// - Add GenerateNKey command
// - Align with NatsOperatorEvents from events/nats_operator.rs
