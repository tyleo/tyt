use crate::{BVoxArrayProperty, BVoxScalarProperty};
use branded_id::U32Id;

/// A palette property id tagged by arity. Returned by
/// [`VoxPalette::property_by_name`](crate::VoxPalette::property_by_name),
/// which looks a name up across both property lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoxPropertyId {
    /// An array property id.
    Array(U32Id<BVoxArrayProperty>),

    /// A scalar property id.
    Scalar(U32Id<BVoxScalarProperty>),
}
