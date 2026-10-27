use ty_math::{TyVector3Ext, TyVector3F64};

/// One scene-frame translation, on MagicaVoxel's Z-up axes, rounded from a
/// node's local position.
pub fn frame_translation(position: TyVector3F64) -> [i32; 3] {
    position.yup_to_zup().round().as_ivec3().to_array()
}
