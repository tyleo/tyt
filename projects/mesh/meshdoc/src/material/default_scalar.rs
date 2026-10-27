use crate::material::{
    ALPHA_CUTOFF, ALPHA_CUTOFF_DEFAULT, EMISSIVE_STRENGTH, EMISSIVE_STRENGTH_DEFAULT, IOR,
    IOR_DEFAULT, METALLIC, METALLIC_DEFAULT, NORMAL_SCALE, NORMAL_SCALE_DEFAULT,
    OCCLUSION_STRENGTH, OCCLUSION_STRENGTH_DEFAULT, ROUGHNESS, ROUGHNESS_DEFAULT, TRANSMISSION,
    TRANSMISSION_DEFAULT,
};

/// The standard default for a scalar property, its `*_DEFAULT` constant, or
/// `None` for a key with no standard default, such as a custom property.
pub fn default_scalar(key: &str) -> Option<f64> {
    match key {
        METALLIC => Some(METALLIC_DEFAULT),
        ROUGHNESS => Some(ROUGHNESS_DEFAULT),
        OCCLUSION_STRENGTH => Some(OCCLUSION_STRENGTH_DEFAULT),
        NORMAL_SCALE => Some(NORMAL_SCALE_DEFAULT),
        EMISSIVE_STRENGTH => Some(EMISSIVE_STRENGTH_DEFAULT),
        IOR => Some(IOR_DEFAULT),
        TRANSMISSION => Some(TRANSMISSION_DEFAULT),
        ALPHA_CUTOFF => Some(ALPHA_CUTOFF_DEFAULT),
        _ => None,
    }
}
