//! Firefly swarm animation for The Cowboy AI theme
//!
//! Creates a mesmerizing firefly effect with organic movement and natural blinking

use iced::{
    widget::canvas::{Cache, Canvas, Geometry, Path, Program, Stroke},
    mouse::Cursor, Color, Element, Length, Point, Rectangle, Size, Theme, Vector,
};
use std::cell::{Cell, RefCell};
use std::f32::consts::PI;
use std::time::Instant;

/// Firefly swarm animation
pub struct FireflySwarm {
    fireflies: RefCell<Vec<Firefly>>,
    cache: RefCell<Cache>,
    last_update: Cell<Instant>,
    time: Cell<f32>,
    update_counter: Cell<u64>,
    colors: super::view_model::ColorPalette,
}

/// A single firefly with organic movement and blinking
struct Firefly {
    position: Point,
    target: Point,
    velocity: Vector,

    // Movement parameters
    wander_angle: f32,
    wander_speed: f32,
    pause_timer: f32,
    is_paused: bool,

    // Visual parameters
    size: f32,
    base_color: Color,
    glow_intensity: f32,
    blink_phase: f32,
    blink_speed: f32,
    next_blink: f32,
    is_blinking: bool,
}

impl Firefly {
    fn new(x: f32, y: f32, index: usize) -> Self {
        // Vary parameters based on index for diversity
        let variety = (index as f32 * 0.618) % 1.0; // Golden ratio for distribution

        // Mix of warm (golden) and cool (blue/cyan) fireflies
        let base_color = if index % 5 < 2 {
            // Warm golden fireflies
            Color::from_rgba(
                0.9 + variety * 0.1,
                0.7 + variety * 0.2,
                0.2 + variety * 0.3,
                0.8,
            )
        } else if index % 5 < 4 {
            // Cool blue fireflies
            Color::from_rgba(
                0.3 + variety * 0.2,
                0.5 + variety * 0.2,
                0.9 + variety * 0.1,
                0.8,
            )
        } else {
            // Cyan fireflies
            Color::from_rgba(
                0.2 + variety * 0.2,
                0.8 + variety * 0.2,
                0.9 + variety * 0.1,
                0.8,
            )
        };

        // Give each firefly an initial target that's different from its position
        let target_angle = variety * PI * 2.0;
        let target_distance = 100.0 + variety * 200.0;
        let target_x = x + target_angle.cos() * target_distance;
        let target_y = y + target_angle.sin() * target_distance;

        Firefly {
            position: Point::new(x, y),
            target: Point::new(target_x, target_y),
            velocity: Vector::new(0.0, 0.0),

            wander_angle: variety * PI * 2.0,
            wander_speed: 0.5 + variety * 1.5,
            pause_timer: 0.0,
            is_paused: false,

            size: 1.5 + variety * 2.0,
            base_color,
            glow_intensity: 0.5 + variety * 0.5,
            blink_phase: variety * PI * 2.0,
            blink_speed: 0.5 + variety * 1.0,
            next_blink: variety * 5.0,
            is_blinking: false,
        }
    }

