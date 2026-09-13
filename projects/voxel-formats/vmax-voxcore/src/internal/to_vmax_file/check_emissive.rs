use crate::{Error, LayoutProperty, PaletteLayout, Result, pool_color};
use branded_id::U32Id;
use voxcore::{
    BVoxMaterial,
    material::{BASE_COLOR, EMISSIVE_COLOR},
};

/// Checks that material `material_id` looks the same on a slot glowing at
/// `sic`. Voxel Max glows in the voxel's base color at `sic`, while a voxcore
/// material glows in `emissiveColor` at `emissiveStrength`, so the two agree
/// only when the material's emissive color is its base color. A slot at zero
/// glows nowhere, whatever the colors. Errors when a glowing slot's material
/// has no emissive color, no base color, or an emissive that differs from its
/// base: Voxel Max would glow in a color the source never showed.
pub(crate) fn check_emissive(
    layout: &PaletteLayout,
    material_id: U32Id<BVoxMaterial>,
    sic: f64,
) -> Result<()> {
    if sic == 0.0 {
        return Ok(());
    }
    let color = |property: Option<&LayoutProperty>, name: &str| -> Result<[f64; 3]> {
        let Some(property) = property else {
            return Err(Error::invalid(format!(
                "material {} sits in a slot glowing at {sic}, but the palette binds no `{name}` \
                 for Voxel Max to glow in",
                material_id.to_u32()
            )));
        };
        let value_id = layout
            .palette
            .value_id(material_id, property.id)
            .expect("a live material has a value id for every property");
        pool_color(property.value_pool, value_id).ok_or_else(|| {
            Error::invalid(format!("`{name}` draws from a value pool holding no color"))
        })
    };
    let emissive = color(layout.emissive_color.as_ref(), EMISSIVE_COLOR)?;
    let base = color(layout.color.as_ref(), BASE_COLOR)?;
    if emissive != base {
        return Err(Error::invalid(format!(
            "material {} glows in linear {emissive:?} over a base color of linear {base:?}, but \
             Voxel Max glows only in the base color, so writing it would change how the model \
             looks",
            material_id.to_u32()
        )));
    }
    Ok(())
}
