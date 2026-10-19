use crate::{BSdfShape2d, BSdfShape3d, SdfAxes3d, SdfSide};
use branded_id::U32Id;
use ty_math::{TyAxis3, TyVector2F64, TyVector3F64};

/// A 3D shape: an entry of [`SdfState::shapes3d`](crate::SdfState::shapes3d).
/// Each variant records one call. A field left `None` takes its default.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfShape3d {
    /// Curls one axis of a shape into an arc toward a side.
    Bend {
        shape_id: U32Id<BSdfShape3d>,

        along: TyAxis3,

        toward: SdfSide,

        radius: f64,

        pivot: Option<TyVector3F64>,
    },

    /// A box between two corners.
    Box {
        min: TyVector3F64,

        max: TyVector3F64,

        round: Option<f64>,
    },

    /// The twelve edges of a box.
    BoxFrame {
        min: TyVector3F64,

        max: TyVector3F64,

        thickness: f64,
    },

    /// A capsule between two points.
    Capsule {
        a: TyVector3F64,

        b: TyVector3F64,

        radius: f64,
    },

    /// A cone between two points with a radius at each.
    Cone {
        a: TyVector3F64,

        b: TyVector3F64,

        radius_a: f64,

        radius_b: f64,
    },

    /// A cylinder between two points.
    Cylinder {
        a: TyVector3F64,

        b: TyVector3F64,

        radius: f64,

        round: Option<f64>,
    },

    /// Roughens a shape's surface with fractal noise.
    Displace {
        shape_id: U32Id<BSdfShape3d>,

        amplitude: f64,

        scale: f64,

        octaves: Option<f64>,

        seed: f64,
    },

    /// Cuts a shape across each axis and moves the halves apart.
    Elongate {
        shape_id: U32Id<BSdfShape3d>,

        lengths: TyVector3F64,

        center: Option<TyVector3F64>,
    },

    /// An ellipsoid.
    Ellipsoid {
        center: TyVector3F64,

        radii: TyVector3F64,
    },

    /// Pushes a 2D shape along an axis.
    Extrude {
        profile_id: U32Id<BSdfShape2d>,

        axis: Option<TyAxis3>,

        from: f64,

        to: f64,
    },

    /// Everything on one side of a plane.
    HalfSpace { side: SdfSide, at: f64 },

    /// The cells every shape covers.
    Intersect { shape_ids: Vec<U32Id<BSdfShape3d>> },

    /// Revolves the outline its points trace about an axis.
    Lathe {
        points: Vec<TyVector2F64>,

        axis: Option<TyAxis3>,

        center: Option<TyVector3F64>,
    },

    /// A shape and its reflections across the axes.
    Mirror {
        shape_id: U32Id<BSdfShape3d>,

        axes: SdfAxes3d,

        center: Option<TyVector3F64>,
    },

    /// An octahedron.
    Octahedron { center: TyVector3F64, radius: f64 },

    /// Grows or shrinks a shape.
    Offset {
        shape_id: U32Id<BSdfShape3d>,

        distance: f64,
    },

    /// Turns a shape until one direction points along another.
    Orient {
        shape_id: U32Id<BSdfShape3d>,

        from: TyVector3F64,

        to: TyVector3F64,

        pivot: Option<TyVector3F64>,
    },

    /// A square pyramid.
    Pyramid {
        base_center: TyVector3F64,

        width: f64,

        height: f64,
    },

    /// Copies a shape along each axis.
    Repeat {
        shape_id: U32Id<BSdfShape3d>,

        step: TyVector3F64,

        count: TyVector3F64,
    },

    /// Copies a shape around an axis.
    RepeatPolar {
        shape_id: U32Id<BSdfShape3d>,

        axis: TyAxis3,

        count: f64,

        center: Option<TyVector3F64>,
    },

    /// Turns a 2D shape about an axis.
    Revolve {
        profile_id: U32Id<BSdfShape2d>,

        axis: Option<TyAxis3>,

        center: Option<TyVector3F64>,
    },

    /// Turns a shape about an axis.
    Rotate {
        shape_id: U32Id<BSdfShape3d>,

        axis: TyAxis3,

        degrees: f64,

        pivot: Option<TyVector3F64>,
    },

    /// A cone with rounded ends.
    RoundCone {
        a: TyVector3F64,

        b: TyVector3F64,

        radius_a: f64,

        radius_b: f64,
    },

    /// Scales a shape by a factor per axis.
    Scale {
        shape_id: U32Id<BSdfShape3d>,

        factor: TyVector3F64,

        pivot: Option<TyVector3F64>,
    },

    /// Keeps a shape's outer wall and hollows the rest.
    Shell {
        shape_id: U32Id<BSdfShape3d>,

        thickness: f64,
    },

    /// An intersection with rounded edges.
    SmoothIntersect {
        radius: f64,

        shape_ids: Vec<U32Id<BSdfShape3d>>,
    },

    /// A subtraction with rounded edges.
    SmoothSubtract {
        radius: f64,

        base_id: U32Id<BSdfShape3d>,

        cutter_ids: Vec<U32Id<BSdfShape3d>>,
    },

    /// A union with filleted corners.
    SmoothUnion {
        radius: f64,

        shape_ids: Vec<U32Id<BSdfShape3d>>,
    },

    /// A sphere.
    Sphere { center: TyVector3F64, radius: f64 },

    /// A base shape with the cutters removed.
    Subtract {
        base_id: U32Id<BSdfShape3d>,

        cutter_ids: Vec<U32Id<BSdfShape3d>>,
    },

    /// Pushes a 2D shape along a smooth curve through points.
    Sweep {
        profile_id: U32Id<BSdfShape2d>,

        points: Vec<TyVector3F64>,

        scales: Option<Vec<f64>>,

        up: Option<TyVector3F64>,
    },

    /// A torus, or a cut of one between two angles.
    Torus {
        center: TyVector3F64,

        ring_radius: f64,

        tube_radius: f64,

        axis: Option<TyAxis3>,

        from: Option<f64>,

        to: Option<f64>,
    },

    /// Moves a shape.
    Translate {
        shape_id: U32Id<BSdfShape3d>,

        offset: TyVector3F64,
    },

    /// Twists a shape about an axis.
    Twist {
        shape_id: U32Id<BSdfShape3d>,

        axis: TyAxis3,

        degrees_per_meter: f64,

        center: Option<TyVector3F64>,
    },

    /// The cells any shape covers.
    Union { shape_ids: Vec<U32Id<BSdfShape3d>> },
}
