use crate::GoxlExtPlacement;
use branded_id::U32Id;

/// A placement stamping object `object_index` at `position`.
pub fn placement(object_index: u32, position: [i32; 3]) -> GoxlExtPlacement {
    GoxlExtPlacement {
        object_id: U32Id::from_u32(object_index),
        position,
    }
}
