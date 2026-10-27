use sdfcore::SdfSide;
use ty_math::TyAxis3;

/// The axis of `side` and the sign, 1 or -1, of its direction along the axis.
pub fn side_direction(side: SdfSide) -> (TyAxis3, f64) {
    match side {
        SdfSide::NegativeX => (TyAxis3::X, -1.0),
        SdfSide::NegativeY => (TyAxis3::Y, -1.0),
        SdfSide::NegativeZ => (TyAxis3::Z, -1.0),
        SdfSide::PositiveX => (TyAxis3::X, 1.0),
        SdfSide::PositiveY => (TyAxis3::Y, 1.0),
        SdfSide::PositiveZ => (TyAxis3::Z, 1.0),
    }
}
