//! Ports (interfaces) for external system integration
//!
//! This module defines the interfaces that our domain uses to interact
//! with external systems. The domain only knows about these interfaces,
//! not the concrete implementations.

pub mod nats;

pub use nats::{NatsKeyPort, NatsKeyOperations};