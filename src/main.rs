use cg8::{
    camera::{Camera, Projection},
    core::{App, ClearColor, Context, Engine},
    filter::{MagFilter, RoundColor, SourceTexture},
    renderer::{
        ColoredPolygonRenderer, ColoredPolygons, ColoredVertex, ColoredVertices, Indices, Instance,
        Instances,
    },
};
use glam::{vec3, vec3a, Affine3A, Mat4};
use winit::event::VirtualKeyCode;

fn main() {
    let engine = Engine::new();
    let app = MyApp::new(&engine.context());
    engine.run(app);
}

pub struct MyApp {
    camera: Camera,
    clear_color: ClearColor,
    renderer: ColoredPolygonRenderer,
    polygons: ColoredPolygons,
    instances: Instances,
    mag_filter: MagFilter,
    round_color: RoundColor,
    frames: Vec<wgpu::Texture>,
    frame_views: Vec<wgpu::TextureView>,
    frame_sources: Vec<SourceTexture>,
}

impl MyApp {
    pub fn new(ctx: &Context) -> Self {
        let transform = Affine3A::IDENTITY;
        let projection =
            Projection::perspective(std::f32::consts::FRAC_PI_4, ctx.aspect_ratio(), 0.1, 1000.0);
        // Projection::Orthographic { left: 0, right: 800, bottom: 500, top: 0, near: (), far: () }
        let scale = 4;
        let width = ctx.size().0 / scale;
        let height = ctx.size().1 / scale;
        let camera = Camera::new(ctx, transform, projection, width, height);
        let renderer = ColoredPolygonRenderer::new(ctx);
        let polygons = octahedron(ctx);
        let t = Mat4::from_translation(vec3(0.0, 0.0, 10.0));
        let instances = Instances::new(
            ctx,
            vec![Instance {
                mat: t.to_cols_array_2d(),
            }],
        );
        let mag_filter = MagFilter::new(ctx);
        let round_color = RoundColor::new(ctx);
        let mut frames = vec![];
        let mut frame_views = vec![];
        let mut frame_sources = vec![];
        for _ in 0..2 {
            let frame = ctx.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("frame"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: ctx.config().format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let frame_source = mag_filter.create_source(
                ctx,
                &frame.create_view(&wgpu::TextureViewDescriptor::default()),
            );
            frame_views.push(frame.create_view(&wgpu::TextureViewDescriptor::default()));
            frames.push(frame);
            frame_sources.push(frame_source);
        }
        Self {
            camera,
            clear_color: ClearColor {
                color: wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            },
            renderer,
            polygons,
            instances,
            mag_filter,
            round_color,
            frames,
            frame_views,
            frame_sources,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context) {
        let mut dx = 0;
        let mut dz = 0;
        if ctx.input().is_pressed(VirtualKeyCode::A) {
            dx -= 1;
        }
        if ctx.input().is_pressed(VirtualKeyCode::D) {
            dx += 1;
        }
        if ctx.input().is_pressed(VirtualKeyCode::W) {
            dz += 1;
        }
        if ctx.input().is_pressed(VirtualKeyCode::S) {
            dz -= 1;
        }
        if dx != 0 || dz != 0 {
            let r = 4.0 / 60.0;
            let d = vec3a(dx as f32, 0.0, dz as f32).normalize() * r;
            self.camera.transform.translation += d;
            self.camera.update_buffer(ctx);
        }

        let t = Mat4::from_translation(vec3(0.0, 0.0, 5.0))
            * Mat4::from_axis_angle(
                vec3(1.0, 2.0, 0.0).normalize(),
                ctx.frame_count() as f32 / 60.0 / std::f32::consts::PI,
            );

        self.instances[0].mat = t.to_cols_array_2d();
        self.instances.update_buffer(ctx);
    }
    fn render(&mut self, ctx: &Context) {
        self.clear_color.render(ctx, &self.frame_views[0]);
        self.renderer.render(
            ctx,
            &self.frame_views[0],
            &self.polygons,
            &self.instances,
            &self.camera,
        );
        self.round_color
            .render(ctx, &self.frame_sources[0], &self.frame_views[1]);
        let frame = ctx.surface().get_current_texture().unwrap();
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.mag_filter
            .render(ctx, &self.frame_sources[1], &frame_view);
        frame.present();
    }
}

fn sphere(ctx: &Context) -> ColoredPolygons {
    fn vertex(x: f32, y: f32, z: f32) -> ColoredVertex {
        let color = [-0.5 * y + 0.5, 0.8, 0.5 * y + 0.5, 1.0];
        ColoredVertex {
            pos: [x, y, z, 0.0],
            color,
        }
    }

    let mut vertices = vec![];
    vertices.push(vertex(0.0, 1.0, 0.0));

    todo!()
}

/*
fn icosahedron(ctx: &Context) -> ColoredPolygons {
    // (0, +-1, +-phi)
    // R = sqrt(2 + phi)
    const R: f64 = 1.902113032590307;
    const PHI: f64 = 1.6180339887498948482;
    const S: f64 = 1.0 / R;
    const T: f64 = PHI / R;

}
 */

fn octahedron(ctx: &Context) -> ColoredPolygons {
    //  0  1  2  3  4  5
    // +x +z -x -z +y -y
    const POS: [[i32; 3]; 6] = [
        [1, 0, 0],
        [0, 0, 1],
        [-1, 0, 0],
        [0, 0, -1],
        [0, 1, 0],
        [0, -1, 0],
    ];
    fn c(x: i32) -> f32 {
        (x as f32 + 1.0) * 0.5
    }
    let vertices = ColoredVertices::new(
        ctx,
        POS.iter()
            .map(|&[x, y, z]| ColoredVertex {
                pos: [x as f32, y as f32, z as f32, 1.0],
                color: [c(x), c(y), c(z), 1.0],
            })
            .collect(),
    );
    let mut indices = vec![];
    for i in 0..4 {
        let j = (i + 1) % 4;
        indices.extend([4, i, j, 5, j, i]);
    }
    ColoredPolygons {
        vertices,
        indices: Indices::new(ctx, indices),
    }
}
