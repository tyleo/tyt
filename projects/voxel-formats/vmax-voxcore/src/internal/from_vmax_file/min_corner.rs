use ty_math::TyVector3I32;
use vmax::snapshots::VMaxVoxel;

/// The minimum `[x, y, z]` corner over `voxels`, or `None` when empty.
pub(crate) fn min_corner(voxels: &[VMaxVoxel]) -> Option<[i32; 3]> {
    voxels
        .iter()
        .fold(None, |acc, v| {
            let position = TyVector3I32::from_array(v.position);
            let acc = acc.unwrap_or(position);
            Some(acc.min(position))
        })
        .map(|corner| corner.to_array())
}
