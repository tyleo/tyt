use crate::PaletteAxes;
use branded_id::{U32Id, UsizeId};
use voxcore::{BVoxEffectiveProperty, BVoxMaterial, color::value_pool_lin_srgba_f64_color};

/// The linear rgb a sample draws for a color-axis property, or `None` when the
/// value pool holds no float vectors.
pub(crate) fn sample_color(
    axes: &PaletteAxes,
    property_id: UsizeId<BVoxEffectiveProperty>,
    sample: &[U32Id<BVoxMaterial>],
) -> Option<[f64; 3]> {
    let property = axes.property(property_id);
    let color =
        value_pool_lin_srgba_f64_color(property.value_pool(), axes.value_id(property_id, sample))?;
    Some([color.red, color.green, color.blue])
}
