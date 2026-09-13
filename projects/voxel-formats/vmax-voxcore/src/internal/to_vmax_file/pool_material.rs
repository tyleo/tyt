use crate::{
    ABSORPTION, Error, PaletteLayout, Result, SHADOWS, flag_value, pbr_factor_to_vm_coefficient,
    scalar_value, unbound_scalar,
};
use branded_id::U32Id;
use vmax::{VMaxMaterial, VMaxMaterialDispersion};
use voxcore::material::{EMISSIVE_STRENGTH, IOR, METALLIC, ROUGHNESS, TRANSMISSION};

/// The Voxel Max material in slot `slot`, read from each material-axis pool at
/// the slot's value id. Metalness and roughness map from the 0 to 1 factor to
/// Voxel Max's 0.1 to 0.9 slider coefficient; see
/// [`pbr_factor_to_vm_coefficient`]. A property the palette does not bind
/// writes its vocabulary default, so it writes what it renders as. Errors when
/// a bound scalar's pool holds no scalar at the slot.
pub(crate) fn pool_material(layout: &PaletteLayout, slot: u8) -> Result<VMaxMaterial> {
    let value_id = U32Id::from_u32(u32::from(slot));
    let scalar = |name: &str| -> Result<Option<f64>> {
        let Some(property) = layout.material_property(name) else {
            return Ok(None);
        };
        scalar_value(property.value_pool, value_id)
            .map(Some)
            .ok_or_else(|| {
                Error::invalid(format!(
                    "`{name}` draws from a value pool holding no scalar at value {slot}"
                ))
            })
    };
    let flag = |name: &str| -> Option<bool> {
        flag_value(layout.material_property(name)?.value_pool, value_id)
    };
    let carries = |name: &str| -> bool { layout.material_property(name).is_some() };
    let dispersed = carries(IOR) || carries(TRANSMISSION) || carries(ABSORPTION);
    Ok(VMaxMaterial {
        mi: (usize::from(slot) + 1).to_string(),
        mc: pbr_factor_to_vm_coefficient(unbound_scalar(scalar(METALLIC)?, METALLIC), METALLIC)?,
        rc: pbr_factor_to_vm_coefficient(unbound_scalar(scalar(ROUGHNESS)?, ROUGHNESS), ROUGHNESS)?,
        // An unbound strength glows nowhere: the loader binds one whenever a
        // material glows.
        sic: scalar(EMISSIVE_STRENGTH)?.unwrap_or(0.0),
        // Voxel Max casts shadows by default.
        sh: flag(SHADOWS).unwrap_or(true),
        tc: None,
        md: match dispersed {
            true => Some(VMaxMaterialDispersion {
                absorption: scalar(ABSORPTION)?.unwrap_or(0.0),
                ior: unbound_scalar(scalar(IOR)?, IOR),
                transmission: unbound_scalar(scalar(TRANSMISSION)?, TRANSMISSION),
            }),
            false => None,
        },
    })
}
