use branded_id::U32Id;
use voxcore::{BVoxAttribute, BVoxPaletteRef, VoxMain, VoxObject, VoxPalette};

/// An object's color reference: the first palette reference whose palette
/// carries an `rgba` or `rgb` color attribute, with that palette and the
/// attribute's id. `None` when the object has no such reference.
pub fn object_color_ref<'a>(
    state: &'a VoxMain,
    object: &VoxObject,
) -> Option<(U32Id<BVoxPaletteRef>, &'a VoxPalette, U32Id<BVoxAttribute>)> {
    object
        .iter_palette_refs()
        .find_map(|(reference, palette_id)| {
            let palette = state.palette(palette_id)?;
            let attribute = color_attribute(palette)?;
            Some((reference, palette, attribute))
        })
}

/// The id of a palette's color attribute, preferring `rgba` over `rgb`, or
/// `None` when it carries neither.
fn color_attribute(palette: &VoxPalette) -> Option<U32Id<BVoxAttribute>> {
    let id = |name: &str| {
        palette
            .iter_attributes()
            .find(|(_, attribute)| *attribute == name)
            .map(|(id, _)| id)
    };
    id("rgba").or_else(|| id("rgb"))
}
