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

const RADIUS: i32 = 4;
const SIGMA: f32 = 3.0;

const TEMP_WIDTH: f32 = 400.0;
const TEMP_HEIGHT: f32 = 300.0;

fn fs_impl(in: VsOut, horizontal: bool) -> vec4<f32> {
    var color_sum = vec3(0.0, 0.0, 0.0);
    var weight_sum = 0.0;

    for (var d = -RADIUS; d <= RADIUS; d++) {
        var coords: vec2<f32>;
        if horizontal {
            coords = vec2(in.tex_coords.x + f32(d) / TEMP_WIDTH, in.tex_coords.y);
        } else {
            coords = vec2(in.tex_coords.x, in.tex_coords.y + f32(d) / TEMP_HEIGHT);
        }
        let color = textureSample(tex, samp, coords);
        let weight = exp(-f32(d * d) / (2.0 * SIGMA * SIGMA));
        color_sum += weight * color.rgb;
        weight_sum += weight;
    }

    return vec4(color_sum / weight_sum, 1.0);
}

@fragment
fn fs_horizontal(in: VsOut) -> @location(0) vec4<f32> {
    return fs_impl(in, true);
}

@fragment
fn fs_vertical(in: VsOut) -> @location(0) vec4<f32> {
    return fs_impl(in, false);
}