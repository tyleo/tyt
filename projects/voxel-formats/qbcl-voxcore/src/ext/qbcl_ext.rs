use crate::ext::{QbclExtMetadata, QbclExtNode, QbclExtThumbnail};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use voxcore::VoxExt;

/// The `qbcl` ext payload stashed on a [`VoxMain`](voxcore::VoxMain):
/// the Qubicle Construction Library `.qbcl` state with no native voxcore home,
/// kept so a file loaded from a `.qbcl` package can be written back exactly.
///
/// Matrix and compound grids become native objects sharing one palette, and the
/// scene tree becomes the hierarchy nodes; this holds the rest, with the
/// per-node entries aligned by index with the hierarchy nodes. The nodes
/// follow the state through the [`VoxExt`](voxcore::VoxExt) hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct QbclExt {
    /// The version of Qubicle that wrote the file, packed.
    #[cfg_attr(feature = "serde", serde(rename = "program-version"))]
    pub program_version: u32,

    /// The file-format version.
    #[cfg_attr(feature = "serde", serde(rename = "file-version"))]
    pub file_version: u32,

    /// The preview thumbnail from the header.
    #[cfg_attr(feature = "serde", serde(default))]
    pub thumbnail: QbclExtThumbnail,

    /// The seven free-text metadata strings.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: QbclExtMetadata,

    /// The 16-byte header chunk of unconfirmed purpose, preserved verbatim.
    pub guid: [u8; 16],

    /// Per scene-node provenance, aligned by index with the hierarchy nodes.
    /// A node retained after the load has `None`. The writer fills it in like
    /// a synthesized node.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub nodes: Vec<Option<QbclExtNode>>,
}

/// The Qubicle Construction Library ext as a state's ext. The nodes follow
/// the hierarchy listing. A retained node takes no entry. The writer fills it
/// in like a synthesized node. The ext lists nothing per object. The writer
/// reaches a node's object through the hierarchy. The masks do not follow
/// the voxel hooks. A repaint retains a live voxel again and fires the same
/// hook as a new voxel, so the list cannot tell them apart.
impl VoxExt for QbclExt {
    fn hierarchy_node_did_retain(&mut self, index: usize) {
        self.nodes.insert(index, None);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        self.nodes.remove(index);
    }
}
