//! Adapters (implementations) for external system integration
//!
//! This module contains the concrete implementations of our ports.
//! These adapters handle the actual integration with external systems.

pub mod nsc;

pub use nsc::NscAdapter;