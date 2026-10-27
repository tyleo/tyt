// B0's indirect draw and dispatch checks.

// A triangle that covers the whole target.
@vertex
fn cover(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));

    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn red() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

@group(0) @binding(0) var<storage, read_write> groups: atomic<u32>;

// Counts the workgroups that run.
@compute @workgroup_size(1)
fn count() {
    atomicAdd(&groups, 1u);
}
