// One multiview pass draws both eyes, and the draw's instance picks the
// placement.

@vertex
fn place(
    @location(0) corner: vec4<u32>,
    @builtin(view_index) eye: u32,
    @builtin(instance_index) placement: u32,
) -> Varyings {
    return place_corner(corner, eye * frame.placements + placement, placement);
}
