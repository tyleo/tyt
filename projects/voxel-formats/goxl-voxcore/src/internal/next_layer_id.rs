use crate::GoxlExtLayer;
use branded_id::U32Id;
use std::collections::BTreeMap;
use voxcore::BVoxHierarchyNode;

/// A layer id no entry in `layers` has: one past the largest, and at least
/// `1` because `0` means no clone base.
pub fn next_layer_id(layers: &BTreeMap<U32Id<BVoxHierarchyNode>, GoxlExtLayer>) -> i32 {
    layers
        .values()
        .map(|layer| layer.id)
        .max()
        .unwrap_or(0)
        .max(0)
        + 1
}
