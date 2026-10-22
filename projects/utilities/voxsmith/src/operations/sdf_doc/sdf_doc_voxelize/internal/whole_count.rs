/// `count` as a whole number of copies, sides, points, or octaves.
///
/// # Panics
///
/// When `count` is not a whole number above zero, which the sdfj format's
/// validation rules out.
pub fn whole_count(count: f64) -> u32 {
    assert!(
        count >= 1.0 && count.fract() == 0.0 && count <= f64::from(u32::MAX),
        "a count is a whole number above zero, but read {count}",
    );

    count as u32
}
