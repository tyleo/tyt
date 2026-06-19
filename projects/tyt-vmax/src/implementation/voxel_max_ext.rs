use crate::{VoxelMaxNode, VoxelMaxPalette, VoxelMaxScene};
use serde::{Deserialize, Serialize};
use vmax_codec::VXObjectStateSerde;

/// The `voxel-max` payload stored under the voxj document's generic `main.ext`
/// namespace. Holds the Voxel Max state with no native voxj representation so
/// `from-voxj` can rebuild a `.vmax` package exactly. The voxj crate itself
/// knows nothing of this shape; only this converter does.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VoxelMaxExt {
    /// Scene-level state (version, camera, renderer/UI settings).
    pub scene: VoxelMaxScene,
    /// Per-node provenance, aligned by index with the voxj `hierarchyNodes`
    /// (groups first, then objects, matching the converter's node order).
    #[serde(rename = "hierarchy-nodes")]
    pub hierarchy_nodes: Vec<VoxelMaxNode>,
    /// Per-palette provenance, aligned by index with the voxj `palettes`
    /// (`None` for color palettes, which need no extra data).
    pub palettes: Vec<Option<VoxelMaxPalette>>,
    /// Per-object Voxel Max editor state (`tools`/`brush`/`cam` and the content
    /// `uuid`/version), aligned by index with the voxj `main.objects`. Restored
    /// into each rebuilt `contents*.vmaxb` so Voxel Max can import the package.
    /// `None` for objects that had no `.vmaxb`.
    #[serde(
        rename = "object-states",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub object_states: Vec<Option<VXObjectStateSerde>>,
}
