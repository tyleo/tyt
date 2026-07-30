use serde::{Deserialize, Serialize};

/// Per-material provenance in the `magica-voxel` ext: the authoritative type
/// token and scalar fields written back to the `MATL` chunk. They also fold
/// into the palette's value pools, which default an absent field, so the
/// exact optionals are kept here.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MagicaVoxelMaterial {
    /// The material id, which is the material index it folds into.
    pub id: i32,

    /// The `_type` shading token.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub material_type: Option<String>,

    /// The `_weight` scalar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,

    /// The `_rough` scalar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rough: Option<f32>,

    /// The `_spec` scalar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<f32>,

    /// The `_ior` scalar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ior: Option<f32>,

    /// The `_att` scalar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub att: Option<f32>,

    /// The `_flux` scalar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flux: Option<f32>,

    /// Any further property keys, preserved verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String)>,
}
