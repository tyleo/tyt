use serde::{Deserialize, Serialize};

/// Per-node Voxel Max provenance preserved in the `voxel-max` ext: the
/// `scene.json` node fields the voxcore hierarchy does not represent natively,
/// kept aligned by index with the hierarchy nodes, groups before objects.
///
/// The voxcore node carries the name, position, and scale, so this holds the
/// rest. The rotation is kept as the original axis-angle because the quaternion
/// voxcore stores cannot be inverted back to it exactly. The content box
/// (`e_c`/`e_mi`/`e_ma`) is not kept: it is derived on write from the native
/// tight bounds for an object and from the subtree's geometry for a group.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VoxelMaxNode {
    /// Node UUID (`id`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,

    /// Parent node UUID (`pid`).
    #[serde(rename = "pid", default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Index triplet (`ind`).
    #[serde(rename = "ind", default, skip_serializing_if = "Option::is_none")]
    pub index: Option<[i64; 3]>,

    /// Axis-angle rotation `[x, y, z, angle]` (`t_r`).
    #[serde(rename = "t_r", default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 4]>,

    /// Alignment enum token (`t_al`).
    #[serde(rename = "t_al", default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<String>,

    /// Pivot-face enum token (`t_pf`).
    #[serde(rename = "t_pf", default, skip_serializing_if = "Option::is_none")]
    pub pivot_face: Option<String>,

    /// Pivot-align enum token (`t_pa`).
    #[serde(rename = "t_pa", default, skip_serializing_if = "Option::is_none")]
    pub pivot_align: Option<String>,

    /// Selected UI flag (`s`).
    #[serde(rename = "s", default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
}
