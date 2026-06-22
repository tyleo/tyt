/// Compresses `bytes` into an LZFSE block stream — the inverse of
/// [`decompress_lzfse`](crate::decompress_lzfse). Grows the output buffer until
/// the encode succeeds (LZFSE always succeeds given a large enough buffer).
pub fn compress_lzfse(bytes: &[u8]) -> Vec<u8> {
    let mut capacity = bytes
        .len()
        .saturating_add(bytes.len() / 16)
        .saturating_add(4096);
    loop {
        let mut out = vec![0u8; capacity];
        if let Ok(len) = lzfse::encode_buffer(bytes, &mut out) {
            out.truncate(len);
            return out;
        }
        capacity = capacity.saturating_mul(2);
    }
}
