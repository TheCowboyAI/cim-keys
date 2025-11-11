//! Firefly swarm behavior using boid-like physics
//!
//! This module implements the swarm behavior from www-egui with:
//! - Separation: particles repel when too close
//! - Cohesion: particles move toward nearby swarm center
//! - Alignment: particles match velocity with neighbors
//! - Flowing attraction: alternate between logo and dialog attraction
//! - Pulsing: independent blinking for each firefly

use std::f32::consts::PI;

// Constants matching www-egui
const DRAG: f32 = 0.98;
const RANDOM_FORCE: f32 = 0.15;
const GRAVITY_STRENGTH: f32 = 0.35;
const GRAVITY_RADIUS: f32 = 500.0;
const REPULSION_STRENGTH: f32 = 4.0;
const REPULSION_RADIUS: f32 = 70.0;
const COHESION_STRENGTH: f32 = 0.08;
const ALIGNMENT_STRENGTH: f32 = 0.05;
const NEIGHBOR_RADIUS: f32 = 120.0;

// Attraction points
const LOGO_X: f32 = 80.0;
const LOGO_Y: f32 = 80.0;

/// Pure mathematical representation of a firefly
#[derive(Debug, Clone, Copy)]
pub struct Firefly {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub pulse_offset: f32,
    pub pulse_speed: f32,
}

impl Firefly {
    pub fn new(position: (f32, f32), index: usize) -> Self {
        // Each firefly has unique pulse pattern
        let pulse_offset = (index as f32 * 2.7) % (2.0 * PI);
        let pulse_speed = 0.8 + (index as f32 * 0.13) % 0.6;

        Self {
            position,
            velocity: (0.0, 0.0),
            pulse_offset,
            pulse_speed,
        }
    }

    /// Calculate brightness based on pulsing (0.0 = off, 1.0 = bright)
    pub fn brightness(&self, time: f32) -> f32 {
        let pulse_phase = (time * self.pulse_speed + self.pulse_offset).sin();
        (pulse_phase + 1.0) * 0.5
    }

