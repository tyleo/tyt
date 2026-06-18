/// An encoded voxel-position block. The chosen encoding fixes the object's
/// canonical voxel order, which the sample channels then follow. Base64 strings
/// are stored already-encoded; serde rendering is handled by `voxj-serde`.
#[derive(Clone, Debug, PartialEq)]
pub enum PositionBlock {
    /// One `[x, y, z]` triple per voxel, in listing order.
    RawJson(Vec<[u32; 3]>),
    /// Dense occupancy bitmap over `bounds`, packed 8 bits per byte MSB-first,
    /// base64-encoded; canonical order is ascending cell index.
    BitmapBase64(String),
    /// Prefix-sum deltas of each voxel's 3D Hilbert-curve index (ascending),
    /// as an unsigned-LEB128 varint stream, base64-encoded.
    HilbertIndexDeltaVarintBase64(String),
}
