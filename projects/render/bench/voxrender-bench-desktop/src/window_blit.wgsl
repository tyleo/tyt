// One eye of a gallery's image over a window's surface, as one triangle.

struct Corner {
    @builtin(position) clip: vec4<f32>,

    // The point across the surface, from its top left.
    @location(0) across: vec2<f32>,
}

@group(0) @binding(0) var eye: texture_2d<f32>;
@group(0) @binding(1) var eye_sampler: sampler;

@vertex
fn corner(@builtin(vertex_index) vertex: u32) -> Corner {
    let across = vec2<f32>(f32(vertex & 1u), f32(vertex >> 1u)) * 2.0;

    return Corner(vec4<f32>(across.x * 2.0 - 1.0, 1.0 - across.y * 2.0, 0.0, 1.0), across);
}

@fragment
fn show(corner: Corner) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSampleLevel(eye, eye_sampler, corner.across, 0.0).rgb, 1.0);
}
