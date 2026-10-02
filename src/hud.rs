use crate::font::{get_char_uv, get_char_width, get_font};
use bytemuck::{Pod, Zeroable};

const SCALE: f32 = 2.0;
const LINE_HEIGHT: f32 = 10.0 * SCALE;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HudVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub is_text: f32,
}

fn ndc(px: f32, py: f32, sw: f32, sh: f32) -> [f32; 2] {
    [px / sw * 2.0 - 1.0, 1.0 - py / sh * 2.0]
}

pub fn build_text_mesh(
    lines: &[String],
    margin_x: f32,
    margin_y: f32,
    sw: f32,
    sh: f32,
) -> (Vec<HudVertex>, Vec<u32>) {
    let mut verts: Vec<HudVertex> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();

    if lines.is_empty() {
        return (verts, idx);
    }

    let bg = [0.0f32, 0.0, 0.0, 0.45];
    let shadow_color = [0.22f32, 0.22, 0.22, 1.0];
    let text_color = [0.88f32, 0.88, 0.88, 1.0];

    for (li, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }

        let mut line_width = 0.0f32;
        for c in line.chars() {
            line_width += get_char_width(c) * SCALE;
        }

        let x0 = margin_x;
        let y0 = margin_y + li as f32 * LINE_HEIGHT;
        let x1 = x0 + line_width + 4.0;
        let y1 = y0 + 9.0 * SCALE + 1.0;

        // Background box
        let base = verts.len() as u32;
        for &[px, py] in &[[x0, y0], [x1, y0], [x1, y1], [x0, y1]] {
            verts.push(HudVertex {
                pos: ndc(px, py, sw, sh),
                uv: [0.0, 0.0],
                color: bg,
                is_text: 0.0,
            });
        }
        idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        // Drop shadow pass
        let mut cur_x = x0 + 2.0;
        let shadow_y = y0 + 1.0 + 1.0 * SCALE;
        for c in line.chars() {
            let cw = get_char_width(c);
            if cw > 0.0 && c != ' ' {
                let (u0, v0, u1, v1) = get_char_uv(c);
                let px0 = cur_x + 1.0 * SCALE;
                let py0 = shadow_y;
                let px1 = px0 + 8.0 * SCALE;
                let py1 = py0 + 8.0 * SCALE;

                let c_base = verts.len() as u32;
                verts.push(HudVertex {
                    pos: ndc(px0, py0, sw, sh),
                    uv: [u0, v0],
                    color: shadow_color,
                    is_text: 1.0,
                });
                verts.push(HudVertex {
                    pos: ndc(px1, py0, sw, sh),
                    uv: [u1, v0],
                    color: shadow_color,
                    is_text: 1.0,
                });
                verts.push(HudVertex {
                    pos: ndc(px1, py1, sw, sh),
                    uv: [u1, v1],
                    color: shadow_color,
                    is_text: 1.0,
                });
                verts.push(HudVertex {
                    pos: ndc(px0, py1, sw, sh),
                    uv: [u0, v1],
                    color: shadow_color,
                    is_text: 1.0,
                });
                idx.extend_from_slice(&[
                    c_base,
                    c_base + 1,
                    c_base + 2,
                    c_base,
                    c_base + 2,
                    c_base + 3,
                ]);
            }
            cur_x += cw * SCALE;
        }

        // Main text pass
        cur_x = x0 + 2.0;
        let text_y = y0 + 1.0;
        for c in line.chars() {
            let cw = get_char_width(c);
            if cw > 0.0 && c != ' ' {
                let (u0, v0, u1, v1) = get_char_uv(c);
                let px0 = cur_x;
                let py0 = text_y;
                let px1 = px0 + 8.0 * SCALE;
                let py1 = py0 + 8.0 * SCALE;

                let c_base = verts.len() as u32;
                verts.push(HudVertex {
                    pos: ndc(px0, py0, sw, sh),
                    uv: [u0, v0],
                    color: text_color,
                    is_text: 1.0,
                });
                verts.push(HudVertex {
                    pos: ndc(px1, py0, sw, sh),
                    uv: [u1, v0],
                    color: text_color,
                    is_text: 1.0,
                });
                verts.push(HudVertex {
                    pos: ndc(px1, py1, sw, sh),
                    uv: [u1, v1],
                    color: text_color,
                    is_text: 1.0,
                });
                verts.push(HudVertex {
                    pos: ndc(px0, py1, sw, sh),
                    uv: [u0, v1],
                    color: text_color,
                    is_text: 1.0,
                });
                idx.extend_from_slice(&[
                    c_base,
                    c_base + 1,
                    c_base + 2,
                    c_base,
                    c_base + 2,
                    c_base + 3,
                ]);
            }
            cur_x += cw * SCALE;
        }
    }

    (verts, idx)
}

