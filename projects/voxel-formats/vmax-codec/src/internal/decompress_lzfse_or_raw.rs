use crate::DecompressLzfse;

/// LZFSE block-stream magics: the compressed v2, LZVN, compressed v1, and
/// uncompressed block kinds.
const LZFSE_MAGICS: [&[u8; 4]; 4] = [b"bvx2", b"bvxn", b"bvx1", b"bvx-"];

/// Decompresses an LZFSE stream through `dependencies`. Bytes that are not
/// LZFSE-framed or fail to decode come back raw. Voxel Max reads them the same
/// way.
pub fn decompress_lzfse_or_raw<D: DecompressLzfse>(dependencies: &D, bytes: &[u8]) -> Vec<u8> {
    let is_lzfse = bytes
        .first_chunk::<4>()
        .is_some_and(|magic| LZFSE_MAGICS.contains(&magic));
    if !is_lzfse {
        return bytes.to_vec();
    }
    dependencies
        .decompress_lzfse(bytes)
        .unwrap_or_else(|_| bytes.to_vec())
}
