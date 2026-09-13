use crate::{Error, PaletteAxes, Result, sample_color, scalar_value, unbound_scalar};
use branded_id::U32Id;
use voxcore::{
    BVoxMaterial, BVoxValuePoolValue,
    material::{EMISSIVE_COLOR, EMISSIVE_STRENGTH},
};

/// The self-illumination coefficient `sic` a sample writes. Voxel Max glows
/// in the voxel's base color at `sic`, while a voxcore material glows in
/// `emissiveColor` at `emissiveStrength`, so the two agree only when the
/// emissive color is the base color, and then `sic` is the strength. A black
/// emissive or a zero strength is no glow and writes zero whatever the
/// colors. A glowing emissive that differs from the base color, or has no
/// base color to match, errors: Voxel Max would glow in a color the source
/// never showed. With no `emissiveColor` bound the strength stands alone, as
/// the from-vmax split emits it, and zero when that is unbound too.
pub(crate) fn emissive_coefficient(
    axes: &PaletteAxes,
    sample: &[U32Id<BVoxMaterial>],
    value_ids: &[U32Id<BVoxValuePoolValue>],
) -> Result<f64> {
    let strength = match axes.material_position(EMISSIVE_STRENGTH) {
        Some(position) => Some(
            scalar_value(axes.property(axes.material[position]), value_ids[position]).ok_or_else(
                || {
                    Error::invalid(format!(
                        "`{EMISSIVE_STRENGTH}` draws from a value pool holding no scalar"
                    ))
                },
            )?,
        ),
        None => None,
    };
    let Some(emissive_color) = axes.emissive_color else {
        return Ok(strength.unwrap_or(0.0));
    };
    let strength = unbound_scalar(strength, EMISSIVE_STRENGTH);
    let emissive = sample_color(axes, emissive_color, sample).ok_or_else(|| {
        Error::invalid(format!(
            "`{EMISSIVE_COLOR}` draws from a value pool holding no color"
        ))
    })?;
    if strength == 0.0 || emissive == [0.0; 3] {
        return Ok(0.0);
    }
    let base = axes
        .color
        .and_then(|color| sample_color(axes, color, sample));
    match base {
        Some(base) if base == emissive => Ok(strength),
        _ => Err(Error::invalid(format!(
            "a voxel sampling materials {:?} glows in linear {emissive:?} over a base color of \
             {}, but Voxel Max glows only in the base color, so writing it would change how the \
             model looks",
            sample.iter().map(|id| id.to_u32()).collect::<Vec<_>>(),
            base.map_or("none".to_owned(), |base| format!("linear {base:?}")),
        ))),
    }
}
