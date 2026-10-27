use ty_math::TyQuaternionF64;

/// Axis-angle stored on a node with no preserved rotation; a degenerate axis
/// decodes to the identity quaternion.
const IDENTITY_AXIS_ANGLE: [f64; 4] = [0.0, 0.0, 0.0, 0.0];

/// The `[x, y, z, angle]` axis-angle that reproduces a quaternion rotation,
/// the inverse of [`decode_axis_angle`](crate::decode_axis_angle). Feeding the
/// result back through the decode, and Voxel Max's, recovers the same
/// rotation.
pub fn encode_axis_angle(rotation: TyQuaternionF64) -> [f64; 4] {
    let (axis, angle) = rotation.to_axis_angle();
    if angle == 0.0 {
        // No rotation: match Voxel Max's `[0, 0, 0, 0]` rather than emit a bare
        // axis.
        return IDENTITY_AXIS_ANGLE;
    }
    [axis.x, axis.y, axis.z, angle]
}
