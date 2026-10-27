use ty_math::{TyTransformF64, TyVector3I32};

/// A translation-only transform from a scene position.
pub fn translation(position: [i32; 3]) -> TyTransformF64 {
    TyTransformF64::from_translation(TyVector3I32::from_array(position).as_dvec3())
}
