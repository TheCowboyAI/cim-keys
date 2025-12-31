// Copyright (c) 2025 - Cowboy AI, LLC.

//! Fold Modules - Natural Transformations by Pipeline Role
//!
//! This module organizes fold implementations by their role in the FRP pipeline:
//!
//! ## Module Structure
//!
//! - **view/** - Folds that execute in view(), producing visual data
//! - **query/** - Folds that execute for queries, producing selection data
//!
//! ## Categorical Foundation
//!
//! Each fold is a natural transformation from the DomainNode coproduct
//! to a specific output type. The organization by pipeline role ensures:
//!
//! 1. **A3 Compliance**: Decoupling - outputs depend only on inputs
//! 2. **A5 Compliance**: Totality - all folds are total functions
//! 3. **A9 Compliance**: Semantic preservation through composition
//!
//! ## Usage
//!
//! ```ignore
//! use cim_keys::gui::folds::view::{FoldVisualization, VisualizationData};
//! use cim_keys::gui::folds::query::{FoldSearchableText, SearchableText};
//!
//! // In view():
//! let viz_data = node.fold(&FoldVisualization);
//!
//! // For queries:
//! let searchable = node.fold(&FoldSearchableText);
//! if searchable.matches("query") { ... }
//! ```

pub mod view;
pub mod query;

// Re-export primary types for convenience
pub use view::{FoldVisualization, VisualizationData};
pub use view::{ThemedVisualizationFold, ThemedVisualizationData, CertificateType, StatusIndicator};
pub use query::{FoldSearchableText, SearchableText};
