use branded_id::U32Id;
use voxcore::BVoxMaterial;

/// The material id `index`.
pub fn material_id(index: u32) -> U32Id<BVoxMaterial> {
    U32Id::from_u32(index)
}
