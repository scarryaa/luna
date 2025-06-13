struct Rect {
    pos   : vec2<f32>,  // top-left in pixels
    size  : vec2<f32>,
    color : vec4<f32>,
};

@group(0) @binding(0) var<uniform> screen : vec2<f32>; // window size

struct VertexOut {
    @builtin(position) pos : vec4<f32>,
    @location(0) color     : vec4<f32>,
    @location(1) local_uv  : vec2<f32>,   // fragment-local position
    @location(2) size      : vec2<f32>,   // flat-interpolated
    @location(3) radius    : f32,         // flat-interpolated
};

@vertex
fn vs_main(
    @location(0) pos   : vec2<f32>,
    @location(1) size  : vec2<f32>,
    @location(2) color : vec4<f32>,
    @location(3) radius: f32,
    @location(4) z     : f32,
    @builtin(vertex_index) vi : u32
) -> VertexOut {
    let x = select(0.0, 1.0, vi == 1u || vi == 2u || vi == 4u);
    let y = select(0.0, 1.0, vi == 2u || vi == 4u || vi == 5u);
    let corner  = vec2(x, y);

    let p   = pos + corner * size;
    let ndc = vec2(p.x / screen.x * 2.0 - 1.0,
                   1.0 - p.y / screen.y * 2.0);

    var o : VertexOut;
    o.pos      = vec4(ndc, z, 1.0);
    o.color    = color;
    o.local_uv = corner * size;
    o.size     = size;
    o.radius   = radius;
    return o;
}

fn sdRoundedBox(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(
    @location(0) color_in : vec4<f32>,
    @location(1) local_uv : vec2<f32>,
    @location(2) size     : vec2<f32>,
    @location(3) radius   : f32
) -> @location(0) vec4<f32> {
    let p = local_uv - size * 0.5;
    let half = size * 0.5 - vec2(radius);
    let dist = sdRoundedBox(p, half, radius);

    let alpha = clamp(0.5 - dist / fwidth(dist), 0.0, 1.0);

    return vec4(color_in.rgb, color_in.a * alpha);
}
