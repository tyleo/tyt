/// Decodes a 3D Morton (Z-order) code into its `[x, y, z]` components by
/// gathering every third bit of `code` onto each axis.
pub fn decode_morton_3d(code: u32) -> [u32; 3] {
    [
        compact_bits(code),
        compact_bits(code >> 1),
        compact_bits(code >> 2),
    ]
}

/// Gathers every third bit of `n` (bits `0, 3, 6, ...`) into the low bits.
fn compact_bits(mut n: u32) -> u32 {
    n &= 0x4924_9249;
    n = (n ^ (n >> 2)) & 0xc30c_30c3;
    n = (n ^ (n >> 4)) & 0x0f00_f00f;
    n = (n ^ (n >> 8)) & 0x00ff_00ff;
    n = (n ^ (n >> 16)) & 0x0000_ffff;
    n
}

#[cfg(test)]
mod tests {
    use crate::decode_morton_3d;

    #[test]
    fn decodes_known_codes() {
        assert_eq!(decode_morton_3d(0), [0, 0, 0]);
        // Bit 0 -> x, bit 1 -> y, bit 2 -> z.
        assert_eq!(decode_morton_3d(1), [1, 0, 0]);
        assert_eq!(decode_morton_3d(2), [0, 1, 0]);
        assert_eq!(decode_morton_3d(4), [0, 0, 1]);
        // Chunk grid coordinates observed in real .vmax files.
        assert_eq!(decode_morton_3d(27), [3, 3, 0]);
        assert_eq!(decode_morton_3d(82), [4, 3, 0]);
        assert_eq!(decode_morton_3d(137), [3, 4, 0]);
        assert_eq!(decode_morton_3d(192), [4, 4, 0]);
        // Full 32^3 chunk extent.
        assert_eq!(decode_morton_3d(32767), [31, 31, 31]);
    }
}
