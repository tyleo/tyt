/// Bit-packs `values` at a fixed `width` bits each, MSB-first, 8 per byte, with
/// the final byte zero-padded. This is the packing shared by the
/// `bitmap-base64` position encoding (`width = 1`) and `packed-base64` samples.
pub fn pack_bits(values: &[u32], width: u32) -> Vec<u8> {
    let total_bits = values.len() * width as usize;
    let mut out = vec![0u8; total_bits.div_ceil(8)];
    let mut bit_pos = 0usize;
    for &value in values {
        for b in (0..width).rev() {
            if (value >> b) & 1 == 1 {
                out[bit_pos / 8] |= 1 << (7 - (bit_pos % 8));
            }
            bit_pos += 1;
        }
    }
    out
}

/// Inverse of [`pack_bits`]: reads `count` values of `width` bits each,
/// MSB-first, 8 per byte. Bytes past the end of `bytes` read as zero.
pub fn unpack_bits(bytes: &[u8], width: u32, count: usize) -> Vec<u32> {
    let mut out = Vec::with_capacity(count);
    let mut bit_pos = 0usize;
    for _ in 0..count {
        let mut value = 0u32;
        for _ in 0..width {
            let byte = bytes.get(bit_pos / 8).copied().unwrap_or(0);
            let bit = (byte >> (7 - bit_pos % 8)) & 1;
            value = (value << 1) | bit as u32;
            bit_pos += 1;
        }
        out.push(value);
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::pack_bits;

    #[test]
    fn bitmap_example() {
        // Cells 0,1,2,3 occupied -> 1111 + 4 zero pad -> 0xF0.
        assert_eq!(pack_bits(&[1, 1, 1, 1], 1), vec![0xF0]);
    }

    #[test]
    fn packed_channel_examples() {
        // Channel 0,0,0,1 at width 1 -> 0b0001_0000 = 0x10.
        assert_eq!(pack_bits(&[0, 0, 0, 1], 1), vec![0x10]);
        // Channel 0,1,1,1 at width 1 -> 0b0111_0000 = 0x70.
        assert_eq!(pack_bits(&[0, 1, 1, 1], 1), vec![0x70]);
    }

    #[test]
    fn multi_bit_width() {
        // Two 3-bit values 5 (101) and 2 (010) -> 101_010_00 -> 0xA8.
        assert_eq!(pack_bits(&[5, 2], 3), vec![0b1010_1000]);
    }
}
