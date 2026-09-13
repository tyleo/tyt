use crate::{Error, PALETTE_COLORS, PaletteLayout, Result};
use branded_id::U32Id;
use voxcore::BVoxMaterial;

/// The 1-based color cell material `material_id` draws: its `baseColor` value
/// id plus one, since the color table is that value pool whole and cell 0 is
/// the empty cell. One when the palette binds no color. Errors when the cell
/// reaches [`PALETTE_COLORS`].
pub(crate) fn color_cell(layout: &PaletteLayout, material_id: U32Id<BVoxMaterial>) -> Result<u8> {
    let Some(color) = &layout.color else {
        return Ok(1);
    };
    let cell = layout
        .palette
        .value_id(material_id, color.id)
        .expect("a live material has a value id for every property")
        .to_u32();
    if cell >= PALETTE_COLORS as u32 {
        return Err(Error::invalid(format!(
            "material {} draws color cell {cell}, but a Voxel Max palette holds only \
             {PALETTE_COLORS} colors",
            material_id.to_u32()
        )));
    }
    Ok(cell as u8 + 1)
}
