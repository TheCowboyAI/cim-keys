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

        // Draw fewer fireflies for better performance
        render_pass.draw(0..6, 0..20); // 6 vertices per quad, 20 fireflies (reduced from 40)
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

// Fast hash using integer bit operations (no sin!)
fn hash(p: vec2<f32>) -> f32 {
    // WGSL doesn't support bitwise ops on vectors, must do scalar operations
    var hx = u32(p.x * 1597.0);
    var hy = u32(p.y * 3797.0);

    // Mix bits using XOR and multiplication (MurmurHash-inspired)
    hx = ((hx >> 16u) ^ hx) * 0x45d9f3bu;
    hx = ((hx >> 16u) ^ hx) * 0x45d9f3bu;
    hx = (hx >> 16u) ^ hx;

    hy = ((hy >> 16u) ^ hy) * 0x45d9f3bu;
    hy = ((hy >> 16u) ^ hy) * 0x45d9f3bu;
    hy = (hy >> 16u) ^ hy;

    return f32(hx ^ hy) / 4294967296.0;
}

// Vector hash for 2D output
fn hash2(p: vec2<f32>) -> vec2<f32> {
    // First hash for x component
    var h1x = u32(p.x * 1597.0);
    var h1y = u32(p.y * 3797.0);

    h1x = ((h1x >> 16u) ^ h1x) * 0x45d9f3bu;
    h1x = ((h1x >> 16u) ^ h1x) * 0x45d9f3bu;
    h1x = (h1x >> 16u) ^ h1x;

    h1y = ((h1y >> 16u) ^ h1y) * 0x45d9f3bu;
    h1y = ((h1y >> 16u) ^ h1y) * 0x45d9f3bu;
    h1y = (h1y >> 16u) ^ h1y;

    // Second hash for y component
    var h2x = u32(p.x * 2711.0);
    var h2y = u32(p.y * 1913.0);

    h2x = ((h2x >> 16u) ^ h2x) * 0x3ad8b31u;
    h2x = ((h2x >> 16u) ^ h2x) * 0x3ad8b31u;
    h2x = (h2x >> 16u) ^ h2x;

    h2y = ((h2y >> 16u) ^ h2y) * 0x3ad8b31u;
    h2y = ((h2y >> 16u) ^ h2y) * 0x3ad8b31u;
    h2y = (h2y >> 16u) ^ h2y;

    return vec2<f32>(f32(h1x ^ h1y), f32(h2x ^ h2y)) / 4294967296.0;
}

// Linear interpolation for smooth transitions
fn lerp2(a: vec2<f32>, b: vec2<f32>, t: f32) -> vec2<f32> {
    return a + (b - a) * t;
}

// 2D rotation matrix (precomputed angles)
fn rotate2D(v: vec2<f32>, angle_index: f32) -> vec2<f32> {
    // Use pre-calculated rotation for common angles to avoid sin/cos
    let idx = u32(angle_index * 7.999) & 7u;

    // WGSL requires constant array indexing, use switch instead
    var cs: vec2<f32>;
    switch idx {
        case 0u: { cs = vec2<f32>(1.0, 0.0); }      // 0°
        case 1u: { cs = vec2<f32>(0.707, 0.707); }  // 45°
        case 2u: { cs = vec2<f32>(0.0, 1.0); }      // 90°
        case 3u: { cs = vec2<f32>(-0.707, 0.707); } // 135°
        case 4u: { cs = vec2<f32>(-1.0, 0.0); }     // 180°
        case 5u: { cs = vec2<f32>(-0.707, -0.707); }// 225°
        case 6u: { cs = vec2<f32>(0.0, -1.0); }     // 270°
        default: { cs = vec2<f32>(0.707, -0.707); } // 315°
    }

    return vec2<f32>(
        v.x * cs.x - v.y * cs.y,
        v.x * cs.y + v.y * cs.x
    );
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var output: VertexOutput;

    let firefly_id = f32(instance_idx);

    // Random starting position and movement vectors
    let seed = hash2(vec2<f32>(firefly_id * 1.234, 5.678));
    let params = hash2(vec2<f32>(firefly_id * 7.89, 10.11));

    // Base position and velocity vector
    var position = seed;
    let velocity_magnitude = 0.1 + params.x * 0.15;

    // Create movement using vector operations
    let time_factor = uniforms.time * velocity_magnitude;
    let time_int = u32(time_factor);
    let time_fract = fract(time_factor);

    // Generate keyframe positions using hash (no trig!)
    let p1 = hash2(vec2<f32>(firefly_id, f32(time_int)));
    let p2 = hash2(vec2<f32>(firefly_id, f32(time_int + 1u)));
    let p3 = hash2(vec2<f32>(firefly_id, f32(time_int + 2u)));

    // Cubic Bezier interpolation for smooth curves (pure linear algebra)
    let t = time_fract;
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    // Control point for curve (creates the swooping motion)
    let control = (p1 + p2) * 0.5 + vec2<f32>(params.y - 0.5, params.x - 0.5) * 0.3;

    // Bezier curve calculation (no trig functions!)
    var center = p1 * mt3 + control * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3;

    // Add rotation using pre-computed rotation matrix
    let rotation_speed = hash(vec2<f32>(firefly_id * 3.14, 2.71));
    let angle_idx = fract(uniforms.time * rotation_speed * 0.5) * 8.0;
    let offset = rotate2D(vec2<f32>(0.1, 0.0), angle_idx);
    center += offset * params.y * 0.5;

    // Simple linear boundary reflection (no trig!)
    let edge_buffer = 0.1;
    let bounds_min = vec2<f32>(edge_buffer);
    let bounds_max = vec2<f32>(1.0 - edge_buffer);

    // Clamp and reflect using linear operations
    if (center.x < bounds_min.x) {
        center.x = bounds_min.x + (bounds_min.x - center.x) * 0.5;
    }
    if (center.x > bounds_max.x) {
        center.x = bounds_max.x - (center.x - bounds_max.x) * 0.5;
    }
    if (center.y < bounds_min.y) {
        center.y = bounds_min.y + (bounds_min.y - center.y) * 0.5;
    }
    if (center.y > bounds_max.y) {
        center.y = bounds_max.y - (center.y - bounds_max.y) * 0.5;
    }

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

    // Linear algebra approach: Use simple inverse falloff instead of exponentials
    // Much faster than exp() and gives similar visual results

    // Core brightness: Sharp falloff for center bright spot
    // Using 1/(1+x) pattern which is pure division, no transcendentals
    let core = 1.0 / (1.0 + dist * dist * 40.0);

    // Inner glow: Medium falloff
    let inner_glow = 1.0 / (1.0 + dist * dist * 10.0);

    // Outer glow: Soft falloff using linear interpolation
    // Clamp and lerp are simple min/max operations, very fast
    let outer_factor = clamp(1.0 - dist * 0.7, 0.0, 1.0);
    let outer_glow = outer_factor * outer_factor;  // Quadratic for smoothness

    // Combine layers using weighted sum (pure linear algebra)
    let glow = core * 1.2 + inner_glow * 0.4 + outer_glow * 0.15;

    // Apply intensity and color
    let final_color = input.color * input.intensity * 2.0;
    let alpha = clamp(glow * input.intensity * 0.8, 0.0, 1.0);

    return vec4<f32>(final_color * glow, alpha);
}
"#;