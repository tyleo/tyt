use crate::{Error, MATERIAL_SLOTS, PaletteLayout, Result};
use branded_id::U32Id;
use voxcore::BVoxMaterial;

/// The Voxel Max material slot material `material_id` draws: the one value id
/// its material-axis properties share, the material byte the loader set. Zero
/// when the palette binds no material axis. Errors when the properties
/// disagree, because the material is then no single slot, and when the slot
/// reaches [`MATERIAL_SLOTS`].
pub(crate) fn material_slot(
    layout: &PaletteLayout,
    material_id: U32Id<BVoxMaterial>,
) -> Result<u8> {
    let mut slot: Option<(u32, &str)> = None;
    for property in &layout.material {
        let value_id = layout
            .palette
            .value_id(material_id, property.id)
            .expect("a live material has a value id for every property")
            .to_u32();
        match slot {
            None => slot = Some((value_id, property.name)),
            Some((slot, _)) if slot == value_id => {}
            Some((slot, first)) => {
                return Err(Error::invalid(format!(
                    "material {} draws `{}` value {value_id} but `{first}` value {slot}, so it \
                     is no single Voxel Max material slot",
                    material_id.to_u32(),
                    property.name
                )));
            }
        }
    }
    let Some((slot, _)) = slot else {
        return Ok(0);
    };
    if slot as usize >= MATERIAL_SLOTS {
        return Err(Error::invalid(format!(
            "material {} draws slot {slot}, but a Voxel Max palette holds only {MATERIAL_SLOTS} \
             material slots",
            material_id.to_u32()
        )));
    }
    Ok(slot as u8)
}
