//! Firefly swarm synchronization using coupled nonlinear oscillators
//!
//! Implements the Kuramoto-inspired firefly synchronization model where fireflies
//! gradually synchronize their flashing patterns through local interactions.

use iced::{
    mouse,
    widget::shader::{self, wgpu, Viewport},
    Rectangle, Size,
};
use bytemuck::{Pod, Zeroable};
use std::mem;

/// Firefly synchronization shader with coupled oscillators
#[derive(Debug, Clone, PartialEq)]
pub struct FireflySynchronization {
    time: f32,
    num_fireflies: u32,
}

impl FireflySynchronization {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            num_fireflies: 50,
        }
    }

    pub fn with_time(time: f32) -> Self {
        Self {
            time,
            num_fireflies: 50,
        }
    }
}

impl<Message> shader::Program<Message> for FireflySynchronization {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        // Debug: Print to verify draw is called with updated time
        eprintln!("FireflySynchronization draw called: time = {}", self.time);
        Primitive::new(bounds.size(), self.time, self.num_fireflies)
    }
}

#[derive(Debug)]
pub struct Primitive {
    size: Size,
    time: f32,
    num_fireflies: u32,
}

impl Primitive {
    pub fn new(size: Size, time: f32, num_fireflies: u32) -> Self {
        Self { size, time, num_fireflies }
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
            storage.store(Pipeline::new(
                device,
                format,
                Size::new(viewport.physical_width(), viewport.physical_height()),
                self.num_fireflies,
            ));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        // Update simulation parameters
        let params = SimulationParams {
            time: self.time,
            delta_time: 0.016, // 60 FPS
            resolution: [bounds.width, bounds.height],
            num_fireflies: self.num_fireflies,
            coupling_strength: 0.5,    // g parameter
            interaction_radius: 150.0,  // R parameter
            noise_strength: 0.1,       // sigma parameter
            adaptation_rate: 0.01,      // epsilon parameter
        };

        queue.write_buffer(&pipeline.params_buffer, 0, bytemuck::bytes_of(&params));
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        _clip_bounds: &Rectangle<u32>,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();

        // First, run compute shader to update firefly states
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Firefly synchronization compute pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&pipeline.compute_pipeline);
            compute_pass.set_bind_group(0, &pipeline.compute_bind_group, &[]);

            let workgroups = (self.num_fireflies + 63) / 64; // 64 threads per workgroup
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Then render the fireflies
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Firefly render pass"),
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
            render_pass.set_bind_group(0, &pipeline.render_bind_group, &[]);

            // Draw fireflies
            render_pass.draw(0..6, 0..self.num_fireflies);
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct SimulationParams {
    time: f32,
    delta_time: f32,
    resolution: [f32; 2],
    num_fireflies: u32,
    coupling_strength: f32,
    interaction_radius: f32,
    noise_strength: f32,
    adaptation_rate: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct FireflyState {
    position: [f32; 2],
    u: f32,           // Oscillator state variable 1
    v: f32,           // Oscillator state variable 2
    lambda: f32,      // Bifurcation parameter
    omega: f32,       // Natural frequency
    phase: f32,       // Current phase
    intensity: f32,   // Current light intensity
}

struct Pipeline {
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    compute_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,
    firefly_buffer: wgpu::Buffer,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat, _target_size: Size<u32>, num_fireflies: u32) -> Self {
        // Create compute shader for updating firefly states
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Firefly synchronization compute shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(COMPUTE_SHADER)),
        });

        // Create render shader for drawing fireflies
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Firefly render shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(RENDER_SHADER)),
        });

        // Create buffers
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Simulation params"),
            size: mem::size_of::<SimulationParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Initialize firefly states
        let mut firefly_states = Vec::with_capacity(num_fireflies as usize);
        for i in 0..num_fireflies {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_fireflies as f32);
            let radius = 200.0 + (i as f32 * 0.618).sin() * 100.0; // Golden ratio distribution

            firefly_states.push(FireflyState {
                position: [
                    0.5 + angle.cos() * radius / 1920.0,
                    0.5 + angle.sin() * radius / 1080.0,
                ],
                u: (i as f32 * 0.1).cos(),
                v: (i as f32 * 0.1).sin(),
                lambda: 0.1,
                omega: 1.0 + (i as f32 * 0.05).sin() * 0.2, // Slight frequency variation
                phase: (i as f32) * 0.5,
                intensity: 0.0,
            });
        }

        // Create buffer and fill with initial data
        let firefly_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Firefly states"),
            size: (mem::size_of::<FireflyState>() * num_fireflies as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });

        {
            let mut buffer_view = firefly_buffer.slice(..).get_mapped_range_mut();
            buffer_view.copy_from_slice(bytemuck::cast_slice(&firefly_states));
        }
        firefly_buffer.unmap();

        // Create compute pipeline
        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: firefly_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute pipeline layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Firefly synchronization compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        // Create render pipeline
        let render_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render bind group layout"),
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

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render bind group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: firefly_buffer.as_entire_binding(),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[&render_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Firefly render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
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
                module: &render_shader,
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
            compute_pipeline,
            render_pipeline,
            compute_bind_group,
            render_bind_group,
            params_buffer,
            firefly_buffer,
        }
    }
}

// Compute shader for updating firefly states using coupled oscillators
const COMPUTE_SHADER: &str = r#"
struct SimulationParams {
    time: f32,
    delta_time: f32,
    resolution: vec2<f32>,
    num_fireflies: u32,
    coupling_strength: f32,
    interaction_radius: f32,
    noise_strength: f32,
    adaptation_rate: f32,
}

struct FireflyState {
    position: vec2<f32>,
    u: f32,
    v: f32,
    lambda: f32,
    omega: f32,
    phase: f32,
    intensity: f32,
}

