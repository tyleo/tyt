use ty_math::TyVector3F64;

/// A map that a transform or a copy applies to its child shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointMap3d {
    /// Reflects each flagged axis across the plane through `center`. No flag
    /// leaves every point in place.
    Reflection {
        axes: [bool; 3],
        center: TyVector3F64,
    },

    /// Turns about `pivot` by the rotation whose rows are `rows`.
    Rotation {
        rows: [TyVector3F64; 3],

        pivot: TyVector3F64,
    },

    /// Scales away from `pivot` by a factor per axis.
    Scaling {
        factor: TyVector3F64,

        pivot: TyVector3F64,
    },

    /// Moves by `offset`.
    Translation { offset: TyVector3F64 },
}

impl PointMap3d {
    /// The point in the child's frame that the map takes to `point`.
    pub fn child_point(self, point: TyVector3F64) -> TyVector3F64 {
        match self {
            PointMap3d::Reflection { axes, center } => reflect(axes, center, point),

            PointMap3d::Rotation { rows, pivot } => {
                let offset = point - pivot;
                let columns = [
                    TyVector3F64::new(rows[0].x, rows[1].x, rows[2].x),
                    TyVector3F64::new(rows[0].y, rows[1].y, rows[2].y),
                    TyVector3F64::new(rows[0].z, rows[1].z, rows[2].z),
                ];
                turn(columns, offset) + pivot
            }

            PointMap3d::Scaling { factor, pivot } => (point - pivot) / factor + pivot,

            PointMap3d::Translation { offset } => point - offset,
        }
    }

    /// The point that the map takes `point` in the child's frame to.
    pub fn parent_point(self, point: TyVector3F64) -> TyVector3F64 {
        match self {
            PointMap3d::Reflection { axes, center } => reflect(axes, center, point),
            PointMap3d::Rotation { rows, pivot } => turn(rows, point - pivot) + pivot,
            PointMap3d::Scaling { factor, pivot } => (point - pivot) * factor + pivot,
            PointMap3d::Translation { offset } => point + offset,
        }
    }
}

fn reflect(axes: [bool; 3], center: TyVector3F64, point: TyVector3F64) -> TyVector3F64 {
    let mut reflected = point;

    for (index, reflects) in axes.into_iter().enumerate() {
        if reflects {
            reflected[index] = 2.0 * center[index] - point[index];
        }
    }

    reflected
}

fn turn(rows: [TyVector3F64; 3], point: TyVector3F64) -> TyVector3F64 {
    TyVector3F64::new(rows[0].dot(point), rows[1].dot(point), rows[2].dot(point))
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{PointMap3d, axis_rotation};
    use ty_math::{TyAxis3, TyVector3F64};

    #[test]
    fn child_point_undoes_parent_point() {
        let point = TyVector3F64::new(0.3, -0.7, 1.1);
        let pivot = TyVector3F64::new(1.0, 2.0, -1.0);
        let (sin, cos) = 0.4_f64.sin_cos();

        for map in [
            PointMap3d::Reflection {
                axes: [true, false, true],
                center: pivot,
            },
            PointMap3d::Rotation {
                rows: axis_rotation(TyAxis3::Y, sin, cos),
                pivot,
            },
            PointMap3d::Scaling {
                factor: TyVector3F64::new(2.0, 0.5, 3.0),
                pivot,
            },
            PointMap3d::Translation { offset: pivot },
        ] {
            assert!((map.child_point(map.parent_point(point)) - point).length() < 1e-14);
        }
    }

    #[test]
    fn a_quarter_turn_maps_exactly() {
        let map = PointMap3d::Rotation {
            rows: axis_rotation(TyAxis3::Z, 1.0, 0.0),
            pivot: TyVector3F64::new(1.0, 0.0, 0.0),
        };

        assert_eq!(
            map.parent_point(TyVector3F64::new(2.0, 0.0, 0.5)),
            TyVector3F64::new(1.0, 1.0, 0.5)
        );
    }
}
