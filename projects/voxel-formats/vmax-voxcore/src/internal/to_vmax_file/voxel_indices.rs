use crate::{PaletteLayout, Result, color_cell, material_slot};
use branded_id::U32Id;
use voxcore::BVoxMaterial;

/// The Voxel Max indices a voxel writes.
#[derive(Clone, Copy)]
pub(crate) struct VoxelIndices {
    /// The 1-based color cell.
    pub(crate) color_idx: u8,

    /// The 0-based material slot.
    pub(crate) material_idx: u8,
}

/// The indices a voxel drawing material `material_id` writes.
pub(crate) fn voxel_indices(
    layout: &PaletteLayout,
    material_id: U32Id<BVoxMaterial>,
) -> Result<VoxelIndices> {
    Ok(VoxelIndices {
        color_idx: color_cell(layout, material_id)?,
        material_idx: material_slot(layout, material_id)?,
    })
}
