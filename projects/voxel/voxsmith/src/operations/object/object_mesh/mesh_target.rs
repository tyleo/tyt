use crate::operations::object::MeshRecord;
use branded_id::U32Id;
use voxcore::BVoxObject;

/// One object of a meshing run under the record it meshes by.
#[derive(Clone, Copy, Debug)]
pub struct MeshTarget<'a> {
    /// The object to mesh.
    pub object_id: U32Id<BVoxObject>,

    /// The record the object meshes under.
    pub record: &'a MeshRecord,
}
