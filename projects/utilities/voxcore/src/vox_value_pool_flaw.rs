use crate::BVoxValuePoolValue;
use branded_id::U32Id;

/// A well-formedness flaw in a value pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VoxValuePoolFlaw {
    /// The value pool holds no values.
    Empty,

    /// A `min`/`max` bound is non-finite, not integer-valued for an `int`
    /// value pool, or `min` is greater than `max`.
    Bound,

    /// The value at this id is malformed for its kind or outside the bounds.
    Value(U32Id<BVoxValuePoolValue>),
}
