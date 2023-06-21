@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(1) @binding(0)
var samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i & 1u);
    let y = f32(i >> 1u);
    out.pos = vec4(2.0 * x - 1.0, 2.0 * y - 1.0, 0.0, 1.0);
    out.tex_coords = vec2(x, 1.0 - y);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let color = textureSample(tex, samp, in.tex_coords);
    let p = (color.r + color.g + color.b) / 2.0;
    let q = max(p - 1.0, 0.0);
    return vec4(q * color.rgb, 1.0);
}
