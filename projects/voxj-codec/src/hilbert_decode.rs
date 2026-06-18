/// Inverse of [`hilbert_encode`](crate::hilbert_encode).
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
    use crate::{hilbert_decode, hilbert_encode};

    #[test]
    fn matches_spec_example() {
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
}
