use crate::vox_value_from_voxj_value;
use branded_id::soa::IdField;
use std::io;
use voxcore::VoxPalette;
use voxj::VoxjPalette;

/// Builds a [`VoxPalette`] from a [`VoxjPalette`], assigning attribute ids and
/// cell ids in listing order so each equals its voxj index. Errors if a cell
/// row's length does not match the attribute count.
pub(crate) fn vox_palette_from_voxj_palette(palette: &VoxjPalette) -> io::Result<VoxPalette> {
    let mut out = VoxPalette::default();

    for attribute in &palette.attributes {
        let id = out.attribute_ids.retain();
        out.attributes.retain(id, attribute.clone());
    }

    for (index, row) in palette.data.iter().enumerate() {
        if row.len() != palette.attributes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "palette cell {index} has {} values but the palette has {} attributes",
                    row.len(),
                    palette.attributes.len()
                ),
            ));
        }

        let cell_id = out.palette_cell_ids.retain();
        let mut cell = IdField::new();
        for attribute_id in &out.attribute_ids {
            let value = &row[attribute_id.to_u32() as usize];
            cell.retain(attribute_id, vox_value_from_voxj_value(value));
        }
        out.palette_cells.retain(cell_id, cell);
    }

    Ok(out)
}
