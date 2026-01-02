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
//!
//! ## Available Folds
//!
//! - `FoldSearchableText` - Extracts searchable text for filtering
//! - `FoldEditFields` - Extracts edit field data for property cards

pub mod edit_fields;
pub mod searchable;

pub use edit_fields::{EditFieldData, EntityType, FoldEditFields, extract_edit_fields_from_lifted};
// Re-export domain types used by EditFieldData for convenience
pub use crate::domain::{PolicyClaim, RoleType, LocationType};
pub use searchable::{FoldSearchableText, SearchableText};
