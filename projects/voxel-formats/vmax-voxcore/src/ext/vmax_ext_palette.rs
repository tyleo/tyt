use crate::ext::VMaxExtMaterial;
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// Per-palette Voxel Max provenance preserved in the `vmax` ext, kept
/// aligned by index with the palettes: a material palette's display name, its
/// exact material list, and the slot each folded material draws. The folded
/// voxcore palette carries none of them losslessly.
///
/// The palette folds each voxel's color and material into one material per
/// distinct color-plus-material combination, and its value pools carry a
/// finite-defaulted neutral copy of the materials, so the exact list is kept
/// here for a byte-exact write-back. It is empty for a color-only palette.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct VMaxExtPalette {
    /// Display name (Voxel Max `name`).
    pub name: String,

    /// The exact materials, aligned by index with the material list a voxel's
    /// `material_idx` selects. Empty for a color-only palette.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub materials: Vec<VMaxExtMaterial>,

    /// The slot in `materials` each folded material draws, aligned by index
    /// with the palette's materials and kept in step by the material hooks.
    /// All zero for a color-only palette.
    #[cfg_attr(feature = "ext", serde(rename = "material-slots"))]
    pub slots: Vec<u8>,
}
