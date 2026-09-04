/// Compresses bytes into a zlib stream, the framing of each matrix's voxel
/// grid in a `.qbt` or `.qbcl` file.
pub trait CompressZlib {
    /// The zlib stream of `bytes`.
    fn compress_zlib(&self, bytes: &[u8]) -> Vec<u8>;
}
