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

#[cfg(test)]
mod tests {
    use crate::varint_encode;

    #[test]
    fn matches_spec_example() {
        // Deltas [0, 3, 1, 3] from the format spec's Hilbert example.
        assert_eq!(varint_encode(&[0, 3, 1, 3]), vec![0x00, 0x03, 0x01, 0x03]);
    }
}
