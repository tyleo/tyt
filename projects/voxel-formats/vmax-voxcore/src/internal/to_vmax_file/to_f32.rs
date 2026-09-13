/// A coefficient snapped to an f32-exact value. Voxel Max decodes material
/// coefficients as 32-bit floats and drops a whole palette whose coefficients
/// are not f32-representable, so every one passes through here.
pub(crate) fn to_f32(value: f64) -> f64 {
    f64::from(value as f32)
}