@group(0) @binding(0) var<uniform> params: SimulationParams;
@group(0) @binding(1) var<storage, read_write> fireflies: array<FireflyState>;

// Pseudo-random number generator
fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

fn random_normal(seed: f32, time: f32) -> f32 {
    // Box-Muller transform for normal distribution
    let u1 = hash(vec2<f32>(seed * 1.23, time));
    let u2 = hash(vec2<f32>(seed * 4.56, time * 1.1));
    return sqrt(-2.0 * log(max(u1, 0.0001))) * cos(2.0 * 3.14159 * u2);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.num_fireflies) {
        return;
    }

    var firefly = fireflies[idx];
    let dt = params.delta_time;

    // Calculate coupling from nearby fireflies
    var coupling = 0.0;
    var neighbor_count = 0.0;

    for (var j = 0u; j < params.num_fireflies; j++) {
        if (j != idx) {
            let other = fireflies[j];
            let dist = distance(firefly.position, other.position) * 1000.0; // Scale to pixels

            if (dist < params.interaction_radius) {
                // Count flashing neighbors (intensity > 0.5 means flashing)
                if (other.intensity > 0.5) {
                    coupling += 1.0;
                    neighbor_count += 1.0;
                }
            }
        }
    }

    if (neighbor_count > 0.0) {
        coupling = coupling / neighbor_count;
    }

    // Coupled oscillator dynamics
    let r_squared = firefly.u * firefly.u + firefly.v * firefly.v;
    let r = sqrt(r_squared);

    // Add noise terms
    let noise_u = random_normal(f32(idx) * 1.0, params.time);
    let noise_v = random_normal(f32(idx) * 2.0, params.time);
    let noise_lambda = random_normal(f32(idx) * 3.0, params.time);

    // Update differential equations
    let du = firefly.u * (firefly.lambda + 1.0 * r - r_squared)
           - firefly.v * (1.0 + firefly.omega)
           + params.coupling_strength * coupling
           + params.noise_strength * noise_u;

    let dv = firefly.v * (firefly.lambda + 1.0 * r - r_squared)
           + firefly.u * (1.0 + firefly.omega)
           + params.noise_strength * noise_v;

    let dlambda = params.adaptation_rate * (0.1 - r) * (1.0 + 0.1)
                + params.noise_strength * 0.1 * noise_lambda;

    // Euler integration
    firefly.u += du * dt;
    firefly.v += dv * dt;
    firefly.lambda += dlambda * dt;

    // Update phase from oscillator state
    firefly.phase = atan2(firefly.v, firefly.u);

    // Map oscillator amplitude to intensity (flashing)
    let amplitude = sqrt(firefly.u * firefly.u + firefly.v * firefly.v);
    firefly.intensity = smoothstep(0.0, 1.0, amplitude * 2.0);

    // Add some movement (optional - fireflies can drift slightly)
    let drift_x = noise_u * 0.0001;
    let drift_y = noise_v * 0.0001;
    firefly.position.x = clamp(firefly.position.x + drift_x, 0.1, 0.9);
    firefly.position.y = clamp(firefly.position.y + drift_y, 0.1, 0.9);

    // Write back updated state
    fireflies[idx] = firefly;
}
"#;

// Render shader for drawing synchronized fireflies
const RENDER_SHADER: &str = r#"
struct SimulationParams {
    time: f32,
    delta_time: f32,
    resolution: vec2<f32>,
    num_fireflies: u32,
    coupling_strength: f32,
    interaction_radius: f32,
    noise_strength: f32,
    adaptation_rate: f32,
}

struct FireflyState {
    position: vec2<f32>,
    u: f32,
    v: f32,
    lambda: f32,
    omega: f32,
    phase: f32,
    intensity: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) intensity: f32,
    @location(2) phase: f32,
    @location(3) color_seed: f32,
}

@group(0) @binding(0) var<uniform> params: SimulationParams;
@group(0) @binding(1) var<storage, read> fireflies: array<FireflyState>;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var output: VertexOutput;

    let firefly = fireflies[instance_idx];

    // Create quad vertices
    let vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );

    let vertex_pos = vertices[vertex_idx];

    // Scale based on intensity (bigger when flashing)
    let size = 0.01 * (1.0 + firefly.intensity * 2.0);

    // Position in clip space
    let world_pos = firefly.position + vertex_pos * size;
    output.clip_position = vec4<f32>(world_pos * 2.0 - 1.0, 0.0, 1.0);
    output.world_pos = vertex_pos;
    output.intensity = firefly.intensity;
    output.phase = firefly.phase;
    output.color_seed = f32(instance_idx) * 0.618; // Golden ratio for color variety

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from center for glow effect
    let dist = length(input.world_pos);

    // Multi-layer glow
    let core = exp(-dist * dist * 30.0);
    let inner_glow = exp(-dist * dist * 10.0);
    let outer_glow = exp(-dist * dist * 2.0);

    // Combine glow layers
    let glow = core + inner_glow * 0.6 + outer_glow * 0.3;

    // Color based on phase and individual variation
    var color: vec3<f32>;
    if (input.color_seed < 0.3) {
        // Golden fireflies
        color = vec3<f32>(1.0, 0.8, 0.3);
    } else if (input.color_seed < 0.6) {
        // Blue fireflies
        color = vec3<f32>(0.3, 0.6, 1.0);
    } else {
        // Cyan fireflies
        color = vec3<f32>(0.2, 0.9, 1.0);
    }

    // Modulate by intensity (synchronized flashing)
    let final_intensity = input.intensity * glow;
    let final_color = color * final_intensity * 3.0;
    let alpha = final_intensity * 0.9;

    return vec4<f32>(final_color, alpha);
}
"#;