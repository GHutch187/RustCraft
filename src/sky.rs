use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

const SUN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/environment/sun.png");
const MOON_PHASES_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/environment/moon_phases.png");
const CLOUDS_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/environment/clouds.png");

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SkyVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl SkyVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SkyVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

struct JavaRandom {
    seed: u64,
}

impl JavaRandom {
    fn new(seed: i64) -> Self {
        let initial_seed = (seed as u64 ^ 0x5DEECE66Du64) & ((1u64 << 48) - 1);
        Self { seed: initial_seed }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self.seed.wrapping_mul(0x5DEECE66Du64).wrapping_add(0xBu64)) & ((1u64 << 48) - 1);
        (self.seed >> (48 - bits)) as i32
    }

    fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1u64 << 24) as f32
    }

    fn next_double(&mut self) -> f64 {
        let high = (self.next(26) as i64) << 27;
        let low = self.next(27) as i64;
        (high + low) as f64 / ((1u64 << 53) as f64)
    }
}

fn generate_minecraft_stars() -> Vec<[Vec3; 4]> {
    let mut random = JavaRandom::new(10842);
    let mut star_quads: Vec<[Vec3; 4]> = Vec::with_capacity(1500);

    for _ in 0..1500 {
        let d0 = (random.next_float() * 2.0 - 1.0) as f64;
        let d1 = (random.next_float() * 2.0 - 1.0) as f64;
        let d2 = (random.next_float() * 2.0 - 1.0) as f64;
        let d3 = (0.15 + random.next_float() * 0.10) as f64;
        let d4 = d0 * d0 + d1 * d1 + d2 * d2;

        if d4 < 1.0 && d4 > 0.01 {
            let inv_d4 = 1.0 / d4.sqrt();
            let nd0 = d0 * inv_d4;
            let nd1 = d1 * inv_d4;
            let nd2 = d2 * inv_d4;
            let d5 = nd0 * 100.0;
            let d6 = nd1 * 100.0;
            let d7 = nd2 * 100.0;
            let d8 = nd0.atan2(nd2);
            let d9 = d8.sin();
            let d10 = d8.cos();
            let d11 = (nd0 * nd0 + nd2 * nd2).sqrt().atan2(nd1);
            let d12 = d11.sin();
            let d13 = d11.cos();
            let d14 = random.next_double() * std::f64::consts::PI * 2.0;
            let d15 = d14.sin();
            let d16 = d14.cos();

            let mut quad = [Vec3::ZERO; 4];
            for j in 0..4 {
                let d17 = 0.0f64;
                let d18 = (((j & 2) as f64) - 1.0) * d3;
                let d19 = ((((j + 1) & 2) as f64) - 1.0) * d3;
                let d20 = d18 * d16 - d19 * d15;
                let d21 = d19 * d16 + d18 * d15;
                let d22 = d20 * d12 + d17 * d13;
                let d23 = d17 * d12 - d20 * d13;
                let d24 = d23 * d9 - d21 * d10;
                let d25 = d21 * d9 + d23 * d10;
                quad[j] = Vec3::new(
                    (d5 + d24) as f32,
                    (d6 + d22) as f32,
                    (d7 + d25) as f32,
                );
            }
            star_quads.push(quad);
        }
    }

    star_quads
}

pub struct SkyRenderer {
    pub dome_pipeline: wgpu::RenderPipeline,
    pub additive_color_pipeline: wgpu::RenderPipeline,
    pub sun_pipeline: wgpu::RenderPipeline,
    pub moon_pipeline: wgpu::RenderPipeline,
    pub clouds_pipeline: wgpu::RenderPipeline,
    pub sun_bind_group: wgpu::BindGroup,
    pub moon_bind_group: wgpu::BindGroup,
    pub dome_vb: wgpu::Buffer,
    pub dome_ib: wgpu::Buffer,
    pub dome_index_count: u32,
    pub sun_vb: wgpu::Buffer,
    pub sun_ib: wgpu::Buffer,
    pub sun_active: bool,
    pub moon_vb: wgpu::Buffer,
    pub moon_ib: wgpu::Buffer,
    pub moon_active: bool,
    pub sunset_vb: wgpu::Buffer,
    pub sunset_ib: wgpu::Buffer,
    pub sunset_index_count: u32,
    pub stars_vb: wgpu::Buffer,
    pub stars_ib: wgpu::Buffer,
    pub stars_index_count: u32,
    pub clouds_vb: wgpu::Buffer,
    pub clouds_ib: wgpu::Buffer,
    pub clouds_index_count: u32,
    static_stars: Vec<[Vec3; 4]>,
    cloud_map: [[bool; 256]; 256],
}

