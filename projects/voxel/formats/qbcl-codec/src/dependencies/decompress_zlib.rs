/// Decompresses a zlib stream, the framing of each matrix's voxel grid in a
/// `.qbt` or `.qbcl` file.
pub trait DecompressZlib {
    /// The bytes `stream` encodes, or the reason it is malformed.
    fn decompress_zlib(&self, stream: &[u8]) -> Result<Vec<u8>, String>;
}
