use serde::{Deserialize, Serialize};

/// Per-palette Voxel Max provenance preserved in the `voxel-max` ext: a material
/// palette's display name, which the voxcore palette cells do not carry, kept
/// aligned by index with the palettes.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct VoxelMaxPalette {
    /// Display name (Voxel Max `name`).
    pub name: String,
}
