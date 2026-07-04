use crate::pool_color;
use branded_id::U32Id;
use voxcore::{BVoxLayer, BVoxPalette, BVoxPaletteBinding, BVoxVoxel, VoxMain, VoxObject};

/// The sRGB `[r, g, b, a]` color one voxel samples through `object`'s color
/// reference: the material it samples in `layer`, resolved through `binding`
/// into the bound color pool. Defaults to transparent black when the voxel is
/// not live in `layer`, the value is absent, or the bound pool is not a color
/// kind.
pub fn cell_color(
    state: &VoxMain,
    object: &VoxObject,
    voxel: U32Id<BVoxVoxel>,
    layer: U32Id<BVoxLayer>,
    palette: U32Id<BVoxPalette>,
    binding: U32Id<BVoxPaletteBinding>,
) -> [u8; 4] {
    object
        .voxel_material(voxel, layer)
        .and_then(|material| state.material_value(palette, material, binding))
        .and_then(|(pool, index)| pool_color(pool, index))
        .unwrap_or([0, 0, 0, 0])
}
