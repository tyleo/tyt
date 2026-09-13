use crate::{
    Error, FALLBACK_PALETTE, PaletteLayout, PalettePlan, Result, VMaxVoxMain, check_emissive,
    color_palette_colors, ext_entry, slot_materials, voxel_indices,
};
use branded_id::U32Id;
use std::collections::BTreeMap;
use voxcore::{
    BVoxPalette,
    material::{BASE_COLOR, EMISSIVE_COLOR},
};

/// Plans the Voxel Max palette for `palette_id`. Voxel Max numbers palette
/// files 1-based (`palette1`, `palette2`, ...), by `colored_count`, since an
/// un-numbered `palette.png` breaks the plist color lookup when no image is
/// written. Errors when the palette has no ext entry or departs from the
/// [`PaletteLayout`]. A palette with materials but no `baseColor` errors
/// because Voxel Max keeps a material list only in a color palette's sidecar.
/// An `emissiveColor` with no material axis errors because its default
/// strength glows where Voxel Max's default materials do not.
pub(crate) fn new_palette_plan(
    main: &VMaxVoxMain,
    palette_id: U32Id<BVoxPalette>,
    colored_count: usize,
) -> Result<PalettePlan> {
    let provenance = ext_entry(
        main.ext().palettes.get(&palette_id),
        "palette",
        palette_id.to_u32(),
    )?;
    let layout = PaletteLayout::resolve(main, palette_id)?;

    let color_table = match &layout.color {
        Some(color) => Some(color_palette_colors(color.value_pool)?),
        None => None,
    };
    if color_table.is_none() && !layout.material.is_empty() {
        return Err(Error::invalid(format!(
            "palette {} binds materials but no `{BASE_COLOR}`, and Voxel Max keeps a material \
             list only in a color palette's sidecar",
            palette_id.to_u32()
        )));
    }
    if layout.material.is_empty() && layout.emissive_color.is_some() {
        return Err(Error::invalid(format!(
            "palette {} binds `{EMISSIVE_COLOR}` but no material property to carry its \
             strength, which Voxel Max's default materials would not glow at",
            palette_id.to_u32()
        )));
    }
    // An empty reference is one Voxel Max cannot resolve, so a colorless
    // palette borrows the default name and writes no file.
    let pal = match color_table {
        Some(_) => format!("palette{}.png", colored_count + 1),
        None => FALLBACK_PALETTE.to_owned(),
    };

    let materials = slot_materials(&layout, provenance)?;
    let mut indices = BTreeMap::new();
    for material_id in layout.palette.iter_materials() {
        let entry = voxel_indices(&layout, material_id)?;
        if let Some(material) = materials.get(usize::from(entry.material_idx)) {
            check_emissive(&layout, material_id, material.sic)?;
        }
        indices.insert(material_id, entry);
    }

    Ok(PalettePlan {
        pal,
        name: provenance.name.clone(),
        color_table,
        indices,
        materials,
    })
}
