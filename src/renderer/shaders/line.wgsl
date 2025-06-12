@group(0) @binding(0)
var<uniform> screen : vec2<f32>; // window size in pixels

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) a      : vec2<f32>,
    @location(1) b      : vec2<f32>,
    @location(2) color  : vec4<f32>,
    @location(3) half_w : f32,
    @builtin(vertex_index) vertex_index : u32
) -> VertexOut {
    let x = select(0.0, 1.0, vertex_index == 1u || vertex_index == 2u || vertex_index == 4u);
    let y = select(0.0, 1.0, vertex_index == 2u || vertex_index == 4u || vertex_index == 5u);
    let local = vec2<f32>(x, y);

    let dir = normalize(b - a);
    let perp = vec2<f32>(-dir.y, dir.x);

    let pos_on_line = a * (1.0 - local.x) + b * local.x;

    let offset = (local.y - 0.5) * half_w * 2.0 * perp;
    let p = pos_on_line + offset;

    // Convert to NDC
    let ndc = vec2<f32>(
        p.x / screen.x * 2.0 - 1.0,
        1.0 - p.y / screen.y * 2.0
    );

    var out: VertexOut;
    out.pos = vec4<f32>(ndc, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}

