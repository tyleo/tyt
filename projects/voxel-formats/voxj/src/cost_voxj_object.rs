use crate::VoxjObject;

/// Costs a candidate encoding of an object: the bytes it takes once the
/// document ships, such as its deflated length. A block search keeps the
/// lowest.
pub trait CostVoxjObject {
    /// The cost of `object` as encoded.
    fn cost_voxj_object(&self, object: &VoxjObject) -> usize;
}