    fn update(&mut self, delta: f32, bounds: Size, time: f32) {
        // Update blinking
        if !self.is_blinking {
            self.next_blink -= delta;
            if self.next_blink <= 0.0 {
                self.is_blinking = true;
                self.blink_phase = 0.0;
                self.next_blink = 3.0 + (time * 1.3).sin() * 2.0; // Vary next blink time
            }
        } else {
            self.blink_phase += delta * self.blink_speed * 2.0;
            if self.blink_phase >= PI {
                self.is_blinking = false;
                self.blink_phase = 0.0;
            }
        }

        // Calculate glow based on blinking
        if self.is_blinking {
            self.glow_intensity = self.blink_phase.sin() * 0.9 + 0.1;
        } else {
            // Subtle ambient pulsing
            self.glow_intensity = 0.3 + (time * 2.0 + self.blink_phase).sin() * 0.1;
        }

        // Update pause state
        if self.is_paused {
            self.pause_timer -= delta;
            if self.pause_timer <= 0.0 {
                self.is_paused = false;
                // Set new target when resuming
                self.set_new_target(bounds);
            }
            return; // Don't move while paused
        }

        // Check if should pause (random chance)
        if (time * 0.3 + self.wander_angle).sin() > 0.98 {
            self.is_paused = true;
            self.pause_timer = 1.0 + (time * 0.7).sin().abs() * 2.0;
            return;
        }

        // Organic movement using sine waves and Perlin-like noise
        let noise_x = (time * 0.3 + self.wander_angle).sin()
                    * (time * 0.7 + self.wander_angle * 2.0).cos();
        let noise_y = (time * 0.4 + self.wander_angle + PI/2.0).sin()
                    * (time * 0.6 + self.wander_angle * 1.5).cos();

        // Update wander angle for curved paths
        self.wander_angle += delta * 0.5 * (noise_x + noise_y);

        // Calculate movement direction
        let dx = self.target.x - self.position.x;
        let dy = self.target.y - self.position.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 20.0 {
            // Reached target, set a new one
            self.set_new_target(bounds);
        } else {
            // Move towards target with organic wandering
            let move_x = (dx / distance) * self.wander_speed + noise_x * 0.3;
            let move_y = (dy / distance) * self.wander_speed + noise_y * 0.3;

            // Smooth acceleration
            self.velocity.x = self.velocity.x * 0.95 + move_x * 0.05;
            self.velocity.y = self.velocity.y * 0.95 + move_y * 0.05;

            // Update position (increase speed for testing)
            self.position.x += self.velocity.x * delta * 60.0;
            self.position.y += self.velocity.y * delta * 60.0;
        }

        // Soft boundaries - gently push back when near edges
        let margin = 50.0;
        if self.position.x < margin {
            self.velocity.x += (margin - self.position.x) * 0.02;
        } else if self.position.x > bounds.width - margin {
            self.velocity.x -= (self.position.x - (bounds.width - margin)) * 0.02;
        }

        if self.position.y < margin {
            self.velocity.y += (margin - self.position.y) * 0.02;
        } else if self.position.y > bounds.height - margin {
            self.velocity.y -= (self.position.y - (bounds.height - margin)) * 0.02;
        }
    }

    fn set_new_target(&mut self, bounds: Size) {
        // Set a new random target within bounds
        let angle = self.wander_angle + (self.position.x * 0.01).sin() * PI;
        let distance = 100.0 + (self.position.y * 0.01).cos() * 200.0;

        self.target = Point::new(
            (self.position.x + angle.cos() * distance)
                .max(50.0)
                .min(bounds.width - 50.0),
            (self.position.y + angle.sin() * distance)
                .max(50.0)
                .min(bounds.height - 50.0),
        );
    }
}

impl Default for FireflySwarm {
    fn default() -> Self {
        Self::new()
    }
}

impl FireflySwarm {
    /// Create a new firefly swarm
    pub fn new() -> Self {
        let mut fireflies = Vec::new();

        // Create fireflies with varied starting positions
        for i in 0..40 {
            let angle = (i as f32) * 2.0 * PI / 40.0;
            let radius = 200.0 + (i as f32 * 0.3).sin() * 100.0;

            fireflies.push(Firefly::new(
                960.0 + angle.cos() * radius,
                540.0 + angle.sin() * radius,
                i,
            ));
        }

        FireflySwarm {
            fireflies: RefCell::new(fireflies),
            cache: RefCell::new(Cache::new()),
            last_update: Cell::new(Instant::now()),
            time: Cell::new(0.0),
            update_counter: Cell::new(0),
            colors: super::view_model::ColorPalette::default(),
        }
    }

    /// Update the firefly positions and states
    pub fn update(&self) {
        let now = Instant::now();
        let delta = (now - self.last_update.get()).as_secs_f32();

        // Only update if enough time has passed
        if delta < 0.001 {
            return;
        }

        self.last_update.set(now);
        let current_time = self.time.get() + delta;
        self.time.set(current_time);

        let mut fireflies = self.fireflies.borrow_mut();
        for firefly in fireflies.iter_mut() {
            // Use a reasonable window size (can be adjusted dynamically later)
            firefly.update(delta, Size::new(1920.0, 1080.0), current_time);
        }

        // Clear cache to force redraw
        self.cache.borrow_mut().clear();

        // Increment counter to signal that state has changed
        self.update_counter.set(self.update_counter.get() + 1);
    }
}