pub struct HudRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub font_bind_group: wgpu::BindGroup,
    pub vertex_buf: Option<wgpu::Buffer>,
    pub index_buf: Option<wgpu::Buffer>,
    pub index_count: u32,
}

impl HudRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let font_info = get_font();

        let font_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("HUD Font Texture"),
            size: wgpu::Extent3d {
                width: font_info.width,
                height: font_info.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &font_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &font_info.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(font_info.width * 4),
                rows_per_image: Some(font_info.height),
            },
            wgpu::Extent3d {
                width: font_info.width,
                height: font_info.height,
                depth_or_array_layers: 1,
            },
        );

        let font_view = font_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let font_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("HUD Font Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("HUD Bind Group Layout"),
            });

        let font_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&font_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&font_sampler),
                },
            ],
            label: Some("HUD Bind Group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("HUD Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
@group(0) @binding(0)
var t_font: texture_2d<f32>;
@group(0) @binding(1)
var s_font: sampler;

struct VIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) is_text: f32,
};
struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) is_text: f32,
};
@vertex
fn vs(v: VIn) -> VOut {
    var o: VOut;
    o.clip = vec4<f32>(v.pos, 0.0, 1.0);
    o.uv = v.uv;
    o.color = v.color;
    o.is_text = v.is_text;
    return o;
}
@fragment
fn fs(v: VOut) -> @location(0) vec4<f32> {
    if (v.is_text > 0.5) {
        let tex = textureSample(t_font, s_font, v.uv);
        if (tex.a < 0.1) {
            discard;
        }
        return vec4<f32>(v.color.rgb * tex.rgb, v.color.a * tex.a);
    } else {
        return v.color;
    }
}
"#
                .into(),
            ),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("HUD Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("HUD Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<HudVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: 32,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::COLOR,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            font_bind_group,
            vertex_buf: None,
            index_buf: None,
            index_count: 0,
        }
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        verts: &[HudVertex],
        indices: &[u32],
    ) {
        if verts.is_empty() {
            self.index_count = 0;
            return;
        }

        let v_bytes: &[u8] = bytemuck::cast_slice(verts);
        let i_bytes: &[u8] = bytemuck::cast_slice(indices);

        let need_new_v = match &self.vertex_buf {
            Some(b) => b.size() < v_bytes.len() as u64,
            None => true,
        };
        if need_new_v {
            let cap = (v_bytes.len() * 2).max(65536);
            self.vertex_buf = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("HUD VB"),
                size: cap as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if let Some(vb) = &self.vertex_buf {
            queue.write_buffer(vb, 0, v_bytes);
        }

        let need_new_i = match &self.index_buf {
            Some(b) => b.size() < i_bytes.len() as u64,
            None => true,
        };
        if need_new_i {
            let cap = (i_bytes.len() * 2).max(98304);
            self.index_buf = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("HUD IB"),
                size: cap as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if let Some(ib) = &self.index_buf {
            queue.write_buffer(ib, 0, i_bytes);
        }

        self.index_count = indices.len() as u32;
    }

    pub fn draw<'rp>(&'rp self, pass: &mut wgpu::RenderPass<'rp>) {
        if self.index_count == 0 {
            return;
        }
        if let (Some(vb), Some(ib)) = (&self.vertex_buf, &self.index_buf) {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.font_bind_group, &[]);
            pass.set_vertex_buffer(0, vb.slice(..));
            pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
    }
}
