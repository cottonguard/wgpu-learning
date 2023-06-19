use crate::core::{Context, Texture};

pub struct MagFilter {
    inner: Filter,
}

struct Filter {
    sampler_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Filter {
    fn new(
        ctx: &Context,
        shader: &wgpu::ShaderModule,
        vertex_entry_point: &str,
        fragment_entry_point: &str,
    ) -> Self {
        let sampler_bind_group_layout =
            ctx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("filter"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    }],
                });
        /*
        let shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shader/sample.wgsl"));
         */
        let pipeline_layout =
            ctx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("MagFilter"),
                    bind_group_layouts: &[
                        ctx.texture_bind_group_layout(),
                        &sampler_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
        let pipeline = ctx
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("MagFilter"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: vertex_entry_point,
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: fragment_entry_point,
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config().format,
                        blend: None, // Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });
        let sampler = ctx.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("filter"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        let sampler_bind_group = ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("filter"),
            layout: &sampler_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            }],
        });
        Self {
            sampler_bind_group,
            pipeline,
        }
    }

    fn render(&self, ctx: &Context, src: &Texture, dst: &Texture) {
        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("filter"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, src.bind_group.as_ref().unwrap(), &[]);
            pass.set_bind_group(1, &self.sampler_bind_group, &[]);
            pass.draw(0..4, 0..1);
        }

        ctx.queue().submit([encoder.finish()]);
    }
}

impl MagFilter {
    pub fn new(ctx: &Context) -> Self {
        let shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shader/sample.wgsl"));
        Self {
            inner: Filter::new(ctx, &shader, "vs_main", "fs_main"),
        }
    }

    pub fn render(&self, ctx: &Context, src: &Texture, dst: &Texture) {
        self.inner.render(ctx, src, dst)
    }
}

pub struct RoundColor {
    inner: Filter,
}

impl RoundColor {
    pub fn new(ctx: &Context) -> Self {
        let shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shader/round_color.wgsl"));
        Self {
            inner: Filter::new(ctx, &shader, "vs_main", "fs_main"),
        }
    }

    pub fn render(&self, ctx: &Context, src: &Texture, dst: &Texture) {
        self.inner.render(ctx, src, dst)
    }
}

pub struct GaussianBlur {
    horizontal: Filter,
    vertical: Filter,
}

impl GaussianBlur {
    pub fn new(ctx: &Context) -> Self {
        let shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shader/gaussian_blur.wgsl"));
        Self {
            horizontal: Filter::new(ctx, &shader, "vs_main", "fs_horizontal"),
            vertical: Filter::new(ctx, &shader, "vs_main", "fs_vertical"),
        }
    }

    pub fn render(&self, ctx: &Context, src: &Texture, dst: &Texture, tmp: &Texture) {
        self.horizontal.render(ctx, src, tmp);
        self.vertical.render(ctx, tmp, dst);
    }
}
