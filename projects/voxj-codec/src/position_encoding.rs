/// The encoding used for a voxel-position block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionEncoding {
    /// One `[x, y, z]` triple per voxel.
    RawJson,

    /// Dense occupancy bitmap over `bounds`, base64-encoded.
    BitmapBase64,

    /// Prefix-sum deltas of each voxel's 3D Hilbert index,
    /// varint+base64-encoded.
    Hilbert,
}
