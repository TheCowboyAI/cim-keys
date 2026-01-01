//! Animated background with moving particles similar to The Cowboy AI website
//!
//! Creates a starfield/particle effect with glowing orbs moving across the screen

use iced::{
    widget::canvas::{Cache, Canvas, Geometry, Path, Program, Stroke},
    mouse::Cursor, Color, Element, Length, Point, Rectangle, Theme, Vector,
};
use std::time::Instant;

/// Animated background with moving particles
pub struct AnimatedBackground {
    particles: Vec<Particle>,
    cache: Cache,
    last_update: Instant,
    colors: super::view_model::ColorPalette,
}

/// A single particle in the animation
struct Particle {
    position: Point,
    velocity: Vector,
    size: f32,
    opacity: f32,
    color: Color,
}

impl Default for AnimatedBackground {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimatedBackground {
    /// Create a new animated background with particles
    pub fn new() -> Self {
        let mut particles = Vec::new();
        let colors = super::view_model::ColorPalette::default();

        // Create initial particles with random positions and velocities
        for i in 0..50 {
            let angle = (i as f32) * 0.3;
            particles.push(Particle {
                position: Point::new(
                    (i as f32 * 47.0) % 1920.0,
                    (i as f32 * 31.0) % 1080.0,
                ),
                velocity: Vector::new(
                    angle.cos() * 0.5,
                    angle.sin() * 0.3,
                ),
                size: 2.0 + (i % 3) as f32,
                opacity: 0.3 + ((i % 5) as f32 * 0.1),
                color: if i % 3 == 0 {
                    colors.particle_blue
                } else if i % 3 == 1 {
                    colors.particle_purple
                } else {
                    colors.particle_cyan
                },
            });
        }

        AnimatedBackground {
            particles,
            cache: Cache::new(),
            last_update: Instant::now(),
            colors,
        }
    }

    /// Update particle positions
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = (now - self.last_update).as_secs_f32();
        self.last_update = now;

        for particle in &mut self.particles {
            // Update position based on velocity
            particle.position.x += particle.velocity.x * delta * 30.0;
            particle.position.y += particle.velocity.y * delta * 30.0;

            // Wrap around screen edges
            if particle.position.x < -50.0 {
                particle.position.x = 1970.0;
            } else if particle.position.x > 1970.0 {
                particle.position.x = -50.0;
            }

            if particle.position.y < -50.0 {
                particle.position.y = 1130.0;
            } else if particle.position.y > 1130.0 {
                particle.position.y = -50.0;
            }

            // Slight oscillation in opacity for twinkling effect
            particle.opacity = (particle.opacity + delta * 0.2).sin().abs() * 0.5 + 0.3;
        }

        // Clear cache to force redraw
        self.cache.clear();
    }
}

impl Program<()> for AnimatedBackground {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Draw each particle
            for particle in &self.particles {
                // Create glow effect with multiple circles
                for i in 0..3 {
                    let glow_size = particle.size * (3.0 - i as f32);
                    let glow_opacity = particle.opacity * 0.2 / (i + 1) as f32;

                    let circle = Path::circle(particle.position, glow_size);
                    let glow_color = self.colors.with_alpha(particle.color, glow_opacity);
                    frame.fill(&circle, glow_color);
                }

                // Draw the main particle
                let circle = Path::circle(particle.position, particle.size);
                let particle_color = self.colors.with_alpha(particle.color, particle.opacity);
                frame.fill(&circle, particle_color);
            }

            // Draw connecting lines between nearby particles
            for i in 0..self.particles.len() {
                for j in i + 1..self.particles.len() {
                    let p1 = &self.particles[i];
                    let p2 = &self.particles[j];

                    let dx = p1.position.x - p2.position.x;
                    let dy = p1.position.y - p2.position.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance < 150.0 {
                        let opacity = (1.0 - distance / 150.0) * 0.1;
                        let line = Path::line(p1.position, p2.position);
                        let line_color = self.colors.with_alpha(self.colors.particle_blue, opacity);
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

/// Create an animated background element
pub fn animated_background<'a>() -> Element<'a, ()> {
    Canvas::new(AnimatedBackground::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}