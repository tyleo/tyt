use branded_id::U32Id;
use voxcore::{BVoxAttribute, BVoxPaletteRef, BVoxVoxel, VoxMain, VoxObject, VoxPalette, VoxValue};

/// The state's ext, but only when it is an [`Object`](VoxValue::Object) carrying
/// `key` as a top-level field. A foreign ext naming another format, or no ext at
/// all, returns `None`, which lets a `to_<format>` writer fall back to synthesis
/// without disturbing its lossless ext path.
pub(crate) fn ext_for<'a>(state: &'a VoxMain, key: &str) -> Option<&'a VoxValue> {
    let ext = state.ext()?;
    match ext {
        VoxValue::Object(map) if map.0.iter().any(|(name, _)| name == key) => Some(ext),
        _ => None,
    }
}

/// Parses a `#RRGGBB` or `#RRGGBBAA` color string into RGBA bytes. A missing
/// alpha defaults to opaque; a missing or malformed value defaults to transparent
/// black.
pub(crate) fn parse_color_hex(value: Option<&VoxValue>) -> [u8; 4] {
    let Some(VoxValue::Text(hex)) = value else {
        return [0, 0, 0, 0];
    };
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|byte| u8::from_str_radix(byte, 16).ok())
    };
    match (byte(0), byte(1), byte(2)) {
        (Some(r), Some(g), Some(b)) => [r, g, b, byte(3).unwrap_or(255)],
        _ => [0, 0, 0, 0],
    }
}

/// An object's color reference: the first palette reference whose palette carries
/// an `rgba` or `rgb` color attribute, with that palette and the attribute's id.
/// `None` when the object has no such reference.
pub(crate) fn object_color_ref<'a>(
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

/// The id of a palette's color attribute, preferring `rgba` over `rgb`, or `None`
/// when it carries neither.
fn color_attribute(palette: &VoxPalette) -> Option<U32Id<BVoxAttribute>> {
    let id = |name: &str| {
        palette
            .iter_attributes()
            .find(|(_, attribute)| *attribute == name)
            .map(|(id, _)| id)
    };
    id("rgba").or_else(|| id("rgb"))
}

/// The RGBA color of one voxel through `object`'s color reference, defaulting to
/// transparent black when the voxel has no cell or the cell has no color value.
pub(crate) fn cell_color(
    object: &VoxObject,
    voxel: U32Id<BVoxVoxel>,
    reference: U32Id<BVoxPaletteRef>,
    palette: &VoxPalette,
    attribute: U32Id<BVoxAttribute>,
) -> [u8; 4] {
    let value = object
        .voxel_cell(voxel, reference)
        .and_then(|cell| palette.cell_value(cell, attribute));
    parse_color_hex(value)
}
