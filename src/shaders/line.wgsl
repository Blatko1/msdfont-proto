struct VertexInput {
    @location(0) pos: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct Matrix {
    v: mat4x4<f32>,
}

@group(0) @binding(2)
var<uniform> global: Matrix;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = global.v * vec4<f32>(in.pos, 1.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.5, 0.5, 0.5, 1.0);
}