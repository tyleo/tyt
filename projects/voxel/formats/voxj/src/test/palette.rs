use crate::{VoxjPalette, VoxjProperty};

/// A palette of `materials` materials: one property binding `baseColor` to
/// value pool 0, its rows the value-indices `0..materials`.
pub fn palette(materials: usize) -> VoxjPalette {
    VoxjPalette {
        properties: vec![VoxjProperty {
            name: "baseColor".to_owned(),
            value_pool: 0,
        }],
        materials: (0..materials).map(|i| vec![i]).collect(),
    }
}
