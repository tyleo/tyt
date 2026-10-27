/// Recommended tolerance for treating a vector's length as zero. A vector
/// shorter than this has no reliable direction. Both `normalize` and
/// `from_axis_angle` require a non-degenerate input under glam's fail-fast
/// contract, so guard them with this and fall back to the zero or identity
/// case.
pub const ZERO_LENGTH_TOLERANCE: f64 = 1e-12;
