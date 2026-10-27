use ty_math::{TyQuaternionF64, TyVector3F64, ZERO_LENGTH_TOLERANCE};

/// Decodes a stored `[x, y, z, angle]` axis-angle rotation into a quaternion,
/// the inverse of [`encode_axis_angle`](crate::encode_axis_angle). An
/// unrotated object stores `[0, 0, 0, 0]`; a zero axis has no direction to
/// normalize, so it decodes to the identity.
pub fn decode_axis_angle(rotation: [f64; 4]) -> TyQuaternionF64 {
    let [x, y, z, angle] = rotation;
    let axis = TyVector3F64::new(x, y, z);
    if axis.length() < ZERO_LENGTH_TOLERANCE {
        return TyQuaternionF64::IDENTITY;
    }
    TyQuaternionF64::from_axis_angle(axis.normalize(), angle)
}
