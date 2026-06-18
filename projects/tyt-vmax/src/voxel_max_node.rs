use serde::{Deserialize, Serialize};
use tyt_injection::serde_json::{Map, Value};

/// Provenance for one Voxel Max scene node (object or group), aligned by index
/// with the voxj `hierarchyNodes`. Captures the `scene.json` fields that have no
/// native voxj representation so `from-voxj` can rebuild the node exactly.
///
/// Numbers that can be fractional (positions, rotation, bounds) are stored as
/// `f64`; index triplets stay integers. Any key not modeled here — legacy `t_a`,
/// `t_po`, or future additions — is preserved verbatim in [`extra`](Self::extra),
/// keeping its exact JSON form.
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
    /// Position `[x, y, z]` (`t_p`).
    #[serde(rename = "t_p", default, skip_serializing_if = "Option::is_none")]
    pub position: Option<[f64; 3]>,
    /// Axis-angle rotation `[x, y, z, angle]` (`t_r`).
    #[serde(rename = "t_r", default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 4]>,
    /// Scale `[x, y, z]` (`t_s`).
    #[serde(rename = "t_s", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f64; 3]>,
    /// Bounds center in model space (`e_c`).
    #[serde(rename = "e_c", default, skip_serializing_if = "Option::is_none")]
    pub center: Option<[f64; 3]>,
    /// Bounds min relative to `center` (`e_mi`).
    #[serde(rename = "e_mi", default, skip_serializing_if = "Option::is_none")]
    pub bounds_min: Option<[f64; 3]>,
    /// Bounds max relative to `center` (`e_ma`).
    #[serde(rename = "e_ma", default, skip_serializing_if = "Option::is_none")]
    pub bounds_max: Option<[f64; 3]>,
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
    /// Hidden UI flag (`h`).
    #[serde(rename = "h", default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// Voxel data filename (`data`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Palette image filename (`pal`).
    #[serde(rename = "pal", default, skip_serializing_if = "Option::is_none")]
    pub palette: Option<String>,
    /// History filename (`hist`).
    #[serde(rename = "hist", default, skip_serializing_if = "Option::is_none")]
    pub history: Option<String>,
    /// Object name (`n`).
    #[serde(rename = "n", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Group name (`name`).
    #[serde(rename = "name", default, skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    /// Any other `scene.json` keys on this node, preserved verbatim.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
