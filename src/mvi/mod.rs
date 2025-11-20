//! MVI (Model-View-Intent) Architecture for cim-keys
//!
//! **Intent-Driven Architecture**:
//! - **Intent**: Unified algebraic type for ALL event sources (UI, domain, async)
//! - **Model**: Pure immutable state
//! - **View**: Pure rendering function `Model → Element<Intent>`
//! - **Update**: Pure state transition `(Model, Intent) → (Model, Command<Intent>)`
//!
//! This differs from traditional TEA by making event sources explicit in the type system.
//!
//! ## N-ary FRP Integration
//!
//! Following n-ary FRP Axiom A1 (Multi-Kinded Signals), this module provides:
//! - **Signal kind classification**: Each Intent has a natural signal kind (Event/Step/Continuous)
//! - **Type aliases**: Ergonomic types for common signal patterns (`EventIntent`, `ModelSignal`)
//! - **Signal vectors**: N-ary update functions operating on `(Model, Intent)` pairs
//!
//! See `signals_aliases` module for type-safe signal operations.

pub mod intent;
pub mod model;
pub mod update;
pub mod view;
pub mod signals_aliases;

pub use intent::{Intent, NodeCreationType, SignalKindMarker};
pub use model::Model;
pub use update::update;
pub use view::view;
pub use signals_aliases::{
    EventIntent, StepValue, ModelSignal,
    UpdateInputs, UpdateOutputs,
    OrganizationNameSignal, PersonEmailSignal, PassphraseSignal,
    AnimationTime, ProgressSignal,
};
