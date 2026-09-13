/// The neutral default material Voxel Max fills unused slots with: matte, not
/// metallic, shadow-casting.
pub(crate) const DEFAULT_METALLIC: f64 = 0.1;

pub(crate) const DEFAULT_ROUGHNESS: f64 = 0.9;

/// The `pal` an object with no color palette borrows. An empty reference makes
/// Voxel Max read the package directory as a file and abort, so a colorless
/// object shares the first color palette's name and writes no file of its own.
pub(crate) const FALLBACK_PALETTE: &str = "palette1.png";

/// How far a node's rotation may drift from its preserved axis-angle before
/// the writer encodes the live rotation instead.
pub(crate) const ROTATION_TOLERANCE: f64 = 1e-9;
