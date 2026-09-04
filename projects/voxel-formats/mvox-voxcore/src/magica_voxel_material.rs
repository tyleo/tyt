#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// Per-material provenance in the `magica-voxel` ext: the authoritative type
/// token and scalar fields written back to the `MATL` chunk. They also fold
/// into the palette's value pools, which default an absent field, so the exact
/// optionals are kept here.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct MagicaVoxelMaterial {
    /// The material id, which is the material index it folds into.
    pub id: i32,

    /// The `_type` shading token.
    #[cfg_attr(
        feature = "ext",
        serde(rename = "type", default, skip_serializing_if = "Option::is_none")
    )]
    pub material_type: Option<String>,

    /// The `_weight` scalar.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub weight: Option<f32>,

    /// The `_rough` scalar.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub rough: Option<f32>,

    /// The `_spec` scalar.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub spec: Option<f32>,

    /// The `_ior` scalar.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ior: Option<f32>,

    /// The `_att` scalar.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub att: Option<f32>,

    /// The `_flux` scalar.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub flux: Option<f32>,

    /// Any further property keys, preserved verbatim.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub extra: Vec<(String, String)>,
}
