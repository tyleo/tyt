// A probe of when the depth test rejects a fragment before it runs. A near
// layer can cover both eyes. Costly layers then draw behind it, each behind
// the last. `DepthProbe` appends `bounded` where the device has early depth
// tests.

struct Frame {
    // The steps of a costly fragment's loop.
    steps: u32,
}

struct Depth {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@group(0) @binding(0) var<uniform> frame: Frame;

// Instance 0 is the near layer. Depth runs reversed, so each later instance
// sits behind the last.
@vertex
fn cover(
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) layer: u32,
) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32(index & 1u), f32(index >> 1u));

    return vec4<f32>(corner * 4.0 - 1.0, 0.9 - 0.1 * f32(layer), 1.0);
}

@fragment
fn near() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}

// A value under one that takes the frame's steps to compute.
fn work(at: vec2<f32>) -> f32 {
    var value = fract(at.x * 0.001);

    for (var n = 0u; n < frame.steps; n++) {
        value = fract(value * 1.618034 + 0.1);
    }

    return value;
}

// Writes color alone.
@fragment
fn plain(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(work(at.xy));
}

// Can discard, and never does.
@fragment
fn kills(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    let value = work(at.xy);

    if value > 2.0 {
        discard;
    }

    return vec4<f32>(value);
}

// Writes the depth the rasterizer gave it.
@fragment
fn written(@builtin(position) at: vec4<f32>) -> Depth {
    return Depth(vec4<f32>(work(at.xy)), at.z);
}
