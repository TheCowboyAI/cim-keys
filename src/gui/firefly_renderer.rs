//! Simplified firefly renderer using pre-computed visual data
//!
//! This shader receives visual data computed by the mathematical model
//! and simply renders it without complex calculations.

use iced::{
    mouse,
    widget::shader::{self, wgpu, Viewport},
    Rectangle, Size,
};

use super::firefly_math::frp::{FireflySystem, FireflyMessage, TimeStep};

const NUM_FIREFLIES: usize = 40;

#[derive(Debug, Clone)]
pub struct FireflyRenderer {
    system: FireflySystem,
}

impl Default for FireflyRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl FireflyRenderer {
    pub fn new() -> Self {
        Self {
            system: FireflySystem::new(NUM_FIREFLIES),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.system = self.system.update(FireflyMessage::Tick(TimeStep(dt)));
    }
}

impl<Message> shader::Program<Message> for FireflyRenderer {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        let visuals = self.system.to_visual();

        // Convert visual data to GPU-friendly format
        let mut positions = Vec::with_capacity(NUM_FIREFLIES * 2);
        let mut colors = Vec::with_capacity(NUM_FIREFLIES * 4);
        let mut sizes = Vec::with_capacity(NUM_FIREFLIES);

        for firefly in &visuals {
            positions.push(firefly.position.0);
            positions.push(firefly.position.1);

            colors.push(firefly.color[0] * firefly.brightness);
            colors.push(firefly.color[1] * firefly.brightness);
            colors.push(firefly.color[2] * firefly.brightness);
            colors.push(1.0);  // Alpha

            sizes.push(firefly.size);
        }

        Primitive::new(bounds.size(), positions, colors, sizes)
    }
}

#[derive(Debug)]
pub struct Primitive {
    _size: Size,  // Reserved for viewport-dependent calculations
    positions: Vec<f32>,
    colors: Vec<f32>,
    sizes: Vec<f32>,
}

impl Primitive {
    pub fn new(size: Size, positions: Vec<f32>, colors: Vec<f32>, sizes: Vec<f32>) -> Self {
        Self { _size: size, positions, colors, sizes }
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

        // Pack data into uniform buffer
        // Using separate arrays for positions, colors, and sizes
        let uniforms = Uniforms {
            positions: array_from_vec(&self.positions),
            colors: array_from_vec(&self.colors),
            sizes: array_from_vec(&self.sizes),
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
        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
        render_pass.draw(0..6, 0..NUM_FIREFLIES as u32);
    }
}

// Helper function to convert Vec to fixed-size array
fn array_from_vec<const N: usize>(vec: &[f32]) -> [f32; N] {
    let mut array = [0.0f32; N];
    for (i, &val) in vec.iter().take(N).enumerate() {
        array[i] = val;
    }
    array
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    positions: [f32; NUM_FIREFLIES * 2],  // x, y pairs
    colors: [f32; NUM_FIREFLIES * 4],     // r, g, b, a quads
    sizes: [f32; NUM_FIREFLIES],          // size per firefly
}

struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Firefly shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER_SOURCE)),
        });

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Firefly uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Firefly bind group layout"),
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
            label: Some("Firefly bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Firefly pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Firefly pipeline"),
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

// Simplified shader that just renders pre-computed data
const SHADER_SOURCE: &str = r#"
struct Uniforms {
    // Each array is properly aligned as a whole
    positions: array<vec2<f32>, 40>,
    colors: array<vec4<f32>, 40>,
    sizes: array<f32, 40>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var output: VertexOutput;

    // Get pre-computed data for this firefly
    let position = uniforms.positions[instance_idx];
    let color = uniforms.colors[instance_idx];
    let size = uniforms.sizes[instance_idx];

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

    // Apply size and position
    output.clip_position = vec4<f32>(
        position.x + local_pos.x * size,
        position.y + local_pos.y * size,
        0.0,
        1.0
    );

    output.color = color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;