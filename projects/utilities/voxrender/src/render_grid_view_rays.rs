use crate::{GRID_FRACTION_BITS, GRID_GUARD_BITS, GRID_RANGE_BITS, RenderGridRay};
use std::array;
use ty_math::TyVector3U32;

/// A view's pixel rays in one placement's grid as integer steps, so every
/// renderer derives the same ray for a pixel. Made by
/// [`RenderViewRays::to_grid_rays`](crate::RenderViewRays::to_grid_rays).
/// Every field below `height` carries [`GRID_GUARD_BITS`] extra fraction
/// bits. A pixel's ray adds the steps and rounds the guard bits off once,
/// halves up.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderGridViewRays {
    /// The image's width in pixels.
    pub width: u32,

    /// The image's height in pixels.
    pub height: u32,

    /// Pixel (0, 0)'s origin.
    pub origin: [i64; 3],

    /// The origin's step to the next pixel along a row.
    pub origin_x: [i64; 3],

    /// The origin's step to the next pixel down a column.
    pub origin_y: [i64; 3],

    /// Pixel (0, 0)'s direction.
    pub direction: [i64; 3],

    /// The direction's step to the next pixel along a row.
    pub direction_x: [i64; 3],

    /// The direction's step to the next pixel down a column.
    pub direction_y: [i64; 3],
}

impl RenderGridViewRays {
    /// The ray through the center of pixel `(x, y)`. Panics if the pixel is
    /// outside the image or the rays are not [in range](Self::in_range).
    pub fn ray(&self, x: u32, y: u32) -> RenderGridRay {
        assert!(
            x < self.width && y < self.height,
            "the pixel is within the image"
        );

        let origin = self
            .origin_at(x, y)
            .map(|coordinate| i32::try_from(coordinate).expect("the rays are within range"));

        let direction =
            step(self.direction, self.direction_x, self.direction_y, x, y).map(|component| {
                i32::try_from(component).expect("a quantized direction fits the direction bits")
            });

        RenderGridRay {
            origin,
            direction,
            end: None,
        }
    }

    /// Whether a grid of `bounds`, every pixel's origin, and the origin's
    /// mirror across the grid lie within [range](GRID_RANGE_BITS). Each
    /// origin is linear across the image before rounding, so the corner
    /// pixels bound the rest.
    pub fn in_range(&self, bounds: TyVector3U32) -> bool {
        let range = 1i64 << (GRID_RANGE_BITS + GRID_FRACTION_BITS);
        let bounds = bounds
            .to_array()
            .map(|side| i64::from(side) << GRID_FRACTION_BITS);

        let last_x = self.width - 1;
        let last_y = self.height - 1;

        bounds.iter().all(|&side| side < range)
            && [(0, 0), (last_x, 0), (0, last_y), (last_x, last_y)]
                .into_iter()
                .all(|(x, y)| {
                    let origin = self.origin_at(x, y);

                    (0..3).all(|a| origin[a].abs() < range && (bounds[a] - origin[a]).abs() < range)
                })
    }

    fn origin_at(&self, x: u32, y: u32) -> [i64; 3] {
        step(self.origin, self.origin_x, self.origin_y, x, y)
    }
}

/// `base` stepped `x` times along a row and `y` times down a column, with
/// its guard bits rounded off, halves up.
fn step(base: [i64; 3], along_x: [i64; 3], along_y: [i64; 3], x: u32, y: u32) -> [i64; 3] {
    let half = 1 << (GRID_GUARD_BITS - 1);

    array::from_fn(|a| {
        (base[a] + along_x[a] * i64::from(x) + along_y[a] * i64::from(y) + half) >> GRID_GUARD_BITS
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        RenderProjection, RenderView, RenderViewRays, grid_point, grid_vector, quantize_point,
    };
    use ty_math::{TyPoseF64, TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3U32};

    #[test]
    fn each_grid_ray_follows_its_float_ray() {
        let pose = TyPoseF64::new(
            TyVector3F64::new(3.0, 4.0, 20.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.2, -0.3, 0.1).normalize(), 0.4),
        );
        let transform = TyTransformF64::new(
            TyVector3F64::new(-1.0, 2.0, 0.5),
            TyQuaternionF64::from_rotation_y(0.7),
            TyVector3F64::new(0.5, 0.25, 0.5),
        );
        let bounds = TyVector3U32::splat(8);

        for projection in [
            RenderProjection::Perspective { fov: 0.8 },
            RenderProjection::Orthographic { scale: 6.0 },
        ] {
            let rays = RenderViewRays::new(&RenderView { pose, projection }, 37, 23);
            let grid = rays.to_grid_rays(&transform, bounds).unwrap();

            for y in 0..23 {
                for x in 0..37 {
                    let float = rays.ray(x, y);
                    let ray = grid.ray(x, y);

                    let origin =
                        quantize_point(grid_point(&transform, float.origin), bounds).unwrap();
                    assert!(
                        (0..3).all(|a| (ray.origin[a] - origin[a]).abs() <= 1),
                        "{ray:?}"
                    );

                    assert!(ray.direction.iter().all(|c| c.abs() <= 1 << 16));

                    let direction = TyVector3F64::from_array(ray.direction.map(f64::from));
                    let exact = grid_vector(&transform, float.direction);
                    assert!(
                        (direction.normalize() - exact.normalize()).length() < 1e-4,
                        "{ray:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_view_out_of_a_grids_range_has_no_grid_rays() {
        let view = RenderView {
            pose: TyPoseF64::new(TyVector3F64::new(0.0, 0.0, 5.0), TyQuaternionF64::IDENTITY),
            projection: RenderProjection::Perspective { fov: 0.8 },
        };
        let tiny = TyTransformF64::new(
            TyVector3F64::ZERO,
            TyQuaternionF64::IDENTITY,
            TyVector3F64::splat(1e-10),
        );

        let bounds = TyVector3U32::new(8, 1, 1);
        assert_eq!(
            RenderViewRays::new(&view, 4, 4).to_grid_rays(&tiny, bounds),
            None
        );

        // The origin and its mirror across the grid each stay under 2^18
        // cells from the grid's origin.
        let at = |x: f64| {
            let view = RenderView {
                pose: TyPoseF64::new(TyVector3F64::new(x, 0.5, 0.5), TyQuaternionF64::IDENTITY),
                ..view
            };

            RenderViewRays::new(&view, 4, 4)
                .to_grid_rays(&TyTransformF64::IDENTITY, bounds)
                .is_some()
        };
        let range = 2f64.powi(18);

        assert!(at(range - 0.5));
        assert!(!at(range + 0.5));
        assert!(at(8.5 - range));
        assert!(!at(7.5 - range));
    }
}
