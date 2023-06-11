@group(0)
@binding(0)
var<uniform> camera: mat4x4<f32>;

struct VsIn {
    @location(0) pos: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) c0: vec4<f32>,
    @location(3) c1: vec4<f32>,
    @location(4) c2: vec4<f32>,
    @location(5) c3: vec4<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VsIn, @builtin(vertex_index) i: u32) -> VsOut {
    var out: VsOut;
    let tr = mat4x4(in.c0, in.c1, in.c2, in.c3);
    out.pos = camera * tr * in.pos;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}