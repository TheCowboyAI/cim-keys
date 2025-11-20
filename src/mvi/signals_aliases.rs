//! Type Aliases for N-ary FRP Signals in MVI Architecture
//!
//! This module provides ergonomic type aliases for working with signals in the
//! cim-keys MVI (Model-View-Intent) architecture, following n-ary FRP axioms.
//!
//! ## Design Principles
//!
//! 1. **Explicit Signal Kinds**: Every signal has its kind (Event/Step/Continuous) explicit at the type level
//! 2. **Type Safety**: Cannot mix event streams with behaviors without explicit conversion
//! 3. **Composability**: Signals compose through signal vectors and routing primitives
//!
//! ## Usage Examples
//!
//! ```rust,ignore
//! use cim_keys::mvi::signals_aliases::*;
//!
//! // Event signal: Button click occurs at discrete time
//! let click_event = EventIntent::event(vec![
//!     (0.0, Intent::UiGenerateRootCAClicked),
//! ]);
//!
//! // Step signal: Organization name holds value until changed
//! let org_name = StepValue::step("Acme Corp".into());
//!
//! // Model signal: Application state is piecewise-constant
//! let model_state = ModelSignal::step(Model::default());
//!
//! // Signal vector: Update function operates on (Model, Intent) pair
//! let inputs = UpdateInputs::new(model_state, click_event);
//! let (model, intent) = inputs.split();
//! ```

use crate::signals::{Signal, SignalVec2, EventKind, StepKind, ContinuousKind};
use super::{Intent, Model};

// =============================================================================
// Intent Signals
// =============================================================================

/// Event signal carrying an Intent
///
/// Most intents (87%) are events - discrete occurrences at specific times:
/// - Button clicks (UiGenerateRootCAClicked)
/// - Completions (DomainCreated, PortX509RootCAGenerated)
/// - Failures (PortX509GenerationFailed)
/// - Domain events (PersonAdded, RootCAGenerated)
///
/// # Semantics
///
/// ```text
/// ⟦EventIntent⟧(t) = [(t₁, intent₁), (t₂, intent₂), ...] where tᵢ ≤ t
/// ```
///
/// # Example
///
/// ```rust,ignore
/// let button_clicks = EventIntent::event(vec![
///     (0.0, Intent::UiGenerateRootCAClicked),
///     (1.5, Intent::UiGenerateSSHKeysClicked),
///     (3.0, Intent::UiExportClicked { output_path: "/mnt/sd".into() }),
/// ]);
///
/// // Query occurrences in time range
/// let clicks_in_first_two_seconds = button_clicks.occurrences(0.0, 2.0);
/// assert_eq!(clicks_in_first_two_seconds.len(), 2);
/// ```
pub type EventIntent = Signal<EventKind, Intent>;

/// Step signal carrying a value that changes discretely
///
/// Step signals hold current values between changes:
/// - Form inputs (organization name, person email)
/// - Configuration values
/// - Current selection state
///
/// # Semantics
///
/// ```text
/// ⟦StepValue<T>⟧(t) = x where x is the value at most recent change ≤ t
/// ```
///
/// # Example
///
/// ```rust,ignore
/// let org_name = StepValue::step("Acme Corp".into());
/// assert_eq!(org_name.sample(0.0), "Acme Corp");
/// assert_eq!(org_name.sample(100.0), "Acme Corp"); // Holds value
///
/// let updated = org_name.with_value("New Name".into());
/// assert_eq!(updated.sample(0.0), "New Name");
/// ```
pub type StepValue<T> = Signal<StepKind, T>;

// =============================================================================
// Model Signals
// =============================================================================

/// Model signal - piecewise-constant application state
///
/// The Model holds the current application state and changes only when
/// intents are processed. This is a Step signal because the model remains
/// constant between discrete state transitions.
///
/// # Semantics
///
/// ```text
/// ⟦ModelSignal⟧(t) = Model at most recent update ≤ t
/// ```
///
/// # Example
///
/// ```rust,ignore
/// let initial_model = ModelSignal::step(Model::default());
/// let current_model = initial_model.sample(0.0);
/// assert_eq!(current_model.current_tab(), Tab::Domain);
/// ```
pub type ModelSignal = Signal<StepKind, Model>;

// =============================================================================
// Signal Vectors for Update Function
// =============================================================================

