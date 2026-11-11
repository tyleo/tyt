/// One eye of a headset's frame, in the headset's tracking space.
#[derive(Clone, Copy, Debug)]
pub struct HeadsetEye {
    /// The eye's position in meters.
    pub position: [f32; 3],

    /// The eye's rotation as a unit quaternion, x, y, z, then w. The eye
    /// looks down its -Z with +Y up.
    pub rotation: [f32; 4],

    /// The tangents of the angles from the eye's axis to its frustum's left,
    /// right, up, and down sides. A side left of or below the axis has a
    /// negative tangent.
    pub tangents: [f32; 4],
}
