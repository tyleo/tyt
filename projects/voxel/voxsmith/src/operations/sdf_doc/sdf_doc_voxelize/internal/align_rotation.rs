use ty_math::TyVector3F64;

/// The rows of the rotation that turns the unit direction `from` onto the unit
/// direction `to` without trigonometry. The two directions never point
/// opposite ways.
pub fn align_rotation(from: TyVector3F64, to: TyVector3F64) -> [TyVector3F64; 3] {
    let v = from.cross(to);
    let c = from.dot(to);
    let k = 1.0 / (1.0 + c);

    [
        TyVector3F64::new(v.x * v.x * k + c, v.x * v.y * k - v.z, v.x * v.z * k + v.y),
        TyVector3F64::new(v.y * v.x * k + v.z, v.y * v.y * k + c, v.y * v.z * k - v.x),
        TyVector3F64::new(v.z * v.x * k - v.y, v.z * v.y * k + v.x, v.z * v.z * k + c),
    ]
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::align_rotation;
    use ty_math::TyVector3F64;

    fn turn(rows: [TyVector3F64; 3], point: TyVector3F64) -> TyVector3F64 {
        TyVector3F64::new(rows[0].dot(point), rows[1].dot(point), rows[2].dot(point))
    }

    #[test]
    fn the_rotation_takes_from_onto_to_and_keeps_lengths() {
        let from = TyVector3F64::new(1.0, 2.0, 2.0) / 3.0;
        let to = TyVector3F64::new(0.0, 0.6, -0.8);
        let rows = align_rotation(from, to);

        assert!((turn(rows, from) - to).length() < 1e-15);

        let point = TyVector3F64::new(0.3, -1.2, 0.7);
        assert!((turn(rows, point).length() - point.length()).abs() < 1e-15);
    }

    #[test]
    fn aligned_directions_take_the_identity() {
        let rows = align_rotation(TyVector3F64::Y, TyVector3F64::Y);
        assert_eq!(rows, [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z]);
    }
}
