use crate::QubicleQbtNode;
use serde::{Deserialize, Serialize};

/// The `qubicle-qbt` ext payload stashed on a [`VoxState`](voxcore::VoxState):
/// the Qubicle Binary Tree `.qbt` state with no native voxcore home, kept so a
/// file loaded from a `.qbt` package can be written back exactly.
///
/// Matrix and compound grids become native objects sharing one palette, and the
/// scene tree becomes the hierarchy nodes; this holds the rest, with the
/// per-node entries aligned by index with the hierarchy nodes.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct QubicleQbtExt {
    /// The `(major, minor)` version from the header.
    pub version: (u8, u8),

    /// The `[x, y, z]` global scale applied to the whole model.
    #[serde(rename = "global-scale")]
    pub global_scale: [f32; 3],

    /// The `COLORMAP` palette, in stored order, as `[r, g, b, a]` entries; empty
    /// when voxels store colors directly.
    #[serde(rename = "color-map", default, skip_serializing_if = "Vec::is_empty")]
    pub color_map: Vec<[u8; 4]>,

    /// Per scene-node provenance, aligned by index with the hierarchy nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<QubicleQbtNode>,
}
