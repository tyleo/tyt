use crate::{FALLBACK_PALETTE, PalettePlan};
use std::collections::BTreeMap;

/// The plan every object with no layer shares. It borrows the default palette
/// name because Voxel Max cannot resolve an empty `pal`, and writes no file.
pub(crate) fn colorless_plan() -> PalettePlan {
    PalettePlan {
        pal: FALLBACK_PALETTE.to_owned(),
        name: String::new(),
        color_table: None,
        indices: BTreeMap::new(),
        materials: Vec::new(),
    }
}
