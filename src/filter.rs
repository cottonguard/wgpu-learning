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
        input_count: usize,
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
        let bind_group_layouts: Vec<_> = (0..input_count)
            .map(|_| ctx.texture_bind_group_layout())
            .chain(Some(&sampler_bind_group_layout))
            .collect();
        let pipeline_layout =
            ctx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("filter"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });
        let pipeline = ctx
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("filter"),
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
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: None,
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

    fn render2(&self, ctx: &Context, src1: &Texture, src2: &Texture, dst: &Texture) {
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
            pass.set_bind_group(0, src1.bind_group.as_ref().unwrap(), &[]);
            pass.set_bind_group(1, src2.bind_group.as_ref().unwrap(), &[]);
            pass.set_bind_group(2, &self.sampler_bind_group, &[]);
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
            inner: Filter::new(ctx, &shader, "vs_main", "fs_main", 1),
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
            inner: Filter::new(ctx, &shader, "vs_main", "fs_main", 1),
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
            horizontal: Filter::new(ctx, &shader, "vs_main", "fs_horizontal", 1),
            vertical: Filter::new(ctx, &shader, "vs_main", "fs_vertical", 1),
        }
    }

    pub fn render(&self, ctx: &Context, src: &Texture, dst: &Texture, tmp: &Texture) {
        self.horizontal.render(ctx, src, tmp);
        self.vertical.render(ctx, tmp, dst);
    }
}

pub struct Bloom {
    threshold: Filter,
    blend: Filter,
}

impl Bloom {
    pub fn new(ctx: &Context) -> Self {
        let threshold_shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shader/bloom_threshold.wgsl"));
        let blend_shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shader/add.wgsl"));
        Self {
            threshold: Filter::new(ctx, &threshold_shader, "vs_main", "fs_main", 1),
            blend: Filter::new(ctx, &blend_shader, "vs_main", "fs_main", 2),
        }
    }

    pub fn render(
        &self,
        ctx: &Context,
        blur: &GaussianBlur,
        src: &Texture,
        dst: &Texture,
        tmp1: &Texture,
        tmp2: &Texture,
    ) {
        self.threshold.render(ctx, src, tmp1);
        blur.render(ctx, tmp1, tmp2, dst);
        self.blend.render2(ctx, src, tmp2, dst);
    }
}
