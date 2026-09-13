use crate::{
    Error, MATERIAL_SLOTS, PaletteLayout, Result, VMaxExtPalette, pool_material, vmax_material,
};
use branded_id::U32Id;
use vmax::VMaxMaterial;

/// The material list a palette writes, in slot order: an exact list in
/// `provenance` as it is, else one material per slot read from the
/// material-axis pools. Empty when the palette binds no material axis,
/// leaving Voxel Max its own defaults.
///
/// Every material-axis pool holds one value per slot, so the pools must be
/// the same length, densely numbered from zero, and no longer than
/// [`MATERIAL_SLOTS`]. An exact list must be that length too. Anything else
/// errors, including a pruned pool not yet compacted.
pub(crate) fn slot_materials(
    layout: &PaletteLayout,
    provenance: &VMaxExtPalette,
) -> Result<Vec<VMaxMaterial>> {
    let Some(first) = layout.material.first() else {
        if !provenance.materials.is_empty() {
            return Err(Error::invalid(format!(
                "vmax ext lists {} exact materials, but the palette binds no material property \
                 to index them",
                provenance.materials.len()
            )));
        }
        return Ok(Vec::new());
    };
    let slot_count = first.value_pool.len();
    for property in &layout.material {
        let dense = property.value_pool.len() == slot_count
            && (0..slot_count).all(|slot| {
                property
                    .value_pool
                    .contains_value(U32Id::from_u32(slot as u32))
            });
        if !dense {
            return Err(Error::invalid(format!(
                "`{}` holds {} values where `{}` holds {slot_count}, but a Voxel Max material \
                 pool holds one value per slot, numbered from zero",
                property.name,
                property.value_pool.len(),
                first.name
            )));
        }
    }
    if slot_count > MATERIAL_SLOTS {
        return Err(Error::invalid(format!(
            "the material pools hold {slot_count} values, but a Voxel Max palette holds only \
             {MATERIAL_SLOTS} material slots"
        )));
    }
    if !provenance.materials.is_empty() {
        if provenance.materials.len() != slot_count {
            return Err(Error::invalid(format!(
                "vmax ext lists {} exact materials, but the material pools hold {slot_count} \
                 values",
                provenance.materials.len()
            )));
        }
        return Ok(provenance
            .materials
            .iter()
            .enumerate()
            .map(|(slot, material)| vmax_material(slot, material))
            .collect());
    }
    (0..slot_count)
        .map(|slot| pool_material(layout, slot as u8))
        .collect()
}
