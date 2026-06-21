/// Encodes a position to its 3D Hilbert-curve index (Skilling's transform),
/// `bits` bits per axis. `bits` must be `<= 17` so the index stays exact.
pub fn encode_hilbert(x: u32, y: u32, z: u32, bits: u32) -> u64 {
    let mut axes = [x, y, z];

    // Skilling's forward transform, branchless. For each axis: a set control bit
    // flips the low bits of axes[0]; a clear one swaps the low bits of axes[0]
    // and axes[i]. `m` is the control bit smeared to a full mask so both cases
    // collapse to the same XORs with no data-dependent branch.
    for shift in (1..bits).rev() {
        let lower = (1u32 << shift) - 1;
        for i in 0..3 {
            let m = ((axes[i] >> shift) & 1).wrapping_neg();
            let t = (axes[0] ^ axes[i]) & lower;
            axes[0] ^= t ^ (m & (t ^ lower)); // m ? lower : t
            axes[i] ^= t & !m; // m ? 0 : t
        }
    }

    // Gray-encode the axes.
    axes[1] ^= axes[0];
    axes[2] ^= axes[1];

    // Fold the tail correction into every axis. The per-bit loop XORed
    // `(1 << s) - 1` for each set bit `s` of axes[2]; the closed form below
    // produces the same value, where bit `j` of `t` is the parity of the axes[2]
    // bits strictly above `j`.
    let mut g = axes[2];
    g ^= g >> 1;
    g ^= g >> 2;
    g ^= g >> 4;
    g ^= g >> 8;
    g ^= g >> 16;
    let t = g >> 1;
    axes[0] ^= t;
    axes[1] ^= t;
    axes[2] ^= t;

    // Interleave the axes into one index (axes[0] most significant within each
    // 3-bit group) by spreading each axis's bits with `split3`.
    split3(axes[0] as u64) << 2 | split3(axes[1] as u64) << 1 | split3(axes[2] as u64)
}

/// Spreads the low 21 bits of `x` so that input bit `i` lands at output bit
/// `3 * i`, zeroing the two bits between each — the bit-interleave half of a 3D
/// Morton code. Inverse of `compact3` in `decode_hilbert`.
fn split3(mut x: u64) -> u64 {
    x &= 0x1fffff;
    x = (x | x << 32) & 0x1f00000000ffff;
    x = (x | x << 16) & 0x1f0000ff0000ff;
    x = (x | x << 8) & 0x100f00f00f00f00f;
    x = (x | x << 4) & 0x10c30c30c30c30c3;
    x = (x | x << 2) & 0x1249249249249249;
    x
}

#[cfg(test)]
mod tests {
    use crate::encode_hilbert;

    #[test]
    fn matches_spec_example() {
        // 2 x 2 x 1 square, bits = 1; encode order from the format spec.
        assert_eq!(encode_hilbert(0, 0, 0, 1), 0);
        assert_eq!(encode_hilbert(0, 1, 0, 1), 3);
        assert_eq!(encode_hilbert(1, 1, 0, 1), 4);
        assert_eq!(encode_hilbert(1, 0, 0, 1), 7);
    }

    #[test]
    fn index_is_a_bijection_over_a_cube() {
        let bits = 4;
        let mut seen = std::collections::HashSet::new();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    assert!(seen.insert(encode_hilbert(x, y, z, bits)));
                }
            }
        }
        assert_eq!(seen.len(), 16 * 16 * 16);
    }
}
