/// One object's voxel geometry and per-palette samples, in listing order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxjCodecObject {
    /// Display name of the object.
    pub name: String,

    /// Indices into [`VoxjMain::palettes`](crate::VoxjMain::palettes), in
    /// resolution order.
    pub palette_refs: Vec<usize>,

    /// `[X, Y, Z]` size in voxels; every voxel lies in
    /// `[0, X) x [0, Y) x [0, Z)`.
    pub bounds: [u32; 3],

    /// Voxel positions as `[x, y, z]`, in listing order.
    pub positions: Vec<[u32; 3]>,

    /// One cell index per referenced palette, per voxel, in listing order.
    pub samples: Vec<Vec<u32>>,
}
