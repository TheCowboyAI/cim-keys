//! MVI (Model-View-Intent) Architecture for cim-keys
//!
//! **Intent-Driven Architecture**:
//! - **Intent**: Unified algebraic type for ALL event sources (UI, domain, async)
//! - **Model**: Pure immutable state
//! - **View**: Pure rendering function `Model → Element<Intent>`
//! - **Update**: Pure state transition `(Model, Intent) → (Model, Command<Intent>)`
//!
//! This differs from traditional TEA by making event sources explicit in the type system.

pub mod intent;
pub mod model;
pub mod update;
pub mod view;

pub use intent::Intent;
pub use model::Model;
pub use update::update;
pub use view::view;
