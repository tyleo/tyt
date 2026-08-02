use crate::BVoxValuePoolValue;
use branded_id::U32Id;

/// A well-formedness flaw in a value pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VoxValuePoolFlaw {
    /// The value pool holds no values.
    Empty,

    /// The value at this id is outside its kind's value domain.
    Value(U32Id<BVoxValuePoolValue>),
}
