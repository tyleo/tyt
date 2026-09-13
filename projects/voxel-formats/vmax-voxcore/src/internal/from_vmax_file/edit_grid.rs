use vmax::VMaxViewBox;

/// The object's build volume (the author's `tools.vp`) as `(bounds, origin)` in
/// the node's local voxel frame, which contains the runtime grid. The `origin`
/// is the build volume's min corner offset from the node, `vp.min - box_min +
/// origin`; an object with no build volume takes a zero-margin volume equal to
/// its runtime grid.
pub(crate) fn edit_grid(
    view_box: Option<&VMaxViewBox>,
    box_min: [i32; 3],
    origin: [i32; 3],
    bounds: [u32; 3],
) -> ([u32; 3], [i32; 3]) {
    match view_box {
        Some(vp) => (
            [
                (vp.max[0] - vp.min[0] + 1).max(0) as u32,
                (vp.max[1] - vp.min[1] + 1).max(0) as u32,
                (vp.max[2] - vp.min[2] + 1).max(0) as u32,
            ],
            [
                vp.min[0] as i32 - box_min[0] + origin[0],
                vp.min[1] as i32 - box_min[1] + origin[1],
                vp.min[2] as i32 - box_min[2] + origin[2],
            ],
        ),
        None => (bounds, origin),
    }
}
