// Copyright (c) 2025 - Cowboy AI, LLC.

//! Label System
//!
//! Provides the types for labelled elements - combinations of icon,
//! text, and styling that form the visual output of the typography
//! bounded context.
//!
//! ## Key Types
//!
//! - `LabelSpec` - Specification for how to render a label
//! - `LabelledElement` - The rendered output with verified typography
//! - `LabelCategory` - Categories of labels (status, entity, action, etc.)

mod spec;
mod element;
mod category;

pub use spec::LabelSpec;
pub use element::LabelledElement;
pub use category::LabelCategory;

// Re-export commonly used types
pub use super::icon_set::{StatusIcon, NavigationIcon, ActionIcon, EntityIcon, SemanticIcon};
