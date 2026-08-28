use crate::VoxelMaxMaterial;
use serde::{Deserialize, Serialize};

/// Per-palette Voxel Max provenance preserved in the `voxel-max` ext, kept
/// aligned by index with the palettes: a material palette's display name and
/// its exact material list, neither of which the folded voxcore palette carries
/// losslessly.
///
/// The palette folds each voxel's color and material into one material per
/// distinct color-plus-material combination, and its value pools carry a
/// finite-defaulted neutral copy of the materials, so the exact list is kept
/// here for a byte-exact write-back. It is empty for a color-only palette.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VoxelMaxPalette {
    /// Display name (Voxel Max `name`).
    pub name: String,

    /// The exact materials, aligned by index with the material list a voxel's
    /// `material_idx` selects. Empty for a color-only palette.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<VoxelMaxMaterial>,
}
