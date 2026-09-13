use crate::material::{
    ALPHA_CUTOFF, EMISSIVE_STRENGTH, IOR, METALLIC, MaterialRange, NORMAL_SCALE,
    OCCLUSION_STRENGTH, ROUGHNESS, TRANSMISSION,
};

/// The range the model holds a scalar property to, or `None` for a key
/// outside the vocabulary. `ior`'s union of exactly `0` and `1..` rides
/// [`MaterialRange::admits_zero`].
pub fn scalar_range(key: &str) -> Option<MaterialRange> {
    match key {
        METALLIC | ROUGHNESS | OCCLUSION_STRENGTH | TRANSMISSION => Some(MaterialRange {
            min: 0.0,
            max: Some(1.0),
            admits_zero: false,
        }),
        NORMAL_SCALE | EMISSIVE_STRENGTH | ALPHA_CUTOFF => Some(MaterialRange {
            min: 0.0,
            max: None,
            admits_zero: false,
        }),
        IOR => Some(MaterialRange {
            min: 1.0,
            max: None,
            admits_zero: true,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::material::{IOR, METALLIC, scalar_range};

    #[test]
    fn the_vocabulary_has_ranges_and_a_custom_key_has_none() {
        assert!(scalar_range(METALLIC).unwrap().contains(1.0));
        assert!(!scalar_range(METALLIC).unwrap().contains(1.5));
        assert!(scalar_range(IOR).unwrap().contains(0.0));
        assert_eq!(scalar_range("subsurface"), None);
    }
}
