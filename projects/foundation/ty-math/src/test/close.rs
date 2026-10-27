/// True when `a` and `b` are within `1e-9` of each other.
pub fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}
