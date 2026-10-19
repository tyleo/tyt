use crate::{SdfjAxes3d, SdfjAxis, SdfjSide};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A 3D shape: an entry of [`SdfjFile::shapes3d`](crate::SdfjFile::shapes3d).
/// `kind` holds the call. A key the model leaves out reads `None`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase",
        deny_unknown_fields
    )
)]
pub enum SdfjShape3d {
    /// Curls one axis of a shape into an arc toward a side.
    Bend {
        shape: usize,

        along: SdfjAxis,

        toward: SdfjSide,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pivot: Option<[f64; 3]>,
    },

    /// A box between two corners.
    Box {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        min: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        max: [f64; 3],

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        round: Option<f64>,
    },

    /// The twelve edges of a box.
    BoxFrame {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        min: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        max: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        thickness: f64,
    },

    /// A capsule between two points.
    Capsule {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        a: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        b: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,
    },

    /// A cone between two points with a radius at each.
    Cone {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        a: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        b: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius_a: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius_b: f64,
    },

    /// A cylinder between two points.
    Cylinder {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        a: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        b: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        round: Option<f64>,
    },

    /// Roughens a shape's surface with fractal noise.
    Displace {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        amplitude: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        scale: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        octaves: Option<f64>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        seed: f64,
    },

    /// Cuts a shape across each axis and moves the halves apart.
    Elongate {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        lengths: [f64; 3],

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        center: Option<[f64; 3]>,
    },

    /// An ellipsoid.
    Ellipsoid {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radii: [f64; 3],
    },

    /// Pushes a 2D shape along an axis.
    Extrude {
        profile: usize,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        axis: Option<SdfjAxis>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        from: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        to: f64,
    },

    /// Everything on one side of a plane.
    HalfSpace {
        side: SdfjSide,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        at: f64,
    },

    /// The cells every shape covers.
    Intersect { shapes: Vec<usize> },

    /// Revolves the outline its points trace about an axis.
    Lathe {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        points: Vec<[f64; 2]>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        axis: Option<SdfjAxis>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        center: Option<[f64; 3]>,
    },

    /// A shape and its reflections across the axes.
    Mirror {
        shape: usize,

        axes: SdfjAxes3d,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        center: Option<[f64; 3]>,
    },

    /// An octahedron.
    Octahedron {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,
    },

    /// Grows or shrinks a shape.
    Offset {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        distance: f64,
    },

    /// Turns a shape until one direction points along another.
    Orient {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        from: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        to: [f64; 3],

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pivot: Option<[f64; 3]>,
    },

    /// A square pyramid.
    Pyramid {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        base_center: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        width: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        height: f64,
    },

    /// Copies a shape along each axis.
    Repeat {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        step: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        count: [f64; 3],
    },

    /// Copies a shape around an axis.
    RepeatPolar {
        shape: usize,

        axis: SdfjAxis,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        count: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        center: Option<[f64; 3]>,
    },

    /// Turns a 2D shape about an axis.
    Revolve {
        profile: usize,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        axis: Option<SdfjAxis>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        center: Option<[f64; 3]>,
    },

    /// Turns a shape about an axis.
    Rotate {
        shape: usize,

        axis: SdfjAxis,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        degrees: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pivot: Option<[f64; 3]>,
    },

    /// A cone with rounded ends.
    RoundCone {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        a: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        b: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius_a: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius_b: f64,
    },

    /// Scales a shape by a factor per axis.
    Scale {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        factor: [f64; 3],

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pivot: Option<[f64; 3]>,
    },

    /// Keeps a shape's outer wall and hollows the rest.
    Shell {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        thickness: f64,
    },

    /// An intersection with rounded edges.
    SmoothIntersect {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,

        shapes: Vec<usize>,
    },

    /// A subtraction with rounded edges.
    SmoothSubtract {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,

        base: usize,

        cutters: Vec<usize>,
    },

    /// A union with filleted corners.
    SmoothUnion {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,

        shapes: Vec<usize>,
    },

    /// A sphere.
    Sphere {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,
    },

    /// A base shape with the cutters removed.
    Subtract { base: usize, cutters: Vec<usize> },

    /// Pushes a 2D shape along a smooth curve through points.
    Sweep {
        profile: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        points: Vec<[f64; 3]>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        scales: Option<Vec<f64>>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        up: Option<[f64; 3]>,
    },

    /// A torus, or a cut of one between two angles.
    Torus {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 3],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        ring_radius: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        tube_radius: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        axis: Option<SdfjAxis>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        from: Option<f64>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        to: Option<f64>,
    },

    /// Moves a shape.
    Translate {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        offset: [f64; 3],
    },

    /// Twists a shape about an axis.
    Twist {
        shape: usize,

        axis: SdfjAxis,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        degrees_per_meter: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        center: Option<[f64; 3]>,
    },

    /// The cells any shape covers.
    Union { shapes: Vec<usize> },
}
