/// The square of the distance from `point` to the box of `bounds`, its low
/// corner and then its high one. Zero inside the box.
pub fn squared_distance(bounds: &[f32; 6], point: [f32; 3]) -> f32 {
    (0..3)
        .map(|axis| {
            let off = point[axis].clamp(bounds[axis], bounds[axis + 3]) - point[axis];

            off * off
        })
        .sum()
}
