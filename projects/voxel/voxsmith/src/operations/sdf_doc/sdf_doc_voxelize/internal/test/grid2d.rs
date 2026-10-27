use ty_math::TyVector2F64;

/// The points of the grid from `min` to `max` with `steps` points along each
/// axis, both ends included.
pub fn grid2d(min: TyVector2F64, max: TyVector2F64, steps: u32) -> Vec<TyVector2F64> {
    let at = |index: u32| f64::from(index) / f64::from(steps - 1);

    (0..steps)
        .flat_map(|i| (0..steps).map(move |j| (i, j)))
        .map(|(i, j)| min + (max - min) * TyVector2F64::new(at(i), at(j)))
        .collect()
}
