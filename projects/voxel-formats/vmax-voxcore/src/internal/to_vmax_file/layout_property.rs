use branded_id::U32Id;
use voxcore::{BVoxProperty, VoxValuePool};

/// A palette property with the value pool it draws from.
pub(crate) struct LayoutProperty<'a> {
    pub(crate) id: U32Id<BVoxProperty>,
    pub(crate) name: &'a str,
    pub(crate) value_pool: &'a VoxValuePool,
}
