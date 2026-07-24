use glam::DMat4;

/// A 4x4 column-major matrix, backed by glam. The bare name is the `f64` form.
/// In a transform matrix `w_axis` is the translation.
pub type TyMatrix4x4 = DMat4;
