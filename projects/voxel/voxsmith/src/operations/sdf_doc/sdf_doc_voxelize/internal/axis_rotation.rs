use ty_math::{TyAxis3, TyVector3F64};

/// The rows of the right-hand rotation about `axis` by the angle whose sine and
/// cosine are `sin` and `cos`.
pub fn axis_rotation(axis: TyAxis3, sin: f64, cos: f64) -> [TyVector3F64; 3] {
    match axis {
        TyAxis3::X => [
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyVector3F64::new(0.0, cos, -sin),
            TyVector3F64::new(0.0, sin, cos),
        ],

        TyAxis3::Y => [
            TyVector3F64::new(cos, 0.0, sin),
            TyVector3F64::new(0.0, 1.0, 0.0),
            TyVector3F64::new(-sin, 0.0, cos),
        ],

        TyAxis3::Z => [
            TyVector3F64::new(cos, -sin, 0.0),
            TyVector3F64::new(sin, cos, 0.0),
            TyVector3F64::new(0.0, 0.0, 1.0),
        ],
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::axis_rotation;
    use ty_math::{TyAxis3, TyVector3F64};

    fn turn(axis: TyAxis3, point: TyVector3F64) -> TyVector3F64 {
        let rows = axis_rotation(axis, 1.0, 0.0);
        TyVector3F64::new(rows[0].dot(point), rows[1].dot(point), rows[2].dot(point))
    }

    #[test]
    fn a_quarter_turn_follows_the_right_hand_rule() {
        assert_eq!(turn(TyAxis3::X, TyVector3F64::Y), TyVector3F64::Z);
        assert_eq!(turn(TyAxis3::Y, TyVector3F64::Z), TyVector3F64::X);
        assert_eq!(turn(TyAxis3::Z, TyVector3F64::X), TyVector3F64::Y);
    }
}
