use serde::{Deserialize, Serialize};

/// Provenance for one Voxel Max material palette, aligned by index with the voxj
/// `palettes`. Carries only what the voxj palette can't represent itself: the
/// palette's display name. The materials (metalness/roughness/emission/shadows)
/// live natively in the voxj palette cells.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VoxelMaxPalette {
    /// Display name (Voxel Max `name`).
    pub name: String,
}
