/// Recommended tolerance for treating a rotation quaternion as unit length.
/// A quaternion whose squared length strays further than this no longer
/// represents a rotation faithfully, so gate an incoming rotation on it with
/// [`TyQuaternionExt::is_normalized_within`](crate::TyQuaternionExt::is_normalized_within).
pub const UNIT_ROTATION_TOLERANCE: f64 = 1e-6;
