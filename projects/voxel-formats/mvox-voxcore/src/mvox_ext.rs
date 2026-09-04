use crate::{MVoxExtCamera, MVoxExtLayer, MVoxExtMaterial, MVoxExtNode, MVoxExtUnknownChunk};
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// The `mvox` ext payload stashed on a [`VoxMain`](voxcore::VoxMain):
/// the MagicaVoxel `.vox` state with no native voxcore home, kept so a file
/// loaded from a MagicaVoxel package can be written back exactly.
///
/// Geometry, colors, and the scene graph become native voxcore entities; this
/// holds the rest, with the per-node and per-material entries aligned by index
/// with the hierarchy nodes and recorded materials so the file rebuilds
/// exactly.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct MVoxExt {
    /// The format version from the header (`version`).
    pub version: u32,

    /// Whether the file carried an `RGBA` palette chunk. When false the colors
    /// came from the built-in palette and no chunk is written back.
    #[cfg_attr(feature = "ext", serde(rename = "palette-present"))]
    pub palette_present: bool,

    /// Per-material provenance, in stored order: the authoritative type and
    /// scalar fields for write-back.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub materials: Vec<MVoxExtMaterial>,

    /// Per scene-node provenance, aligned by index with the hierarchy nodes.
    #[cfg_attr(
        feature = "ext",
        serde(rename = "scene-nodes", default, skip_serializing_if = "Vec::is_empty")
    )]
    pub scene_nodes: Vec<MVoxExtNode>,

    /// The layer definitions (`LAYR`), preserved verbatim.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub layers: Vec<MVoxExtLayer>,

    /// The render-settings chunks (`rOBJ`), each an ordered key/value list.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "render-objects",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub render_objects: Vec<Vec<(String, String)>>,

    /// The render cameras (`rCAM`), preserved verbatim.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub cameras: Vec<MVoxExtCamera>,

    /// The palette color names (`NOTE`), in stored order.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "palette-notes",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub palette_notes: Vec<String>,

    /// The palette index map (`IMAP`) as its 256 bytes, or `None` when the file
    /// omits it. Held as a `Vec` because serde does not derive for `[u8; 256]`.
    #[cfg_attr(
        feature = "ext",
        serde(rename = "index-map", default, skip_serializing_if = "Option::is_none")
    )]
    pub index_map: Option<Vec<u8>>,

    /// Chunks the mvox crate does not model, preserved verbatim.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "unknown-chunks",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub unknown_chunks: Vec<MVoxExtUnknownChunk>,
}
