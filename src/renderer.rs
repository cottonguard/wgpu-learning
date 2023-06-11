use std::{
    mem,
    ops::{Deref, DerefMut},
};

use wgpu::util::DeviceExt;

use crate::{camera::Camera, core::Context};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredVertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

pub struct ColoredVertices {
    data: Vec<ColoredVertex>,
    buffer: wgpu::Buffer,
}

impl ColoredVertices {
    pub fn new(ctx: &Context, data: Vec<ColoredVertex>) -> Self {
        let buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("colored vertices"),
                usage: wgpu::BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&data),
            });
        Self { data, buffer }
    }
}

pub struct Indices {
    data: Vec<u32>,
    buffer: wgpu::Buffer,
}

impl Indices {
    pub fn new(ctx: &Context, data: Vec<u32>) -> Self {
        let buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                usage: wgpu::BufferUsages::INDEX,
                contents: bytemuck::cast_slice(&data),
            });
        Self { data, buffer }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub mat: [[f32; 4]; 4],
}

pub struct Instances {
    data: Vec<Instance>,
    buffer: wgpu::Buffer,
}

impl Instances {
    pub fn new(ctx: &Context, data: Vec<Instance>) -> Self {
        let buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&data),
            });
        Self { data, buffer }
    }
    pub fn update_buffer(&self, ctx: &Context) {
        ctx.queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }
}

impl Deref for Instances {
    type Target = [Instance];
    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}
impl DerefMut for Instances {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.data
    }
}

pub struct ColoredPolygons {
    pub vertices: ColoredVertices,
    pub indices: Indices,
}

impl ColoredPolygons {
    pub fn new(vertices: ColoredVertices, indices: Indices) -> Self {
        Self { vertices, indices }
    }
}

pub struct ColoredPolygonRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl ColoredPolygonRenderer {
    pub fn new(ctx: &Context) -> Self {
        let shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("shader/colored.wgsl"));
        let pipeline_layout =
            ctx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("colored"),
                    bind_group_layouts: &[ctx.camera_bind_group_layout()],
                    push_constant_ranges: &[],
                });
        let pipeline = ctx
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("colored"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: mem::size_of::<ColoredVertex>() as _,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![
                                0 => Float32x4,
                                1 => Float32x4,
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: mem::size_of::<Instance>() as _,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![
                                2 => Float32x4,
                                3 => Float32x4,
                                4 => Float32x4,
                                5 => Float32x4,
                            ],
                        },
                    ],
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config().format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });
        Self { pipeline }
    }

    pub fn render(
        &self,
        ctx: &Context,
        dst: &wgpu::TextureView,
        data: &ColoredPolygons,
        instances: &Instances,
        camera: &Camera,
    ) {
        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("1") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("1"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &camera.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &camera.bind_group, &[]);
            pass.set_vertex_buffer(0, data.vertices.buffer.slice(..));
            pass.set_vertex_buffer(1, instances.buffer.slice(..));
            pass.set_index_buffer(data.indices.buffer.slice(..), wgpu::IndexFormat::Uint32);

            pass.draw_indexed(0..data.indices.data.len() as u32, 0, 0..1);
        }
        ctx.queue().submit([encoder.finish()]);
    }
}
