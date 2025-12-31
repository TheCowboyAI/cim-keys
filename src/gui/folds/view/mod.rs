// Copyright (c) 2025 - Cowboy AI, LLC.

//! View Folds - Natural Transformations for Rendering
//!
//! These folds execute in the view() function, transforming domain nodes
//! into visual representation data. They are pure and produce no side effects.
//!
//! ## FRP Role
//!
//! View folds implement the Model → Element transformation:
//! ```text
//! Model → view() → [Folds] → Element
//! ```
//!
//! ## Typography Integration
//!
//! The `themed_visualization` module provides folds that use the Typography
//! bounded context to guarantee all icons will render (no tofu boxes).

pub mod visualization;
pub mod themed_visualization;

pub use visualization::{FoldVisualization, VisualizationData};
pub use themed_visualization::{
    ThemedVisualizationFold, ThemedVisualizationData,
    CertificateType, StatusIndicator,
};
