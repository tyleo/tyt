use crate::pool_color;
use branded_id::U32Id;
use voxcore::{BVoxArrayProperty, BVoxLayer, BVoxPalette, BVoxVoxel, VoxMain, VoxObject};

/// The sRGB `[r, g, b, a]` color one voxel samples through `object`'s color
/// reference: the material it samples in `layer`, resolved through
/// `array_property` into its color pool. Defaults to transparent black when
/// the voxel is not live in `layer`, the value is absent, or the pool is not
/// a color kind.
pub fn cell_color(
    state: &VoxMain,
    object: &VoxObject,
    voxel: U32Id<BVoxVoxel>,
    layer: U32Id<BVoxLayer>,
    palette: U32Id<BVoxPalette>,
    array_property: U32Id<BVoxArrayProperty>,
) -> [u8; 4] {
    object
        .voxel_material(voxel, layer)
        .and_then(|material| state.material_value(palette, material, array_property))
        .and_then(|(pool, value_id)| pool_color(pool, value_id))
        .unwrap_or([0, 0, 0, 0])
}
