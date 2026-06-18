use crate::{VoxelMaxNode, VoxelMaxPalette, VoxelMaxScene};
use serde::{Deserialize, Serialize};

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
}
