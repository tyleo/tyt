use crate::ext::QbtExtNode;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use voxcore::VoxExt;

/// The `qbt` ext payload stashed on a [`VoxMain`](voxcore::VoxMain):
/// the Qubicle Binary Tree `.qbt` state with no native voxcore home, kept so a
/// file loaded from a `.qbt` package can be written back exactly.
///
/// Matrix and compound grids become native objects sharing one palette, and the
/// scene tree becomes the hierarchy nodes; this holds the rest, with the
/// per-node entries aligned by index with the hierarchy nodes. The nodes
/// follow the state through the [`VoxExt`](voxcore::VoxExt) hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct QbtExt {
    /// The `(major, minor)` version from the header.
    pub version: (u8, u8),

    /// The `[x, y, z]` global scale applied to the whole model.
    #[cfg_attr(feature = "serde", serde(rename = "global-scale"))]
    pub global_scale: [f32; 3],

    /// The `COLORMAP` palette, in stored order, as `[r, g, b, a]` entries;
    /// empty when voxels store colors directly.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "color-map", default, skip_serializing_if = "Vec::is_empty")
    )]
    pub color_map: Vec<[u8; 4]>,

    /// Per scene-node provenance, aligned by index with the hierarchy nodes.
    /// A node retained after the load has `None`. The writer fills it in like
    /// a synthesized node.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub nodes: Vec<Option<QbtExtNode>>,
}

/// The Qubicle Binary Tree ext as a state's ext. The nodes follow the
/// hierarchy listing. A retained node takes no entry. The writer fills it in
/// like a synthesized node. The ext lists nothing per object. The writer
/// reaches a node's object through the hierarchy. The masks do not follow
/// the voxel hooks. A repaint retains a live voxel again and fires the same
/// hook as a new voxel, so the list cannot tell them apart.
impl VoxExt for QbtExt {
    fn hierarchy_node_did_retain(&mut self, index: usize) {
        self.nodes.insert(index, None);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        self.nodes.remove(index);
    }
}
