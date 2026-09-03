/// Encodes `[x, y, z]` into a 3D Morton (Z-order) code by spreading each axis's
/// bits to every third position, the inverse of
/// [`decode_morton_3d`](crate::snapshots::decode_morton_3d). Each component
/// must fit in 10 bits (`0..=1023`), far above the 32-per-axis chunk extent
/// it is used for.
pub fn encode_morton_3d(coords: [u32; 3]) -> u32 {
    spread_bits(coords[0]) | (spread_bits(coords[1]) << 1) | (spread_bits(coords[2]) << 2)
}

/// Spreads the low 10 bits of `n` to every third bit (`0, 3, 6, ...`).
fn spread_bits(mut n: u32) -> u32 {
    n &= 0x0000_03ff;
    n = (n ^ (n << 16)) & 0xff00_00ff;
    n = (n ^ (n << 8)) & 0x0300_f00f;
    n = (n ^ (n << 4)) & 0x030c_30c3;
    n = (n ^ (n << 2)) & 0x0924_9249;
    n
}

#[cfg(test)]
mod tests {
    use crate::snapshots::{decode_morton_3d, encode_morton_3d};

    #[test]
    fn encodes_known_codes() {
        assert_eq!(encode_morton_3d([0, 0, 0]), 0);
        assert_eq!(encode_morton_3d([1, 0, 0]), 1);
        assert_eq!(encode_morton_3d([0, 1, 0]), 2);
        assert_eq!(encode_morton_3d([0, 0, 1]), 4);
        assert_eq!(encode_morton_3d([3, 3, 0]), 27);
        assert_eq!(encode_morton_3d([31, 31, 31]), 32767);
    }

    #[test]
    fn round_trips_over_a_chunk() {
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    assert_eq!(decode_morton_3d(encode_morton_3d([x, y, z])), [x, y, z]);
                }
            }
        }
    }
}