impl Program<()> for FireflySwarm {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        // Get the current time for animation
        let _current_time = self.time.get();
        // Note: current_time reserved for future time-based effects

        // Force redraw for animation by clearing cache
        self.cache.borrow_mut().clear();

        let geometry = self.cache.borrow_mut().draw(renderer, bounds.size(), |frame| {
            // Draw each firefly
            let fireflies = self.fireflies.borrow();
            for firefly in fireflies.iter() {
                // Use the firefly's current position (already updated by update())
                let animated_position = firefly.position;

                // Calculate current glow intensity
                let opacity = firefly.glow_intensity;

                // Skip if too dim
                if opacity < 0.05 {
                    continue;
                }

                // Draw multiple layers for glow effect
                // Outer glow (largest, most transparent)
                let outer_glow = Path::circle(animated_position, firefly.size * 8.0);
                let outer_color = self.colors.with_alpha(firefly.base_color, opacity * 0.05);
                frame.fill(&outer_glow, outer_color);

                // Middle glow
                let middle_glow = Path::circle(animated_position, firefly.size * 4.0);
                let middle_color = self.colors.with_alpha(firefly.base_color, opacity * 0.1);
                frame.fill(&middle_glow, middle_color);

                // Inner glow (brightened)
                let inner_glow = Path::circle(animated_position, firefly.size * 2.0);
                let bright_color = self.colors.lighten(firefly.base_color, 0.2);
                let inner_color = self.colors.with_alpha(bright_color, opacity * 0.3);
                frame.fill(&inner_glow, inner_color);

                // Core (brightest)
                let core = Path::circle(animated_position, firefly.size);
                let core_bright = self.colors.lighten(firefly.base_color, 0.4);
                let core_color = self.colors.with_alpha(core_bright, opacity * 0.8);
                frame.fill(&core, core_color);
            }

            // Draw subtle connections between nearby bright fireflies
            for i in 0..fireflies.len() {
                for j in i + 1..fireflies.len() {
                    let f1 = &fireflies[i];
                    let f2 = &fireflies[j];

                    // Use current positions
                    let pos1 = f1.position;
                    let pos2 = f2.position;

                    // Only connect if both are glowing
                    if f1.glow_intensity < 0.5 || f2.glow_intensity < 0.5 {
                        continue;
                    }

                    let dx = pos1.x - pos2.x;
                    let dy = pos1.y - pos2.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance < 100.0 {
                        let connection_opacity = (1.0 - distance / 100.0)
                            * f1.glow_intensity
                            * f2.glow_intensity
                            * 0.05;

                        let line = Path::line(pos1, pos2);
                        let line_color = self.colors.with_alpha(self.colors.firefly_connection, connection_opacity);
                        frame.stroke(
                            &line,
                            Stroke::default()
                                .with_width(0.5)
                                .with_color(line_color),
                        );
                    }
                }
            }
        });

        vec![geometry]
    }
}

impl FireflySwarm {
    /// Create a canvas element for rendering this firefly swarm
    pub fn view(&self) -> Element<'_, ()> {
        // Create a canvas that redraws on every frame
        struct FireflyRenderer<'a> {
            swarm: &'a FireflySwarm,
        }

        impl<'a> Program<()> for FireflyRenderer<'a> {
            type State = ();

            fn draw(
                &self,
                _state: &Self::State,
                renderer: &iced::Renderer,
                _theme: &Theme,
                bounds: Rectangle,
                _cursor: Cursor,
            ) -> Vec<Geometry> {
                // Always clear cache to force redraw
                self.swarm.cache.borrow_mut().clear();
                self.swarm.draw(_state, renderer, _theme, bounds, _cursor)
            }
        }

        Canvas::new(FireflyRenderer { swarm: self })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}