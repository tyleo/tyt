/// Usable colors in a Voxel Max palette. Color indices are 1-based: `color_idx`
/// is `cell + 1`, runs 1..=255, and 0 is the empty cell. Colors are stored
/// 0-based; a `palette*.png` appends a transparent terminator (256 entries),
/// the plist `colors` table does not (255 entries).
pub const PALETTE_COLORS: usize = 255;

/// The material slots every Voxel Max palette carries. A color cell's material
/// is a bit in the settings `lc` byte, so at most 8 (0..=7) fit; the sidecar
/// always lists exactly this many, real materials in the low slots and the rest
/// padded with the neutral default.
pub const MATERIAL_SLOTS: usize = 8;
