use crate::{
    ABSORPTION, Error, PaletteAxes, Result, SHADOWS, flag_value, pbr_factor_to_vm_coefficient,
    scalar_value, unbound_scalar,
};
use branded_id::U32Id;
use vmax::{VMaxMaterial, VMaxMaterialDispersion};
use voxcore::{
    BVoxValuePoolValue,
    material::{IOR, METALLIC, ROUGHNESS, TRANSMISSION},
};

/// One derived Voxel Max material. A coefficient reads from its property's
/// value pool at the slot key's value id, and `sic` is the coefficient
/// [`emissive_coefficient`](crate::emissive_coefficient) gives. Metalness and
/// roughness map from the 0 to 1 glTF factor to Voxel Max's 0.1 to 0.9 slider
/// coefficient; see [`pbr_factor_to_vm_coefficient`].
pub(crate) fn derived_material(
    axes: &PaletteAxes,
    slot: u8,
    value_ids: &[U32Id<BVoxValuePoolValue>],
    sic: f64,
) -> Result<VMaxMaterial> {
    // `None` when no layer supplies the property, so the caller takes the
    // vocabulary default. A property bound to a value pool holding no scalar
    // errors instead: the palette names a value this writer cannot read, and a
    // default would write a material the source never described.
    let scalar = |name: &str| -> Result<Option<f64>> {
        let Some(position) = axes.material_position(name) else {
            return Ok(None);
        };
        scalar_value(axes.property(axes.material[position]), value_ids[position])
            .map(Some)
            .ok_or_else(|| {
                Error::invalid(format!(
                    "`{name}` draws from a value pool holding no scalar"
                ))
            })
    };
    let flag = |name: &str| -> Option<bool> {
        let position = axes.material_position(name)?;
        flag_value(axes.property(axes.material[position]), value_ids[position])
    };
    let carries = |name: &str| -> bool { axes.material_position(name).is_some() };
    let dispersed = carries(IOR) || carries(TRANSMISSION) || carries(ABSORPTION);
    Ok(VMaxMaterial {
        mi: (usize::from(slot) + 1).to_string(),
        // An unbound property renders at its vocabulary default, the same one
        // the glTF export writes, so the two exporters read one source model
        // the same way.
        mc: pbr_factor_to_vm_coefficient(unbound_scalar(scalar(METALLIC)?, METALLIC), METALLIC)?,
        rc: pbr_factor_to_vm_coefficient(unbound_scalar(scalar(ROUGHNESS)?, ROUGHNESS), ROUGHNESS)?,
        sic,
        // Voxel Max casts shadows by default; a source without a shadows flag,
        // such as glTF, takes that default.
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
