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
