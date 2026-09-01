use crate::GltfRange;

/// The `[0, 1]` every component of a recommended color property lies in, per
/// the glTF schema of `baseColorFactor` and `emissiveFactor`.
pub(crate) const COLOR_RANGE: GltfRange = GltfRange {
    min: 0.0,
    max: Some(1.0),
    admits_zero: false,
};
