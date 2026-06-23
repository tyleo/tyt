use crate::{MVoxDict, MVoxMaterialType};

/// A material definition (`MATL`). The documented property keys are lifted into
/// fields; any other keys are preserved in [`extra`](Self::extra).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MVoxMaterial {
    /// The material id this chunk defines; it matches a palette index.
    pub id: i32,

    /// `_type`: the shading model.
    pub material_type: Option<MVoxMaterialType>,

    /// `_weight`: blend weight, in `0..=1`.
    pub weight: Option<f32>,

    /// `_rough`: roughness.
    pub rough: Option<f32>,

    /// `_spec`: specular.
    pub spec: Option<f32>,

    /// `_ior`: index of refraction.
    pub ior: Option<f32>,

    /// `_att`: attenuation.
    pub att: Option<f32>,

    /// `_flux`: radiant flux (emission strength).
    pub flux: Option<f32>,

    /// Any further property keys, preserved verbatim.
    pub extra: MVoxDict,
}
