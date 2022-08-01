use std::collections::HashMap;

use artery_font::ArteryFont;
use wgpu::util::DeviceExt;

use crate::{camera::Camera, text::Glyph, Graphics};

pub struct Requisites {
    pub atlas_texture: wgpu::Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    pub glyphs: HashMap<u32, Glyph>,
    pub matrix_buffer: wgpu::Buffer,
}

impl Requisites {
    pub fn init(
        gfx: &Graphics,
        arfont: &ArteryFont,
    ) -> Self {
        let image = arfont.images.first().unwrap();
        let image_data = &image.data;
        let variants = arfont.variants.first().unwrap();

        let mut glyphs: HashMap<u32, Glyph> = HashMap::new();
        for g in &variants.glyphs {
            let glyph = Glyph {
                advance_x: g.advance.horizontal,
                plane_bounds: g.plane_bounds,
                atlas_bounds: g.image_bounds.scaled(
                    1.0 / image.width as f32,
                    1.0 / image.height as f32,
                ),
            };
            glyphs.insert(g.codepoint, glyph);
        }

        let size = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };

        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let texture_view =
            texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        gfx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * image.width),
                rows_per_image: None,
            },
            size,
        );

        let matrix_buffer = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Matrix Buffer"),
            size: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let bind_group_layout = gfx.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        );

        let bind_group =
            gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &texture_view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: matrix_buffer.as_entire_binding(),
                    },
                ],
            });

        Self {
            atlas_texture: texture,
            bind_group_layout,
            bind_group,

            glyphs,
            matrix_buffer,
        }
    }
}

pub fn pipeline1(gfx: &Graphics, reqs: &Requisites) -> wgpu::RenderPipeline {
    let shader = gfx
        .device
        .create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
    let layout =
        gfx.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline 1 Layout"),
                bind_group_layouts: &[&reqs.bind_group_layout],
                push_constant_ranges: &[],
            });
    gfx.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline 1"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Quad::buffer_layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: gfx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Quad {
    pub top_left: [f32; 3],
    pub bottom_right: [f32; 2],
    pub tex_top_left: [f32; 2],
    pub tex_bottom_right: [f32; 2],
}

impl Quad {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Quad>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 3]>()
                        as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 5]>()
                        as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 7]>()
                        as wgpu::BufferAddress,
                    shader_location: 3,
                },
            ],
        }
    }
}
