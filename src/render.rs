use crate::camera::CameraUniform;
use crate::hud::HudRenderer;
use crate::mesh::{MeshData, Vertex};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CrosshairVertex {
    pub position: [f32; 2],
}

impl CrosshairVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CrosshairVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

pub struct ChunkGpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub transparent_vertex_buffer: Option<wgpu::Buffer>,
    pub transparent_index_buffer: Option<wgpu::Buffer>,
    pub transparent_index_count: u32,
    pub end_portal_vertex_buffer: Option<wgpu::Buffer>,
    pub end_portal_index_buffer: Option<wgpu::Buffer>,
    pub end_portal_index_count: u32,
}

pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub transparent_pipeline: wgpu::RenderPipeline,
    pub end_portal_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    pub end_portal_bind_group_layout: wgpu::BindGroupLayout,
    pub end_portal_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    pub end_sky_texture: wgpu::Texture,
    #[allow(dead_code)]
    pub end_portal_texture: wgpu::Texture,
    #[allow(dead_code)]
    pub end_portal_sampler: wgpu::Sampler,
    pub crosshair_pipeline: wgpu::RenderPipeline,
    pub crosshair_buffer: wgpu::Buffer,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    pub atlas_texture: wgpu::Texture,
    #[allow(dead_code)]
    pub atlas_view: wgpu::TextureView,
    #[allow(dead_code)]
    pub atlas_sampler: wgpu::Sampler,
    pub depth_texture_view: wgpu::TextureView,
    pub chunk_meshes: HashMap<(i32, i32), ChunkGpuMesh>,
    pub player_meshes: HashMap<String, ChunkGpuMesh>,
    pub entity_mesh: Option<ChunkGpuMesh>,
    pub skin_manager: crate::skin::SkinManager,
    pub hud: HudRenderer,
    pub sky: crate::sky::SkyRenderer,
    pub start_time: std::time::Instant,
    pub anim_timer: std::time::Instant,
    pub anim_frame: usize,
    pub current_sky_color: [f32; 4],
}

fn create_crosshair_vertices(aspect: f32) -> Vec<CrosshairVertex> {
    let len = 0.018;
    let thick = 0.0025;
    let len_x = len / aspect;
    let thick_x = thick / aspect;

    vec![
        CrosshairVertex {
            position: [-len_x, -thick],
        },
        CrosshairVertex {
            position: [len_x, -thick],
        },
        CrosshairVertex {
            position: [len_x, thick],
        },
        CrosshairVertex {
            position: [-len_x, -thick],
        },
        CrosshairVertex {
            position: [len_x, thick],
        },
        CrosshairVertex {
            position: [-len_x, thick],
        },
        CrosshairVertex {
            position: [-thick_x, -len],
        },
        CrosshairVertex {
            position: [thick_x, -len],
        },
        CrosshairVertex {
            position: [thick_x, len],
        },
        CrosshairVertex {
            position: [-thick_x, -len],
        },
        CrosshairVertex {
            position: [thick_x, len],
        },
        CrosshairVertex {
            position: [-thick_x, len],
        },
    ]
}

