@group(0) @binding(0)
var<uniform> screen : vec2<f32>;

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) v_center: vec2<f32>,
    @location(1) v_radius: f32,
    @location(2) v_color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) center : vec2<f32>,
    @location(1) radius : f32,
    @location(2) _pad   : f32,
    @location(3) color  : vec4<f32>,
    @location(4) z    : f32,
    @builtin(vertex_index) vertex_index : u32
) -> VertexOut {
    let x = select(-1.0, 1.0, vertex_index == 1u || vertex_index == 2u || vertex_index == 4u);
    let y = select(-1.0, 1.0, vertex_index == 2u || vertex_index == 4u || vertex_index == 5u);
    let quad = vec2<f32>(x, y);

    let p = center + quad * radius;

    let ndc = vec2<f32>(
        p.x / screen.x * 2.0 - 1.0,
        1.0 - p.y / screen.y * 2.0
    );

    var out: VertexOut;
    out.v_center = center;
    out.v_radius = radius;
    out.v_color = color;
    out.pos = vec4<f32>(ndc, z, 1.0);
    return out;
}

@fragment
fn fs_main(
    @location(0) center: vec2<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
    @builtin(position) frag_pos: vec4<f32>
) -> @location(0) vec4<f32> {
    let frag_px = vec2<f32>(
        (frag_pos.x * 0.5 + 0.5) * screen.x,
        (1.0 - (frag_pos.y * 0.5 + 0.5)) * screen.y
    );

    let dist = distance(frag_px, center);
    if (dist > radius) {
        discard;
    }
    return color;
}

