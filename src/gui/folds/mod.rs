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
//! Each fold is a natural transformation from the LiftedNode coproduct
//! to a specific output type. The organization by pipeline role ensures:
//!
//! 1. **A3 Compliance**: Decoupling - outputs depend only on inputs
//! 2. **A5 Compliance**: Totality - all folds are total functions
//! 3. **A9 Compliance**: Semantic preservation through composition
//!
//! ## Usage
//!
//! ```ignore
//! use cim_keys::gui::folds::view::VisualizationData;
//! use cim_keys::lifting::LiftedNode;
//!
//! // In view(), use LiftedNode methods:
//! let viz_data = lifted_node.themed_visualization(theme);
//! ```

pub mod view;
pub mod query;

// Re-export primary types for convenience
pub use view::{VisualizationData, palette};
pub use view::{ThemedVisualizationFold, ThemedVisualizationData, CertificateType, StatusIndicator};
pub use query::SearchableText;
