@group(0) @binding(0)
var<uniform> screen: vec2<f32>;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) radius: f32,
    @location(4) z: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(instance: VertexInput, @builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var quad_pos: vec2<f32>;
    switch in_vertex_index {
        case 0u: {
            quad_pos = vec2<f32>(0.0, 0.0);
        }
        case 1u: {
            quad_pos = vec2<f32>(1.0, 0.0);
        }
        case 2u: {
            quad_pos = vec2<f32>(0.0, 1.0);
        }
        case 3u: {
            quad_pos = vec2<f32>(0.0, 1.0);
        }
        case 4u: {
            quad_pos = vec2<f32>(1.0, 0.0);
        }
        case 5u: {
            quad_pos = vec2<f32>(1.0, 1.0);
        }
        default: {
            quad_pos = vec2<f32>(0.0, 0.0);
        }
    }

    let world_pos = instance.pos + quad_pos * instance.size;

    let clip_pos = (world_pos / screen) * vec2(2.0, -2.0) - vec2(1.0, -1.0);

    var out: VertexOutput;
    out.clip_position = vec4(clip_pos.x, clip_pos.y, instance.z, 1.0);
    out.uv = quad_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv);
}
