/// One object's decoded voxel geometry and per-palette samples, in listing
/// order: the codec backend's object representation (see
/// [`VoxjCodecBackend`](crate::VoxjCodecBackend)). It mirrors
/// [`VoxjSerdeObject`](crate::VoxjSerdeObject) but holds raw positions and samples instead
/// of encoded blocks; `voxj-codec`'s `encode_object` turns it into a
/// `VoxjSerdeObject` and `decode_object` recovers it. Palette cell counts are not
/// stored: they are derived from `palette_refs` and the document's palettes
/// where the codec needs them.
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

    /// `[x, y, z]` positions in listing order.
    pub positions: Vec<[u32; 3]>,

    /// `samples[voxel][palette]` = the cell index that voxel samples in each
    /// referenced palette, in listing order.
    pub samples: Vec<Vec<u32>>,
}
