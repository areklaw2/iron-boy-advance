use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::gpu::GpuContext;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    scale: [f32; 2],
    offset: [f32; 2],
}

pub struct Renderer {
    _window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,
    viewport_width: usize,
    viewport_height: usize,

    frame_texture: wgpu::Texture,
    _frame_view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(gpu: &GpuContext, window: Arc<Window>, viewport_width: usize, viewport_height: usize) -> Self {
        let size = window.inner_size();

        let surface = gpu
            .instance()
            .create_surface(window.clone())
            .expect("failed to create wgpu surface");

        let caps = surface.get_capabilities(gpu.adapter());
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Bgra8UnormSrgb)
            .or_else(|| caps.formats.iter().copied().find(|f| f.is_srgb()))
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(gpu.device(), &config);

        let frame_texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("gba-frame-texture"),
            size: wgpu::Extent3d {
                width: viewport_width as u32,
                height: viewport_height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let frame_view = frame_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gpu.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("gba-frame-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = gpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gba-frame-uniforms"),
            contents: bytemuck::bytes_of(&compute_uniforms(size.width, size.height, viewport_width, viewport_height)),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = gpu.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gba-frame-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gba-frame-bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&frame_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader = gpu.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gba-frame-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/frame.wgsl").into()),
        });

        let pipeline_layout = gpu.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gba-frame-pl"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = gpu.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gba-frame-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            _window: window,
            surface,
            config,
            surface_format,
            viewport_width,
            viewport_height,
            frame_texture,
            _frame_view: frame_view,
            _sampler: sampler,
            uniform_buffer,
            bind_group,
            pipeline,
        }
    }

    pub fn resize(&mut self, gpu: &GpuContext, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        // Clamp to the device's texture-dimension limit. wgpu::Surface::configure
        // panics if either dimension exceeds max_texture_dimension_2d, which is
        // easy to hit when the user resizes the window bigger than the limit
        // (2048 on downlevel defaults).
        let max_dim = gpu.device().limits().max_texture_dimension_2d;
        self.config.width = size.width.min(max_dim);
        self.config.height = size.height.min(max_dim);
        self.surface.configure(gpu.device(), &self.config);
        let uniforms = compute_uniforms(
            self.config.width,
            self.config.height,
            self.viewport_width,
            self.viewport_height,
        );
        gpu.queue()
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn upload_frame(&self, gpu: &GpuContext, frame: &[u32]) {
        debug_assert_eq!(frame.len(), self.viewport_width * self.viewport_height);
        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.frame_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(frame),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((self.viewport_width * 4) as u32),
                rows_per_image: Some(self.viewport_height as u32),
            },
            wgpu::Extent3d {
                width: self.viewport_width as u32,
                height: self.viewport_height as u32,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn draw_frame(&self, rpass: &mut wgpu::RenderPass<'static>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..4, 0..1);
    }

    pub fn acquire(&self) -> wgpu::CurrentSurfaceTexture {
        self.surface.get_current_texture()
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}

fn compute_uniforms(window_w: u32, window_h: u32, viewport_width: usize, viewport_height: usize) -> Uniforms {
    let w = window_w.max(1) as f32;
    let h = window_h.max(1) as f32;
    let s = (w / viewport_width as f32).min(h / viewport_height as f32);
    let rendered_w = viewport_width as f32 * s;
    let rendered_h = viewport_height as f32 * s;
    Uniforms {
        scale: [rendered_w / w, rendered_h / h],
        offset: [0.0, 0.0],
    }
}
