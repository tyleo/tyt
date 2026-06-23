/// Converts a Voxel Max axis-angle rotation `[x, y, z, angle]` to a unit
/// quaternion `[x, y, z, w]`.
pub(crate) fn axis_angle_to_quat(axis_angle: [f64; 4]) -> [f64; 4] {
    let [ax, ay, az, angle] = axis_angle;
    let length = (ax * ax + ay * ay + az * az).sqrt();
    if length < 1e-12 || angle == 0.0 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let half = angle / 2.0;
    let s = half.sin() / length;
    [ax * s, ay * s, az * s, half.cos()]
}
