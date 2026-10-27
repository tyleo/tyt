use ty_math::TyVector3U32;

/// A hash of `seed` and `position` spread over [0, 1).
pub fn cell_noise(seed: u64, position: TyVector3U32) -> f64 {
    let mut hash = seed
        .wrapping_add(u64::from(position.x) << 42)
        .wrapping_add(u64::from(position.y) << 21)
        .wrapping_add(u64::from(position.z))
        .wrapping_mul(0x9e37_79b9_7f4a_7c15);

    hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    hash = (hash ^ (hash >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    hash ^= hash >> 31;

    (hash >> 11) as f64 / (1u64 << 53) as f64
}