/// Input signals for update function: (Model, Intent)
///
/// The update function operates on a signal vector combining:
/// - Model (StepKind): Current application state
/// - Intent (EventKind): Discrete user/system/domain event
///
/// # Type Signature
///
/// ```text
/// update : SignalVec2<StepKind, EventKind, Model, Intent>
///       → SignalVec2<StepKind, EventKind, Model, Intent>
/// ```
///
/// # Example
///
/// ```rust,ignore
/// fn update_with_signals(
///     inputs: UpdateInputs,
/// ) -> UpdateOutputs {
///     let (model, intent) = inputs.split();
///
///     // Process intent and update model
///     let updated_model = process(model.sample(0.0), intent);
///     let result_intent = generate_response();
///
///     UpdateOutputs::new(
///         ModelSignal::step(updated_model),
///         EventIntent::event(vec![(0.0, result_intent)]),
///     )
/// }
/// ```
pub type UpdateInputs = SignalVec2<StepKind, EventKind, Model, Intent>;

/// Output signals from update function: (Model, Intent)
///
/// After processing, the update function returns:
/// - Updated Model (StepKind): New application state
/// - Result Intent (EventKind): Response event or command
pub type UpdateOutputs = SignalVec2<StepKind, EventKind, Model, Intent>;

// =============================================================================
// Domain-Specific Signals
// =============================================================================

/// Organization name as a step signal
///
/// Holds the current organization name value between user edits.
pub type OrganizationNameSignal = StepValue<String>;

/// Person email as a step signal
///
/// Holds the current email value for a person between edits.
pub type PersonEmailSignal = StepValue<String>;

/// Passphrase as a step signal
///
/// Holds the current passphrase value (note: should be cleared after use).
pub type PassphraseSignal = StepValue<String>;

// =============================================================================
// Future: Continuous Signals
// =============================================================================

/// Animation time as a continuous signal
///
/// Not yet used in cim-keys, but would represent smooth animation time:
///
/// ```text
/// ⟦AnimationTime⟧(t) = t (identity function)
/// ```
///
/// # Example
///
/// ```rust,ignore
/// let time = AnimationTime::continuous(|t| t as f32);
/// assert_eq!(time.sample(0.0), 0.0);
/// assert_eq!(time.sample(1.5), 1.5);
/// ```
pub type AnimationTime = Signal<ContinuousKind, f32>;

/// Progress indicator as a continuous signal (0.0 to 1.0)
///
/// Could be used for smooth progress animations.
///
/// # Example
///
/// ```rust,ignore
/// // Linear progress from 0 to 1 over 10 seconds
/// let progress = ProgressSignal::continuous(|t| (t / 10.0).min(1.0));
/// assert_eq!(progress.sample(5.0), 0.5);
/// assert_eq!(progress.sample(10.0), 1.0);
/// assert_eq!(progress.sample(15.0), 1.0); // Clamped
/// ```
pub type ProgressSignal = Signal<ContinuousKind, f64>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_intent_signal() {
        let events = EventIntent::event(vec![
            (0.0, Intent::UiGenerateRootCAClicked),
            (1.0, Intent::UiGenerateSSHKeysClicked),
        ]);

        let occurrences = events.occurrences(0.0, 2.0);
        assert_eq!(occurrences.len(), 2);
    }

    #[test]
    fn test_step_value_signal() {
        let org_name = OrganizationNameSignal::step("Acme Corp".into());
        assert_eq!(org_name.sample(0.0), "Acme Corp");
        assert_eq!(org_name.sample(100.0), "Acme Corp"); // Holds value
    }

    #[test]
    fn test_model_signal() {
        let model = ModelSignal::step(Model::default());
        let current = model.sample(0.0);
        // Model is created successfully with default tab
        assert_eq!(current.current_tab, super::super::model::Tab::Welcome);
    }

    #[test]
    fn test_update_inputs_vector() {
        let model = ModelSignal::step(Model::default());
        let intent = EventIntent::event(vec![(0.0, Intent::UiGenerateRootCAClicked)]);

        let inputs = UpdateInputs::new(model, intent);
        let (m, i) = inputs.split();

        assert_eq!(m.sample(0.0).current_tab, super::super::model::Tab::Welcome);
        assert_eq!(i.count(0.0, 1.0), 1);
    }

    #[test]
    fn test_animation_time_continuous() {
        let time = AnimationTime::continuous(|t| t as f32);
        assert_eq!(time.sample(0.0), 0.0);
        assert_eq!(time.sample(1.5), 1.5);
        assert_eq!(time.sample(10.0), 10.0);
    }

    #[test]
    fn test_progress_signal_clamped() {
        // Linear progress from 0 to 1 over 10 seconds, clamped at 1.0
        let progress = ProgressSignal::continuous(|t| (t / 10.0).min(1.0));

        assert_eq!(progress.sample(0.0), 0.0);
        assert_eq!(progress.sample(5.0), 0.5);
        assert_eq!(progress.sample(10.0), 1.0);
        assert_eq!(progress.sample(15.0), 1.0); // Clamped
    }
}
