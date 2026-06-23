use crate::MVoxDict;

/// A render camera (`rCAM`). The documented keys are lifted into fields; any
/// other keys are preserved in [`extra`](Self::extra).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MVoxCamera {
    /// The camera id.
    pub id: i32,

    /// `_mode`: projection mode, e.g. `"pers"`.
    pub mode: Option<String>,

    /// `_focus`: the focal point `[x, y, z]`.
    pub focus: Option<[f32; 3]>,

    /// `_angle`: the orbit angles `[x, y, z]` in degrees.
    pub angle: Option<[f32; 3]>,

    /// `_radius`: orbit distance.
    pub radius: Option<i32>,

    /// `_frustum`: frustum scale.
    pub frustum: Option<f32>,

    /// `_fov`: vertical field of view in degrees.
    pub fov: Option<i32>,

    /// Any further attribute keys, preserved verbatim.
    pub extra: MVoxDict,
}
