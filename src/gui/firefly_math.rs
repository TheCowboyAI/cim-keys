//! Mathematical models for firefly synchronization using Category Theory
//!
//! This module defines the pure mathematical functions for the Kuramoto model
//! and provides Category Theory morphisms to map them to rendering points.

use std::f32::consts::PI;

/// Pure mathematical representation of a firefly's state
#[derive(Debug, Clone, Copy)]
pub struct FireflyState {
    /// Phase in radians [0, 2π]
    pub phase: f32,
    /// Natural frequency in Hz
    pub frequency: f32,
    /// Position in 2D space
    pub position: (f32, f32),
}

/// Kuramoto model parameters
#[derive(Debug, Clone, Copy)]
pub struct KuramotoParams {
    /// Coupling strength K
    pub coupling_strength: f32,
    /// Number of oscillators
    pub num_oscillators: usize,
    /// Time step dt
    pub dt: f32,
}

/// Pure mathematical functions for the Kuramoto model
pub mod kuramoto {
    use super::*;

    /// Calculate phase velocity for oscillator i given all oscillator phases
    /// dθᵢ/dt = ωᵢ + (K/N) Σⱼ sin(θⱼ - θᵢ)
    pub fn phase_velocity(
        i: usize,
        phases: &[f32],
        frequencies: &[f32],
        params: &KuramotoParams,
    ) -> f32 {
        let mut coupling_term = 0.0;

        for (j, &phase_j) in phases.iter().enumerate() {
            if i != j {
                coupling_term += (phase_j - phases[i]).sin();
            }
        }

        frequencies[i] + (params.coupling_strength / params.num_oscillators as f32) * coupling_term
    }

    /// Update phase using Euler integration
    /// θᵢ(t + dt) = θᵢ(t) + dθᵢ/dt * dt
    pub fn integrate_phase(
        current_phase: f32,
        phase_velocity: f32,
        dt: f32,
    ) -> f32 {
        (current_phase + phase_velocity * dt) % (2.0 * PI)
    }

    /// Calculate order parameter (synchronization measure)
    /// r * e^(iψ) = (1/N) Σⱼ e^(iθⱼ)
    pub fn order_parameter(phases: &[f32]) -> (f32, f32) {
        let n = phases.len() as f32;
        let mut real_sum = 0.0;
        let mut imag_sum = 0.0;

        for &phase in phases {
            real_sum += phase.cos();
            imag_sum += phase.sin();
        }

        let r = ((real_sum / n).powi(2) + (imag_sum / n).powi(2)).sqrt();
        let psi = (imag_sum / n).atan2(real_sum / n);

        (r, psi)
    }
}

/// Category Theory morphisms for mapping mathematical objects to rendering
pub mod morphisms {
    use super::*;

    /// Functor from FireflyState to visual properties
    pub trait VisualFunctor {
        type Visual;

        fn map(&self, state: &FireflyState) -> Self::Visual;
    }

    /// Maps phase to brightness (0.0 to 1.0)
    pub struct PhaseToBrightness;

    impl VisualFunctor for PhaseToBrightness {
        type Visual = f32;

        fn map(&self, state: &FireflyState) -> f32 {
            // Use a sharp transition for firefly-like flashing
            let sin_phase = state.phase.sin();
            if sin_phase > 0.7 {
                (sin_phase - 0.7) * 3.333  // Map [0.7, 1.0] to [0, 1]
            } else {
                0.0
            }
        }
    }

    /// Maps position and phase to screen coordinates with movement
    pub struct PositionToScreen {
        pub screen_width: f32,
        pub screen_height: f32,
        pub time: f32,
    }

    impl VisualFunctor for PositionToScreen {
        type Visual = (f32, f32);

        fn map(&self, state: &FireflyState) -> (f32, f32) {
            // Base position in normalized coordinates [-1, 1]
            let base_x = state.position.0 * 2.0 - 1.0;
            let base_y = state.position.1 * 2.0 - 1.0;

            // Add organic movement based on phase and time
            let drift_x = (self.time * 0.3 + state.phase).sin() * 0.08;
            let drift_y = (self.time * 0.21 + state.phase * 1.5).sin() * 0.06;

            (base_x + drift_x, base_y + drift_y)
        }
    }

    /// Maps phase to color with rainbow effect
    pub struct PhaseToColor;

