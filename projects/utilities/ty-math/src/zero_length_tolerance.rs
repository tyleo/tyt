/// Recommended tolerance for treating a vector's length as zero. A vector
/// shorter than this has no reliable direction, so guard a `normalize` or
/// `from_axis_angle` with it - both require a non-degenerate input under glam's
/// fail-fast contract - and fall back to the zero or identity case.
pub const ZERO_LENGTH_TOLERANCE: f64 = 1e-12;