impl RenderState {
    pub async fn new(window: Arc<Window>, pending_skins: Arc<std::sync::Mutex<Vec<(String, Vec<u8>)>>>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = if surface_caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else if surface_caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        let alpha_mode = if surface_caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            surface_caps.alpha_modes[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        let depth_texture_view = Self::create_depth_texture(&device, &config);

        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let atlas_rgba = crate::texture::build_block_atlas();
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Block Texture Atlas"),
            size: wgpu::Extent3d {
                width: crate::texture::ATLAS_WIDTH,
                height: crate::texture::ATLAS_HEIGHT,
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
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(crate::texture::ATLAS_WIDTH * 4),
                rows_per_image: Some(crate::texture::ATLAS_HEIGHT),
            },
            wgpu::Extent3d {
                width: crate::texture::ATLAS_WIDTH,
                height: crate::texture::ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
        );

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Block Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("camera_and_texture_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
            label: Some("camera_and_texture_bind_group"),
        });

        let mut skin_manager = crate::skin::SkinManager::new(pending_skins);
        skin_manager.init_default(
            &device,
            &queue,
            &camera_bind_group_layout,
            &camera_buffer,
            &atlas_sampler,
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let transparent_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Transparent Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::COLOR,
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
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::COLOR,
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
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let crosshair_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Crosshair Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.85);
}
"#
                .into(),
            ),
        });

        let crosshair_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Crosshair Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let crosshair_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Crosshair Pipeline"),
                layout: Some(&crosshair_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &crosshair_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[CrosshairVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &crosshair_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::OneMinusDst,
                                dst_factor: wgpu::BlendFactor::Zero,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::COLOR,
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
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let aspect = size.width as f32 / size.height.max(1) as f32;
        let crosshair_verts = create_crosshair_vertices(aspect);
        let crosshair_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crosshair Vertex Buffer"),
            contents: bytemuck::cast_slice(&crosshair_verts),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let hud = HudRenderer::new(&device, &queue, surface_format, wgpu::TextureFormat::Depth32Float);
        let sky = crate::sky::SkyRenderer::new(
            &device,
            &queue,
            &config,
            &camera_bind_group_layout,
            &camera_buffer,
            &atlas_sampler,
        );

        let end_sky_bytes = crate::resource_pack::get_texture(
            "textures/environment/end_sky.png",
            crate::texture::END_SKY_BYTES,
        );
        let end_sky_img = image::load_from_memory(&end_sky_bytes)
            .map(|img| img.to_rgba8())
            .unwrap_or_else(|_| image::RgbaImage::new(128, 128));
        let (end_sky_texture, end_sky_view) =
            Self::create_rgba_texture(&device, &queue, &end_sky_img, "End Sky Texture");

        let end_portal_bytes = crate::resource_pack::get_texture(
            "textures/entity/end_portal.png",
            crate::texture::END_PORTAL_BYTES,
        );
        let end_portal_img = image::load_from_memory(&end_portal_bytes)
            .map(|img| img.to_rgba8())
            .unwrap_or_else(|_| image::RgbaImage::new(256, 256));
        let (end_portal_texture, end_portal_view) =
            Self::create_rgba_texture(&device, &queue, &end_portal_img, "End Portal Texture");

        let end_portal_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("End Portal Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let end_portal_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("End Portal Bind Group Layout"),
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
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let end_portal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("End Portal Bind Group"),
            layout: &end_portal_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&end_sky_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&end_portal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&end_portal_sampler),
                },
            ],
        });

        let end_portal_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("End Portal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("end_portal_shader.wgsl").into()),
        });

        let end_portal_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("End Portal Pipeline Layout"),
                bind_group_layouts: &[&end_portal_bind_group_layout],
                push_constant_ranges: &[],
            });

        let end_portal_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("End Portal Pipeline"),
            layout: Some(&end_portal_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &end_portal_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &end_portal_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::COLOR,
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
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            transparent_pipeline,
            end_portal_pipeline,
            end_portal_bind_group_layout,
            end_portal_bind_group,
            end_sky_texture,
            end_portal_texture,
            end_portal_sampler,
            crosshair_pipeline,
            crosshair_buffer,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            atlas_texture,
            atlas_view,
            atlas_sampler,
            depth_texture_view,
            chunk_meshes: HashMap::new(),
            player_meshes: HashMap::new(),
            entity_mesh: None,
            skin_manager,
            hud,
            sky,
            start_time: std::time::Instant::now(),
            anim_timer: std::time::Instant::now(),
            anim_frame: 0,
            current_sky_color: [0.53, 0.81, 0.92, 1.0],
        }
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_rgba_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::RgbaImage,
        label: &str,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: img.width(),
                height: img.height(),
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
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(img.width() * 4),
                rows_per_image: Some(img.height()),
            },
            wgpu::Extent3d {
                width: img.width(),
                height: img.height(),
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture_view = Self::create_depth_texture(&self.device, &self.config);

            let aspect = new_size.width as f32 / new_size.height as f32;
            let crosshair_verts = create_crosshair_vertices(aspect);
            self.queue.write_buffer(
                &self.crosshair_buffer,
                0,
                bytemuck::cast_slice(&crosshair_verts),
            );
        }
    }

    pub fn upload_chunk_mesh(&mut self, cx: i32, cz: i32, mesh: &MeshData) {
        if mesh.is_empty() {
            self.chunk_meshes.remove(&(cx, cz));
            return;
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let transparent_vertex_buffer = if !mesh.transparent_indices.is_empty() {
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Transparent Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.transparent_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }))
        } else {
            None
        };

        let transparent_index_buffer = if !mesh.transparent_indices.is_empty() {
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Transparent Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.transparent_indices),
                usage: wgpu::BufferUsages::INDEX,
            }))
        } else {
            None
        };

        let (end_portal_vertex_buffer, end_portal_index_buffer) = if !mesh.end_portal_indices.is_empty() {
            (
                Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Chunk End Portal Vertex Buffer"),
                    contents: bytemuck::cast_slice(&mesh.end_portal_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })),
                Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Chunk End Portal Index Buffer"),
                    contents: bytemuck::cast_slice(&mesh.end_portal_indices),
                    usage: wgpu::BufferUsages::INDEX,
                })),
            )
        } else {
            (None, None)
        };

        self.chunk_meshes.insert(
            (cx, cz),
            ChunkGpuMesh {
                vertex_buffer,
                index_buffer,
                index_count: mesh.indices.len() as u32,
                transparent_vertex_buffer,
                transparent_index_buffer,
                transparent_index_count: mesh.transparent_indices.len() as u32,
                end_portal_vertex_buffer,
                end_portal_index_buffer,
                end_portal_index_count: mesh.end_portal_indices.len() as u32,
            },
        );
    }

    #[allow(dead_code)]
    pub fn upload_entities_mesh(&mut self, mesh: &MeshData) {
        if mesh.is_empty() {
            self.entity_mesh = None;
            return;
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Entities Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Entities Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.entity_mesh = Some(ChunkGpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
            transparent_vertex_buffer: None,
            transparent_index_buffer: None,
            transparent_index_count: 0,
            end_portal_vertex_buffer: None,
            end_portal_index_buffer: None,
            end_portal_index_count: 0,
        });
    }

    pub fn upload_player_mesh(&mut self, player_key: String, mesh: &MeshData) {
        if mesh.is_empty() {
            self.player_meshes.remove(&player_key);
            return;
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Player Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Player Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.player_meshes.insert(
            player_key,
            ChunkGpuMesh {
                vertex_buffer,
                index_buffer,
                index_count: mesh.indices.len() as u32,
                transparent_vertex_buffer: None,
                transparent_index_buffer: None,
                transparent_index_count: 0,
                end_portal_vertex_buffer: None,
                end_portal_index_buffer: None,
                end_portal_index_count: 0,
            },
        );
    }

    pub fn prune_player_meshes(&mut self, active_keys: &std::collections::HashSet<String>) {
        self.player_meshes.retain(|k, _| active_keys.contains(k));
    }

    pub fn prune_distant_chunks(&mut self, player_cx: i32, player_cz: i32, max_radius: i32) {
        self.chunk_meshes.retain(|&(cx, cz), _| {
            let dx = (cx - player_cx).abs();
            let dz = (cz - player_cz).abs();
            dx <= max_radius && dz <= max_radius
        });
    }

    pub fn update_camera(&mut self, eye: glam::Vec3, yaw: f32, pitch: f32, time_of_day: i64) {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let elapsed_seconds = self.start_time.elapsed().as_secs_f32();
        self.camera_uniform.update_view_proj(eye, yaw, pitch, aspect, time_of_day, elapsed_seconds);
        self.current_sky_color = self.camera_uniform.sky_color;

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.sky.update(&self.queue, eye, time_of_day, self.current_sky_color);
    }

    pub fn render(&mut self, show_crosshair: bool) -> Result<(), wgpu::SurfaceError> {
        if self.anim_timer.elapsed().as_millis() >= 60 {
            self.anim_timer = std::time::Instant::now();
            self.anim_frame = (self.anim_frame + 1) % 32;

            let still_rgba = crate::texture::get_water_frame_rgba(self.anim_frame, false);
            let col29 = 29 % crate::texture::TILES_PER_ROW;
            let row29 = 29 / crate::texture::TILES_PER_ROW;
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: col29 * crate::texture::TILE_SIZE,
                        y: row29 * crate::texture::TILE_SIZE,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &still_rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(crate::texture::TILE_SIZE * 4),
                    rows_per_image: Some(crate::texture::TILE_SIZE),
                },
                wgpu::Extent3d {
                    width: crate::texture::TILE_SIZE,
                    height: crate::texture::TILE_SIZE,
                    depth_or_array_layers: 1,
                },
            );

            let flow_rgba = crate::texture::get_water_frame_rgba(self.anim_frame, true);
            let col82 = 82 % crate::texture::TILES_PER_ROW;
            let row82 = 82 / crate::texture::TILES_PER_ROW;
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: col82 * crate::texture::TILE_SIZE,
                        y: row82 * crate::texture::TILE_SIZE,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &flow_rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(crate::texture::TILE_SIZE * 4),
                    rows_per_image: Some(crate::texture::TILE_SIZE),
                },
                wgpu::Extent3d {
                    width: crate::texture::TILE_SIZE,
                    height: crate::texture::TILE_SIZE,
                    depth_or_array_layers: 1,
                },
            );

            let portal_rgba = crate::texture::get_portal_frame_rgba(self.anim_frame);
            let col220 = 220 % crate::texture::TILES_PER_ROW;
            let row220 = 220 / crate::texture::TILES_PER_ROW;
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: col220 * crate::texture::TILE_SIZE,
                        y: row220 * crate::texture::TILE_SIZE,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &portal_rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(crate::texture::TILE_SIZE * 4),
                    rows_per_image: Some(crate::texture::TILE_SIZE),
                },
                wgpu::Extent3d {
                    width: crate::texture::TILE_SIZE,
                    height: crate::texture::TILE_SIZE,
                    depth_or_array_layers: 1,
                },
            );

            let fire_rgba = crate::texture::get_fire_frame_rgba(self.anim_frame);
            let col222 = 222 % crate::texture::TILES_PER_ROW;
            let row222 = 222 / crate::texture::TILES_PER_ROW;
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: col222 * crate::texture::TILE_SIZE,
                        y: row222 * crate::texture::TILE_SIZE,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &fire_rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(crate::texture::TILE_SIZE * 4),
                    rows_per_image: Some(crate::texture::TILE_SIZE),
                },
                wgpu::Extent3d {
                    width: crate::texture::TILE_SIZE,
                    height: crate::texture::TILE_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.skin_manager.process_pending(
            &self.device,
            &self.queue,
            &self.camera_bind_group_layout,
            &self.camera_buffer,
            &self.atlas_sampler,
        );

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.current_sky_color[0] as f64,
                            g: self.current_sky_color[1] as f64,
                            b: self.current_sky_color[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.sky.draw_sky(&mut render_pass);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for mesh in self.chunk_meshes.values() {
                if mesh.index_count > 0 {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                }
            }

            for (player_key, mesh) in &self.player_meshes {
                if mesh.index_count > 0 {
                    if let Some(skin_bg) = self.skin_manager.get_bind_group(player_key) {
                        render_pass.set_bind_group(0, skin_bg, &[]);
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    }
                }
            }

            if let Some(ref emesh) = self.entity_mesh {
                if emesh.index_count > 0 {
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, emesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(emesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..emesh.index_count, 0, 0..1);
                }
            }

            render_pass.set_pipeline(&self.end_portal_pipeline);
            render_pass.set_bind_group(0, &self.end_portal_bind_group, &[]);
            for mesh in self.chunk_meshes.values() {
                if mesh.end_portal_index_count > 0 {
                    if let (Some(vb), Some(ib)) = (&mesh.end_portal_vertex_buffer, &mesh.end_portal_index_buffer) {
                        render_pass.set_vertex_buffer(0, vb.slice(..));
                        render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..mesh.end_portal_index_count, 0, 0..1);
                    }
                }
            }

            self.sky.draw_clouds(&mut render_pass);

            render_pass.set_pipeline(&self.transparent_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            let cam_x = self.camera_uniform.eye_pos[0];
            let cam_z = self.camera_uniform.eye_pos[2];

            let mut sorted_transparent: Vec<(&(i32, i32), &ChunkGpuMesh)> = self
                .chunk_meshes
                .iter()
                .filter(|(_, mesh)| mesh.transparent_index_count > 0)
                .collect();

            sorted_transparent.sort_by(|(pos_a, _), (pos_b, _)| {
                let center_a_x = (pos_a.0 * 16 + 8) as f32;
                let center_a_z = (pos_a.1 * 16 + 8) as f32;
                let dist_a_sq = (center_a_x - cam_x).powi(2) + (center_a_z - cam_z).powi(2);

                let center_b_x = (pos_b.0 * 16 + 8) as f32;
                let center_b_z = (pos_b.1 * 16 + 8) as f32;
                let dist_b_sq = (center_b_x - cam_x).powi(2) + (center_b_z - cam_z).powi(2);

                dist_b_sq.partial_cmp(&dist_a_sq).unwrap_or(std::cmp::Ordering::Equal)
            });

            for (_, mesh) in sorted_transparent {
                if let (Some(vb), Some(ib)) = (&mesh.transparent_vertex_buffer, &mesh.transparent_index_buffer) {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.transparent_index_count, 0, 0..1);
                }
            }



            if show_crosshair {
                render_pass.set_pipeline(&self.crosshair_pipeline);
                render_pass.set_vertex_buffer(0, self.crosshair_buffer.slice(..));
                render_pass.draw(0..12, 0..1);
            }

            self.hud.draw(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update_hud(&mut self, lines: &[String]) {
        let sw = self.config.width as f32;
        let sh = self.config.height as f32;
        let (verts, indices) = crate::hud::build_text_mesh(lines, 4.0, 4.0, sw, sh);
        self.hud.upload(&self.device, &self.queue, &verts, &indices);
    }
}
