use std::f64::consts::FRAC_1_SQRT_2;

/// The sine and cosine of `degrees`, exact for every multiple of 30 or 45
/// degrees. A quarter turn then keeps a box's corners on the cell corners.
pub fn exact_sin_cos(degrees: f64) -> (f64, f64) {
    let reduced = degrees.rem_euclid(360.0);

    // A tiny negative angle reduces to 360 by rounding.
    let reduced = if reduced == 360.0 { 0.0 } else { reduced };

    let half_sqrt3 = 3.0_f64.sqrt() / 2.0;

    match reduced {
        0.0 => (0.0, 1.0),
        30.0 => (0.5, half_sqrt3),
        45.0 => (FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        60.0 => (half_sqrt3, 0.5),
        90.0 => (1.0, 0.0),
        120.0 => (half_sqrt3, -0.5),
        135.0 => (FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        150.0 => (0.5, -half_sqrt3),
        180.0 => (0.0, -1.0),
        210.0 => (-0.5, -half_sqrt3),
        225.0 => (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        240.0 => (-half_sqrt3, -0.5),
        270.0 => (-1.0, 0.0),
        300.0 => (-half_sqrt3, 0.5),
        315.0 => (-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        330.0 => (-0.5, half_sqrt3),
        _ => reduced.to_radians().sin_cos(),
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::exact_sin_cos;

    #[test]
    fn quarter_turns_take_exact_zeros_and_ones() {
        assert_eq!(exact_sin_cos(90.0), (1.0, 0.0));
        assert_eq!(exact_sin_cos(-90.0), (-1.0, 0.0));
        assert_eq!(exact_sin_cos(450.0), (1.0, 0.0));
        assert_eq!(exact_sin_cos(-1e-20), (0.0, 1.0));
    }

    #[test]
    fn other_angles_agree_with_the_library() {
        for degrees in [12.5, 30.0, 45.0, 60.0, 135.0, 200.0, 315.0, -30.0] {
            let (sin, cos) = exact_sin_cos(degrees);
            let (expected_sin, expected_cos) = degrees.to_radians().sin_cos();
            assert!((sin - expected_sin).abs() < 1e-15, "{degrees}");
            assert!((cos - expected_cos).abs() < 1e-15, "{degrees}");
        }
    }
}
