/// A block placement inside a [`GoxlLayer`](crate::GoxlLayer): which shared
/// [`GoxlBlock`](crate::GoxlBlock) to stamp and where to stamp it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GoxlLayerBlock {
    /// Index into [`GoxlFile::blocks`](crate::GoxlFile::blocks) of the block to
    /// place.
    pub block_index: i32,

    /// `[x, y, z]` voxel position of the block's lower corner in the layer.
    pub position: [i32; 3],
}
