use glam::DMat4;

/// A 4x4 column-major matrix with `f64` components, backed by glam. In a
/// transform matrix `w_axis` is the translation.
pub type TyMatrix4x4F64 = DMat4;
