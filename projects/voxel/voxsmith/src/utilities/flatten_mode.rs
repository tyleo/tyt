/// How much of a hierarchy a voxelizer flattens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlattenMode {
    /// A node for each placement, nested as the hierarchy nests.
    None,

    /// One node for each root, holding every object below the root on one
    /// grid.
    Nodes,

    /// One node for each root, holding one object of every voxel below the
    /// root.
    Objects,
}
