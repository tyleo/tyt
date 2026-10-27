use ty_math::TyVector2F64;

/// `point` turned by the rotation that takes the unit direction of largest dot
/// product with it onto +v, with u folded to `abs(u)`. The earliest direction
/// wins a tie.
pub fn fold_to_direction(point: TyVector2F64, directions: &[TyVector2F64]) -> TyVector2F64 {
    let direction = directions
        .iter()
        .copied()
        .reduce(|nearest, direction| {
            if direction.dot(point) > nearest.dot(point) {
                direction
            } else {
                nearest
            }
        })
        .expect("a fold has a direction");

    TyVector2F64::new(
        (direction.y * point.x - direction.x * point.y).abs(),
        direction.x * point.x + direction.y * point.y,
    )
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::fold_to_direction;
    use ty_math::TyVector2F64;

    #[test]
    fn the_nearest_direction_turns_onto_v() {
        let directions = [
            TyVector2F64::new(0.0, -1.0),
            TyVector2F64::new(1.0, 0.0),
            TyVector2F64::new(0.0, 1.0),
            TyVector2F64::new(-1.0, 0.0),
        ];

        assert_eq!(
            fold_to_direction(TyVector2F64::new(2.0, 0.5), &directions),
            TyVector2F64::new(0.5, 2.0)
        );
        assert_eq!(
            fold_to_direction(TyVector2F64::new(-0.5, -3.0), &directions),
            TyVector2F64::new(0.5, 3.0)
        );
    }
}
