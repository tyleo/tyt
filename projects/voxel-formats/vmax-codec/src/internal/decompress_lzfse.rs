/// LZFSE block-stream magics that mark a compressed payload (compressed v1/v2,
/// LZVN, and uncompressed blocks).
const LZFSE_MAGICS: [&[u8; 4]; 4] = [b"bvx2", b"bvxn", b"bvx1", b"bvx-"];

/// Decompresses an LZFSE stream, falling back to the raw bytes when the input
/// is not LZFSE-framed or cannot be decoded (matching Voxel Max's own
/// try-decompress-then-raw behavior).
pub fn decompress_lzfse(bytes: &[u8]) -> Vec<u8> {
    let is_lzfse = bytes
        .first_chunk::<4>()
        .is_some_and(|magic| LZFSE_MAGICS.contains(&magic));
    if !is_lzfse {
        return bytes.to_vec();
    }

    let mut capacity = bytes.len().saturating_mul(8).max(4096);
    let ceiling = bytes.len().saturating_mul(8192).max(1 << 20);
    loop {
        let mut out = vec![0u8; capacity];
        match lzfse::decode_buffer(bytes, &mut out) {
            Ok(len) if len < capacity => {
                out.truncate(len);
                return out;
            }
            // A full buffer may mean truncation, so grow and retry.
            Ok(_) | Err(_) if capacity < ceiling => capacity = capacity.saturating_mul(2),
            // Give up and treat the payload as already decompressed.
            _ => return bytes.to_vec(),
        }
    }
}
