use serde::{Deserialize, Serialize};

/// A render camera (`rCAM`) preserved verbatim in the `magica-voxel` ext.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MagicaVoxelCamera {
    /// The camera id.
    pub id: i32,

    /// `_mode`: projection mode, e.g. `"pers"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// `_focus`: the focal point `[x, y, z]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<[f32; 3]>,

    /// `_angle`: the orbit angles `[x, y, z]` in degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<[f32; 3]>,

    /// `_radius`: orbit distance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<i32>,

    /// `_frustum`: frustum scale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frustum: Option<f32>,

    /// `_fov`: vertical field of view in degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fov: Option<i32>,

    /// Any further attribute keys, preserved verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String)>,
}
