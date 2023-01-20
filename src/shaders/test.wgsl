struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) top_left: vec3<f32>,
    @location(1) bottom_right: vec2<f32>,
    @location(2) tex_top_left: vec2<f32>,
    @location(3) tex_bottom_right: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_pos: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct Matrix {
    v: mat4x4<f32>,
}

@group(0) @binding(2)
var<uniform> global: Matrix;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var pos: vec2<f32>;
    var left_x: f32 = in.top_left.x;
    var right_x: f32 = in.bottom_right.x;
    var top_y: f32 = in.top_left.y;
    var bottom_y: f32 = in.bottom_right.y;

    switch (i32(in.vertex_index)) {
        case 0: {
            pos = vec2<f32>(left_x, top_y);
            out.tex_pos = in.tex_top_left;
            break;
        }
        case 1: {
            pos = vec2<f32>(right_x, top_y);
            out.tex_pos = vec2<f32>(in.tex_bottom_right.x, in.tex_top_left.y);
            break;
        }
        case 2: {
            pos = vec2<f32>(left_x, bottom_y);
            out.tex_pos = vec2<f32>(in.tex_top_left.x, in.tex_bottom_right.y);
            break;
        }
        case 3: {
            pos = vec2<f32>(right_x, bottom_y);
            out.tex_pos = in.tex_bottom_right;
            break;
        }
        default: {}
    }

    out.clip_position = global.v * vec4<f32>(pos, in.top_left.z, 1.0);
    out.color = vec3<f32>(0.7, 0.2, 0.1);

    return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}

// Bigger the text, bigger the screenPxRange.
fn screenPxRange(texCoord: vec2<f32>) -> f32 {
    let unitRange = vec2<f32>(4.0) / vec2<f32>(textureDimensions(texture));
    let screenTexSize = vec2<f32>(1.0) / fwidth(texCoord);
    return max(0.5 * dot(unitRange, screenTexSize), 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(texture, tex_sampler, in.tex_pos).rgba;
    let d = median(texel.r, texel.g, texel.b) - 0.5;
    let px_range = screenPxRange(in.tex_pos);

    var fg_color = vec4<f32>(0.8, 0.4, 0.1, 1.0);
    var bg_color = vec4<f32>(0.3, 0.2, 0.9, 0.3);
    var outline_color = vec4<f32>(0.9, 0.2, 0.3, 0.8);

    ///////////////////////// TESTING /////////////////////////
    //let od = clamp(d, 0.45, 0.55) - 0.5;
    let od = d - 0.3;
    let outline = clamp(od * px_range + 0.5, 0.0, 1.0);

    let td = d;
    let body = clamp(td * px_range + 0.5, 0.0, 1.0);

    let alpha = body - outline;

    //let pixel_dist = px_range * dist;
    //let alpha = clamp(pixel_dist + 0.5, 0.0, 1.0);

    ////////////// JUST SDF (only alpha channel) /////////////
    //let alpha = smoothstep(0.5, 0.55, texel);

    //////////////////// GAMMA CORRECTION /////////////////

    //let gamma = 2.2;
    //let alpha = pow(/*fg_color.a * */alpha, 1.0 / gamma);

    //let color = vec4<f32>(mix(outline_color, fg_color, alpha).rgb, alpha);
    //let color = vec4<f32>(mix(outline_color.rgb, fg_color.rgb, body_alpha), alpha);

    return mix(outline_color, fg_color, alpha);
}