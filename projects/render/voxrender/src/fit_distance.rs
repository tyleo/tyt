use crate::fit_radius;
use ty_math::TyBoundsF64;

/// The distance from the center of `bounds` at which a perspective view fits
/// the bounding sphere of `bounds` into the shorter image axis with
/// [`FIT_MARGIN`](crate::FIT_MARGIN) around it.
///
/// # Arguments
/// * `fov` - the vertical field of view, in radians.
/// * `width`, `height` - the image size in pixels. Panics if either is zero.
pub fn fit_distance(bounds: &TyBoundsF64, fov: f64, width: u32, height: u32) -> f64 {
    assert!(width > 0 && height > 0, "an image has two positive sides");

    let half = (fov / 2.0).tan();

    // The horizontal half angle is the narrower one on a tall image.
    let half = if width < height {
        (half * f64::from(width) / f64::from(height)).atan()
    } else {
        fov / 2.0
    };

    fit_radius(bounds) / half.sin()
}

#[cfg(test)]
mod tests {
    use crate::{FIT_MARGIN, fit_distance};
    use std::f64::consts::PI;
    use ty_math::{TyBoundsF64, TyVector3F64};

    #[test]
    fn the_sphere_fits_the_shorter_axis() {
        let bounds = TyBoundsF64::new(TyVector3F64::ZERO, TyVector3F64::new(3.0, 4.0, 0.0));
        let radius = 5.0 * (1.0 + FIT_MARGIN);

        // A square image fits the vertical field of view.
        let square = fit_distance(&bounds, PI / 2.0, 100, 100);
        assert!((square - radius / (PI / 4.0).sin()).abs() < 1e-9);

        // A wide image still fits the vertical field of view.
        assert_eq!(fit_distance(&bounds, PI / 2.0, 200, 100), square);

        // A tall image fits the narrower horizontal field of view.
        let tall = fit_distance(&bounds, PI / 2.0, 50, 100);
        let half = ((PI / 4.0).tan() * 0.5).atan();
        assert!((tall - radius / half.sin()).abs() < 1e-9);
        assert!(tall > square);
    }
}
