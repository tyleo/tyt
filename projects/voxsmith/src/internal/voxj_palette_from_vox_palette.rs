use crate::voxj_value_from_vox_value;
use voxcore::VoxPalette;
use voxj::VoxjPalette;

/// Builds a [`VoxjPalette`] from a [`VoxPalette`], emitting attributes and cells
/// in id order so each lands at its original index.
pub(crate) fn voxj_palette_from_vox_palette(palette: &VoxPalette) -> VoxjPalette {
    // Attribute ids, reused for each cell's values.
    let attribute_ids: Vec<_> = palette.iter_attributes().map(|(id, _)| id).collect();
    let attributes = palette
        .iter_attributes()
        .map(|(_, name)| name.to_owned())
        .collect();

    let data = palette
        .iter_cells()
        .map(|cell_id| {
            attribute_ids
                .iter()
                .map(|&attribute_id| {
                    let value = palette
                        .cell_value(cell_id, attribute_id)
                        .expect("a cell has a value for every attribute");
                    voxj_value_from_vox_value(value)
                })
                .collect()
        })
        .collect();

    VoxjPalette { attributes, data }
}
