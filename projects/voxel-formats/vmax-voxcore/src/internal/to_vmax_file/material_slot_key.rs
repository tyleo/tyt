use branded_id::U32Id;
use voxcore::BVoxValuePoolValue;

/// The Voxel Max material slot a derived sample lands in. Equal keys share a
/// slot.
#[derive(Clone, Eq, Hash, PartialEq)]
pub(crate) struct MaterialSlotKey {
    /// The material-axis value ids, in property order.
    pub(crate) value_ids: Vec<U32Id<BVoxValuePoolValue>>,

    /// The bits of the self-illumination coefficient the color axis gives.
    pub(crate) sic_bits: u64,
}
