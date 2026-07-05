use crate::{
    EMISSIVE_STRENGTH, IOR, METALLIC_FACTOR, OCCLUSION_STRENGTH, ROUGHNESS_FACTOR,
    TRANSMISSION_FACTOR,
};

/// The spec default for a recommended scalar attribute, so a map never fails on
/// a material that omits it. A key with no standard default (a custom attribute)
/// returns `None`, which the bake reads as `0`.
pub(crate) fn default_scalar(key: &str) -> Option<f64> {
    match key {
        METALLIC_FACTOR => Some(1.0),
        ROUGHNESS_FACTOR => Some(1.0),
        OCCLUSION_STRENGTH => Some(1.0),
        EMISSIVE_STRENGTH => Some(1.0),
        IOR => Some(1.5),
        TRANSMISSION_FACTOR => Some(0.0),
        _ => None,
    }
}
