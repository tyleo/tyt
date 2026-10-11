/// The material slots every Voxel Max palette carries. A color cell's material
/// is a bit in the settings `lc` byte, so at most 8 (0..=7) fit; the sidecar
/// always lists exactly this many, real materials in the low slots and the rest
/// padded with the neutral default.
pub const MATERIAL_SLOTS: usize = 8;
