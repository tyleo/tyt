//! The shared glTF metallic-roughness attribute vocabulary. Every converter, the
//! glTF pipeline, and the CLI bind and read palette attributes by these names, so
//! one table keeps producers and consumers in agreement. voxcore attribute names
//! stay free strings, so a format's custom attributes remain expressible; these
//! constants name only the recommended glTF-aligned set.

use crate::GltfRange;
#[cfg(feature = "gltf")]
use ty_math::TyLinSrgbaF64;

/// The base color, a color attribute. glTF `baseColorFactor`.
pub const BASE_COLOR: &str = "baseColor";

/// Metalness, a `0..1` scalar. glTF `metallicFactor`.
pub const METALLIC: &str = "metallic";

/// Roughness, a `0..1` scalar. glTF `roughnessFactor`.
pub const ROUGHNESS: &str = "roughness";

/// Ambient-occlusion strength, a `0..1` scalar. glTF `occlusionStrength`.
pub const OCCLUSION_STRENGTH: &str = "occlusionStrength";

/// Transmission, a `0..1` scalar. glTF `transmissionFactor`.
pub const TRANSMISSION: &str = "transmission";

/// Index of refraction: `0` for "does not refract", else a `1..` scalar.
/// glTF `ior`.
pub const IOR: &str = "ior";

/// The emissive color, a color attribute with no alpha. glTF `emissiveFactor`.
pub const EMISSIVE_COLOR: &str = "emissiveColor";

/// Emissive strength scaling [`EMISSIVE_COLOR`], a `0..` scalar. glTF's
/// `KHR_materials_emissive_strength` `emissiveStrength`.
pub const EMISSIVE_STRENGTH: &str = "emissiveStrength";

/// The glTF spec default for a recommended scalar attribute, or `None` for a key
/// with no standard default, such as a custom attribute.
pub(crate) fn default_scalar(key: &str) -> Option<f64> {
    match key {
        METALLIC => Some(1.0),
        ROUGHNESS => Some(1.0),
        OCCLUSION_STRENGTH => Some(1.0),
        EMISSIVE_STRENGTH => Some(1.0),
        IOR => Some(1.5),
        TRANSMISSION => Some(0.0),
        _ => None,
    }
}

/// The range the glTF spec gives a recommended scalar attribute, or `None`
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

/// The `[0, 1]` every component of a recommended color attribute lies in, per
/// the glTF schema of `baseColorFactor` and `emissiveFactor`.
pub(crate) const COLOR_RANGE: GltfRange = GltfRange {
    min: 0.0,
    max: Some(1.0),
    admits_zero: false,
};

/// The glTF spec default for a recommended color attribute, the linear factor
/// the glTF schema states, or `None` for a key with no standard default, such
/// as a custom attribute. A three-component color takes opaque alpha.
#[cfg(feature = "gltf")]
pub(crate) fn default_lin_srgba_f64_color(key: &str) -> Option<TyLinSrgbaF64> {
    match key {
        BASE_COLOR => Some(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0)),
        EMISSIVE_COLOR => Some(TyLinSrgbaF64::new(0.0, 0.0, 0.0, 1.0)),
        _ => None,
    }
}
