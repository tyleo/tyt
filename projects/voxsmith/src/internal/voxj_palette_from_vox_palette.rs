use crate::voxj_value_from_vox_value;
use voxcore::VoxPalette;
use voxj::VoxjPalette;

/// Builds a [`VoxjPalette`] from a [`VoxPalette`], emitting attributes and cells
/// in id order so each lands at its original index.
pub(crate) fn voxj_palette_from_vox_palette(palette: &VoxPalette) -> VoxjPalette {
    let attributes = palette
        .attribute_ids
        .iter()
        .map(|id| unsafe { palette.attributes.get(id) }.clone())
        .collect();

    let data = palette
        .palette_cell_ids
        .iter()
        .map(|cell_id| {
            let cell = unsafe { palette.palette_cells.get(cell_id) };
            palette
                .attribute_ids
                .iter()
                .map(|attribute_id| voxj_value_from_vox_value(unsafe { cell.get(attribute_id) }))
                .collect()
        })
        .collect();

    VoxjPalette { attributes, data }
}
