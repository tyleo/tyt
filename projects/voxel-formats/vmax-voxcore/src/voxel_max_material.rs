use crate::VoxelMaxMaterialDispersion;
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// One Voxel Max material's exact coefficients, preserved in the `voxel-max`
/// ext, aligned by index with a palette's material list.
///
/// The folded palette's value pools carry a neutral copy of these for a
/// cross-format consumer, but a material's optional dispersion and a NaN
/// coefficient cannot ride in a value pool, so the exact material is kept here
/// and read back on write. The `mi` token is re-derived
/// from the slot and the transparency color `tc` is dropped, matching the
/// writer's behavior.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct VoxelMaxMaterial {
    /// Metalness coefficient (Voxel Max `mc`).
    pub metallic: f64,

    /// Roughness coefficient (Voxel Max `rc`).
    pub roughness: f64,

    /// Self-illumination coefficient (Voxel Max `sic`).
    pub emissive: f64,

    /// Whether the material casts shadows (Voxel Max `sh`).
    pub shadows: bool,

    /// Transmission color coefficient (Voxel Max `tc`), present on some
    /// materials such as MagicaVoxel exports. It has no palette representation,
    /// so it rides here to round-trip exactly.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub transmission_color: Option<f64>,

    /// Dispersion parameters, present on some materials.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dispersion: Option<VoxelMaxMaterialDispersion>,
}
