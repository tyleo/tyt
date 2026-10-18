use crate::{GRID_FRACTION_BITS, GRID_RANGE_BITS};
use ty_math::{TyVector3F64, TyVector3U32};

/// `point` in grid units rounded to fixed point, or `None` when a coordinate
/// is not finite or the point, its mirror across a grid of `bounds`, or the
/// grid lies out of [range](GRID_RANGE_BITS).
pub fn quantize_point(point: TyVector3F64, bounds: TyVector3U32) -> Option<[i32; 3]> {
    let range = 2f64.powi(GRID_RANGE_BITS as i32);
    let unit = 2f64.powi(GRID_FRACTION_BITS as i32);

    let mut fixed = [0; 3];

    for ((fixed, coordinate), side) in fixed
        .iter_mut()
        .zip(point.to_array())
        .zip(bounds.to_array())
    {
        let rounded = (coordinate * unit).round() / unit;
        let mirror = f64::from(side) - rounded;

        if !coordinate.is_finite()
            || rounded.abs() >= range
            || mirror.abs() >= range
            || f64::from(side) >= range
        {
            return None;
        }

        *fixed = (rounded * unit) as i32;
    }

    Some(fixed)
}

#[cfg(test)]
mod tests {
    use crate::quantize_point;
    use ty_math::{TyVector3F64, TyVector3U32};

    #[test]
    fn a_point_rounds_to_fixed_point_within_the_range() {
        let bounds = TyVector3U32::splat(4);

        assert_eq!(
            quantize_point(TyVector3F64::new(1.0, -0.5, 0.3), bounds),
            Some([8192, -4096, 2458])
        );
        assert_eq!(
            quantize_point(TyVector3F64::new(0.0, 0.0, f64::NAN), bounds),
            None
        );

        // The point and its mirror across the grid both stay under 2^18
        // cells from the grid's origin.
        let range = 2f64.powi(18);
        let point = |x| quantize_point(TyVector3F64::new(x, 0.0, 0.0), bounds);

        assert!(point(range - 1.0).is_some());
        assert_eq!(point(range), None);
        assert!(point(5.0 - range).is_some());
        assert_eq!(point(4.0 - range), None);

        let long = TyVector3U32::new(1 << 18, 1, 1);
        assert_eq!(quantize_point(TyVector3F64::ZERO, long), None);
    }
}
