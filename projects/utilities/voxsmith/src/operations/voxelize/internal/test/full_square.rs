/// A full square UV layout over a quad, mapping each corner to a texture
/// corner.
pub(crate) fn full_square() -> [[f64; 2]; 4] {
    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
}
