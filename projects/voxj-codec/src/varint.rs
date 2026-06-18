/// Encodes unsigned integers as an unsigned-LEB128 varint byte stream.
pub fn varint_encode(values: &[u64]) -> Vec<u8> {
    let mut out = Vec::new();
    for &value in values {
        let mut v = value;
        while v >= 0x80 {
            out.push((v as u8 & 0x7f) | 0x80);
            v >>= 7;
        }
        out.push(v as u8);
    }
    out
}

/// Decodes an unsigned-LEB128 varint byte stream back into integers.
pub fn varint_decode(bytes: &[u8]) -> Vec<u64> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let mut v = 0u64;
        let mut shift = 0;
        loop {
            let b = bytes[i];
            i += 1;
            v |= ((b & 0x7f) as u64) << shift;
            shift += 7;
            if b & 0x80 == 0 {
                break;
            }
        }
        out.push(v);
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::{varint_decode, varint_encode};

    #[test]
    fn matches_spec_example() {
        // Deltas [0, 3, 1, 3] from the format spec's Hilbert example.
        assert_eq!(varint_encode(&[0, 3, 1, 3]), vec![0x00, 0x03, 0x01, 0x03]);
    }

    #[test]
    fn round_trips_multibyte() {
        let values = [0, 1, 127, 128, 300, 16384, 1_000_000, u32::MAX as u64];
        assert_eq!(varint_decode(&varint_encode(&values)), values);
    }
}
