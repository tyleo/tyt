use crate::{BSdfShape2d, SdfAxes2d, SdfCaps};
use branded_id::U32Id;
use ty_math::TyVector2F64;

/// A 2D shape: an entry of [`SdfState::shapes2d`](crate::SdfState::shapes2d).
/// Each variant records one call. A field left `None` takes its default.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfShape2d {
    /// A band along a circle between two angles.
    Arc {
        center: TyVector2F64,

        radius: f64,

        from_degrees: f64,

        to_degrees: f64,

        width: f64,

        caps: Option<SdfCaps>,
    },

    /// A rectangle topped with a half circle.
    Arch {
        min: TyVector2F64,

        max: TyVector2F64,
    },

    /// A circle.
    Circle { center: TyVector2F64, radius: f64 },

    /// An ellipse.
    Ellipse {
        center: TyVector2F64,

        radii: TyVector2F64,
    },

    /// The area every shape covers.
    Intersect { shape_ids: Vec<U32Id<BSdfShape2d>> },

    /// A shape and its reflections across the axes.
    Mirror {
        shape_id: U32Id<BSdfShape2d>,

        axes: SdfAxes2d,

        center: Option<TyVector2F64>,
    },

    /// A regular polygon.
    Ngon {
        center: TyVector2F64,

        sides: f64,

        radius: f64,
    },

    /// Grows or shrinks a shape.
    Offset {
        shape_id: U32Id<BSdfShape2d>,

        distance: f64,
    },

    /// A closed outline.
    Polygon { points: Vec<TyVector2F64> },

    /// An open line with a width.
    Polyline {
        points: Vec<TyVector2F64>,

        width: f64,
    },

    /// A rectangle between two corners.
    Rect {
        min: TyVector2F64,

        max: TyVector2F64,

        chamfer: Option<f64>,

        round: Option<f64>,
    },

    /// Copies a shape along each axis.
    Repeat {
        shape_id: U32Id<BSdfShape2d>,

        step: TyVector2F64,

        count: TyVector2F64,
    },

    /// Copies a shape around a center.
    RepeatPolar {
        shape_id: U32Id<BSdfShape2d>,

        count: f64,

        center: Option<TyVector2F64>,
    },

    /// Turns a shape.
    Rotate {
        shape_id: U32Id<BSdfShape2d>,

        degrees: f64,

        pivot: Option<TyVector2F64>,
    },

    /// Scales a shape by a factor per axis.
    Scale {
        shape_id: U32Id<BSdfShape2d>,

        factor: TyVector2F64,

        pivot: Option<TyVector2F64>,
    },

    /// A slice of a circle between two angles.
    Sector {
        center: TyVector2F64,

        radius: f64,

        from_degrees: f64,

        to_degrees: f64,
    },

    /// Keeps a shape's outer wall and hollows the rest.
    Shell {
        shape_id: U32Id<BSdfShape2d>,

        thickness: f64,
    },

    /// An intersection with rounded corners.
    SmoothIntersect {
        radius: f64,

        shape_ids: Vec<U32Id<BSdfShape2d>>,
    },

    /// A subtraction with rounded corners.
    SmoothSubtract {
        radius: f64,

        base_id: U32Id<BSdfShape2d>,

        cutter_ids: Vec<U32Id<BSdfShape2d>>,
    },

    /// A union with filleted corners.
    SmoothUnion {
        radius: f64,

        shape_ids: Vec<U32Id<BSdfShape2d>>,
    },

    /// A smooth curve through points, closed into an outline or open with a
    /// width.
    Spline {
        points: Vec<TyVector2F64>,

        width: Option<f64>,
    },

    /// A star.
    Star {
        center: TyVector2F64,

        points: f64,

        outer_radius: f64,

        inner_radius: f64,
    },

    /// A base shape with the cutters removed.
    Subtract {
        base_id: U32Id<BSdfShape2d>,

        cutter_ids: Vec<U32Id<BSdfShape2d>>,
    },

    /// Moves a shape.
    Translate {
        shape_id: U32Id<BSdfShape2d>,

        offset: TyVector2F64,
    },

    /// The area any shape covers.
    Union { shape_ids: Vec<U32Id<BSdfShape2d>> },

    /// A pointed lens between two points.
    Vesica {
        a: TyVector2F64,

        b: TyVector2F64,

        width: f64,
    },
}
