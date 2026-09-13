use crate::{extend_bounds, object_box_local, transform_half};
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::TyVector3F64;
use voxcore::{BVoxHierarchyNode, VoxExt, VoxMain};

/// The bounding box `(center, half)` of all geometry under `node_id`, in that
/// node's own local frame: the union of each child object's content box and
/// each child node's box mapped through the child's transform. Voxel Max stores
/// this per group as `e_c`/`e_mi`/`e_ma`; it is the union of the subtree, so it
/// is derived here rather than kept in the ext. Memoized by node id so a
/// subtree shared across parents is walked once. A node with no geometry
/// collapses to a zero box.
pub(crate) fn subtree_box_local<T: VoxExt>(
    main: &VoxMain<T>,
    node_id: U32Id<BVoxHierarchyNode>,
    memo: &mut HashMap<u32, ([f64; 3], [f64; 3])>,
) -> ([f64; 3], [f64; 3]) {
    if let Some(&box_local) = memo.get(&node_id.to_u32()) {
        return box_local;
    }
    let node = main
        .hierarchy_node(node_id)
        .expect("a valid hierarchy node");
    let mut bounds: Option<([f64; 3], [f64; 3])> = None;
    for &object_id in &node.child_object_ids {
        let (center, half) = object_box_local(main, object_id);
        extend_bounds(&mut bounds, center, half);
    }
    for &child_id in &node.child_node_ids {
        let (child_center, child_half) = subtree_box_local(main, child_id, memo);
        let transform = main
            .hierarchy_node(child_id)
            .expect("a valid child node")
            .transform;
        let center = transform
            .transform_point(TyVector3F64::from_array(child_center))
            .to_array();
        let half = transform_half(&transform, child_half);
        extend_bounds(&mut bounds, center, half);
    }
    let (min, max) = bounds.unwrap_or(([0.0; 3], [0.0; 3]));
    let box_local = (
        [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ],
        [
            (max[0] - min[0]) / 2.0,
            (max[1] - min[1]) / 2.0,
            (max[2] - min[2]) / 2.0,
        ],
    );
    memo.insert(node_id.to_u32(), box_local);
    box_local
}
