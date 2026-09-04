//! Voxel Max's custom palette-attribute names. The rest of a Voxel Max
//! material maps onto the shared glTF vocabulary (`baseColor`,
//! `metallic`, `roughness`, `emissiveStrength`, `ior`,
//! `transmission`); these two have no glTF counterpart and stay custom, so
//! the reader and writer share one spelling.

/// Whether a material casts shadows, a custom bool attribute (Voxel Max `sh`).
pub const SHADOWS: &str = "shadows";

/// A material's dispersion absorption, a custom scalar attribute (Voxel Max
/// dispersion `a`).
pub const ABSORPTION: &str = "absorption";
