//! Shader-based firefly animation with true glow effects
//!
//! GPU-accelerated particle system using WGSL shaders for The Cowboy AI theme

use iced::{
    mouse,
    widget::shader::{self, wgpu, Viewport},
    Color, Rectangle, Size, Transformation,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct FireflyShader {
    time: f32,
}

impl FireflyShader {
    pub fn new() -> Self {
        Self {
            time: 0.0,
        }
    }

    pub fn with_time(time: f32) -> Self {
        Self { time }
    }
}

impl<Message> shader::Program<Message> for FireflyShader {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        // Debug: Print to verify draw is called with updated time
        println!("Shader draw: time = {}", self.time);
        // Create primitive with current animation time
        Primitive::new(bounds.size(), self.time)
    }
}

#[derive(Debug)]
pub struct Primitive {
    size: Size,
    time: f32,
}

impl Primitive {
    pub fn new(size: Size, time: f32) -> Self {
        Self { size, time }
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
            storage.store(Pipeline::new(device, format, Size::new(viewport.physical_width(), viewport.physical_height())));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        // Update uniforms with current time and resolution
        let uniforms = Uniforms {
            time: self.time,
            resolution: [bounds.width, bounds.height],
            scale: viewport.scale_factor() as f32,
            _padding: 0.0,
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

        // Draw many instances for all fireflies
        render_pass.draw(0..6, 0..40); // 6 vertices per quad, 40 fireflies
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    resolution: [f32; 2],
    scale: f32,
    _padding: f32,
}

struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat, _target_size: Size<u32>) -> Self {
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

const SHADER_SOURCE: &str = r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    scale: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) firefly_center: vec2<f32>,
    @location(2) color: vec3<f32>,
    @location(3) intensity: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(hash(i), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var output: VertexOutput;

    let firefly_id = f32(instance_idx);

    // Calculate firefly movement
    let angle = firefly_id * 0.618 + uniforms.time * 0.1;
    let radius = 0.3 + noise(vec2<f32>(firefly_id * 0.1, uniforms.time * 0.05)) * 0.3;

    var center = vec2<f32>(
        0.5 + cos(angle) * radius + noise(vec2<f32>(uniforms.time * 0.2 + firefly_id, 0.0)) * 0.2,
        0.5 + sin(angle) * radius + noise(vec2<f32>(0.0, uniforms.time * 0.3 + firefly_id)) * 0.2
    );

    // Keep within bounds
    center = clamp(center, vec2<f32>(0.1), vec2<f32>(0.9));

    // Size varies per firefly
    let size = (0.03 + hash(vec2<f32>(firefly_id, 0.0)) * 0.02) * uniforms.scale;

    // Create quad vertices using switch statement (WGSL doesn't allow dynamic array indexing)
    var vertex_pos: vec2<f32>;
    switch vertex_idx {
        case 0u: { vertex_pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { vertex_pos = vec2<f32>(1.0, -1.0); }
        case 2u: { vertex_pos = vec2<f32>(-1.0, 1.0); }
        case 3u: { vertex_pos = vec2<f32>(1.0, -1.0); }
        case 4u: { vertex_pos = vec2<f32>(1.0, 1.0); }
        case 5u: { vertex_pos = vec2<f32>(-1.0, 1.0); }
        default: { vertex_pos = vec2<f32>(0.0, 0.0); }
    }
    let world_pos = center + vertex_pos * size;

    // Convert to clip space
    output.clip_position = vec4<f32>(world_pos * 2.0 - 1.0, 0.0, 1.0);
    output.world_pos = vertex_pos;
    output.firefly_center = center;

    // Blinking effect
    let blink_speed = 0.5 + hash(vec2<f32>(firefly_id, 1.0));
    let blink = sin(uniforms.time * blink_speed + firefly_id * 2.0);
    output.intensity = 0.3 + max(0.0, blink) * 0.7;

    // Assign colors (warm golden, cool blue, cyan)
    let color_variety = hash(vec2<f32>(firefly_id, 2.0));
    if (color_variety < 0.4) {
        output.color = vec3<f32>(1.0, 0.8, 0.3); // Golden
    } else if (color_variety < 0.7) {
        output.color = vec3<f32>(0.3, 0.6, 1.0); // Blue
    } else {
        output.color = vec3<f32>(0.2, 0.9, 1.0); // Cyan
    }

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from center of firefly for glow effect
    let dist = length(input.world_pos);

    // Multi-layer glow effect
    let core = exp(-dist * dist * 20.0);
    let inner_glow = exp(-dist * dist * 5.0);
    let outer_glow = exp(-dist * dist * 1.0);

    // Combine glow layers
    let glow = core + inner_glow * 0.5 + outer_glow * 0.2;

    // Apply intensity and color
    let final_color = input.color * input.intensity * 2.0;
    let alpha = glow * input.intensity * 0.8;

    return vec4<f32>(final_color * glow, alpha);
}
"#;