/// `seed` as the 32-bit two's-complement integer the hash reads.
///
/// # Panics
///
/// When `seed` is not a whole number within the 32-bit range, which model
/// evaluation's checks rule out.
pub fn noise_seed(seed: f64) -> u32 {
    assert!(
        seed.fract() == 0.0 && seed >= f64::from(i32::MIN) && seed <= f64::from(i32::MAX),
        "a seed is a whole number within the 32-bit range, but read {seed}",
    );

    seed as i32 as u32
}
