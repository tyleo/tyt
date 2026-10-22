use crate::operations::sdf_doc::{Bounds2d, Bounds3d, side_direction};
use sdfcore::SdfSide;
use std::f64::consts::FRAC_PI_2;
use ty_math::{TyAxis3, TyVector2F64, TyVector3F64};

/// The map a `bend` applies from each point back to the unbent shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BendMap {
    along: usize,

    toward: usize,

    toward_sign: f64,

    radius: f64,

    pivot: TyVector3F64,

    center: TyVector3F64,

    middle_angle: f64,

    middle_sin: f64,

    middle_cos: f64,
}

impl BendMap {
    /// The bend of the shape with box `child` that curls `along` into an arc
    /// of `radius` toward `toward`. The slice through `pivot` stays in place.
    ///
    /// # Panics
    ///
    /// When `along` and `toward` lie on one axis, which the sdfj format's
    /// validation rules out.
    pub fn new(
        along: TyAxis3,
        toward: SdfSide,
        radius: f64,
        pivot: TyVector3F64,
        child: &Bounds3d,
    ) -> Self {
        let (toward_axis, toward_sign) = side_direction(toward);

        assert_ne!(
            along, toward_axis,
            "a bend's along and toward lie on different axes"
        );

        let along = along.index();
        let toward = toward_axis.index();

        let mut center = pivot;
        center[toward] += toward_sign * radius;

        let middle = (child.min[along] + child.max[along]) / 2.0;
        let middle_angle = (middle - pivot[along]) / radius;
        let (middle_sin, middle_cos) = middle_angle.sin_cos();

        Self {
            along,
            toward,
            toward_sign,
            radius,
            pivot,
            center,
            middle_angle,
            middle_sin,
            middle_cos,
        }
    }

    /// The point of the unbent shape that the bend takes to `point`.
    pub fn child_point(self, point: TyVector3F64) -> TyVector3F64 {
        let toward_offset = point[self.toward] - self.center[self.toward];
        let along_offset = point[self.along] - self.center[self.along];

        // The offset in the bend plane, measured toward the pivot and along
        // +along.
        let u = TyVector2F64::new(-self.toward_sign * toward_offset, along_offset);

        let cross = self.middle_cos * u.y - self.middle_sin * u.x;
        let dot = self.middle_cos * u.x + self.middle_sin * u.y;
        let angle = self.middle_angle + cross.atan2(dot);

        let distance = if self.toward < self.along {
            TyVector2F64::new(toward_offset, along_offset)
        } else {
            TyVector2F64::new(along_offset, toward_offset)
        }
        .length();

        let mut unbent = point;
        unbent[self.along] = self.pivot[self.along] + angle * self.radius;
        unbent[self.toward] = self.pivot[self.toward] + self.toward_sign * (self.radius - distance);
        unbent
    }

    /// The box around the annulus sector that the box `child` bends into.
    pub fn bounds(&self, child: &Bounds3d) -> Bounds3d {
        let first_angle = (child.min[self.along] - self.pivot[self.along]) / self.radius;
        let last_angle = (child.max[self.along] - self.pivot[self.along]) / self.radius;

        let offsets = [child.min[self.toward], child.max[self.toward]]
            .map(|coordinate| self.toward_sign * (coordinate - self.pivot[self.toward]));
        let inner = self.radius - offsets[0].max(offsets[1]);
        let outer = self.radius - offsets[0].min(offsets[1]);

        // The box in the bend plane about the arc's center, measured toward the
        // pivot and along +along. A box reaching past the center takes the
        // whole disk.
        let plane = if inner < 0.0 {
            Bounds2d::around(TyVector2F64::ZERO, TyVector2F64::splat(outer.max(-inner)))
        } else {
            let point_at = |angle: f64, radius: f64| {
                let (sin, cos) = angle.sin_cos();
                TyVector2F64::new(cos, sin) * radius
            };

            let first_quarter = (first_angle / FRAC_PI_2).ceil() as i64;
            let last_quarter = (last_angle / FRAC_PI_2).floor() as i64;

            let crossings =
                (first_quarter..=last_quarter).map(|quarter| match quarter.rem_euclid(4) {
                    0 => TyVector2F64::new(outer, 0.0),
                    1 => TyVector2F64::new(0.0, outer),
                    2 => TyVector2F64::new(-outer, 0.0),
                    _ => TyVector2F64::new(0.0, -outer),
                });

            Bounds2d::from_points(
                [
                    point_at(first_angle, inner),
                    point_at(first_angle, outer),
                    point_at(last_angle, inner),
                    point_at(last_angle, outer),
                ]
                .into_iter()
                .chain(crossings),
            )
        };

        let toward_ends = [plane.min.x, plane.max.x]
            .map(|offset| self.center[self.toward] - self.toward_sign * offset);

        let mut bounds = *child;
        bounds.min[self.along] = self.center[self.along] + plane.min.y;
        bounds.max[self.along] = self.center[self.along] + plane.max.y;
        bounds.min[self.toward] = toward_ends[0].min(toward_ends[1]);
        bounds.max[self.toward] = toward_ends[0].max(toward_ends[1]);
        bounds
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{BendMap, Bounds3d};
    use sdfcore::SdfSide;
    use std::f64::consts::{FRAC_PI_2, PI};
    use ty_math::{TyAxis3, TyVector3F64};

    #[test]
    fn the_arc_keeps_lengths_along_the_axis() {
        let child = Bounds3d {
            min: TyVector3F64::new(0.0, -0.1, -0.1),
            max: TyVector3F64::new(2.0, 0.1, 0.1),
        };
        let map = BendMap::new(
            TyAxis3::X,
            SdfSide::PositiveY,
            1.0,
            TyVector3F64::ZERO,
            &child,
        );

        // The pivot stays, and a quarter of the arc reaches the top of the
        // circle about [0, 1, 0].
        assert!((map.child_point(TyVector3F64::ZERO) - TyVector3F64::ZERO).length() < 1e-15);

        let quarter = map.child_point(TyVector3F64::new(1.0, 1.0, 0.3));
        let expected = TyVector3F64::new(FRAC_PI_2, 0.0, 0.3);
        assert!((quarter - expected).length() < 1e-15, "{quarter}");

        // A point outside the arc maps below the slice it bends from.
        let outside = map.child_point(TyVector3F64::new(1.1, 1.0, 0.0));
        assert!((outside - TyVector3F64::new(FRAC_PI_2, -0.1, 0.0)).length() < 1e-15);
    }

    #[test]
    fn the_box_wraps_the_annulus_sector() {
        let child = Bounds3d {
            min: TyVector3F64::new(0.0, -0.1, -0.2),
            max: TyVector3F64::new(PI, 0.1, 0.2),
        };
        let map = BendMap::new(
            TyAxis3::X,
            SdfSide::PositiveY,
            1.0,
            TyVector3F64::ZERO,
            &child,
        );

        // Half a turn about [0, 1, 0] between radii 0.9 and 1.1.
        let bounds = map.bounds(&child);
        let expected = Bounds3d {
            min: TyVector3F64::new(0.0, -0.1, -0.2),
            max: TyVector3F64::new(1.1, 2.1, 0.2),
        };
        assert!((bounds.min - expected.min).length() < 1e-15, "{bounds:?}");
        assert!((bounds.max - expected.max).length() < 1e-15, "{bounds:?}");
    }
}
