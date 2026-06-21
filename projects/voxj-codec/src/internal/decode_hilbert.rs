/// Inverse of [`encode_hilbert`](crate::encode_hilbert).
pub fn decode_hilbert(index: u64, bits: u32) -> [u32; 3] {
    // De-interleave the index back into the three axes (axes[0] held the most
    // significant bit of each 3-bit group) by gathering each axis with
    // `compact3`.
    let mut axes = [
        compact3(index >> 2) as u32,
        compact3(index >> 1) as u32,
        compact3(index) as u32,
    ];

    // Invert the gray-code and tail fold.
    let gray = axes[2] >> 1;
    axes[2] ^= axes[1];
    axes[1] ^= axes[0];
    axes[0] ^= gray;

    // Invert Skilling's transform, branchless (mirror of the encode loop). `m`
    // smears the control bit to a full mask so the set/clear cases share one
    // pair of XORs with no data-dependent branch.
    let size = 1u32 << bits;
    let mut mask = 2u32;
    let mut shift = 1u32;
    while mask != size {
        let lower = mask - 1;
        for i in (0..3).rev() {
            let m = ((axes[i] >> shift) & 1).wrapping_neg();
            let t = (axes[0] ^ axes[i]) & lower;
            axes[0] ^= t ^ (m & (t ^ lower));
            axes[i] ^= t & !m;
        }
        mask <<= 1;
        shift += 1;
    }

    axes
}

/// Gathers the bits of `x` at positions `3 * i` back down to position `i`,
/// discarding the rest — the de-interleave half of a 3D Morton code. Inverse of
/// `split3` in `encode_hilbert`.
fn compact3(mut x: u64) -> u64 {
    x &= 0x1249249249249249;
    x = (x | x >> 2) & 0x10c30c30c30c30c3;
    x = (x | x >> 4) & 0x100f00f00f00f00f;
    x = (x | x >> 8) & 0x1f0000ff0000ff;
    x = (x | x >> 16) & 0x1f00000000ffff;
    x = (x | x >> 32) & 0x1fffff;
    x
}

#[cfg(test)]
mod tests {
    use crate::{decode_hilbert, encode_hilbert};

    #[test]
    fn matches_spec_example() {
        assert_eq!(decode_hilbert(0, 1), [0, 0, 0]);
        assert_eq!(decode_hilbert(3, 1), [0, 1, 0]);
        assert_eq!(decode_hilbert(4, 1), [1, 1, 0]);
        assert_eq!(decode_hilbert(7, 1), [1, 0, 0]);
    }

    #[test]
    fn round_trips_over_a_cube() {
        let bits = 5; // 32^3
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let i = encode_hilbert(x, y, z, bits);
                    assert_eq!(decode_hilbert(i, bits), [x, y, z]);
                }
            }
        }
    }
}
