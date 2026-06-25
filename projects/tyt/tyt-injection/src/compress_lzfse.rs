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

#[cfg(test)]
mod tests {
    use crate::{compress_lzfse, decompress_lzfse};

    #[test]
    fn round_trips() {
        let highly_compressible = vec![7u8; 50_000];
        let incompressible: Vec<u8> = (0..5000u32)
            .map(|i| (i.wrapping_mul(2654435761) >> 24) as u8)
            .collect();
        for data in [vec![1, 2, 3, 4, 5], highly_compressible, incompressible] {
            let compressed = compress_lzfse(&data);
            assert_eq!(decompress_lzfse(&compressed), data);
        }
    }
}
