use ty_math::TyVector3F64;

/// The points of the grid from `min` to `max` with `steps` points along each
/// axis, both ends included.
pub fn grid3d(min: TyVector3F64, max: TyVector3F64, steps: u32) -> Vec<TyVector3F64> {
    let at = |index: u32| f64::from(index) / f64::from(steps - 1);

    (0..steps)
        .flat_map(|i| (0..steps).map(move |j| (i, j)))
        .flat_map(|(i, j)| (0..steps).map(move |k| (i, j, k)))
        .map(|(i, j, k)| min + (max - min) * TyVector3F64::new(at(i), at(j), at(k)))
        .collect()
}
