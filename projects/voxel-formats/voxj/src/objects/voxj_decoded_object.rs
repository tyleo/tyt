/// One object's geometry and samples, decoded into flat per-voxel lists. The
/// decoded counterpart of [`VoxjObject`](crate::VoxjObject), produced by
/// [`decode_voxj_object`](crate::objects::decode_voxj_object()) and re-encoded by
/// [`encode_voxj_object`](crate::objects::encode_voxj_object()) /
/// [`encode_voxj_object_optimized`](crate::objects::encode_voxj_object_optimized()).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxjDecodedObject {
    /// Display name of the object.
    pub name: String,

    /// Palette indices into the document's palettes, one per layer, ordered
    /// back to front. Two layers may reference the same palette; layers do not
    /// merge.
    pub layers: Vec<usize>,

    /// `[X, Y, Z]` size in voxels, as in
    /// [`VoxjObject::bounds`](crate::VoxjObject::bounds).
    pub bounds: [u32; 3],

    /// `[X, Y, Z]` translation from the placing node to the grid's min corner,
    /// as in [`VoxjObject::origin`](crate::VoxjObject::origin).
    pub origin: [i32; 3],

    /// Voxel positions `[x, y, z]`, in listing order.
    pub positions: Vec<[u32; 3]>,

    /// One material index per layer, per voxel, aligned to
    /// [`positions`](Self::positions). Entries follow
    /// [`layers`](Self::layers) order. Each index addresses a material in
    /// that layer's palette.
    pub samples: Vec<Vec<u32>>,
}
