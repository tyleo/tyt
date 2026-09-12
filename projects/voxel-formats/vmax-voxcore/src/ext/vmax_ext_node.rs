#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per-node Voxel Max provenance preserved in the `vmax` ext: the
/// `scene.json` node fields the voxcore hierarchy does not represent.
///
/// The voxcore node carries the name, position, scale, and parent. The
/// rotation stays as the file's axis-angle because the quaternion voxcore
/// stores cannot be inverted back to it exactly. The writer uses it while it
/// still decodes to the node's rotation and encodes the live rotation once it
/// does not. The content box fields `e_c`, `e_mi`, and `e_ma` are derived on
/// write, an object's from its tight bounds and a group's from its subtree.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxExtNode {
    /// Node UUID (`id`).
    pub id: String,

    /// Index triplet (`ind`). Voxel Max collapses nodes that share one.
    #[cfg_attr(feature = "serde", serde(rename = "ind"))]
    pub index: [i64; 3],

    /// Axis-angle rotation `[x, y, z, angle]` (`t_r`).
    #[cfg_attr(feature = "serde", serde(rename = "t_r"))]
    pub rotation: [f64; 4],

    /// Alignment enum token (`t_al`).
    #[cfg_attr(feature = "serde", serde(rename = "t_al"))]
    pub alignment: String,

    /// Pivot-face enum token (`t_pf`).
    #[cfg_attr(feature = "serde", serde(rename = "t_pf"))]
    pub pivot_face: String,

    /// Pivot-align enum token (`t_pa`).
    #[cfg_attr(feature = "serde", serde(rename = "t_pa"))]
    pub pivot_align: String,

    /// Selected UI flag (`s`).
    #[cfg_attr(
        feature = "serde",
        serde(rename = "s", default, skip_serializing_if = "Option::is_none")
    )]
    pub selected: Option<bool>,
}
