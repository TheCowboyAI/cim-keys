//! Simple Kuramoto synchronization shader - minimal implementation
//!
//! This shader implements basic Kuramoto model synchronization where fireflies
//! gradually synchronize their flashing based on neighbor interactions.

use iced::{
    mouse,
    widget::shader::{self, wgpu, Viewport},
    Rectangle, Size,
};

const NUM_FIREFLIES: u32 = 40;
const COUPLING_STRENGTH: f32 = 0.1; // Weak coupling for gradual sync

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleKuramotoShader {
    pub time: f32,
    phases: Vec<f32>,      // Current phase of each firefly
    frequencies: Vec<f32>, // Natural frequency of each firefly
}

impl SimpleKuramotoShader {
    pub fn new() -> Self {
        let mut phases = Vec::with_capacity(NUM_FIREFLIES as usize);
        let mut frequencies = Vec::with_capacity(NUM_FIREFLIES as usize);

        // Initialize with random phases and slightly different frequencies
        for i in 0..NUM_FIREFLIES {
            // Distribute phases evenly with some randomness
            phases.push((i as f32 * 0.157) % (2.0 * std::f32::consts::PI));
            // Natural frequencies around 1 Hz with small variations
            frequencies.push(1.0 + (i as f32 * 0.02) % 0.2 - 0.1);
        }

        Self {
            time: 0.0,
            phases,
            frequencies,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Simple Kuramoto update - each firefly adjusts based on all others
        let mut new_phases = self.phases.clone();

        for i in 0..NUM_FIREFLIES as usize {
            let mut phase_adjustment = 0.0;

            // Calculate influence from all other fireflies
            for j in 0..NUM_FIREFLIES as usize {
                if i != j {
                    // Kuramoto coupling: sin(phase_j - phase_i)
                    let phase_diff = self.phases[j] - self.phases[i];
                    phase_adjustment += phase_diff.sin();
                }
            }

            // Update phase with natural frequency + coupling
            new_phases[i] = self.phases[i] + dt * (
                self.frequencies[i] +
                COUPLING_STRENGTH * phase_adjustment / (NUM_FIREFLIES as f32 - 1.0)
            );

            // Keep phase in [0, 2π]
            new_phases[i] = new_phases[i] % (2.0 * std::f32::consts::PI);
        }

        self.phases = new_phases;
        self.time += dt;
    }
}

impl<Message> shader::Program<Message> for SimpleKuramotoShader {
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
        _bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, format));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        // Pack uniforms with phase data as intensities
        let mut phase_data = [0.0f32; NUM_FIREFLIES as usize];
        for (i, &phase) in self.phases.iter().enumerate() {
            // Convert phase to intensity (0 to 1)
            phase_data[i] = (phase.sin() + 1.0) * 0.5;
        }

        let uniforms = Uniforms {
            time: self.time,
            num_fireflies: NUM_FIREFLIES as f32,
            _padding: [0.0, 0.0],
            phase_data,
        };

        queue.write_buffer(&pipeline.uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));
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
            label: Some("Simple Kuramoto render pass"),
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
        render_pass.draw(0..6, 0..NUM_FIREFLIES);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    num_fireflies: f32,
    _padding: [f32; 2], // Add padding to ensure phase_data starts at 16-byte boundary
    phase_data: [f32; NUM_FIREFLIES as usize],
}

struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Simple Kuramoto shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER_SOURCE)),
        });

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kuramoto uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Kuramoto bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Kuramoto bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
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
        }
    }
}

const SHADER_SOURCE: &str = r#"
struct Uniforms {
    time: f32,
    num_fireflies: f32,
    _padding: vec2<f32>, // Padding to align phase_data to 16 bytes
    phase_data: array<f32, 40>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) intensity: f32,
    @location(1) firefly_id: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var output: VertexOutput;

    // Store firefly ID and get its intensity from phase data
    output.firefly_id = f32(instance_idx);
    output.intensity = uniforms.phase_data[instance_idx];

    // Grid layout
    let grid_width = 8u;
    let col = instance_idx % grid_width;
    let row = instance_idx / grid_width;

    // Position in normalized space
    let cell_width = 2.0 / f32(grid_width);
    let cell_height = 2.0 / 5.0;
    let center_x = -1.0 + (f32(col) + 0.5) * cell_width;
    let center_y = -1.0 + (f32(row) + 0.5) * cell_height;

    // Simple movement based on time
    let offset_x = sin(uniforms.time * 0.5 + output.firefly_id * 0.2) * 0.03;
    let offset_y = cos(uniforms.time * 0.7 + output.firefly_id * 0.15) * 0.03;

    // Quad vertices
    var local_pos: vec2<f32>;
    switch vertex_idx % 6u {
        case 0u: { local_pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { local_pos = vec2<f32>( 1.0, -1.0); }
        case 2u: { local_pos = vec2<f32>(-1.0,  1.0); }
        case 3u: { local_pos = vec2<f32>( 1.0, -1.0); }
        case 4u: { local_pos = vec2<f32>( 1.0,  1.0); }
        case 5u: { local_pos = vec2<f32>(-1.0,  1.0); }
        default: { local_pos = vec2<f32>(0.0, 0.0); }
    }

    // Size based on intensity (synchronized pulsing)
    let size = 0.04 + 0.04 * output.intensity;

    output.clip_position = vec4<f32>(
        center_x + offset_x + local_pos.x * size,
        center_y + offset_y + local_pos.y * size,
        0.0,
        1.0
    );

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Yellow-green firefly color with intensity-based brightness
    let base_color = vec3<f32>(0.8, 0.9, 0.3);
    let brightness = 0.3 + 0.7 * input.intensity;

    return vec4<f32>(base_color * brightness, 1.0);
}
"#;