impl SkyRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
        sampler: &wgpu::Sampler,
    ) -> Self {
        let sun_bytes = crate::resource_pack::get_texture("textures/environment/sun.png", SUN_BYTES);
        let sun_img = match image::load_from_memory_with_format(&sun_bytes, image::ImageFormat::Png) {
            Ok(i) => i.to_rgba8(),
            Err(_) => image::RgbaImage::new(32, 32),
        };


        let sun_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Sun Texture"),
            size: wgpu::Extent3d {
                width: sun_img.width(),
                height: sun_img.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &sun_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &sun_img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(sun_img.width() * 4),
                rows_per_image: Some(sun_img.height()),
            },
            wgpu::Extent3d {
                width: sun_img.width(),
                height: sun_img.height(),
                depth_or_array_layers: 1,
            },
        );

        let sun_view = sun_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sun_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&sun_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("Sun Bind Group"),
        });

        let moon_bytes = crate::resource_pack::get_texture("textures/environment/moon_phases.png", MOON_PHASES_BYTES);
        let moon_img = match image::load_from_memory_with_format(&moon_bytes, image::ImageFormat::Png) {
            Ok(i) => i.to_rgba8(),
            Err(_) => image::RgbaImage::new(128, 64),
        };

        let moon_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Moon Texture"),
            size: wgpu::Extent3d {
                width: moon_img.width(),
                height: moon_img.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &moon_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &moon_img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(moon_img.width() * 4),
                rows_per_image: Some(moon_img.height()),
            },
            wgpu::Extent3d {
                width: moon_img.width(),
                height: moon_img.height(),
                depth_or_array_layers: 1,
            },
        );

        let moon_view = moon_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let moon_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&moon_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("Moon Bind Group"),
        });

        let clouds_bytes = crate::resource_pack::get_texture("textures/environment/clouds.png", CLOUDS_BYTES);
        let clouds_img = match image::load_from_memory_with_format(&clouds_bytes, image::ImageFormat::Png) {
            Ok(i) => {
                let img = i.to_rgba8();
                if img.width() == 256 && img.height() == 256 {
                    img
                } else {
                    image::imageops::resize(&img, 256, 256, image::imageops::FilterType::Nearest)
                }
            }
            Err(_) => image::RgbaImage::new(256, 256),
        };

        let mut cloud_map = [[false; 256]; 256];
        for y in 0..clouds_img.height().min(256) {
            for x in 0..clouds_img.width().min(256) {
                if clouds_img.get_pixel(x, y)[3] > 100 {
                    cloud_map[x as usize][y as usize] = true;
                }
            }
        }


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sky Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sky_shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sky Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let dome_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sky Dome Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_sky"),
                buffers: &[SkyVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_sky_color"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let additive_color_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sky Additive Color Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_sky"),
                buffers: &[SkyVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_sky_color"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sun_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sun Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_sky"),
                buffers: &[SkyVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_sun"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let moon_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Moon Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_sky"),
                buffers: &[SkyVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_moon"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let clouds_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Clouds Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_clouds"),
                buffers: &[SkyVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_sky_color"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let dummy_v = SkyVertex {
            position: [0.0, 0.0, 0.0],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 0.0, 0.0],
        };

        let dome_vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dome VB"),
            size: 256 * std::mem::size_of::<SkyVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let dome_ib = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dome IB"),
            size: 1024 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sun_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sun VB"),
            contents: bytemuck::cast_slice(&[dummy_v; 4]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let sun_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sun IB"),
            contents: bytemuck::cast_slice(&[0u32, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let moon_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Moon VB"),
            contents: bytemuck::cast_slice(&[dummy_v; 4]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let moon_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Moon IB"),
            contents: bytemuck::cast_slice(&[0u32, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let sunset_vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sunset VB"),
            size: 128 * std::mem::size_of::<SkyVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sunset_ib = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sunset IB"),
            size: 512 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let stars_vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Stars VB"),
            size: 8192 * std::mem::size_of::<SkyVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let stars_ib = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Stars IB"),
            size: 16384 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let clouds_vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Clouds VB"),
            size: 32768 * std::mem::size_of::<SkyVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let clouds_ib = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Clouds IB"),
            size: 49152 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let static_stars = generate_minecraft_stars();

        Self {
            dome_pipeline,
            additive_color_pipeline,
            sun_pipeline,
            moon_pipeline,
            clouds_pipeline,
            sun_bind_group,
            moon_bind_group,
            dome_vb,
            dome_ib,
            dome_index_count: 0,
            sun_vb,
            sun_ib,
            sun_active: false,
            moon_vb,
            moon_ib,
            moon_active: false,
            sunset_vb,
            sunset_ib,
            sunset_index_count: 0,
            stars_vb,
            stars_ib,
            stars_index_count: 0,
            clouds_vb,
            clouds_ib,
            clouds_index_count: 0,
            static_stars,
            cloud_map,
        }
    }

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        eye: Vec3,
        time_of_day: i64,
        sky_color: [f32; 4],
    ) {
        let (celestial_angle, raw_sun, _sun_brightness) = crate::camera::get_sun_factors(time_of_day);
        let angle = celestial_angle * std::f32::consts::PI * 2.0;

        // 1. Build Dome Mesh
        let mut dome_verts: Vec<SkyVertex> = Vec::with_capacity(50);
        let mut dome_indices: Vec<u32> = Vec::with_capacity(200);

        // Vanilla 1.7.10 sky color blending:
        // At night, uniform (8, 10, 15). At day, transitions to saturated deep blue.
        let night_sky = [0.0314f32, 0.0392f32, 0.0588f32];
        let day_zenith = [0.541f32, 0.706f32, 0.961f32];
        let day_upper = [0.541f32, 0.706f32, 0.961f32];

        let zenith_color = [
            night_sky[0] + (day_zenith[0] - night_sky[0]) * raw_sun,
            night_sky[1] + (day_zenith[1] - night_sky[1]) * raw_sun,
            night_sky[2] + (day_zenith[2] - night_sky[2]) * raw_sun,
            1.0,
        ];
        let upper_color = [
            night_sky[0] + (day_upper[0] - night_sky[0]) * raw_sun,
            night_sky[1] + (day_upper[1] - night_sky[1]) * raw_sun,
            night_sky[2] + (day_upper[2] - night_sky[2]) * raw_sun,
            1.0,
        ];
        let horizon_color = [sky_color[0], sky_color[1], sky_color[2], 1.0];
        let void_color = [0.0, 0.0, 0.0, 1.0];

        // Zenith vertex
        dome_verts.push(SkyVertex {
            position: [eye.x, eye.y + 100.0, eye.z],
            uv: [0.0, 0.0],
            color: zenith_color,
        });

        let segments = 16;
        for i in 0..segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            // Upper ring
            dome_verts.push(SkyVertex {
                position: [eye.x + cos_t * 86.6, eye.y + 50.0, eye.z + sin_t * 86.6],
                uv: [0.0, 0.0],
                color: upper_color,
            });
        }

        for i in 0..segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            // Horizon ring
            dome_verts.push(SkyVertex {
                position: [eye.x + cos_t * 100.0, eye.y, eye.z + sin_t * 100.0],
                uv: [0.0, 0.0],
                color: horizon_color,
            });
        }

        for i in 0..segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            // Lower void ring
            dome_verts.push(SkyVertex {
                position: [eye.x + cos_t * 100.0, eye.y - 20.0, eye.z + sin_t * 100.0],
                uv: [0.0, 0.0],
                color: void_color,
            });
        }

        // Nadir vertex
        let nadir_idx = dome_verts.len() as u32;
        dome_verts.push(SkyVertex {
            position: [eye.x, eye.y - 100.0, eye.z],
            uv: [0.0, 0.0],
            color: void_color,
        });

        // Zenith cap triangles
        for i in 0..segments {
            let next = (i + 1) % segments;
            dome_indices.extend_from_slice(&[0, 1 + i, 1 + next]);
        }

        // Upper to Horizon quads
        for i in 0..segments {
            let next = (i + 1) % segments;
            let u0 = 1 + i;
            let u1 = 1 + next;
            let h0 = 1 + segments + i;
            let h1 = 1 + segments + next;
            dome_indices.extend_from_slice(&[u0, h0, h1, u0, h1, u1]);
        }

        // Horizon to Void quads
        for i in 0..segments {
            let next = (i + 1) % segments;
            let h0 = 1 + segments + i;
            let h1 = 1 + segments + next;
            let v0 = 1 + segments * 2 + i;
            let v1 = 1 + segments * 2 + next;
            dome_indices.extend_from_slice(&[h0, v0, v1, h0, v1, h1]);
        }

        // Void ring to Nadir triangles
        for i in 0..segments {
            let next = (i + 1) % segments;
            let v0 = 1 + segments * 2 + i;
            let v1 = 1 + segments * 2 + next;
            dome_indices.extend_from_slice(&[v0, nadir_idx, v1]);
        }

        queue.write_buffer(&self.dome_vb, 0, bytemuck::cast_slice(&dome_verts));
        queue.write_buffer(&self.dome_ib, 0, bytemuck::cast_slice(&dome_indices));
        self.dome_index_count = dome_indices.len() as u32;

        // Minecraft celestial rotation matrix:
        // Rotate -90 degrees around Y, then rotate by celestial angle around X.
        let rot = Mat4::from_rotation_y(-90.0f32.to_radians())
            * Mat4::from_rotation_x(angle);

        // 2. Build Sun Quad
        let sun_center_local = Vec3::new(0.0, 100.0, 0.0);
        let sun_center_world = eye + rot.transform_point3(sun_center_local);
        let sun_dir = (sun_center_world - eye).normalize();
        let sun_alpha = (sun_dir.y + 0.15).clamp(0.0, 1.0);

        if sun_alpha > 0.001 {
            let s = 30.0f32;
            let local_verts = [
                Vec3::new(-s, 100.0, -s),
                Vec3::new(s, 100.0, -s),
                Vec3::new(s, 100.0, s),
                Vec3::new(-s, 100.0, s),
            ];

            let col = [1.0, 1.0, 1.0, sun_alpha];
            let sun_quad = [
                SkyVertex {
                    position: (eye + rot.transform_point3(local_verts[0])).to_array(),
                    uv: [0.0, 0.0],
                    color: col,
                },
                SkyVertex {
                    position: (eye + rot.transform_point3(local_verts[1])).to_array(),
                    uv: [1.0, 0.0],
                    color: col,
                },
                SkyVertex {
                    position: (eye + rot.transform_point3(local_verts[2])).to_array(),
                    uv: [1.0, 1.0],
                    color: col,
                },
                SkyVertex {
                    position: (eye + rot.transform_point3(local_verts[3])).to_array(),
                    uv: [0.0, 1.0],
                    color: col,
                },
            ];
            queue.write_buffer(&self.sun_vb, 0, bytemuck::cast_slice(&sun_quad));
            self.sun_active = true;
        } else {
            self.sun_active = false;
        }

        // 3. Build Moon Quad
        let moon_center_local = Vec3::new(0.0, -100.0, 0.0);
        let moon_center_world = eye + rot.transform_point3(moon_center_local);
        let moon_dir = (moon_center_world - eye).normalize();
        let moon_alpha = (moon_dir.y + 0.15).clamp(0.0, 1.0) * (1.0 - raw_sun).clamp(0.0, 1.0);

        if moon_alpha > 0.001 {
            let s = 20.0f32;
            let phase = ((time_of_day.div_euclid(24000)) % 8) as usize;
            let col_idx = (phase % 4) as f32;
            let row_idx = (phase / 4) as f32;
            let u0 = col_idx * 0.25;
            let u1 = (col_idx + 1.0) * 0.25;
            let v0 = row_idx * 0.5;
            let v1 = (row_idx + 1.0) * 0.5;

            let col = [1.0, 1.0, 1.0, moon_alpha];
            let moon_quad = [
                SkyVertex {
                    position: (eye + rot.transform_point3(Vec3::new(-s, -100.0, s))).to_array(),
                    uv: [u1, v1],
                    color: col,
                },
                SkyVertex {
                    position: (eye + rot.transform_point3(Vec3::new(s, -100.0, s))).to_array(),
                    uv: [u0, v1],
                    color: col,
                },
                SkyVertex {
                    position: (eye + rot.transform_point3(Vec3::new(s, -100.0, -s))).to_array(),
                    uv: [u0, v0],
                    color: col,
                },
                SkyVertex {
                    position: (eye + rot.transform_point3(Vec3::new(-s, -100.0, -s))).to_array(),
                    uv: [u1, v0],
                    color: col,
                },
            ];
            queue.write_buffer(&self.moon_vb, 0, bytemuck::cast_slice(&moon_quad));
            self.moon_active = true;
        } else {
            self.moon_active = false;
        }


        // 4. Build Sunset / Sunrise Horizon Fan (Minecraft 1.7.10 canonical horizon fan)
        if let Some(sc) = crate::camera::calc_sunrise_sunset_colors(celestial_angle) {
            let r_y = glam::Mat4::from_rotation_y(-90.0f32.to_radians());
            let r_x = glam::Mat4::from_rotation_x(90.0f32.to_radians());
            let is_sunset = (celestial_angle * std::f32::consts::PI * 2.0).sin() < 0.0;
            let r_z1 = glam::Mat4::from_rotation_z(if is_sunset { 180.0f32.to_radians() } else { 0.0 });
            let r_z2 = glam::Mat4::from_rotation_z(90.0f32.to_radians());
            let mat = r_y * r_x * r_z1 * r_z2;

            let mut sf_verts = Vec::with_capacity(17);
            let mut sf_indices = Vec::with_capacity(48);

            let local_center = glam::Vec3::new(0.0, 100.0, 0.0);
            let center_world = eye + mat.transform_point3(local_center);

            sf_verts.push(SkyVertex {
                position: center_world.to_array(),
                uv: [0.0, 0.0],
                color: sc,
            });

            let outer_col = [sc[0], sc[1], sc[2], 0.0];

            for i in 0..16 {
                let theta = (i as f32 / 16.0) * std::f32::consts::PI * 2.0;
                let local_pos = glam::Vec3::new(
                    theta.sin() * 120.0,
                    theta.cos() * 120.0,
                    -theta.cos() * 40.0 * sc[3],
                );
                let world_pos = eye + mat.transform_point3(local_pos);
                sf_verts.push(SkyVertex {
                    position: world_pos.to_array(),
                    uv: [0.0, 0.0],
                    color: outer_col,
                });
            }

            for i in 0..16 {
                let next = (i + 1) % 16;
                sf_indices.extend_from_slice(&[0, 1 + i, 1 + next]);
            }

            queue.write_buffer(&self.sunset_vb, 0, bytemuck::cast_slice(&sf_verts));
            queue.write_buffer(&self.sunset_ib, 0, bytemuck::cast_slice(&sf_indices));
            self.sunset_index_count = sf_indices.len() as u32;
        } else {
            self.sunset_index_count = 0;
        }

        // 5. Build Stars (Exact Minecraft 1.7.10 catalog & transformation)
        let star_brightness = crate::camera::get_star_brightness(celestial_angle);
        if star_brightness > 0.005 {
            let mut star_verts = Vec::with_capacity(4000);
            let mut star_indices = Vec::with_capacity(6000);

            for quad in &self.static_stars {
                let center_local = (quad[0] + quad[2]) * 0.5;
                let center_world = eye + rot.transform_point3(center_local);

                if center_world.y >= eye.y - 5.0 {
                    let height_factor = ((center_world.y - eye.y) / 100.0).clamp(0.0, 1.0);
                    let alpha = star_brightness * (height_factor * 8.0).clamp(0.0, 1.0);
                    let col = [alpha, alpha, alpha, alpha];


                    let p0 = eye + rot.transform_point3(quad[0]);
                    let p1 = eye + rot.transform_point3(quad[1]);
                    let p2 = eye + rot.transform_point3(quad[2]);
                    let p3 = eye + rot.transform_point3(quad[3]);

                    let base_idx = star_verts.len() as u32;
                    star_verts.push(SkyVertex {
                        position: p0.to_array(),
                        uv: [0.0, 0.0],
                        color: col,
                    });
                    star_verts.push(SkyVertex {
                        position: p1.to_array(),
                        uv: [1.0, 0.0],
                        color: col,
                    });
                    star_verts.push(SkyVertex {
                        position: p2.to_array(),
                        uv: [1.0, 1.0],
                        color: col,
                    });
                    star_verts.push(SkyVertex {
                        position: p3.to_array(),
                        uv: [0.0, 1.0],
                        color: col,
                    });

                    star_indices.extend_from_slice(&[
                        base_idx,
                        base_idx + 1,
                        base_idx + 2,
                        base_idx,
                        base_idx + 2,
                        base_idx + 3,
                    ]);
                }
            }

            queue.write_buffer(&self.stars_vb, 0, bytemuck::cast_slice(&star_verts));
            queue.write_buffer(&self.stars_ib, 0, bytemuck::cast_slice(&star_indices));
            self.stars_index_count = star_indices.len() as u32;
        } else {
            self.stars_index_count = 0;
        }

        // 6. Build 3D Volumetric Clouds (Minecraft 1.7.10 Fancy graphics)
        let scroll_x = (time_of_day as f64) * 0.03;
        let center_cell_x = ((eye.x as f64 - scroll_x) / 12.0).floor() as i32;
        let center_cell_z = (eye.z as f64 / 12.0).floor() as i32;

        let y0 = 128.0f32;
        let y1 = 132.0f32;

        let mut cloud_r = 0.098f32 + (1.0f32 - 0.098f32) * raw_sun;
        let mut cloud_g = 0.098f32 + (1.0f32 - 0.098f32) * raw_sun;
        let mut cloud_b = 0.133f32 + (1.0f32 - 0.133f32) * raw_sun;

        if let Some(sc) = crate::camera::calc_sunrise_sunset_colors(celestial_angle) {
            let sunset_weight = sc[3] * 0.6;
            cloud_r = cloud_r * (1.0 - sunset_weight) + sc[0] * sunset_weight;
            cloud_g = cloud_g * (1.0 - sunset_weight) + sc[1] * sunset_weight;
            cloud_b = cloud_b * (1.0 - sunset_weight) + sc[2] * sunset_weight;
        }

        let base_cloud_col = [cloud_r, cloud_g, cloud_b, 0.80f32];

        let col_top = [base_cloud_col[0], base_cloud_col[1], base_cloud_col[2], base_cloud_col[3]];
        let col_bottom = [base_cloud_col[0] * 0.7, base_cloud_col[1] * 0.7, base_cloud_col[2] * 0.7, base_cloud_col[3]];
        let col_x = [base_cloud_col[0] * 0.9, base_cloud_col[1] * 0.9, base_cloud_col[2] * 0.9, base_cloud_col[3]];
        let col_z = [base_cloud_col[0] * 0.8, base_cloud_col[1] * 0.8, base_cloud_col[2] * 0.8, base_cloud_col[3]];

        let mut cloud_verts: Vec<SkyVertex> = Vec::with_capacity(16384);
        let mut cloud_indices: Vec<u32> = Vec::with_capacity(24576);

        let is_cloud = |cx: i32, cz: i32| -> bool {
            let px = cx.rem_euclid(256) as usize;
            let pz = cz.rem_euclid(256) as usize;
            self.cloud_map[px][pz]
        };

        for dx in -32..=32 {
            let cx = center_cell_x + dx;
            for dz in -32..=32 {
                let cz = center_cell_z + dz;
                if is_cloud(cx, cz) {
                    let x0 = (cx as f32) * 12.0 + (scroll_x as f32);
                    let x1 = x0 + 12.0;
                    let z0 = (cz as f32) * 12.0;
                    let z1 = z0 + 12.0;

                    if cloud_verts.len() + 24 > 32768 || cloud_indices.len() + 36 > 49152 {
                        break;
                    }

                    // Bottom face (y = y0, facing -Y):
                    let b_idx = cloud_verts.len() as u32;
                    cloud_verts.push(SkyVertex { position: [x0, y0, z0], uv: [0.0, 0.0], color: col_bottom });
                    cloud_verts.push(SkyVertex { position: [x1, y0, z0], uv: [1.0, 0.0], color: col_bottom });
                    cloud_verts.push(SkyVertex { position: [x1, y0, z1], uv: [1.0, 1.0], color: col_bottom });
                    cloud_verts.push(SkyVertex { position: [x0, y0, z1], uv: [0.0, 1.0], color: col_bottom });
                    cloud_indices.extend_from_slice(&[b_idx, b_idx + 1, b_idx + 2, b_idx, b_idx + 2, b_idx + 3]);

                    // Top face (y = y1, facing +Y):
                    let t_idx = cloud_verts.len() as u32;
                    cloud_verts.push(SkyVertex { position: [x0, y1, z1], uv: [0.0, 0.0], color: col_top });
                    cloud_verts.push(SkyVertex { position: [x1, y1, z1], uv: [1.0, 0.0], color: col_top });
                    cloud_verts.push(SkyVertex { position: [x1, y1, z0], uv: [1.0, 1.0], color: col_top });
                    cloud_verts.push(SkyVertex { position: [x0, y1, z0], uv: [0.0, 1.0], color: col_top });
                    cloud_indices.extend_from_slice(&[t_idx, t_idx + 1, t_idx + 2, t_idx, t_idx + 2, t_idx + 3]);

                    // West face (x = x0, facing -X):
                    if !is_cloud(cx - 1, cz) {
                        let w_idx = cloud_verts.len() as u32;
                        cloud_verts.push(SkyVertex { position: [x0, y0, z1], uv: [0.0, 0.0], color: col_x });
                        cloud_verts.push(SkyVertex { position: [x0, y1, z1], uv: [1.0, 0.0], color: col_x });
                        cloud_verts.push(SkyVertex { position: [x0, y1, z0], uv: [1.0, 1.0], color: col_x });
                        cloud_verts.push(SkyVertex { position: [x0, y0, z0], uv: [0.0, 1.0], color: col_x });
                        cloud_indices.extend_from_slice(&[w_idx, w_idx + 1, w_idx + 2, w_idx, w_idx + 2, w_idx + 3]);
                    }

                    // East face (x = x1, facing +X):
                    if !is_cloud(cx + 1, cz) {
                        let e_idx = cloud_verts.len() as u32;
                        cloud_verts.push(SkyVertex { position: [x1, y0, z0], uv: [0.0, 0.0], color: col_x });
                        cloud_verts.push(SkyVertex { position: [x1, y1, z0], uv: [1.0, 0.0], color: col_x });
                        cloud_verts.push(SkyVertex { position: [x1, y1, z1], uv: [1.0, 1.0], color: col_x });
                        cloud_verts.push(SkyVertex { position: [x1, y0, z1], uv: [0.0, 1.0], color: col_x });
                        cloud_indices.extend_from_slice(&[e_idx, e_idx + 1, e_idx + 2, e_idx, e_idx + 2, e_idx + 3]);
                    }

                    // North face (z = z0, facing -Z):
                    if !is_cloud(cx, cz - 1) {
                        let n_idx = cloud_verts.len() as u32;
                        cloud_verts.push(SkyVertex { position: [x1, y0, z0], uv: [0.0, 0.0], color: col_z });
                        cloud_verts.push(SkyVertex { position: [x0, y0, z0], uv: [1.0, 0.0], color: col_z });
                        cloud_verts.push(SkyVertex { position: [x0, y1, z0], uv: [1.0, 1.0], color: col_z });
                        cloud_verts.push(SkyVertex { position: [x1, y1, z0], uv: [0.0, 1.0], color: col_z });
                        cloud_indices.extend_from_slice(&[n_idx, n_idx + 1, n_idx + 2, n_idx, n_idx + 2, n_idx + 3]);
                    }

                    // South face (z = z1, facing +Z):
                    if !is_cloud(cx, cz + 1) {
                        let s_idx = cloud_verts.len() as u32;
                        cloud_verts.push(SkyVertex { position: [x0, y0, z1], uv: [0.0, 0.0], color: col_z });
                        cloud_verts.push(SkyVertex { position: [x1, y0, z1], uv: [1.0, 0.0], color: col_z });
                        cloud_verts.push(SkyVertex { position: [x1, y1, z1], uv: [1.0, 1.0], color: col_z });
                        cloud_verts.push(SkyVertex { position: [x0, y1, z1], uv: [0.0, 1.0], color: col_z });
                        cloud_indices.extend_from_slice(&[s_idx, s_idx + 1, s_idx + 2, s_idx, s_idx + 2, s_idx + 3]);
                    }
                }
            }
        }

        if !cloud_verts.is_empty() {
            queue.write_buffer(&self.clouds_vb, 0, bytemuck::cast_slice(&cloud_verts));
            queue.write_buffer(&self.clouds_ib, 0, bytemuck::cast_slice(&cloud_indices));
            self.clouds_index_count = cloud_indices.len() as u32;
        } else {
            self.clouds_index_count = 0;
        }
    }

    pub fn draw_sky<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.dome_index_count > 0 {
            render_pass.set_pipeline(&self.dome_pipeline);
            render_pass.set_bind_group(0, &self.sun_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.dome_vb.slice(..));
            render_pass.set_index_buffer(self.dome_ib.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.dome_index_count, 0, 0..1);
        }

        if self.sunset_index_count > 0 {
            render_pass.set_pipeline(&self.additive_color_pipeline);
            render_pass.set_bind_group(0, &self.sun_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sunset_vb.slice(..));
            render_pass.set_index_buffer(self.sunset_ib.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.sunset_index_count, 0, 0..1);
        }

        if self.stars_index_count > 0 {
            render_pass.set_pipeline(&self.additive_color_pipeline);
            render_pass.set_bind_group(0, &self.sun_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.stars_vb.slice(..));
            render_pass.set_index_buffer(self.stars_ib.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.stars_index_count, 0, 0..1);
        }

        if self.sun_active {
            render_pass.set_pipeline(&self.sun_pipeline);
            render_pass.set_bind_group(0, &self.sun_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sun_vb.slice(..));
            render_pass.set_index_buffer(self.sun_ib.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        if self.moon_active {
            render_pass.set_pipeline(&self.moon_pipeline);
            render_pass.set_bind_group(0, &self.moon_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.moon_vb.slice(..));
            render_pass.set_index_buffer(self.moon_ib.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }

    pub fn draw_clouds<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.clouds_index_count > 0 {
            render_pass.set_pipeline(&self.clouds_pipeline);
            render_pass.set_bind_group(0, &self.sun_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.clouds_vb.slice(..));
            render_pass.set_index_buffer(self.clouds_ib.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.clouds_index_count, 0, 0..1);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clouds_texture_validity() {
        let clouds_img = image::load_from_memory_with_format(CLOUDS_BYTES, image::ImageFormat::Png)
            .expect("Valid clouds PNG")
            .to_rgba8();
        assert_eq!(clouds_img.width(), 256);
        assert_eq!(clouds_img.height(), 256);
        let mut has_transparent = false;
        let mut has_opaque = false;
        for p in clouds_img.pixels() {
            if p[3] < 10 {
                has_transparent = true;
            } else if p[3] > 200 {
                has_opaque = true;
            }
        }
        assert!(has_transparent, "Clouds must have transparent pixels");
        assert!(has_opaque, "Clouds must have opaque pixels");
    }

    #[test]
    fn test_minecraft_stars_count() {
        assert_eq!(generate_minecraft_stars().len(), 780);
    }

    #[test]
    fn test_3d_clouds_face_count() {
        let clouds_img = image::load_from_memory_with_format(CLOUDS_BYTES, image::ImageFormat::Png).unwrap().to_rgba8();
        let is_cloud = |x: i32, z: i32| -> bool {
            let px = x.rem_euclid(256) as u32;
            let pz = z.rem_euclid(256) as u32;
            clouds_img.get_pixel(px, pz)[3] > 100
        };

        let mut quads = 0;
        for x in -32..32 {
            for z in -32..32 {
                if is_cloud(x, z) {
                    quads += 2;
                    if !is_cloud(x - 1, z) { quads += 1; }
                    if !is_cloud(x + 1, z) { quads += 1; }
                    if !is_cloud(x, z - 1) { quads += 1; }
                    if !is_cloud(x, z + 1) { quads += 1; }
                }
            }
        }
        assert!(quads > 2000 && quads < 5000);
    }
}