    /// Update firefly position based on swarm physics
    pub fn update(
        &mut self,
        dt: f32,
        time: f32,
        others: &[Firefly],
        dialog_center: (f32, f32),
        screen_size: (f32, f32),
    ) {
        // Flowing attraction between logo and dialog
        let logo_pos = (LOGO_X, LOGO_Y);
        let to_logo = (logo_pos.0 - self.position.0, logo_pos.1 - self.position.1);
        let logo_dist = (to_logo.0 * to_logo.0 + to_logo.1 * to_logo.1).sqrt();

        let to_dialog = (dialog_center.0 - self.position.0, dialog_center.1 - self.position.1);
        let dialog_dist = (to_dialog.0 * to_dialog.0 + to_dialog.1 * to_dialog.1).sqrt();

        let attraction_threshold = 150.0;

        if logo_dist < dialog_dist {
            // Closer to logo, be attracted to dialog
            if dialog_dist > 1.0 {
                let flow_strength = GRAVITY_STRENGTH * 1.5;
                let gravity_force = (flow_strength / (dialog_dist / 100.0).max(0.5))
                    * (1.0 - (dialog_dist / GRAVITY_RADIUS).min(1.0));
                let norm_factor = dialog_dist.max(0.001);
                self.velocity.0 += (to_dialog.0 / norm_factor) * gravity_force;
                self.velocity.1 += (to_dialog.1 / norm_factor) * gravity_force;
            }
            // Weak repulsion from logo when very close
            if logo_dist < attraction_threshold && logo_dist > 1.0 {
                let repel = 0.1 * (attraction_threshold - logo_dist) / attraction_threshold;
                let norm_factor = logo_dist.max(0.001);
                self.velocity.0 -= (to_logo.0 / norm_factor) * repel;
                self.velocity.1 -= (to_logo.1 / norm_factor) * repel;
            }
        } else {
            // Closer to dialog, be attracted to logo
            if logo_dist > 1.0 {
                let flow_strength = GRAVITY_STRENGTH * 1.5;
                let gravity_force = (flow_strength / (logo_dist / 100.0).max(0.5))
                    * (1.0 - (logo_dist / GRAVITY_RADIUS).min(1.0));
                let norm_factor = logo_dist.max(0.001);
                self.velocity.0 += (to_logo.0 / norm_factor) * gravity_force;
                self.velocity.1 += (to_logo.1 / norm_factor) * gravity_force;
            }
            // Weak repulsion from dialog when very close
            if dialog_dist < attraction_threshold && dialog_dist > 1.0 {
                let repel = 0.1 * (attraction_threshold - dialog_dist) / attraction_threshold;
                let norm_factor = dialog_dist.max(0.001);
                self.velocity.0 -= (to_dialog.0 / norm_factor) * repel;
                self.velocity.1 -= (to_dialog.1 / norm_factor) * repel;
            }
        }

        // Screen boundary forces
        let boundary_margin = 50.0;
        let boundary_strength = 2.0;

        if self.position.0 < boundary_margin {
            self.velocity.0 += boundary_strength * (boundary_margin - self.position.0) / boundary_margin;
        }
        if self.position.0 > screen_size.0 - boundary_margin {
            self.velocity.0 -= boundary_strength * (self.position.0 - (screen_size.0 - boundary_margin)) / boundary_margin;
        }
        if self.position.1 < boundary_margin {
            self.velocity.1 += boundary_strength * (boundary_margin - self.position.1) / boundary_margin;
        }
        if self.position.1 > screen_size.1 - boundary_margin {
            self.velocity.1 -= boundary_strength * (self.position.1 - (screen_size.1 - boundary_margin)) / boundary_margin;
        }

        // Swarm behaviors
        let mut separation = (0.0, 0.0);
        let mut cohesion_center = (0.0, 0.0);
        let mut alignment = (0.0, 0.0);
        let mut neighbor_count = 0;

        for other in others {
            let to_other = (other.position.0 - self.position.0, other.position.1 - self.position.1);
            let distance = (to_other.0 * to_other.0 + to_other.1 * to_other.1).sqrt();

            if distance > 0.0 {
                // Separation
                if distance < REPULSION_RADIUS {
                    let repulsion = (REPULSION_STRENGTH / distance.max(10.0))
                        * (1.0 - distance / REPULSION_RADIUS);
                    let norm_factor = distance.max(0.001);
                    separation.0 -= (to_other.0 / norm_factor) * repulsion;
                    separation.1 -= (to_other.1 / norm_factor) * repulsion;
                }

                // Cohesion & Alignment
                if distance < NEIGHBOR_RADIUS {
                    cohesion_center.0 += other.position.0;
                    cohesion_center.1 += other.position.1;
                    alignment.0 += other.velocity.0;
                    alignment.1 += other.velocity.1;
                    neighbor_count += 1;
                }
            }
        }

        // Apply separation
        self.velocity.0 += separation.0;
        self.velocity.1 += separation.1;

        // Apply cohesion
        if neighbor_count > 0 {
            cohesion_center.0 /= neighbor_count as f32;
            cohesion_center.1 /= neighbor_count as f32;
            let to_center = (cohesion_center.0 - self.position.0, cohesion_center.1 - self.position.1);
            let center_dist = (to_center.0 * to_center.0 + to_center.1 * to_center.1).sqrt();
            if center_dist > 0.0 {
                let norm_factor = center_dist.max(0.001);
                self.velocity.0 += (to_center.0 / norm_factor) * COHESION_STRENGTH;
                self.velocity.1 += (to_center.1 / norm_factor) * COHESION_STRENGTH;
            }

            // Apply alignment
            alignment.0 /= neighbor_count as f32;
            alignment.1 /= neighbor_count as f32;
            self.velocity.0 += (alignment.0 - self.velocity.0) * ALIGNMENT_STRENGTH;
            self.velocity.1 += (alignment.1 - self.velocity.1) * ALIGNMENT_STRENGTH;
        }

        // Random drift
        let noise_x = (time * 0.5 + self.position.0 * 0.01).sin() * RANDOM_FORCE;
        let noise_y = (time * 0.7 + self.position.1 * 0.01).cos() * RANDOM_FORCE;
        self.velocity.0 += noise_x;
        self.velocity.1 += noise_y;

        // Apply drag
        self.velocity.0 *= DRAG;
        self.velocity.1 *= DRAG;

        // Update position
        self.position.0 += self.velocity.0 * dt;
        self.position.1 += self.velocity.1 * dt;
    }
}

/// FRP (Functional Reactive Programming) layer
pub mod frp {
    use super::*;

