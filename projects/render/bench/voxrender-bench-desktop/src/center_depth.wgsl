// The depth at the middle of a window's first eye, as a target's one texel.

@group(0) @binding(0) var depth: texture_depth_2d;

@vertex
fn corner(@builtin(vertex_index) vertex: u32) -> @builtin(position) vec4<f32> {
    let across = vec2<f32>(f32(vertex & 1u), f32(vertex >> 1u)) * 2.0;

    return vec4<f32>(across.x * 2.0 - 1.0, 1.0 - across.y * 2.0, 0.0, 1.0);
}

@fragment
fn middle() -> @location(0) vec4<f32> {
    let at = vec2<i32>(textureDimensions(depth) / 2u);

    return vec4<f32>(textureLoad(depth, at, 0), 0.0, 0.0, 1.0);
}
