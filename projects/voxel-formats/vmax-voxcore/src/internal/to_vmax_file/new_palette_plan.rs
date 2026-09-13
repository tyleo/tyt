use crate::{
    FALLBACK_PALETTE, PaletteAxes, PalettePlan, Result, VMaxVoxMain, color_palette_colors,
    ext_entry, vmax_material,
};
use std::collections::HashMap;
use voxcore::VoxObject;

/// Starts the plan for `object`'s layers. A one-layer palette whose ext holds
/// the exact Voxel Max material list writes that list and the slots its
/// materials draw. Any other layering derives its materials from the samples.
/// Voxel Max numbers palette files 1-based (`palette1`, `palette2`, ...), by
/// `colored_count`, since an un-numbered `palette.png` breaks the plist color
/// lookup when no image is written. Errors when a layer's palette has no ext
/// entry.
pub(crate) fn new_palette_plan(
    main: &VMaxVoxMain,
    axes: &PaletteAxes,
    object: &VoxObject,
    colored_count: usize,
) -> Result<PalettePlan> {
    let mut name = String::new();
    for (_, palette_id) in object.iter_layers() {
        let provenance = ext_entry(
            main.ext().palettes.get(&palette_id),
            "palette",
            palette_id.to_u32(),
        )?;
        // The palette Voxel Max shows is the one the colors come from.
        if axes.color.map(|color| axes.property(color).palette_id()) == Some(palette_id) {
            name = provenance.name.clone();
        }
    }
    let color_table = match axes.color {
        Some(color) => Some(color_palette_colors(axes.property(color).value_pool())?),
        None => None,
    };
    // An empty reference is one Voxel Max cannot resolve, so a colorless
    // palette borrows the default name and writes no file.
    let pal = match color_table {
        Some(_) => format!("palette{}.png", colored_count + 1),
        None => FALLBACK_PALETTE.to_owned(),
    };

    let mut exact_slots = None;
    let mut materials = Vec::new();
    let layer_palette_ids: Vec<_> = object.iter_layers().map(|(_, id)| id).collect();
    if let [palette_id] = layer_palette_ids.as_slice()
        && !axes.material.is_empty()
    {
        let provenance = &main.ext().palettes[palette_id];
        if !provenance.materials.is_empty() {
            exact_slots = Some(provenance.slots.clone());
            materials = provenance
                .materials
                .iter()
                .enumerate()
                .map(|(slot, material)| vmax_material(slot, material))
                .collect();
        }
    }

    Ok(PalettePlan {
        pal,
        name,
        color_table,
        exact_slots,
        samples: HashMap::new(),
        slot_index_of: HashMap::new(),
        materials,
    })
}
