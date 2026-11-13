use crate::{Error, Result};
use std::f64::consts::PI;

/// How a view maps the scene onto the image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderProjection {
    /// Rays fan out from the view's position between the image's four
    /// edges. An edge is the slope of its side of the image from the view's
    /// -Z, the tangent of its angle. An edge toward the view's -X or -Y is
    /// negative.
    Perspective {
        /// The edge toward the view's -X.
        left: f64,

        /// The edge toward the view's +X.
        right: f64,

        /// The edge toward the view's +Y.
        up: f64,

        /// The edge toward the view's -Y.
        down: f64,
    },

    /// Rays run parallel along the view's -Z.
    Orthographic {
        /// The world units across the shorter image axis.
        scale: f64,
    },
}

impl RenderProjection {
    /// The perspective projection of a view centered on its -Z over an image
    /// of square pixels. Errors unless `fov` is within `(0, pi)` and the
    /// image has two positive sides.
    ///
    /// # Arguments
    ///
    /// 1. `fov`: the radians between the top and bottom edges
    /// 2. `width`, `height`: the image's shape, which sets how far apart the
    ///    left and right edges lie
    pub fn from_fov(fov: f64, width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::ImageSide { width, height });
        }

        if !(fov > 0.0 && fov < PI) {
            return Err(Error::FieldOfViewOutOfRange { fov });
        }

        let half = (fov / 2.0).tan();
        let across = half * (f64::from(width) / f64::from(height));

        Ok(RenderProjection::Perspective {
            left: -across,
            right: across,
            up: half,
            down: -half,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, RenderProjection};
    use std::f64::consts::PI;

    #[test]
    fn a_field_of_view_centers_its_edges_over_the_images_shape() {
        let RenderProjection::Perspective {
            left,
            right,
            up,
            down,
        } = RenderProjection::from_fov(PI / 2.0, 4, 2).unwrap()
        else {
            panic!("a field of view gives a perspective projection");
        };

        assert!((up - 1.0).abs() < 1e-12);
        assert!((right - 2.0).abs() < 1e-12);
        assert_eq!(left, -right);
        assert_eq!(down, -up);
    }

    #[test]
    fn a_field_of_view_holds_to_its_range_and_the_image_to_its_sides() {
        assert_eq!(
            RenderProjection::from_fov(PI, 4, 2),
            Err(Error::FieldOfViewOutOfRange { fov: PI })
        );
        assert_eq!(
            RenderProjection::from_fov(0.0, 4, 2),
            Err(Error::FieldOfViewOutOfRange { fov: 0.0 })
        );
        assert_eq!(
            RenderProjection::from_fov(1.0, 0, 2),
            Err(Error::ImageSide {
                width: 0,
                height: 2
            })
        );
    }
}
