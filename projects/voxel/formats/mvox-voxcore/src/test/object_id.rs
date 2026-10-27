use branded_id::U32Id;
use voxcore::BVoxObject;

/// The object id `index`.
pub fn object_id(index: u32) -> U32Id<BVoxObject> {
    U32Id::from_u32(index)
}
