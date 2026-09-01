use crate::GltfRange;
use voxcore::material::{
    EMISSIVE_STRENGTH, IOR, METALLIC, OCCLUSION_STRENGTH, ROUGHNESS, TRANSMISSION,
};

/// The range the glTF spec gives a recommended scalar property, or `None`
/// for a key outside the vocabulary. `ior`'s union of exactly `0` and
/// `[1, inf)` rides [`GltfRange::admits_zero`].
pub(crate) fn scalar_range(key: &str) -> Option<GltfRange> {
    match key {
        METALLIC | ROUGHNESS | OCCLUSION_STRENGTH | TRANSMISSION => Some(GltfRange {
            min: 0.0,
            max: Some(1.0),
            admits_zero: false,
        }),
        EMISSIVE_STRENGTH => Some(GltfRange {
            min: 0.0,
            max: None,
            admits_zero: false,
        }),
        IOR => Some(GltfRange {
            min: 1.0,
            max: None,
            admits_zero: true,
        }),
        _ => None,
    }
}
