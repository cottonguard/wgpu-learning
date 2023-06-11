@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var sampler: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) i: i32) -> vec4<f32> {
    var out: VsOut;
    let x = f32(i & 1);
    let y = f32(i >> 1);
    out.pos = vec4(2 * x - 1, 2 * y - 1, 0.0, 1.0);
    out.tex_coords = vec2(x, -y);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> vec4<f32> {
    return textureSample(tex, sampler, in.tex_coords);
}