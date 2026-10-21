use crate::{
    Error, GRID_DIRECTION_BITS, GRID_FRACTION_BITS, GRID_GUARD_BITS, RenderGridViewRays,
    RenderProjection, RenderRay, RenderScene, RenderView, Result, grid_point, grid_vector,
    quantize_point,
};
use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};

/// The rays through the pixel centers of a view over an image. Rows run top
/// to bottom. A perspective ray fans out from the view's position by its
/// field of view and the image's aspect. An orthographic ray runs along the
/// view's -Z from a point on an image plane whose shorter axis spans the
/// view's scale.
///
/// [`new`](Self::new) lowers the projection into pixel steps.
/// [`ray`](Self::ray) adds those steps without branching on the projection.
/// [`to_grid_rays`](Self::to_grid_rays) carries the steps into a
/// placement's grid as integers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderViewRays {
    width: u32,

    height: u32,

    // Pixel (0, 0)'s ray. The `_x` fields step to the next pixel along a row
    // and the `_y` fields step down a column. `ray` normalizes the direction.
    origin: TyVector3F64,

    origin_x: TyVector3F64,

    origin_y: TyVector3F64,

    direction: TyVector3F64,

    direction_x: TyVector3F64,

    direction_y: TyVector3F64,
}

impl RenderViewRays {
    /// The rays of `view` over a `width` by `height` image. Panics if a side
    /// is zero.
    pub fn new(view: &RenderView, width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "an image has two positive sides");

        let position = view.pose.position;
        let rotation = view.pose.rotation;
        let forward = rotation * -TyVector3F64::Z;

        let columns = f64::from(width);
        let rows = f64::from(height);
        let aspect = columns / rows;

        // Pixel (0, 0)'s offset from the image center and the steps to the
        // next pixel, on an image plane with these half extents.
        let spread = |half_x: f64, half_y: f64| {
            let right = rotation * TyVector3F64::X * half_x;
            let up = rotation * TyVector3F64::Y * half_y;

            (
                right * (1.0 / columns - 1.0) + up * (1.0 - 1.0 / rows),
                right * (2.0 / columns),
                up * (-2.0 / rows),
            )
        };

        match view.projection {
            RenderProjection::Perspective { fov } => {
                let half = (fov / 2.0).tan();
                let (corner, step_x, step_y) = spread(half * aspect, half);

                Self {
                    width,
                    height,
                    origin: position,
                    origin_x: TyVector3F64::ZERO,
                    origin_y: TyVector3F64::ZERO,
                    direction: forward + corner,
                    direction_x: step_x,
                    direction_y: step_y,
                }
            }

            RenderProjection::Orthographic { scale } => {
                let (half_x, half_y) = if width >= height {
                    (scale / 2.0 * aspect, scale / 2.0)
                } else {
                    (scale / 2.0, scale / 2.0 / aspect)
                };

                let (corner, step_x, step_y) = spread(half_x, half_y);

                Self {
                    width,
                    height,
                    origin: position + corner,
                    origin_x: step_x,
                    origin_y: step_y,
                    direction: forward,
                    direction_x: TyVector3F64::ZERO,
                    direction_y: TyVector3F64::ZERO,
                }
            }
        }
    }

    /// The ray through the center of pixel `(x, y)`. Panics if the pixel is
    /// outside the image.
    pub fn ray(&self, x: u32, y: u32) -> RenderRay {
        assert!(
            x < self.width && y < self.height,
            "the pixel is within the image"
        );

        let x = f64::from(x);
        let y = f64::from(y);

        RenderRay {
            origin: self.origin + self.origin_x * x + self.origin_y * y,
            direction: (self.direction + self.direction_x * x + self.direction_y * y).normalize(),
        }
    }

    /// Pixel (0, 0)'s ray origin, then the steps to the next pixel along a
    /// row and down a column.
    pub fn origin_steps(&self) -> [TyVector3F64; 3] {
        [self.origin, self.origin_x, self.origin_y]
    }

    /// Pixel (0, 0)'s ray direction before [`ray`](Self::ray) normalizes it,
    /// then the steps to the next pixel along a row and down a column.
    pub fn direction_steps(&self) -> [TyVector3F64; 3] {
        [self.direction, self.direction_x, self.direction_y]
    }

    /// These rays in the grid of `bounds` of a placement under `transform`,
    /// or `None` when a pixel's origin lies out of the grid's
    /// [range](crate::GRID_RANGE_BITS). The direction scale leaves room under
    /// `1 << GRID_DIRECTION_BITS` for the steps' rounding.
    pub fn to_grid_rays(
        &self,
        transform: &TyTransformF64,
        bounds: TyVector3U32,
    ) -> Option<RenderGridViewRays> {
        let origin = grid_point(transform, self.origin);
        let origin_x = grid_vector(transform, self.origin_x);
        let origin_y = grid_vector(transform, self.origin_y);
        let direction = grid_vector(transform, self.direction);
        let direction_x = grid_vector(transform, self.direction_x);
        let direction_y = grid_vector(transform, self.direction_y);

        // Every ray lies between the corner pixels' rays.
        let last_x = f64::from(self.width - 1);
        let last_y = f64::from(self.height - 1);
        let corners = [(0.0, 0.0), (last_x, 0.0), (0.0, last_y), (last_x, last_y)];

        // A far-out corner fails here before its steps could overflow. The
        // integer corners settle the range's edge.
        for (x, y) in corners {
            quantize_point(origin + origin_x * x + origin_y * y, bounds)?;
        }

        let largest = corners
            .iter()
            .map(|&(x, y)| {
                (direction + direction_x * x + direction_y * y)
                    .abs()
                    .max_element()
            })
            .fold(0.0, f64::max);

        let margin = 1 + ((u64::from(self.width) + u64::from(self.height)) >> GRID_GUARD_BITS);
        let reach = 2f64.powi(GRID_DIRECTION_BITS as i32) - margin as f64;

        let guard = 2f64.powi(GRID_GUARD_BITS as i32);
        let origin_scale = guard * 2f64.powi(GRID_FRACTION_BITS as i32);
        let direction_scale = guard * reach / largest;

        let rays = RenderGridViewRays {
            width: self.width,
            height: self.height,
            origin: fixed(origin, origin_scale),
            origin_x: fixed(origin_x, origin_scale),
            origin_y: fixed(origin_y, origin_scale),
            direction: fixed(direction, direction_scale),
            direction_x: fixed(direction_x, direction_scale),
            direction_y: fixed(direction_y, direction_scale),
        };

        rays.in_range(bounds).then_some(rays)
    }

    /// The rays in each placement's grid of `scene`, in placement order.
    /// Errors with `GridRange` when a placement's rays leave the range.
    pub fn to_scene_grid_rays(&self, scene: &RenderScene) -> Result<Vec<RenderGridViewRays>> {
        scene
            .iter_placements()
            .map(|(placement_id, placement)| {
                let bounds = scene
                    .object(placement.object_id)
                    .expect("a placement's object is one of the scene's")
                    .bounds();

                self.to_grid_rays(&placement.transform, bounds)
                    .ok_or(Error::GridRange { placement_id })
            })
            .collect()
    }
}

