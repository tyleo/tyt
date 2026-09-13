use vmax::snapshots::VMaxVoxel;

/// The color-and-material combination key for a voxel: its color index and, when
/// the object has materials, its material index, else zero.
pub(crate) fn combo_key(voxel: &VMaxVoxel, has_materials: bool) -> (u8, u8) {
    (
        voxel.color_idx,
        if has_materials { voxel.material_idx } else { 0 },
    )
}
