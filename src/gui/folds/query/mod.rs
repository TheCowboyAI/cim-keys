// Copyright (c) 2025 - Cowboy AI, LLC.

//! Query Folds - Natural Transformations for Selection
//!
//! These folds execute during query/filter operations, transforming
//! domain nodes into searchable/filterable data structures.
//!
//! ## FRP Role
//!
//! Query folds implement the selection transformation:
//! ```text
//! Model + Query → [Folds] → bool (matches)
//! ```

pub mod searchable;

pub use searchable::{FoldSearchableText, SearchableText};
