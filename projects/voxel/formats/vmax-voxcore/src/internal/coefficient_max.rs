use crate::COEFFICIENT_MIN;

/// The metalness or roughness coefficient a Voxel Max slider stores at 100
/// percent. Voxel Max sums 0.1 and 0.8 in single precision. The sum lands one
/// step above the nearest single to 0.9.
pub const COEFFICIENT_MAX: f32 = COEFFICIENT_MIN + 0.8;
