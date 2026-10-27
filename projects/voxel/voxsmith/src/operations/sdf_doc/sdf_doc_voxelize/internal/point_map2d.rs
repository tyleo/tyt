use ty_math::TyVector2F64;

/// A map that a transform or a copy applies to its child profile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointMap2d {
    /// Reflects each flagged axis across the line through `center`. No flag
    /// leaves every point in place.
    Reflection {
        axes: [bool; 2],
        center: TyVector2F64,
    },

    /// Turns counterclockwise about `pivot` by the angle whose sine and cosine
    /// are `sin` and `cos`.
    Rotation {
        sin: f64,

        cos: f64,

        pivot: TyVector2F64,
    },

    /// Scales away from `pivot` by a factor per axis.
    Scaling {
        factor: TyVector2F64,

        pivot: TyVector2F64,
    },

    /// Moves by `offset`.
    Translation { offset: TyVector2F64 },
}

impl PointMap2d {
    /// The point in the child's frame that the map takes to `point`.
    pub fn child_point(self, point: TyVector2F64) -> TyVector2F64 {
        match self {
            PointMap2d::Reflection { axes, center } => reflect(axes, center, point),
            PointMap2d::Rotation { sin, cos, pivot } => turn(-sin, cos, point - pivot) + pivot,
            PointMap2d::Scaling { factor, pivot } => (point - pivot) / factor + pivot,
            PointMap2d::Translation { offset } => point - offset,
        }
    }

    /// The point that the map takes `point` in the child's frame to.
    pub fn parent_point(self, point: TyVector2F64) -> TyVector2F64 {
        match self {
            PointMap2d::Reflection { axes, center } => reflect(axes, center, point),
            PointMap2d::Rotation { sin, cos, pivot } => turn(sin, cos, point - pivot) + pivot,
            PointMap2d::Scaling { factor, pivot } => (point - pivot) * factor + pivot,
            PointMap2d::Translation { offset } => point + offset,
        }
    }
}

fn reflect(axes: [bool; 2], center: TyVector2F64, point: TyVector2F64) -> TyVector2F64 {
    let mut reflected = point;

    for (index, reflects) in axes.into_iter().enumerate() {
        if reflects {
            reflected[index] = 2.0 * center[index] - point[index];
        }
    }

    reflected
}

fn turn(sin: f64, cos: f64, point: TyVector2F64) -> TyVector2F64 {
    TyVector2F64::new(point.x * cos - point.y * sin, point.x * sin + point.y * cos)
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::PointMap2d;
    use ty_math::TyVector2F64;

    #[test]
    fn child_point_undoes_parent_point() {
        let point = TyVector2F64::new(0.3, -0.7);
        let pivot = TyVector2F64::new(1.0, 2.0);
        let (sin, cos) = 0.4_f64.sin_cos();

        for map in [
            PointMap2d::Reflection {
                axes: [true, true],
                center: pivot,
            },
            PointMap2d::Rotation { sin, cos, pivot },
            PointMap2d::Scaling {
                factor: TyVector2F64::new(2.0, 0.5),
                pivot,
            },
            PointMap2d::Translation { offset: pivot },
        ] {
            assert!((map.child_point(map.parent_point(point)) - point).length() < 1e-14);
        }
    }
}
