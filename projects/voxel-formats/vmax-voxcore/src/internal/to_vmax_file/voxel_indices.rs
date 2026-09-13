use crate::{
    Error, MATERIAL_SLOTS, MaterialSlotKey, PALETTE_COLORS, PaletteAxes, PalettePlan, Result,
    derived_material, emissive_coefficient,
};
use branded_id::U32Id;
use voxcore::BVoxMaterial;

/// The Voxel Max indices a voxel writes.
#[derive(Clone, Copy)]
pub(crate) struct VoxelIndices {
    /// The 1-based color cell.
    pub(crate) color_idx: u8,

    /// The 0-based material slot.
    pub(crate) material_idx: u8,
}

/// The indices a voxel sampling `sample` writes. The color cell is the
/// `baseColor` value id, since the color table is that value pool whole, so a
/// padded source palette is fine as long as its referenced colors fit; a
/// colorless voxel takes cell 1, since 0 is the empty cell. The slot is the
/// exact one when the plan has them, else the [`MaterialSlotKey`]'s, adding
/// the derived material on a new key. Errors when the cell reaches
/// [`PALETTE_COLORS`], when an exact slot is missing or reaches
/// [`MATERIAL_SLOTS`], or when the derived materials would exceed
/// [`MATERIAL_SLOTS`], rather than silently wrapping.
pub(crate) fn voxel_indices(
    plan: &mut PalettePlan,
    axes: &PaletteAxes,
    sample: &[U32Id<BVoxMaterial>],
) -> Result<VoxelIndices> {
    let color_idx = match axes.color {
        Some(color) => {
            let cell = axes.value_id(color, sample).to_u32();
            if cell >= PALETTE_COLORS as u32 {
                return Err(Error::invalid(format!(
                    "a voxel references color cell {cell}, but a Voxel Max palette holds only \
                     {PALETTE_COLORS} colors, so the source has more colors than fit"
                )));
            }
            cell as u8 + 1
        }
        None => 1,
    };

    let material_idx = if axes.material.is_empty() {
        0
    } else if let Some(exact_slots) = &plan.exact_slots {
        let material_id = sample[0];
        let Some(&slot) = exact_slots.get(&material_id) else {
            return Err(Error::invalid(format!(
                "vmax ext palette holds no material slot for material {}",
                material_id.to_u32()
            )));
        };
        if usize::from(slot) >= MATERIAL_SLOTS {
            return Err(Error::invalid(format!(
                "a voxel references material {slot}, but a Voxel Max palette holds only \
                 {MATERIAL_SLOTS} material slots"
            )));
        }
        slot
    } else {
        let value_ids: Vec<_> = axes
            .material
            .iter()
            .map(|&property_id| axes.value_id(property_id, sample))
            .collect();
        let sic = emissive_coefficient(axes, sample, &value_ids)?;
        let key = MaterialSlotKey {
            value_ids,
            sic_bits: sic.to_bits(),
        };
        match plan.slot_index_of.get(&key) {
            Some(&slot) => slot,
            None => {
                if plan.materials.len() >= MATERIAL_SLOTS {
                    return Err(Error::invalid(format!(
                        "an object needs more than {MATERIAL_SLOTS} materials, but a Voxel Max \
                         palette holds only that many material slots"
                    )));
                }
                let slot = plan.materials.len() as u8;
                plan.materials
                    .push(derived_material(axes, slot, &key.value_ids, sic)?);
                plan.slot_index_of.insert(key, slot);
                slot
            }
        }
    };

    Ok(VoxelIndices {
        color_idx,
        material_idx,
    })
}
