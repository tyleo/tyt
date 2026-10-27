// One pass draws each eye, and the draw's instance picks the eye's matrix
// for the placement.

@vertex
fn place(@location(0) corner: vec4<u32>, @builtin(instance_index) clip: u32) -> Varyings {
    return place_corner(corner, clip, clip % frame.placements);
}