fn fixed(vector: TyVector3F64, scale: f64) -> [i64; 3] {
    vector
        .to_array()
        .map(|component| (component * scale).round() as i64)
}

#[cfg(test)]
mod tests {
    use crate::{RenderProjection, RenderView, RenderViewRays};
    use std::f64::consts::PI;
    use ty_math::{TyPoseF64, TyQuaternionF64, TyVector3F64};

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        (a - b).length() < 1e-9
    }

    #[test]
    fn perspective_rays_fan_from_the_position_by_the_field_of_view() {
        let view = RenderView {
            pose: TyPoseF64::new(TyVector3F64::new(0.0, 0.0, 5.0), TyQuaternionF64::IDENTITY),
            projection: RenderProjection::Perspective { fov: PI / 2.0 },
        };

        let rays = RenderViewRays::new(&view, 4, 2);

        // A 4 by 2 image: the center of the top-left pixel sits at three
        // quarters of the half width and half of the half height.
        let ray = rays.ray(0, 0);
        assert_eq!(ray.origin, TyVector3F64::new(0.0, 0.0, 5.0));
        assert!(close(
            ray.direction,
            TyVector3F64::new(-1.5, 0.5, -1.0).normalize()
        ));

        // The two middle pixels of the bottom row straddle the center.
        assert!(close(
            rays.ray(1, 1).direction,
            TyVector3F64::new(-0.5, -0.5, -1.0).normalize()
        ));
        assert!(close(
            rays.ray(2, 1).direction,
            TyVector3F64::new(0.5, -0.5, -1.0).normalize()
        ));
    }

    #[test]
    fn orthographic_rays_run_parallel_across_the_scale() {
        let turned = TyQuaternionF64::from_axis_angle(TyVector3F64::Y, PI / 2.0);
        let view = RenderView {
            pose: TyPoseF64::new(TyVector3F64::new(3.0, 0.0, 0.0), turned),
            projection: RenderProjection::Orthographic { scale: 2.0 },
        };

        // The scale spans the height, the shorter axis, so the width spans 4.
        let ray = RenderViewRays::new(&view, 4, 2).ray(0, 0);
        assert!(close(ray.direction, -TyVector3F64::X));
        assert!(close(
            ray.origin,
            TyVector3F64::new(3.0, 0.5, 0.0) + turned * TyVector3F64::new(-1.5, 0.0, 0.0)
        ));

        // On a tall image the scale spans the width instead.
        let tall = RenderViewRays::new(&view, 2, 4).ray(0, 0);
        assert!(close(
            tall.origin,
            TyVector3F64::new(3.0, 1.5, 0.0) + turned * TyVector3F64::new(-0.5, 0.0, 0.0)
        ));
    }

    #[test]
    fn the_steps_sum_to_each_pixels_ray() {
        let projections = [
            RenderProjection::Perspective { fov: 1.2 },
            RenderProjection::Orthographic { scale: 3.0 },
        ];

        for projection in projections {
            let view = RenderView {
                pose: TyPoseF64::new(
                    TyVector3F64::new(1.0, 2.0, 3.0),
                    TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 0.3),
                ),
                projection,
            };

            let rays = RenderViewRays::new(&view, 5, 3);
            let [origin, origin_x, origin_y] = rays.origin_steps();
            let [direction, direction_x, direction_y] = rays.direction_steps();

            for (x, y) in [(0, 0), (4, 0), (2, 1), (4, 2)] {
                let ray = rays.ray(x, y);
                let (x, y) = (f64::from(x), f64::from(y));

                assert!(close(ray.origin, origin + origin_x * x + origin_y * y));
                assert!(close(
                    ray.direction,
                    (direction + direction_x * x + direction_y * y).normalize()
                ));
            }
        }
    }
}
