use crate::operations::sdf_doc::SdfCustomValue;
use std::collections::BTreeMap;
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};

/// One material's properties as a voxj palette binds them. A named property
/// the material leaves out holds its default.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfMaterialProperties {
    /// The base color in linear light with straight alpha.
    pub base_color: TyLinSrgbaF64,

    /// The emitted color in linear light.
    pub emissive_color: TyLinSrgbF64,

    /// The multiplier on the emitted color.
    pub emissive_strength: f64,

    /// The index of refraction.
    pub ior: f64,

    /// The metalness.
    pub metallic: f64,

    /// The strength of the flat ambient occlusion.
    pub occlusion_strength: f64,

    /// The roughness.
    pub roughness: f64,

    /// The share of light passing through the surface.
    pub transmission: f64,

    /// Every custom property the model's materials set, by name. A material
    /// that leaves one out holds the kind's empty value.
    pub custom: BTreeMap<String, SdfCustomValue>,
}
