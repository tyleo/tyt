use crate::BASE_COLOR_FACTOR;
use branded_id::U32Id;
use voxcore::{BVoxLayer, BVoxPalette, BVoxPaletteBinding, VoxMain, VoxObject};

/// An object's color reference: the first layer whose palette binds
/// `baseColorFactor`, with that palette's id and the binding's id. `None` when
/// the object has no such layer.
pub fn object_color_ref(
    state: &VoxMain,
    object: &VoxObject,
) -> Option<(
    U32Id<BVoxLayer>,
    U32Id<BVoxPalette>,
    U32Id<BVoxPaletteBinding>,
)> {
    object.iter_layers().find_map(|(layer, palette_id)| {
        let palette = state.palette(palette_id)?;
        let binding = palette.binding_by_attribute(BASE_COLOR_FACTOR)?;
        Some((layer, palette_id, binding))
    })
}
