use crate::VMaxExtMaterial;
use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use voxcore::BVoxMaterial;

/// Per-palette Voxel Max provenance preserved in the `vmax` ext: a material
/// palette's display name, its exact material list, and the slot each folded
/// material draws. The folded voxcore palette carries none of them losslessly.
///
/// The palette folds each voxel's color and material into one material per
/// distinct color-plus-material combination, and its value pools carry a
/// finite-defaulted neutral copy of the materials, so the exact list is kept
/// here for a byte-exact write-back. A palette with no exact list, a
/// color-only one or one another format produced, writes materials derived
/// from its value pools and keeps no slots.
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

    /// The slot in `materials` each folded material draws, by material id.
    /// Empty when `materials` is.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "material-slots",
            default,
            skip_serializing_if = "BTreeMap::is_empty"
        )
    )]
    pub slots: BTreeMap<U32Id<BVoxMaterial>, u8>,
}
