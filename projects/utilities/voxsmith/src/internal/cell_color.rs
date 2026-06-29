use crate::parse_color_hex;
use branded_id::U32Id;
use voxcore::{BVoxAttribute, BVoxPaletteRef, BVoxVoxel, VoxObject, VoxPalette};

/// The RGBA color of one voxel through `object`'s color reference, defaulting
/// to transparent black when the voxel has no cell or the cell has no color
/// value.
pub fn cell_color(
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
