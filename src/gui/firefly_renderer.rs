//! Simplified firefly renderer using pre-computed visual data
//!
//! This shader receives visual data computed by the mathematical model
//! and simply renders it without complex calculations.

use iced::{
    mouse,
    widget::shader::{self, wgpu, Viewport},
    Rectangle, Size,
};
use wgpu::util::DeviceExt;

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
        let system = std::mem::replace(&mut self.system, FireflySystem::new(NUM_FIREFLIES));
        self.system = system.update(FireflyMessage::Tick(TimeStep(dt)));
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
        let connections = self.system.get_connections();

        // Convert visual data to GPU-friendly format
        let mut positions = Vec::with_capacity(NUM_FIREFLIES * 2);
        let mut colors = Vec::with_capacity(NUM_FIREFLIES * 4);
        let mut sizes = Vec::with_capacity(NUM_FIREFLIES);

        // Convert from pixel coordinates to NDC [-1, 1]
        let width = bounds.width;
        let height = bounds.height;

        for firefly in &visuals {
            // Convert pixel coords to NDC
            let ndc_x = (firefly.position.0 / width) * 2.0 - 1.0;
            let ndc_y = 1.0 - (firefly.position.1 / height) * 2.0;  // Flip Y

            positions.push(ndc_x);
            positions.push(ndc_y);

            colors.push(firefly.color[0]);
            colors.push(firefly.color[1]);
            colors.push(firefly.color[2]);
            colors.push(firefly.brightness);  // Alpha varies with brightness

            // Convert size from pixels to NDC
            let ndc_size = (firefly.size / width.min(height)) * 2.0;
            sizes.push(ndc_size);
        }

        // Convert connection lines to NDC
        let mut line_data = Vec::new();
        for conn in &connections {
            let from_x = (conn.from.0 / width) * 2.0 - 1.0;
            let from_y = 1.0 - (conn.from.1 / height) * 2.0;
            let to_x = (conn.to.0 / width) * 2.0 - 1.0;
            let to_y = 1.0 - (conn.to.1 / height) * 2.0;

            line_data.push((from_x, from_y, to_x, to_y, conn.fade));
        }

        Primitive::new(bounds.size(), positions, colors, sizes, line_data)
    }
}

#[derive(Debug)]
pub struct Primitive {
    _size: Size,  // Reserved for viewport-dependent calculations
    positions: Vec<f32>,
    colors: Vec<f32>,
    sizes: Vec<f32>,
    line_data: Vec<(f32, f32, f32, f32, f32)>,  // (from_x, from_y, to_x, to_y, fade)
}

