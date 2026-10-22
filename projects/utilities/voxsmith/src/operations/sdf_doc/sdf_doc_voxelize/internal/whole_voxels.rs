use crate::operations::sdf_doc::WHOLE_TOLERANCE;
use ty_math::{TyVector3F64, TyVector3I32};

/// `voxels` rounded to whole voxels, or `None` when a component sits farther
/// than [`WHOLE_TOLERANCE`] from a whole number.
pub fn whole_voxels(voxels: TyVector3F64) -> Option<TyVector3I32> {
    let rounded = voxels.round();

    (voxels - rounded)
        .abs()
        .cmple(TyVector3F64::splat(WHOLE_TOLERANCE))
        .all()
        .then(|| rounded.as_ivec3())
}
