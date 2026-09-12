use branded_id::U32Id;
use voxcore::BVoxObject;

/// One matrix of the flattened scene.
pub struct QbPlacement {
    pub object_id: U32Id<BVoxObject>,
    pub name: String,
    pub position: [i32; 3],
}
