struct Rect {
    pos   : vec2<f32>,  // top-left in pixels
    size  : vec2<f32>,
    color : vec4<f32>,
};

@group(0) @binding(0) var<uniform> screen : vec2<f32>; // window size

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) pos   : vec2<f32>,
    @location(1) size  : vec2<f32>,
    @location(2) color : vec4<f32>,
    @builtin(vertex_index) vertex_index : u32
) -> VertexOut {
    let x = select(0.0, 1.0, vertex_index == 1u || vertex_index == 2u || vertex_index == 4u);
    let y = select(0.0, 1.0, vertex_index == 2u || vertex_index == 4u || vertex_index == 5u);
    let corner = vec2<f32>(x, y);

    let p = pos + corner * size;
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
