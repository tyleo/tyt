/// One object's geometry and samples, decoded into flat per-voxel lists. The
/// decoded counterpart of [`VoxjObject`](voxj::VoxjObject), produced by
/// [`decode_voxj_object`](crate::decode_voxj_object()) and re-encoded by
/// [`encode_voxj_object`](crate::encode_voxj_object()) /
/// [`encode_voxj_object_smallest`](crate::encode_voxj_object_smallest()).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxjDecodedObject {
    /// Display name of the object.
    pub name: String,

    /// Indices into the document's palettes, in resolution order.
    pub palette_refs: Vec<usize>,

    /// `[X, Y, Z]` size in voxels, as in
    /// [`VoxjObject::bounds`](voxj::VoxjObject::bounds).
    pub bounds: [u32; 3],

    /// Voxel positions `[x, y, z]`, in listing order.
    pub positions: Vec<[u32; 3]>,

    /// One cell index per referenced palette, per voxel, aligned to
    /// [`positions`](Self::positions).
    pub samples: Vec<Vec<u32>>,
}
