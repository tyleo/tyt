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

#[cfg(test)]
mod tests {
    use crate::hilbert_encode;

    #[test]
    fn matches_spec_example() {
        // 2 x 2 x 1 square, bits = 1; encode order from the format spec.
        assert_eq!(hilbert_encode(0, 0, 0, 1), 0);
        assert_eq!(hilbert_encode(0, 1, 0, 1), 3);
        assert_eq!(hilbert_encode(1, 1, 0, 1), 4);
        assert_eq!(hilbert_encode(1, 0, 0, 1), 7);
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
