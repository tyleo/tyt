/// One object's voxel geometry and per-palette samples, in listing order, ready
/// to be encoded into a [`VoxjObject`](voxj::VoxjObject) by
/// [`encode_object`](crate::encode_object).
#[derive(Clone)]
pub struct VoxelData {
    /// `[x, y, z]` positions in listing order.
    pub positions: Vec<[u32; 3]>,
    /// `samples[voxel][palette]` = the cell index that voxel samples in each
    /// referenced palette, in listing order.
    pub samples: Vec<Vec<u32>>,
    /// `[X, Y, Z]` object bounds.
    pub bounds: [u32; 3],
    /// Cell count of each referenced palette, in `palette_refs` order.
    pub palette_cell_counts: Vec<usize>,
}
