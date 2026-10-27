use crate::operations::node::{OriginView, TransformView};

/// The per-node and per-object subtree views
/// [`node_list`](crate::operations::node::node_list()) can append.
#[derive(Clone, Copy, Debug, Default)]
pub struct NodeListViews {
    /// Prepend each node's transform subtree.
    pub transforms: Option<TransformView>,

    /// Append each object's edit-grid origin, the build volume's min corner.
    pub edit_origins: Option<OriginView>,

    /// Append each object's edit-grid bounds subtree at this precision.
    pub edit_bounds: Option<usize>,

    /// Append each object's edit-grid extents at this precision.
    pub edit_extents: Option<usize>,

    /// Append each object's runtime-grid origin, the tight live box's min
    /// corner.
    pub runtime_origins: Option<OriginView>,

    /// Append each object's runtime-grid bounds subtree at this precision.
    pub runtime_bounds: Option<usize>,

    /// Append each object's runtime-grid extents at this precision.
    pub runtime_extents: Option<usize>,

    /// Append each object's filled (live) voxel count.
    pub voxel_counts: bool,

    /// Append each object's layers subtree, one child per layer.
    pub layers: bool,
}
