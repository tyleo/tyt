/// One object's voxel geometry and per-palette samples, decoded into flat lists.
///
/// The codec form of a [`VoxjObject`](voxj::VoxjObject): its encoded position and
/// sample blocks are flattened into per-voxel [`positions`](Self::positions) and
/// [`samples`](Self::samples), in listing order. Produced by
/// [`decode_voxj_object`](crate::decode_voxj_object) and consumed by
/// [`encode_voxj_object`](crate::encode_voxj_object) /
/// [`encode_voxj_object_smallest`](crate::encode_voxj_object_smallest).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxjDecodedObject {
    /// Display name of the object.
    pub name: String,

    /// Indices into the document's palettes, in resolution order.
    pub palette_refs: Vec<usize>,

    /// `[X, Y, Z]` size in voxels; every voxel lies in
    /// `[0, X) x [0, Y) x [0, Z)`.
    pub bounds: [u32; 3],

    /// Voxel positions as `[x, y, z]`, in listing order.
    pub positions: Vec<[u32; 3]>,

    /// One cell index per referenced palette, per voxel, in listing order.
    pub samples: Vec<Vec<u32>>,
}