    impl VisualFunctor for PhaseToColor {
        type Visual = [f32; 3];  // RGB

        fn map(&self, state: &FireflyState) -> [f32; 3] {
            let hue = state.phase / (2.0 * PI);

            // HSV to RGB conversion
            let c = 1.0;  // Chroma
            let x = c * (1.0 - ((hue * 6.0) % 2.0 - 1.0).abs());
            let m = 0.0;

            let (r, g, b) = match (hue * 6.0) as usize {
                0 => (c, x, 0.0),
                1 => (x, c, 0.0),
                2 => (0.0, c, x),
                3 => (0.0, x, c),
                4 => (x, 0.0, c),
                _ => (c, 0.0, x),
            };

            [r + m, g + m, b + m]
        }
    }
}

/// FRP integration layer for iced
pub mod frp {
    use super::*;

    /// Event representing a time step in the simulation
    #[derive(Debug, Clone)]
    pub struct TimeStep(pub f32);

    /// Message for updating firefly states
    #[derive(Debug, Clone)]
    pub enum FireflyMessage {
        Tick(TimeStep),
        UpdateCoupling(f32),
        Reset,
    }

    /// State container with FRP update semantics
    #[derive(Debug, Clone)]
    pub struct FireflySystem {
        states: Vec<FireflyState>,
        params: KuramotoParams,
        time: f32,
    }

    impl FireflySystem {
        pub fn new(num_fireflies: usize) -> Self {
            let mut states = Vec::with_capacity(num_fireflies);

            // Initialize with grid positions and random phases
            let grid_width = 8;
            for i in 0..num_fireflies {
                let col = i % grid_width;
                let row = i / grid_width;

                states.push(FireflyState {
                    phase: (i as f32 * 0.618_034) % (2.0 * PI),
                    frequency: 1.0 + (i as f32 * 0.02) % 0.2 - 0.1,
                    position: (
                        col as f32 / grid_width as f32,
                        row as f32 / 5.0,
                    ),
                });
            }

            Self {
                states,
                params: KuramotoParams {
                    coupling_strength: 0.1,
                    num_oscillators: num_fireflies,
                    dt: 0.016,  // 60 FPS
                },
                time: 0.0,
            }
        }

        /// Pure functional update - returns new state without mutation
        pub fn update(&self, msg: FireflyMessage) -> Self {
            match msg {
                FireflyMessage::Tick(TimeStep(dt)) => {
                    let mut new_states = self.states.clone();
                    let phases: Vec<f32> = self.states.iter().map(|s| s.phase).collect();
                    let frequencies: Vec<f32> = self.states.iter().map(|s| s.frequency).collect();

                    // Update each firefly's phase using Kuramoto model
                    for (i, state) in new_states.iter_mut().enumerate() {
                        let velocity = kuramoto::phase_velocity(
                            i,
                            &phases,
                            &frequencies,
                            &self.params,
                        );
                        state.phase = kuramoto::integrate_phase(state.phase, velocity, dt);
                    }

                    Self {
                        states: new_states,
                        params: self.params,
                        time: self.time + dt,
                    }
                }
                FireflyMessage::UpdateCoupling(k) => {
                    let mut new_params = self.params;
                    new_params.coupling_strength = k;
                    Self {
                        states: self.states.clone(),
                        params: new_params,
                        time: self.time,
                    }
                }
                FireflyMessage::Reset => Self::new(self.params.num_oscillators),
            }
        }

        /// Get visual representation using morphisms
        pub fn to_visual(&self) -> Vec<VisualFirefly> {
            use morphisms::*;

            let brightness_functor = PhaseToBrightness;
            let position_functor = PositionToScreen {
                screen_width: 800.0,
                screen_height: 600.0,
                time: self.time,
            };
            let color_functor = PhaseToColor;

            self.states
                .iter()
                .map(|state| VisualFirefly {
                    position: position_functor.map(state),
                    brightness: brightness_functor.map(state),
                    color: color_functor.map(state),
                    size: 0.02 + 0.04 * brightness_functor.map(state),
                })
                .collect()
        }
    }

    /// Visual representation of a firefly
    #[derive(Debug, Clone)]
    pub struct VisualFirefly {
        pub position: (f32, f32),
        pub brightness: f32,
        pub color: [f32; 3],
        pub size: f32,
    }
}