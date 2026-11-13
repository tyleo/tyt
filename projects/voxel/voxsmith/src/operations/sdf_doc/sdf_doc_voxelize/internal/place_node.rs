use ty_math::{TyTransformF64, TyVector3F64};

/// One node a document holds for a part.
pub struct PlaceNode {
    /// The part's name.
    pub name: String,

    /// Whether the node is a root node.
    pub root: bool,

    /// The indices of the nodes under this one.
    pub child_indices: Vec<usize>,

    /// Where the node sits in its parent's frame.
    pub transform: TyTransformF64,

    /// The objects at the place.
    pub object_indices: Vec<usize>,

    /// The offset in voxels from a lattice index to the node's frame: `f` in
    /// model evaluation.
    pub lattice_offset: TyVector3F64,
}
