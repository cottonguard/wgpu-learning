use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{camera::Camera, input::Input};

pub struct Engine {
    event_loop: EventLoop<()>,
    window: Window,
    ctx: Context,
}

pub struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,

    camera_bind_group_layout: wgpu::BindGroupLayout,

    input: Input,
    frame_count: u64,
}

impl Engine {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();

        let ctx = Context::new(&window);

        Self {
            event_loop,
            window,
            ctx,
        }
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    pub fn run<A: App + 'static>(self, mut app: A) {
        let Self {
            window,
            event_loop,
            mut ctx,
        } = self;

        window.set_visible(true);

        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => {
                app.update(&ctx);
                ctx.input.next_tick();
                ctx.frame_count += 1;
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                app.render(&ctx);
            }
            Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            ctx.input.press(keycode);
                        } else {
                            ctx.input.release(keycode);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        });
    }
}

impl Context {
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn size(&self) -> (u32, u32) {
        let config = self.config();
        (config.width, config.height)
    }

    pub fn aspect_ratio(&self) -> f32 {
        let (w, h) = self.size();
        w as f32 / h as f32
    }

    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    fn new(window: &Window) -> Self {
        smol::block_on(Self::new_async(window))
    }

    async fn new_async(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();
        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let camera_bind_group_layout = Camera::bind_group_layout(&device);

        Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            config,

            camera_bind_group_layout,

            input: Input::default(),
            frame_count: 0,
        }
    }
}

pub trait App {
    fn update(&mut self, ctx: &Context);
    fn render(&mut self, ctx: &Context);
}

pub struct ClearColor {
    pub color: wgpu::Color,
}

impl ClearColor {
    pub fn render(&mut self, ctx: &Context, dst: &wgpu::TextureView) {
        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }
        ctx.queue().submit([encoder.finish()]);
    }
}
