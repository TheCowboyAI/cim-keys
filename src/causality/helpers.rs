//! Causality Helper Functions
//!
//! Convenience functions for integrating causality with cim-keys domain events.

use super::types::{CausalEvent, CausalId};
use crate::events::KeyEvent;

/// Wrap a KeyEvent in a CausalEvent with no dependencies
///
/// Use this for root events that don't depend on anything.
///
/// # Example
///
/// ```rust
/// use cim_keys::events::{KeyEvent, KeyGeneratedEvent};
/// use cim_keys::causality::helpers::wrap_event;
///
/// let key_event = KeyEvent::KeyGenerated(KeyGeneratedEvent {
///     key_id: uuid::Uuid::new_v4(),
///     public_key: vec![1, 2, 3],
///     algorithm: "RSA-4096".to_string(),
///     metadata: Default::default(),
/// });
///
/// let causal_event = wrap_event(key_event);
/// assert!(causal_event.dependencies().is_empty());
/// ```
pub fn wrap_event(event: KeyEvent) -> CausalEvent<KeyEvent> {
    CausalEvent::new(event)
}

/// Wrap a KeyEvent in a CausalEvent with explicit dependencies
///
/// Use this for events that depend on previous events.
///
/// # Example
///
/// ```rust
/// use cim_keys::events::{KeyEvent, KeyGeneratedEvent, KeyExportedEvent};
/// use cim_keys::causality::helpers::{wrap_event, wrap_dependent_event};
///
/// let root_event = wrap_event(KeyEvent::KeyGenerated(KeyGeneratedEvent {
///     key_id: uuid::Uuid::new_v4(),
///     public_key: vec![1, 2, 3],
///     algorithm: "RSA-4096".to_string(),
///     metadata: Default::default(),
/// }));
///
/// let dependent = wrap_dependent_event(
///     KeyEvent::KeyExported(KeyExportedEvent {
///         key_id: uuid::Uuid::new_v4(),
///         export_path: "/mnt/encrypted/key.pem".to_string(),
///         format: "PEM".to_string(),
///         encrypted: true,
///     }),
///     vec![root_event.id()],
/// );
///
/// assert_eq!(dependent.dependencies().len(), 1);
/// ```
pub fn wrap_dependent_event(event: KeyEvent, dependencies: Vec<CausalId>) -> CausalEvent<KeyEvent> {
    CausalEvent::caused_by(event, dependencies)
}

/// Extract the KeyEvent from a CausalEvent
///
/// # Example
///
/// ```rust
/// use cim_keys::events::{KeyEvent, KeyGeneratedEvent};
/// use cim_keys::causality::helpers::{wrap_event, unwrap_event};
///
/// let original = KeyEvent::KeyGenerated(KeyGeneratedEvent {
///     key_id: uuid::Uuid::new_v4(),
///     public_key: vec![1, 2, 3],
///     algorithm: "RSA-4096".to_string(),
///     metadata: Default::default(),
/// });
///
/// let causal = wrap_event(original.clone());
/// let extracted = unwrap_event(causal);
///
/// assert_eq!(extracted, original);
/// ```
pub fn unwrap_event(causal_event: CausalEvent<KeyEvent>) -> KeyEvent {
    causal_event.into_data()
}

/// Extract a reference to the KeyEvent from a CausalEvent
///
/// # Example
///
/// ```rust
/// use cim_keys::events::{KeyEvent, KeyGeneratedEvent};
/// use cim_keys::causality::helpers::{wrap_event, peek_event};
///
/// let original = KeyEvent::KeyGenerated(KeyGeneratedEvent {
///     key_id: uuid::Uuid::new_v4(),
///     public_key: vec![1, 2, 3],
///     algorithm: "RSA-4096".to_string(),
///     metadata: Default::default(),
/// });
///
/// let causal = wrap_event(original.clone());
/// let peeked = peek_event(&causal);
///
/// assert_eq!(peeked, &original);
/// ```
pub fn peek_event(causal_event: &CausalEvent<KeyEvent>) -> &KeyEvent {
    causal_event.data()
}

// Note: Tests for these helpers are in examples/causality_integration.rs
// which demonstrates real-world usage with actual KeyEvent types.
