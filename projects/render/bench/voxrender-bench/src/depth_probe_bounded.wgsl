// Writes the depth the rasterizer gave it and declares that it writes no
// nearer depth.
@fragment
@early_depth_test(less_equal)
fn bounded(@builtin(position) at: vec4<f32>) -> Depth {
    return Depth(vec4<f32>(work(at.xy)), at.z);
}
