//! Realistic firefly shader with synchronized flashing

use iced::{
    mouse,
    widget::shader::{self, wgpu, Viewport}, Rectangle, Size,
};

const NUM_FIREFLIES: u32 = 40;
const COUPLING_STRENGTH: f32 = 0.05; // Weak coupling for gradual sync

#[derive(Debug, Clone, PartialEq)]
pub struct DebugFireflyShader {
    pub time: f32,
    phases: Vec<f32>,      // Phase of each firefly's flash cycle
    frequencies: Vec<f32>, // Natural flash frequency
}

impl Default for DebugFireflyShader {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugFireflyShader {
    pub fn new() -> Self {
        let mut phases = Vec::with_capacity(NUM_FIREFLIES as usize);
        let mut frequencies = Vec::with_capacity(NUM_FIREFLIES as usize);

        // Initialize with random phases and slightly different frequencies
        for i in 0..NUM_FIREFLIES {
            // Random initial phases
            phases.push((i as f32 * 0.618_034) % (2.0 * std::f32::consts::PI));
            // Natural frequencies around 0.5 Hz with variations
            frequencies.push(0.5 + (i as f32 * 0.03) % 0.15 - 0.075);
        }

        Self {
            time: 0.0,
            phases,
            frequencies,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Simple Kuramoto synchronization
        let mut new_phases = self.phases.clone();

        for i in 0..NUM_FIREFLIES as usize {
            let mut coupling = 0.0;

            // Only couple with nearby fireflies (simple grid neighbors)
            let col = i % 8;
            let row = i / 8;

            for j in 0..NUM_FIREFLIES as usize {
                let j_col = j % 8;
                let j_row = j / 8;

                // Check if neighbor (Manhattan distance <= 2)
                let dist = ((col as i32 - j_col as i32).abs() +
                           (row as i32 - j_row as i32).abs()) as f32;

                if dist <= 2.0 && dist > 0.0 {
                    // Coupling strength decreases with distance
                    let strength = COUPLING_STRENGTH / dist;
                    coupling += strength * (self.phases[j] - self.phases[i]).sin();
                }
            }

            // Update phase with natural frequency + coupling
            new_phases[i] = self.phases[i] + dt * (self.frequencies[i] + coupling);

            // Wrap phase to [0, 2π]
            if new_phases[i] > 2.0 * std::f32::consts::PI {
                new_phases[i] -= 2.0 * std::f32::consts::PI;
            }
        }

        self.phases = new_phases;
        self.time += dt;
    }
}

impl<Message> shader::Program<Message> for DebugFireflyShader {
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
    _size: Size,  // Reserved for viewport-dependent calculations
    time: f32,
    phases: Vec<f32>,
}

impl Primitive {
    pub fn new(size: Size, time: f32, phases: Vec<f32>) -> Self {
        Self { _size: size, time, phases }
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

        // Convert phases to flash intensities
        let mut phase_data = [0.0f32; NUM_FIREFLIES as usize];
        for (i, &phase) in self.phases.iter().enumerate() {
            // Create discrete flashing: bright when sin(phase) > 0.8
            let sin_phase = phase.sin();
            // Sharp on/off transition with smooth edges
            phase_data[i] = if sin_phase > 0.7 {
                (sin_phase - 0.7) * 3.333  // Map [0.7, 1.0] to [0, 1]
            } else {
                0.0
            };
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
            label: Some("Debug Firefly render pass"),
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

        // Draw 40 instances of 6 vertices (quads)
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
            label: Some("Debug Firefly shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER_SOURCE)),
        });

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug bind group layout"),
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
            label: Some("Debug bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug pipeline"),
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
    @location(0) uv: vec2<f32>,
    @location(1) intensity: f32,
    @location(2) firefly_id: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var output: VertexOutput;

    // Store firefly ID and get its flash intensity
    output.firefly_id = f32(instance_idx);
    output.intensity = uniforms.phase_data[instance_idx];

    // Grid layout
    let grid_width = 8u;
    let col = instance_idx % grid_width;
    let row = instance_idx / grid_width;

    // Base position
    let cell_width = 2.0 / f32(grid_width);
    let cell_height = 2.0 / 5.0;
    let base_x = -1.0 + (f32(col) + 0.5) * cell_width;
    let base_y = -1.0 + (f32(row) + 0.5) * cell_height;

    // Organic movement - drift pattern like real fireflies
    let drift_speed = 0.3;
    let drift_x = sin(uniforms.time * drift_speed + output.firefly_id * 1.618) * 0.08;
    let drift_y = sin(uniforms.time * drift_speed * 0.7 + output.firefly_id * 2.3) * 0.06;

    // Add small random-like movements
    let wobble_x = sin(uniforms.time * 2.0 + output.firefly_id * 3.14) * 0.02;
    let wobble_y = cos(uniforms.time * 2.3 + output.firefly_id * 2.7) * 0.02;

    let center_x = base_x + drift_x + wobble_x;
    let center_y = base_y + drift_y + wobble_y;

    // Generate quad vertices
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

    // Size based on flash intensity (bigger when flashing)
    let base_size = 0.015;  // Small when not flashing
    let flash_size = 0.06;  // Bigger when flashing
    let size = mix(base_size, flash_size, output.intensity);

    output.clip_position = vec4<f32>(
        center_x + local_pos.x * size,
        center_y + local_pos.y * size,
        0.0,
        1.0
    );

    output.uv = local_pos;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from center for glow effect
    let dist = length(input.uv);

    // Soft glow falloff
    let glow = 1.0 - smoothstep(0.0, 1.0, dist);

    // Firefly yellow-green color
    let dim_color = vec3<f32>(0.1, 0.15, 0.0);  // Very dim when not flashing
    let bright_color = vec3<f32>(0.9, 1.0, 0.3); // Bright yellow-green when flashing

    // Mix colors based on intensity
    let color = mix(dim_color, bright_color, input.intensity);

    // Apply glow
    let final_color = color * glow;

    // Fade out edges
    let alpha = glow * (0.2 + 0.8 * input.intensity);

    return vec4<f32>(final_color, alpha);
}
"#;