/// A floored grid coordinate clamped to `0..=last`.
pub fn clamp_index(value: f64, last: usize) -> usize {
    if value < 0.0 {
        0
    } else {
        (value as usize).min(last)
    }
}