    const INITIAL_PARTICLE_COUNT: usize = 9;
    const MAX_PARTICLE_COUNT: usize = 80;
    const CONNECTION_DISTANCE: f32 = 150.0;
    const CONNECTION_DURATION_MIN: f32 = 1.0;
    const CONNECTION_DURATION_MAX: f32 = 3.0;
    const MAX_CONNECTIONS_PER_PARTICLE: usize = 2;

    #[derive(Clone, Copy)]
    pub struct TimeStep(pub f32);

    #[derive(Clone)]
    pub enum FireflyMessage {
        Tick(TimeStep),
        Resize(f32, f32),
    }

    #[derive(Clone, Debug)]
    struct Connection {
        particle_a: usize,
        particle_b: usize,
        birth_time: f32,
        duration: f32,
        random_seed: f32,
    }

    #[derive(Clone, Debug)]
    pub struct FireflySystem {
        fireflies: Vec<Firefly>,
        connections: Vec<Connection>,
        last_update: f32,
        next_spawn_time: f32,
        screen_size: (f32, f32),
    }

    impl FireflySystem {
        pub fn new(_num_fireflies: usize) -> Self {
            let screen_size = (1024.0, 768.0); // Default size
            let mut fireflies = Vec::with_capacity(MAX_PARTICLE_COUNT);

            // Start with initial particles
            for i in 0..INITIAL_PARTICLE_COUNT {
                let angle = (i as f32 / INITIAL_PARTICLE_COUNT as f32) * 2.0 * PI;
                let spawn_distance = 150.0;

                let x = screen_size.0 / 2.0 + angle.cos() * spawn_distance;
                let y = screen_size.1 / 2.0 + angle.sin() * spawn_distance;

                let mut firefly = Firefly::new((x, y), i);
                // Initial velocity toward center
                let to_center_x = screen_size.0 / 2.0 - x;
                let to_center_y = screen_size.1 / 2.0 - y;
                let norm = (to_center_x * to_center_x + to_center_y * to_center_y).sqrt().max(0.001);
                firefly.velocity = (to_center_x / norm, to_center_y / norm);
                fireflies.push(firefly);
            }

            Self {
                fireflies,
                connections: Vec::new(),
                last_update: 0.0,
                next_spawn_time: 1.0,
                screen_size,
            }
        }

