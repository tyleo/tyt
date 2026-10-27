/// Hashes `seed` and the lattice `coordinates` into 32 bits through the
/// lowbias32 mixer.
pub fn lattice_hash(seed: u32, coordinates: &[i32]) -> u32 {
    coordinates
        .iter()
        .fold(lowbias32(seed), |hash, &coordinate| {
            lowbias32(hash ^ coordinate as u32)
        })
}

/// Chris Wellons's lowbias32 integer mixer.
fn lowbias32(value: u32) -> u32 {
    let mut x = value;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::lattice_hash;

    #[test]
    fn the_hash_is_pinned() {
        assert_eq!(lattice_hash(0, &[]), 0);
        assert_eq!(lattice_hash(1, &[]), 0x6889_90c0);
        assert_eq!(lattice_hash(7, &[-1, 0, 2]), 0xb9b8_79ba);
    }

    #[test]
    fn each_coordinate_and_the_seed_change_the_hash() {
        let hash = lattice_hash(3, &[1, 2, 3]);

        assert_ne!(hash, lattice_hash(4, &[1, 2, 3]));
        assert_ne!(hash, lattice_hash(3, &[2, 1, 3]));
        assert_ne!(hash, lattice_hash(3, &[1, 2, -3]));
    }
}
