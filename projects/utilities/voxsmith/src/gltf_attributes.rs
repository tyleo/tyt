//! The shared glTF metallic-roughness attribute vocabulary. Every converter, the
//! glTF pipeline, and the CLI bind and read palette attributes by these names, so
//! one table keeps producers and consumers in agreement. voxcore attribute names
//! stay free strings, so a format's custom attributes remain expressible; these
//! constants name only the recommended glTF-aligned set.

/// The base color, a color attribute. glTF `baseColorFactor`.
pub const BASE_COLOR_FACTOR: &str = "baseColorFactor";

/// Metalness, a `0..1` scalar. glTF `metallicFactor`.
pub const METALLIC_FACTOR: &str = "metallicFactor";

/// Roughness, a `0..1` scalar. glTF `roughnessFactor`.
pub const ROUGHNESS_FACTOR: &str = "roughnessFactor";

/// Ambient-occlusion strength, a `0..1` scalar. glTF `occlusionStrength`.
pub const OCCLUSION_STRENGTH: &str = "occlusionStrength";

/// Transmission, a `0..1` scalar. glTF `transmissionFactor`.
pub const TRANSMISSION_FACTOR: &str = "transmissionFactor";

/// Index of refraction, a `1..` scalar. glTF `ior`.
pub const IOR: &str = "ior";

/// The emissive color, a color attribute with no alpha. glTF `emissiveFactor`.
pub const EMISSIVE_FACTOR: &str = "emissiveFactor";

/// Emissive strength scaling [`EMISSIVE_FACTOR`], a `0..` scalar. glTF's
/// `KHR_materials_emissive_strength` `emissiveStrength`.
pub const EMISSIVE_STRENGTH: &str = "emissiveStrength";

/// The glTF spec default for a recommended scalar attribute, or `None` for a key
/// with no standard default, such as a custom attribute.
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

/// The range the glTF spec gives a recommended scalar attribute, as an
/// inclusive `(min, max)` with a `max` of `None` for an unbounded top, or `None`
/// for a key outside the vocabulary.
pub(crate) fn scalar_range(key: &str) -> Option<(f64, Option<f64>)> {
    match key {
        METALLIC_FACTOR | ROUGHNESS_FACTOR | OCCLUSION_STRENGTH | TRANSMISSION_FACTOR => {
            Some((0.0, Some(1.0)))
        }
        EMISSIVE_STRENGTH => Some((0.0, None)),
        IOR => Some((1.0, None)),
        _ => None,
    }
}

/// The glTF spec default for a recommended color attribute, as sRGB `[r, g, b,
/// a]` bytes, or `None` for a key with no standard default, such as a custom
/// attribute. A three-component color takes opaque alpha.
#[cfg(feature = "gltf")]
pub(crate) fn default_color(key: &str) -> Option<[u8; 4]> {
    match key {
        BASE_COLOR_FACTOR => Some([255, 255, 255, 255]),
        EMISSIVE_FACTOR => Some([0, 0, 0, 255]),
        _ => None,
    }
}