        pub fn update(mut self, message: FireflyMessage) -> Self {
            match message {
                FireflyMessage::Tick(TimeStep(dt)) => {
                    let current_time = self.last_update + dt;
                    let actual_dt = dt.min(0.1);

                    let dialog_center = (self.screen_size.0 / 2.0, self.screen_size.1 / 2.0);

                    // Spawn new particles
                    if self.fireflies.len() < MAX_PARTICLE_COUNT && current_time >= self.next_spawn_time {
                        let angle = (self.fireflies.len() as f32 * 7.3) % (2.0 * PI);
                        let spawn_distance = (self.screen_size.0.max(self.screen_size.1) / 2.0) + 100.0;

                        let x = dialog_center.0 + angle.cos() * spawn_distance;
                        let y = dialog_center.1 + angle.sin() * spawn_distance;

                        let mut new_firefly = Firefly::new((x, y), self.fireflies.len());
                        let to_center_x = dialog_center.0 - x;
                        let to_center_y = dialog_center.1 - y;
                        let norm = (to_center_x * to_center_x + to_center_y * to_center_y).sqrt().max(0.001);
                        new_firefly.velocity = (to_center_x / norm * 2.0, to_center_y / norm * 2.0);
                        self.fireflies.push(new_firefly);

                        // Random spawn delay
                        let seed = ((current_time * 1000.0 + self.fireflies.len() as f32 * 7.3) % 1500.0).abs();
                        self.next_spawn_time = current_time + 0.5 + (seed / 1000.0);
                    }

                    // Update all fireflies
                    let fireflies_clone = self.fireflies.clone();
                    for firefly in &mut self.fireflies {
                        firefly.update(actual_dt, current_time, &fireflies_clone, dialog_center, self.screen_size);
                    }

                    // Update connections - remove expired ones
                    self.connections.retain(|conn| {
                        current_time - conn.birth_time < conn.duration
                    });

                    // Try to form new connections between nearby particles
                    if (current_time * 37.0).sin() > 0.96 {  // Random chance based on time
                        for i in 0..self.fireflies.len() {
                            // Count existing connections for this particle
                            let conn_count = self.connections.iter()
                                .filter(|c| c.particle_a == i || c.particle_b == i)
                                .count();

                            if conn_count >= MAX_CONNECTIONS_PER_PARTICLE {
                                continue;
                            }

                            // Look for nearby particles to connect to
                            for j in (i + 1)..self.fireflies.len() {
                                let dx = self.fireflies[i].position.0 - self.fireflies[j].position.0;
                                let dy = self.fireflies[i].position.1 - self.fireflies[j].position.1;
                                let dist = (dx * dx + dy * dy).sqrt();

                                if dist < CONNECTION_DISTANCE {
                                    // Check if both particles are visible (brightness > threshold)
                                    let brightness_i = self.fireflies[i].brightness(current_time);
                                    let brightness_j = self.fireflies[j].brightness(current_time);

                                    if brightness_i > 0.1 && brightness_j > 0.1 {
                                        // Check if already connected
                                        let already_connected = self.connections.iter()
                                            .any(|c| (c.particle_a == i && c.particle_b == j) || (c.particle_a == j && c.particle_b == i));

                                        if !already_connected {
                                            // Check if particle j also has space
                                            let j_conn_count = self.connections.iter()
                                                .filter(|c| c.particle_a == j || c.particle_b == j)
                                                .count();

                                            if j_conn_count < MAX_CONNECTIONS_PER_PARTICLE {
                                                // Random duration and seed for variation
                                                let random_val = ((current_time * 1000.0 + i as f32 * 13.7 + j as f32 * 7.3) % 1000.0) / 1000.0;
                                                let duration = CONNECTION_DURATION_MIN + random_val * (CONNECTION_DURATION_MAX - CONNECTION_DURATION_MIN);
                                                let random_seed = ((current_time * 537.0 + i as f32 * 41.0 + j as f32 * 23.0) % 1000.0) / 1000.0;

                                                // Form connection
                                                self.connections.push(Connection {
                                                    particle_a: i,
                                                    particle_b: j,
                                                    birth_time: current_time,
                                                    duration,
                                                    random_seed,
                                                });
                                                break;  // Only form one connection per particle per update
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    self.last_update = current_time;
                    self
                }
                FireflyMessage::Resize(width, height) => {
                    self.screen_size = (width, height);
                    self
                }
            }
        }

        pub fn to_visual(&self) -> Vec<FireflyVisual> {
            self.fireflies.iter().map(|f| FireflyVisual {
                position: f.position,
                brightness: f.brightness(self.last_update),
                color: [220.0 / 255.0, 255.0 / 255.0, 80.0 / 255.0], // Yellow-green
                size: 15.0,
            }).collect()
        }

        pub fn get_connections(&self) -> Vec<ConnectionVisual> {
            self.connections.iter().filter_map(|conn| {
                // Check if both particles are still visible
                let brightness_a = self.fireflies[conn.particle_a].brightness(self.last_update);
                let brightness_b = self.fireflies[conn.particle_b].brightness(self.last_update);

                if brightness_a < 0.1 || brightness_b < 0.1 {
                    return None; // One or both particles are invisible
                }

                // Calculate fade based on connection lifetime
                let age = self.last_update - conn.birth_time;
                let life_progress = age / conn.duration;

                if life_progress >= 1.0 {
                    return None; // Expired
                }

                // Fade in/out matching www-egui
                let base_fade = if life_progress < 0.3 {
                    life_progress / 0.3  // Fade in
                } else if life_progress > 0.7 {
                    (1.0 - life_progress) / 0.3  // Fade out
                } else {
                    1.0  // Full brightness
                };

                // Also scale by particle brightness (so line fades with particles)
                let brightness_factor = brightness_a.min(brightness_b);
                let fade = base_fade * (0.5 + conn.random_seed * 0.5) * brightness_factor;

                Some(ConnectionVisual {
                    from: self.fireflies[conn.particle_a].position,
                    to: self.fireflies[conn.particle_b].position,
                    fade,
                })
            }).collect()
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct FireflyVisual {
        pub position: (f32, f32),
        pub brightness: f32,
        pub color: [f32; 3],
        pub size: f32,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct ConnectionVisual {
        pub from: (f32, f32),
        pub to: (f32, f32),
        pub fade: f32,
    }
}
