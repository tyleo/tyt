use crate::{
    MagicaVoxelCamera, MagicaVoxelLayer, MagicaVoxelMaterial, MagicaVoxelNode,
    MagicaVoxelUnknownChunk,
};
use serde::{Deserialize, Serialize};

/// The `magica-voxel` ext payload stashed on a [`VoxState`](voxcore::VoxState):
/// the MagicaVoxel `.vox` state with no native voxcore home, kept so a file
/// loaded from a MagicaVoxel package can be written back exactly.
///
/// Geometry, colors, and the scene graph become native voxcore entities; this
/// holds the rest, with the per-node and per-material entries aligned by index
/// with the hierarchy nodes and recorded materials so the file rebuilds exactly.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MagicaVoxelExt {
    /// The format version from the header (`version`).
    pub version: u32,

    /// Whether the file carried an `RGBA` palette chunk. When false the colors
    /// came from the built-in palette and no chunk is written back.
    #[serde(rename = "palette-present")]
    pub palette_present: bool,

    /// Per-material provenance, in stored order. The scalar and type fields fold
    /// into the palette cell named by `id`; this keeps the parts that do not.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<MagicaVoxelMaterial>,

    /// Per scene-node provenance, aligned by index with the hierarchy nodes.
    #[serde(rename = "scene-nodes", default, skip_serializing_if = "Vec::is_empty")]
    pub scene_nodes: Vec<MagicaVoxelNode>,

    /// The layer definitions (`LAYR`), preserved verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<MagicaVoxelLayer>,

    /// The render-settings chunks (`rOBJ`), each an ordered key/value list.
    #[serde(
        rename = "render-objects",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub render_objects: Vec<Vec<(String, String)>>,

    /// The render cameras (`rCAM`), preserved verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cameras: Vec<MagicaVoxelCamera>,

    /// The palette color names (`NOTE`), in stored order.
    #[serde(
        rename = "palette-notes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub palette_notes: Vec<String>,

    /// The palette index map (`IMAP`) as its 256 bytes, or `None` when the file
    /// omits it. Held as a `Vec` because serde does not derive for `[u8; 256]`.
    #[serde(rename = "index-map", default, skip_serializing_if = "Option::is_none")]
    pub index_map: Option<Vec<u8>>,

    /// Chunks the mvox crate does not model, preserved verbatim.
    #[serde(
        rename = "unknown-chunks",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub unknown_chunks: Vec<MagicaVoxelUnknownChunk>,
}
