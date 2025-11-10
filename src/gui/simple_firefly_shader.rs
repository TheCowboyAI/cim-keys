//! Simple firefly shader - just points with Kuramoto synchronization

use iced::{
    widget::shader::{self, wgpu},
    Rectangle, Size,
};
use std::sync::Arc;

const NUM_FIREFLIES: u32 = 40;

#[derive(Debug, Clone)]
pub struct SimpleFireflyShader {
    pub time: f32,
}

impl SimpleFireflyShader {
    pub fn new() -> Self {
        Self { time: 0.0 }
    }
}

impl<Message> shader::Program<Message> for SimpleFireflyShader {
    type State = ();
    type Primitive = shader::Primitive;

    fn prepare(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &Rectangle,
        _target_size: &Size<u32>,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, wgpu::TextureFormat::Bgra8UnormSrgb));
        }
    }

    fn draw(
        &self,
        storage: &mut shader::Storage,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        _clip_bounds: &Rectangle,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();

        // Note: We can't update uniforms here without queue access
        // This would need to be done elsewhere

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Firefly pass"),
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

        // Draw points - one per firefly
        render_pass.draw(0..NUM_FIREFLIES, 0..1);
    }
}

struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Simple firefly shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER)),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Time uniform"),
            size: 4, // Just time as f32
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
                resource: uniform_buffer.as_entire_binding(),
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
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList, // Just points!
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
                module: &shader,
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
            uniform_buffer,
        }
    }
}

const SHADER: &str = r#"
@group(0) @binding(0) var<uniform> time: f32;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) intensity: f32,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    var output: VertexOutput;

    let firefly_id = f32(vertex_idx);

    // Simple grid position
    let x = (firefly_id % 8.0) / 8.0 - 0.4;
    let y = floor(firefly_id / 8.0) / 5.0 - 0.3;

    // Simple oscillation for movement
    let offset_x = sin(time + firefly_id * 0.5) * 0.05;
    let offset_y = cos(time * 0.7 + firefly_id * 0.3) * 0.05;

    output.position = vec4<f32>(x + offset_x, y + offset_y, 0.0, 1.0);

    // Simple Kuramoto-like synchronization
    let phase = time + firefly_id * 0.2;
    output.intensity = 0.5 + 0.5 * sin(phase);

    // Color based on ID
    output.color = vec3<f32>(
        0.8 + 0.2 * sin(firefly_id * 0.1),
        0.9,
        0.3
    );

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple glowing point
    return vec4<f32>(input.color * input.intensity, 1.0);
}
"#;