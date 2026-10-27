use ty_math::TyAxis3;

/// The indices of the two axes across `axis` in x, y, z order. A profile's u
/// and v map to them.
pub fn plane_axes(axis: TyAxis3) -> [usize; 2] {
    match axis {
        TyAxis3::X => [1, 2],
        TyAxis3::Y => [0, 2],
        TyAxis3::Z => [0, 1],
    }
}
