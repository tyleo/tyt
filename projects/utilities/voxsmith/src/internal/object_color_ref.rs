use crate::BASE_COLOR_FACTOR;
use branded_id::U32Id;
use voxcore::{BVoxArrayProperty, BVoxLayer, BVoxPalette, VoxMain, VoxObject};

/// An object's color reference: the first layer whose palette carries a
/// `baseColorFactor` array property, with that palette's id and the property's
/// id. `None` when the object has no such layer.
pub fn object_color_ref(
    state: &VoxMain,
    object: &VoxObject,
) -> Option<(
    U32Id<BVoxLayer>,
    U32Id<BVoxPalette>,
    U32Id<BVoxArrayProperty>,
)> {
    object.iter_layers().find_map(|(layer, palette_id)| {
        let palette = state.palette(palette_id)?;
        let array_property = palette.array_property_by_name(BASE_COLOR_FACTOR)?;
        Some((layer, palette_id, array_property))
    })
}
