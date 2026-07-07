/// A scene node's transform resolved to a chosen space, with its rotation
/// reduced to euler angles for display.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedNodeTransform {
    /// Translation in the resolved space.
    pub position: [f64; 3],
    /// Rotation as euler angles in radians, ordered `[x, y, z]` for the
    /// `Rz * Ry * Rx` convention.
    pub rotation: [f64; 3],
    /// Per-axis scale in the resolved space.
    pub scale: [f64; 3],
}
