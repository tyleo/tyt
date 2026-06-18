/// Number of binary digits in `n`, with `bit_length(0) == 0`.
pub fn bit_length(mut n: u32) -> u32 {
    let mut len = 0;
    while n > 0 {
        n /= 2;
        len += 1;
    }
    len
}

/// Hilbert `bits` per axis for the given `[X, Y, Z]` bounds: the bit width
/// needed to index the largest axis.
pub fn hilbert_bits(bounds: [u32; 3]) -> u32 {
    let max_dim = bounds[0].max(bounds[1]).max(bounds[2]);
    bit_length(max_dim.saturating_sub(1)).max(1)
}

/// Bit-packing width for a palette with `cell_count` cells.
pub fn packed_width(cell_count: usize) -> u32 {
    bit_length((cell_count.max(1) - 1) as u32).max(1)
}

/// Encodes a position to its 3D Hilbert-curve index (Skilling's transform),
/// `bits` bits per axis. `bits` must be `<= 17` so the index stays exact.
pub fn hilbert_encode(x: u32, y: u32, z: u32, bits: u32) -> u64 {
    let mut axes = [x, y, z];
    let top_bit = 1u32 << (bits - 1);

    let mut mask = top_bit;
    while mask > 1 {
        let lower = mask - 1;
        for i in 0..3 {
            if axes[i] & mask != 0 {
                axes[0] ^= lower;
            } else {
                let t = (axes[0] ^ axes[i]) & lower;
                axes[0] ^= t;
                axes[i] ^= t;
            }
        }
        mask >>= 1;
    }

    for i in 1..3 {
        axes[i] ^= axes[i - 1];
    }
    let mut t = 0;
    let mut mask = top_bit;
    while mask > 1 {
        if axes[2] & mask != 0 {
            t ^= mask - 1;
        }
        mask >>= 1;
    }
    for axis in &mut axes {
        *axis ^= t;
    }

    // Interleave into a single index (axes[0] most significant).
    let mut index = 0u64;
    for k in (0..bits).rev() {
        for axis in axes {
            index = index * 2 + ((axis >> k) & 1) as u64;
        }
    }
    index
}

/// Inverse of [`hilbert_encode`].
pub fn hilbert_decode(index: u64, bits: u32) -> [u32; 3] {
    let total_bits = 3 * bits;
    let mut axes = [0u32; 3];

    // De-interleave the index back into the three axes.
    for p in 0..total_bits {
        let bit_value = ((index >> (total_bits - 1 - p)) & 1) as u32;
        let k = bits - 1 - p / 3;
        axes[(p % 3) as usize] |= bit_value << k;
    }

    // Invert the encode transform.
    let size = 2u32 << (bits - 1);
    let gray = axes[2] >> 1;
    for i in (1..3).rev() {
        axes[i] ^= axes[i - 1];
    }
    axes[0] ^= gray;
    let mut mask = 2u32;
    while mask != size {
        let lower = mask - 1;
        for i in (0..3).rev() {
            if axes[i] & mask != 0 {
                axes[0] ^= lower;
            } else {
                let t = (axes[0] ^ axes[i]) & lower;
                axes[0] ^= t;
                axes[i] ^= t;
            }
        }
        mask <<= 1;
    }

    axes
}

#[cfg(test)]
mod tests {
    use crate::{bit_length, hilbert_bits, hilbert_decode, hilbert_encode, packed_width};

    #[test]
    fn bit_length_matches_reference() {
        assert_eq!(bit_length(0), 0);
        assert_eq!(bit_length(1), 1);
        assert_eq!(bit_length(2), 2);
        assert_eq!(bit_length(255), 8);
        assert_eq!(packed_width(2), 1);
        assert_eq!(packed_width(256), 8);
        assert_eq!(hilbert_bits([2, 2, 1]), 1);
    }

    #[test]
    fn matches_spec_example() {
        // 2 x 2 x 1 square, bits = 1; encode order from the format spec.
        assert_eq!(hilbert_encode(0, 0, 0, 1), 0);
        assert_eq!(hilbert_encode(0, 1, 0, 1), 3);
        assert_eq!(hilbert_encode(1, 1, 0, 1), 4);
        assert_eq!(hilbert_encode(1, 0, 0, 1), 7);
        // Indices decode back to those positions.
        assert_eq!(hilbert_decode(0, 1), [0, 0, 0]);
        assert_eq!(hilbert_decode(3, 1), [0, 1, 0]);
        assert_eq!(hilbert_decode(4, 1), [1, 1, 0]);
        assert_eq!(hilbert_decode(7, 1), [1, 0, 0]);
    }

    #[test]
    fn round_trips_over_a_cube() {
        let bits = 5; // 32^3
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let i = hilbert_encode(x, y, z, bits);
                    assert_eq!(hilbert_decode(i, bits), [x, y, z]);
                }
            }
        }
    }

    #[test]
    fn index_is_a_bijection_over_a_cube() {
        let bits = 4;
        let mut seen = std::collections::HashSet::new();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    assert!(seen.insert(hilbert_encode(x, y, z, bits)));
                }
            }
        }
        assert_eq!(seen.len(), 16 * 16 * 16);
    }
}
