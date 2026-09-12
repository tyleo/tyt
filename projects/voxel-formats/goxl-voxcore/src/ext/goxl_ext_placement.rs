use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use voxcore::BVoxObject;

/// One block stamp of a layer: the object stamped and the world position of
/// its lower corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GoxlExtPlacement {
    /// The stamped object.
    #[cfg_attr(feature = "serde", serde(rename = "object-id"))]
    pub object_id: U32Id<BVoxObject>,

    /// The `[x, y, z]` world position of the block's lower corner.
    pub position: [i32; 3],
}
