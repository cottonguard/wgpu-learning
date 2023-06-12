use std::mem;

use glam::{Affine3A, Mat4};

use crate::core::Context;

pub struct Camera {
    pub transform: Affine3A,
    pub projection: Projection,
    buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    // depth_texture: wgpu::Texture,
    pub(crate) depth_texture_view: wgpu::TextureView,
}

impl Camera {
    pub fn new(
        ctx: &Context,
        transform: Affine3A,
        projection: Projection,
        width: u32,
        height: u32,
    ) -> Self {
        let buffer = ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: mem::size_of::<[f32; 16]>() as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: ctx.camera_bind_group_layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let depth_texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let res = Self {
            transform,
            projection,
            buffer,
            bind_group,
            // depth_texture,
            depth_texture_view,
        };
        res.update_buffer(ctx);
        res
    }

    pub fn update_buffer(&self, ctx: &Context) {
        ctx.queue().write_buffer(
            &self.buffer,
            0,
            bytemuck::bytes_of(&self.matrix().to_cols_array()),
        );
    }

    fn matrix(&self) -> Mat4 {
        self.projection.matrix() * self.transform.inverse()
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

pub enum Projection {
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov_y_radians: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
    },
}

impl Projection {
    pub fn perspective(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self::Perspective {
            fov_y_radians,
            aspect_ratio,
            z_near,
            z_far,
        }
    }
    fn matrix(&self) -> Mat4 {
        match *self {
            Self::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Mat4::orthographic_lh(left, right, bottom, top, near, far),
            Self::Perspective {
                fov_y_radians,
                aspect_ratio,
                z_near,
                z_far,
            } => Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_near, z_far),
        }
    }
}
