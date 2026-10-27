/// The base color, a color property.
pub const BASE_COLOR: &str = "baseColor";

/// Metalness, a `0..1` scalar.
pub const METALLIC: &str = "metallic";

/// The standard default for [`METALLIC`].
pub const METALLIC_DEFAULT: f64 = 1.0;

/// Roughness, a `0..1` scalar.
pub const ROUGHNESS: &str = "roughness";

/// The standard default for [`ROUGHNESS`].
pub const ROUGHNESS_DEFAULT: f64 = 1.0;

/// Ambient-occlusion strength, a `0..1` scalar.
pub const OCCLUSION_STRENGTH: &str = "occlusionStrength";

/// The standard default for [`OCCLUSION_STRENGTH`].
pub const OCCLUSION_STRENGTH_DEFAULT: f64 = 1.0;

/// Transmission, a `0..1` scalar.
pub const TRANSMISSION: &str = "transmission";

/// The standard default for [`TRANSMISSION`].
pub const TRANSMISSION_DEFAULT: f64 = 0.0;

/// Index of refraction: `0` for "does not refract", else a `1..` scalar.
pub const IOR: &str = "ior";

/// The standard default for [`IOR`].
pub const IOR_DEFAULT: f64 = 1.5;

/// The emissive color, a color property with no alpha.
pub const EMISSIVE_COLOR: &str = "emissiveColor";

/// Emissive strength scaling [`EMISSIVE_COLOR`], a `0..` scalar.
pub const EMISSIVE_STRENGTH: &str = "emissiveStrength";

/// The standard default for [`EMISSIVE_STRENGTH`].
pub const EMISSIVE_STRENGTH_DEFAULT: f64 = 1.0;
