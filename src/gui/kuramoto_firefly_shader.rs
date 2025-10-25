//! GPU-optimized firefly swarm with Kuramoto synchronization model
//! Uses SDF for glow effects and Taylor series for fast trigonometry

use iced::{
    mouse,
    widget::shader::{self, wgpu, Viewport},
    Color, Rectangle, Size, Transformation,
};
use std::sync::Arc;

// Swarm configuration constants
const NUM_FIREFLIES: u32 = 40;  // Increased for better synchronization effect
const COUPLING_STRENGTH: f32 = 2.5;  // K parameter for Kuramoto model
const COUPLING_RADIUS: f32 = 0.3;  // Spatial coupling radius

#[derive(Debug, Clone, PartialEq)]
pub struct KuramotoFireflyShader {
    pub time: f32,
    // Phase data for synchronization
    phases: Vec<f32>,
    // Intrinsic frequencies for each firefly
    frequencies: Vec<f32>,
}

impl KuramotoFireflyShader {
    pub fn new() -> Self {
        // Initialize with random phases and frequencies
        let mut phases = Vec::with_capacity(NUM_FIREFLIES as usize);
        let mut frequencies = Vec::with_capacity(NUM_FIREFLIES as usize);

        for i in 0..NUM_FIREFLIES {
            // Random initial phases
            phases.push((i as f32 * 0.618033988749895) % (2.0 * std::f32::consts::PI));
            // Slightly different intrinsic frequencies (around 1 Hz)
            frequencies.push(0.8 + (i as f32 * 0.1) % 0.4);
        }

        Self {
            time: 0.0,
            phases,
            frequencies,
        }
    }

    pub fn with_time(time: f32) -> Self {
        let mut shader = Self::new();

        // Calculate delta time for Kuramoto phase updates
        let delta_time = time - shader.time;
        shader.time = time;

        // Update phases using Kuramoto model on CPU side
        // (In production, this would be in a compute shader)
        if delta_time > 0.0 && delta_time < 1.0 {  // Sanity check for reasonable delta
            shader.update_kuramoto_phases(delta_time);
        }
        shader
    }

    pub fn update_kuramoto_phases(&mut self, dt: f32) {
        let mut new_phases = self.phases.clone();

        for i in 0..NUM_FIREFLIES as usize {
            let mut coupling_sum = 0.0;
            let mut neighbor_count = 0;

            // Calculate coupling from all fireflies (spatial locality would optimize this)
            for j in 0..NUM_FIREFLIES as usize {
                if i != j {
                    let phase_diff = self.phases[j] - self.phases[i];
                    // Use fast_sin approximation
                    coupling_sum += fast_sin(phase_diff);
                    neighbor_count += 1;
                }
            }

            // Kuramoto update equation
            if neighbor_count > 0 {
                let phase_velocity = self.frequencies[i] +
                    (COUPLING_STRENGTH / neighbor_count as f32) * coupling_sum;
                new_phases[i] = self.phases[i] + dt * phase_velocity;
            }
        }

        self.phases = new_phases;
    }
}

// Fast sine approximation using 5th order Taylor series
fn fast_sin(x: f32) -> f32 {
    // Normalize to [-PI, PI]
    let x = x - (x / (2.0 * std::f32::consts::PI)).floor() * 2.0 * std::f32::consts::PI;
    let x = if x > std::f32::consts::PI {
        x - 2.0 * std::f32::consts::PI
    } else if x < -std::f32::consts::PI {
        x + 2.0 * std::f32::consts::PI
    } else {
        x
    };

    // 5th order Taylor series: x - x³/6 + x⁵/120
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    x - x3 / 6.0 + x5 / 120.0
}

impl<Message> shader::Program<Message> for KuramotoFireflyShader {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new(bounds.size(), self.time, self.phases.clone())
    }
}

#[derive(Debug)]
pub struct Primitive {
    size: Size,
    time: f32,
    phases: Vec<f32>,
}

impl Primitive {
    pub fn new(size: Size, time: f32, phases: Vec<f32>) -> Self {
        Self { size, time, phases }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, format));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        // Pack uniforms with phase data
        let uniforms = Uniforms {
            time: self.time,
            _padding: 0.0,
            resolution: [bounds.width, bounds.height],
            num_fireflies: NUM_FIREFLIES as f32,
            coupling_strength: COUPLING_STRENGTH,
        };

        queue.write_buffer(&pipeline.uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Upload phase data as storage buffer
        let phase_data: Vec<f32> = self.phases.iter()
            .map(|p| fast_sin(*p) * 0.5 + 0.5)  // Convert phase to intensity
            .collect();

        queue.write_buffer(
            &pipeline.phase_buffer,
            0,
            bytemuck::cast_slice(&phase_data),
        );
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        _clip_bounds: &Rectangle<u32>,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Kuramoto Firefly render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&pipeline.render_pipeline);
        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);

        // Draw all fireflies as a single batch of triangles
        // 6 vertices per firefly * 40 fireflies = 240 vertices total
        render_pass.draw(0..(6 * NUM_FIREFLIES), 0..1);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    _padding: f32,  // Padding for vec2 alignment in WGSL
    resolution: [f32; 2],
    num_fireflies: f32,
    coupling_strength: f32,
}

struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
    phase_buffer: wgpu::Buffer,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Kuramoto Firefly shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER_SOURCE)),
        });

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kuramoto uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let phase_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Phase data"),
            size: (NUM_FIREFLIES * 4) as u64,  // f32 per firefly
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Kuramoto bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Kuramoto bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: phase_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Kuramoto pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Kuramoto pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            render_pipeline,
            bind_group,
            uniforms_buffer,
            phase_buffer,
        }
    }
}

const SHADER_SOURCE: &str = r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    num_fireflies: f32,
    coupling_strength: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) firefly_id: f32,
    @location(2) intensity: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> phase_intensities: array<f32>;

// Fast hash for position generation
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Taylor series approximation for sine (5th order)
fn taylor_sin(x: f32) -> f32 {
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    return x - x3 * 0.16666667 + x5 * 0.00833333;
}

// Taylor series approximation for cosine (6th order)
fn taylor_cos(x: f32) -> f32 {
    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    return 1.0 - x2 * 0.5 + x4 * 0.04166667 - x6 * 0.00138889;
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var output: VertexOutput;

    // Calculate which firefly this vertex belongs to
    let firefly_idx = vertex_idx / 6u;  // Each firefly has 6 vertices
    let firefly_id = f32(firefly_idx);
    output.firefly_id = firefly_id;

    // Get synchronized intensity from phase data
    output.intensity = phase_intensities[firefly_idx];

    // DEBUG: Spread fireflies in a visible pattern
    let x_pos = (f32(firefly_idx % 10u) * 0.1) + 0.05;  // 10 columns
    let y_pos = (f32(firefly_idx / 10u) * 0.2) + 0.2;   // 4 rows
    var base_pos = vec2<f32>(x_pos, y_pos);  // Grid pattern

    // Complex organic movement using multiple Lissajous frequencies - INCREASED AMPLITUDE
    let movement_phase = uniforms.time * 0.5 + firefly_id * 0.618033988749895;  // Increased speed

    // Primary motion - large scale figure-8 patterns (INCREASED)
    let primary_x = taylor_sin(movement_phase * 1.3) * 0.15;  // Was 0.25, now more reasonable
    let primary_y = taylor_cos(movement_phase * 0.7 + 1.5707963267948966) * 0.12;

    // Secondary motion - smaller circular orbits
    let secondary_phase = movement_phase * 2.7;
    let secondary_x = taylor_sin(secondary_phase) * 0.05;  // Reduced from 0.08
    let secondary_y = taylor_cos(secondary_phase * 1.1) * 0.05;

    // Tertiary motion - tiny vibrations for liveliness
    let vibration = taylor_sin(movement_phase * 8.3) * 0.01;  // Reduced from 0.02

    // Combine movement components
    var velocity = vec2<f32>(
        primary_x + secondary_x + vibration * taylor_cos(movement_phase * 5.0),
        primary_y + secondary_y + vibration * taylor_sin(movement_phase * 5.0)
    );

    // DEBUG: Disable movement to see static grid
    // base_pos += velocity;

    // Soft boundary constraints with elegant turning
    // Based on FIREFLY_SWARM_MODELS.md Section 4: Movement Dynamics
    let boundary_margin = 0.05;      // 5% margin from edges
    let boundary_softness = 0.1;     // Soft zone for smooth transitions
    let reflection_factor = 0.8;     // 80% velocity retained on reflection

    // Calculate edge proximity for smooth steering
    let edge_proximity = vec4<f32>(
        base_pos.x,                  // Distance from left
        1.0 - base_pos.x,             // Distance from right
        base_pos.y,                   // Distance from bottom
        1.0 - base_pos.y              // Distance from top
    );

    // Create vector field for elegant turning near boundaries
    var steering = vec2<f32>(0.0, 0.0);

    // Left boundary - smooth rightward steering
    if (edge_proximity.x < boundary_margin + boundary_softness) {
        let proximity_factor = smoothstep(0.0, boundary_margin + boundary_softness, edge_proximity.x);
        // Steer right with increasing strength as we approach boundary
        steering.x += (1.0 - proximity_factor) * 0.2;
        // Also add subtle upward/downward component based on current y position
        steering.y += (hash21(vec2<f32>(firefly_id, uniforms.time)) - 0.5) * 0.1 * (1.0 - proximity_factor);
    }

    // Right boundary - smooth leftward steering
    if (edge_proximity.y < boundary_margin + boundary_softness) {
        let proximity_factor = smoothstep(0.0, boundary_margin + boundary_softness, edge_proximity.y);
        steering.x -= (1.0 - proximity_factor) * 0.2;
        steering.y += (hash21(vec2<f32>(firefly_id + 1.0, uniforms.time)) - 0.5) * 0.1 * (1.0 - proximity_factor);
    }

    // Bottom boundary - smooth upward steering
    if (edge_proximity.z < boundary_margin + boundary_softness) {
        let proximity_factor = smoothstep(0.0, boundary_margin + boundary_softness, edge_proximity.z);
        steering.y += (1.0 - proximity_factor) * 0.2;
        steering.x += (hash21(vec2<f32>(firefly_id + 2.0, uniforms.time)) - 0.5) * 0.1 * (1.0 - proximity_factor);
    }

    // Top boundary - smooth downward steering
    if (edge_proximity.w < boundary_margin + boundary_softness) {
        let proximity_factor = smoothstep(0.0, boundary_margin + boundary_softness, edge_proximity.w);
        steering.y -= (1.0 - proximity_factor) * 0.2;
        steering.x += (hash21(vec2<f32>(firefly_id + 3.0, uniforms.time)) - 0.5) * 0.1 * (1.0 - proximity_factor);
    }

    // Apply steering forces with velocity blending
    base_pos += steering;

    // Soft reflection if we're very close to boundary
    if (base_pos.x < boundary_margin) {
        base_pos.x = boundary_margin + (boundary_margin - base_pos.x) * reflection_factor;
    }
    if (base_pos.x > 1.0 - boundary_margin) {
        base_pos.x = 1.0 - boundary_margin - (base_pos.x - (1.0 - boundary_margin)) * reflection_factor;
    }
    if (base_pos.y < boundary_margin) {
        base_pos.y = boundary_margin + (boundary_margin - base_pos.y) * reflection_factor;
    }
    if (base_pos.y > 1.0 - boundary_margin) {
        base_pos.y = 1.0 - boundary_margin - (base_pos.y - (1.0 - boundary_margin)) * reflection_factor;
    }

    // Final safety clamp (should rarely trigger with proper steering)
    base_pos = clamp(base_pos, vec2<f32>(boundary_margin), vec2<f32>(1.0 - boundary_margin));

    // Create quad vertices (two triangles for a square)
    var vertex_pos: vec2<f32>;
    let vtx = vertex_idx % 6u;  // Need modulo for proper vertex generation
    switch vtx {
        case 0u: { vertex_pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { vertex_pos = vec2<f32>(1.0, -1.0); }
        case 2u: { vertex_pos = vec2<f32>(-1.0, 1.0); }
        case 3u: { vertex_pos = vec2<f32>(1.0, -1.0); }
        case 4u: { vertex_pos = vec2<f32>(1.0, 1.0); }
        case 5u: { vertex_pos = vec2<f32>(-1.0, 1.0); }
        default: { vertex_pos = vec2<f32>(0.0, 0.0); }
    }

    // Scale firefly based on intensity (synchronized pulsing) - BIGGER DEBUG SIZE
    let size = 0.05;  // Fixed large size for debugging
    let world_pos = base_pos + vertex_pos * size;

    output.clip_position = vec4<f32>(world_pos * 2.0 - 1.0, 0.0, 1.0);
    output.uv = vertex_pos;

    return output;
}

// SDF circle for efficient glow rendering
fn sdf_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

// Smooth minimum for SDF blending
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / k;
    return min(a, b) - h * h * k * 0.25;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // SDF-based ethereal glow effect
    let dist = sdf_circle(input.uv, 0.05);  // Smaller core

    // Multi-layer glow using SDF for ethereal appearance
    let core = smoothstep(0.02, 0.0, dist) * 2.0;  // Bright, small core
    let inner_glow = smoothstep(0.3, 0.0, dist);    // Soft inner halo
    let mid_glow = smoothstep(0.6, 0.0, dist);      // Medium halo
    let outer_glow = smoothstep(1.2, 0.0, dist);    // Far-reaching outer glow

    // Combine glow layers with synchronized intensity for ethereal effect
    let glow = core * 0.8 + inner_glow * 0.5 + mid_glow * 0.3 + outer_glow * 0.15;
    let final_intensity = glow * (0.4 + input.intensity * 0.6);  // Always some base glow

    // DEBUG: Unique color for each firefly to verify instancing
    let hue = input.firefly_id / 40.0;  // Different hue for each firefly
    let color = vec3<f32>(
        0.5 + 0.5 * cos(hue * 6.28318 + 0.0),
        0.5 + 0.5 * cos(hue * 6.28318 + 2.094),
        0.5 + 0.5 * cos(hue * 6.28318 + 4.189)
    );
    let final_color = color;

    // Final output with soft alpha falloff
    let alpha = final_intensity * 0.7;  // More transparent for ethereal look
    return vec4<f32>(final_color * final_intensity, alpha);
}
"#;