impl Primitive {
    pub fn new(
        size: Size,
        positions: Vec<f32>,
        colors: Vec<f32>,
        sizes: Vec<f32>,
        line_data: Vec<(f32, f32, f32, f32, f32)>,
    ) -> Self {
        Self { _size: size, positions, colors, sizes, line_data }
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

        // Pack data into uniform buffer with proper 16-byte alignment per element
        let mut positions = [[0.0f32; 4]; NUM_FIREFLIES];
        let mut colors = [[0.0f32; 4]; NUM_FIREFLIES];
        let mut sizes = [[0.0f32; 4]; NUM_FIREFLIES];

        for i in 0..NUM_FIREFLIES.min(self.positions.len() / 2) {
            positions[i] = [
                self.positions[i * 2],      // x
                self.positions[i * 2 + 1],  // y
                0.0,                        // padding
                0.0,                        // padding
            ];
        }

        for i in 0..NUM_FIREFLIES.min(self.colors.len() / 4) {
            colors[i] = [
                self.colors[i * 4],
                self.colors[i * 4 + 1],
                self.colors[i * 4 + 2],
                self.colors[i * 4 + 3],
            ];
        }

        for i in 0..NUM_FIREFLIES.min(self.sizes.len()) {
            sizes[i] = [
                self.sizes[i],  // size
                0.0,            // padding
                0.0,            // padding
                0.0,            // padding
            ];
        }

        let uniforms = Uniforms {
            positions,
            colors,
            sizes,
        };

        queue.write_buffer(&pipeline.uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Build line geometry (two triangles per line, making a thin quad)
        if !self.line_data.is_empty() {
            let mut line_vertices: Vec<f32> = Vec::new();

            for (from_x, from_y, to_x, to_y, fade) in &self.line_data {
                // Calculate perpendicular direction for line width
                let dx = to_x - from_x;
                let dy = to_y - from_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len < 0.0001 {
                    continue;
                }

                // Perpendicular vector (normalized)
                let perp_x = -dy / len;
                let perp_y = dx / len;

                // Line width in NDC (very thin for subtle effect)
                let width = 0.001 * fade;  // Scale by fade

                // Yellow-green color (220, 255, 80) with transparency based on fade
                // Two layers: glow and core
                let glow_alpha = 8.0 / 255.0 * fade;
                let core_alpha = 20.0 / 255.0 * fade;

                // Build quad for glow layer (thicker, more transparent)
                let glow_width = width * 1.5;
                let v1 = [
                    from_x + perp_x * glow_width,
                    from_y + perp_y * glow_width,
                    220.0 / 255.0,
                    1.0,
                    80.0 / 255.0,
                    glow_alpha,
                ];
                let v2 = [
                    from_x - perp_x * glow_width,
                    from_y - perp_y * glow_width,
                    220.0 / 255.0,
                    1.0,
                    80.0 / 255.0,
                    glow_alpha,
                ];
                let v3 = [
                    to_x + perp_x * glow_width,
                    to_y + perp_y * glow_width,
                    220.0 / 255.0,
                    1.0,
                    80.0 / 255.0,
                    glow_alpha,
                ];
                let v4 = [
                    to_x - perp_x * glow_width,
                    to_y - perp_y * glow_width,
                    220.0 / 255.0,
                    1.0,
                    80.0 / 255.0,
                    glow_alpha,
                ];

                // Two triangles for quad: v1-v2-v3, v2-v4-v3
                line_vertices.extend_from_slice(&v1);
                line_vertices.extend_from_slice(&v2);
                line_vertices.extend_from_slice(&v3);
                line_vertices.extend_from_slice(&v2);
                line_vertices.extend_from_slice(&v4);
                line_vertices.extend_from_slice(&v3);

                // Build quad for core layer (thinner, brighter)
                let core_width = width * 0.5;
                let c1 = [
                    from_x + perp_x * core_width,
                    from_y + perp_y * core_width,
                    240.0 / 255.0,
                    1.0,
                    120.0 / 255.0,
                    core_alpha,
                ];
                let c2 = [
                    from_x - perp_x * core_width,
                    from_y - perp_y * core_width,
                    240.0 / 255.0,
                    1.0,
                    120.0 / 255.0,
                    core_alpha,
                ];
                let c3 = [
                    to_x + perp_x * core_width,
                    to_y + perp_y * core_width,
                    240.0 / 255.0,
                    1.0,
                    120.0 / 255.0,
                    core_alpha,
                ];
                let c4 = [
                    to_x - perp_x * core_width,
                    to_y - perp_y * core_width,
                    240.0 / 255.0,
                    1.0,
                    120.0 / 255.0,
                    core_alpha,
                ];

                line_vertices.extend_from_slice(&c1);
                line_vertices.extend_from_slice(&c2);
                line_vertices.extend_from_slice(&c3);
                line_vertices.extend_from_slice(&c2);
                line_vertices.extend_from_slice(&c4);
                line_vertices.extend_from_slice(&c3);
            }

            // Create or update line buffer
            if !line_vertices.is_empty() {
                let line_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Line vertex buffer"),
                    contents: bytemuck::cast_slice(&line_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                pipeline.line_buffer = Some(line_buffer);
            }
        }
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

        // Draw connection lines first (behind fireflies)
        if !self.line_data.is_empty() && pipeline.line_buffer.is_some() {
            render_pass.set_pipeline(&pipeline.line_pipeline);
            render_pass.set_vertex_buffer(0, pipeline.line_buffer.as_ref().unwrap().slice(..));
            render_pass.draw(0..(self.line_data.len() * 6) as u32, 0..1);  // 6 vertices per line (2 triangles)
        }

        // Draw fireflies on top
        render_pass.set_pipeline(&pipeline.render_pipeline);
        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
        render_pass.draw(0..6, 0..NUM_FIREFLIES as u32);
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    // Each array element aligned to 16 bytes with @align(16) in WGSL
    positions: [[f32; 4]; NUM_FIREFLIES],  // vec2 padded to 16 bytes [x, y, pad, pad]
    colors: [[f32; 4]; NUM_FIREFLIES],     // vec4 naturally 16 bytes [r, g, b, a]
    sizes: [[f32; 4]; NUM_FIREFLIES],      // f32 padded to 16 bytes [size, pad, pad, pad]
}

struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
    line_buffer: Option<wgpu::Buffer>,
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

        // Create line shader and pipeline
        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(LINE_SHADER_SOURCE)),
        });

        let line_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line pipeline"),
            layout: Some(&line_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &line_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 6 * std::mem::size_of::<f32>() as u64,  // position (vec2) + color (vec4)
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 2 * std::mem::size_of::<f32>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
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
                module: &line_shader,
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
            line_pipeline,
            bind_group,
            uniforms_buffer,
            line_buffer: None,
        }
    }
}

// Simplified shader that just renders pre-computed data
const SHADER_SOURCE: &str = r#"
struct Uniforms {
    // Use vec4 for everything to ensure natural 16-byte alignment
    positions: array<vec4<f32>, 40>,  // [x, y, 0, 0]
    colors: array<vec4<f32>, 40>,     // [r, g, b, a]
    sizes: array<vec4<f32>, 40>,      // [size, 0, 0, 0]
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,  // Position within quad for radial gradient
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
    let position = uniforms.positions[instance_idx].xy;  // Extract vec2 from vec4
    let color = uniforms.colors[instance_idx];
    let size = uniforms.sizes[instance_idx].x;           // Extract f32 from vec4

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
    output.local_pos = local_pos;  // Pass to fragment shader for radial gradient
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Create radial gradient from center (0,0) to edge (1,1)
    let dist = length(input.local_pos);

    // Smooth fade from center to edge matching www-egui:
    // Center alpha ~100/255 = 0.39, outer alpha ~5/255 = 0.02
    let center_alpha = 0.39;
    let outer_alpha = 0.02;

    // Smooth gradient with exponential falloff for ethereal glow
    let alpha_factor = smoothstep(1.0, 0.0, dist);
    let alpha = mix(outer_alpha, center_alpha, alpha_factor * alpha_factor);

    // Discard pixels beyond radius for soft edges
    if dist > 1.0 {
        discard;
    }

    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
"#;

// Simple line shader for connection lines
const LINE_SHADER_SOURCE: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;