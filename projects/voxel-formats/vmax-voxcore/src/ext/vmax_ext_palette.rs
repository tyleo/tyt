use crate::VMaxExtMaterial;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per-palette Voxel Max provenance preserved in the `vmax` ext: a material
/// palette's display name and its exact material list, by slot.
///
/// The palette's value pools carry the materials on the shared vocabulary,
/// which narrows metalness and roughness to the slider span and has no place
/// for a transmission color, an absent dispersion, or a NaN. The exact list is
/// kept here for a byte-exact write-back. A palette with no exact list, such
/// as a color-only one or one another format produced, writes its materials
/// from its value pools.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxExtPalette {
    /// Display name (Voxel Max `name`).
    pub name: String,

    /// The exact materials, aligned by index with the material list a voxel's
    /// `material_idx` selects. Empty for a palette with no exact list.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub materials: Vec<VMaxExtMaterial>,
}
