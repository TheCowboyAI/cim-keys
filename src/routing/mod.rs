//! Compositional Routing for N-ary FRP
//!
//! This module implements Axiom A6: Explicit Routing.
//!
//! Instead of pattern matching on Intent variants, we compose pure signal functions
//! using routing primitives. This enables:
//! - **Compositional reasoning**: Understand parts independently
//! - **Reusability**: Routes work across different frameworks
//! - **Testability**: Test routes in isolation
//! - **Type safety**: Compiler verifies composition
//!
//! ## Routing Primitives
//!
//! - **`id`**: Identity route (A → A)
//! - **`>>>`**: Sequential composition (A → B, B → C = A → C)
//! - **`***`**: Parallel composition (A → B, C → D = (A, C) → (B, D))
//! - **`&&&`**: Fanout (A → B, A → C = A → (B, C))
//!
//! ## Example
//!
//! ```rust,ignore
//! use cim_keys::routing::*;
//!
//! // Compose a workflow from pure functions
//! let workflow =
//!     validate_passphrase
//!     .then(generate_seed)
//!     .then(derive_keys)
//!     .then(store_projection);
//!
//! // Or using >>> operator
//! let workflow2 =
//!     validate_passphrase >>>
//!     generate_seed >>>
//!     derive_keys >>>
//!     store_projection;
//! ```
//!
//! ## Category Theory Foundation
//!
//! Routes form a category where:
//! - **Objects**: Signal types (Signal<K, T>)
//! - **Morphisms**: Routes (A → B)
//! - **Identity**: `id`
//! - **Composition**: `>>>`
//!
//! Laws:
//! ```text
//! id >>> f = f            (left identity)
//! f >>> id = f            (right identity)
//! (f >>> g) >>> h = f >>> (g >>> h)  (associativity)
//! ```

pub mod primitives;
pub mod builder;
pub mod subject;
pub mod subject_algebra;

#[cfg(test)]
mod laws;  // Property tests for compositional laws (Axiom A9)

pub use primitives::{Route, id, compose, parallel, fanout};
pub use builder::RouteBuilder;
pub use subject::{
    IntentCategory,
    SubjectPattern,
    SubjectPatternError,
    SubjectIntent,
    SubjectRouter,
    HierarchicalRouter,
};
pub use subject_algebra::{
    Monoid,
    Token,
    Subject as AlgebraicSubject,
    SubjectBuilder as AlgebraicSubjectBuilder,
    ParseError,
    patterns,
};
