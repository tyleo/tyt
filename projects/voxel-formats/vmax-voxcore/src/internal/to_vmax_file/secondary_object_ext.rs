use crate::{VMaxExtNode, secondary_uuid};
use std::collections::HashSet;

/// The per-object ext for an extra object on a node placing several, such as a
/// Goxel layer's blocks. It takes a distinct id and a distinct index triplet
/// and inherits the node's rotation and alignment to stay a sibling of the
/// node's first object. Its content box is derived from its own bounds on
/// write.
pub(crate) fn secondary_object_ext(
    node_ext: &VMaxExtNode,
    slot: usize,
    used_indices: &mut HashSet<[i64; 3]>,
) -> VMaxExtNode {
    let index = (0..)
        .map(|counter| [0, 0, counter])
        .find(|index| !used_indices.contains(index))
        .expect("a fresh counter exists");
    used_indices.insert(index);
    VMaxExtNode {
        id: secondary_uuid(&node_ext.id, slot),
        index,
        ..node_ext.clone()
    }
}
