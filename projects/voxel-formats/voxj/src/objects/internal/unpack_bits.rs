/// Inverse of [`pack_bits`](crate::objects::pack_bits): reads `count` values of `width`
/// bits each, MSB-first, 8 per byte. `bytes` must cover all `count * width` of
/// them, which each caller checks against the decoded length first.
pub fn unpack_bits(bytes: &[u8], width: u32, count: usize) -> Vec<u32> {
    let mut out = Vec::with_capacity(count);
    let mut bit_pos = 0usize;
    for _ in 0..count {
        let mut value = 0u32;
        for _ in 0..width {
            let byte = *bytes
                .get(bit_pos / 8)
                .expect("the caller checked the bytes cover count * width bits");
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
    use crate::objects::{pack_bits, unpack_bits};

    #[test]
    fn round_trips_pack_bits() {
        let values = [0, 1, 5, 2, 7, 0, 255, 128];
        for width in [1, 3, 8] {
            let masked: Vec<u32> = values.iter().map(|v| v & ((1 << width) - 1)).collect();
            let packed = pack_bits(&masked, width);
            assert_eq!(
                unpack_bits(&packed, width, masked.len()),
                masked,
                "width {width}"
            );
        }
    }
}
