use crate::{SdfjAxes2d, SdfjCaps, SdfjTextAlign};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A 2D shape: an entry of [`SdfjFile::shapes2d`](crate::SdfjFile::shapes2d).
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
pub enum SdfjShape2d {
    /// A band along a circle between two angles.
    Arc {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        from_degrees: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        to_degrees: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        width: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        caps: Option<SdfjCaps>,
    },

    /// A rectangle topped with a half circle.
    Arch {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        min: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        max: [f64; 2],
    },

    /// A circle.
    Circle {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,
    },

    /// An ellipse.
    Ellipse {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radii: [f64; 2],
    },

    /// The area every shape covers.
    Intersect { shapes: Vec<usize> },

    /// A shape and its reflections across the axes.
    Mirror {
        shape: usize,

        axes: SdfjAxes2d,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        center: Option<[f64; 2]>,
    },

    /// A regular polygon.
    Ngon {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        sides: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,
    },

    /// Grows or shrinks a shape.
    Offset {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        distance: f64,
    },

    /// A closed outline.
    Polygon {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        points: Vec<[f64; 2]>,
    },

    /// An open line with a width.
    Polyline {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        points: Vec<[f64; 2]>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        width: f64,
    },

    /// A rectangle between two corners.
    Rect {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        min: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        max: [f64; 2],

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        chamfer: Option<f64>,

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

    /// Copies a shape along each axis.
    Repeat {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        step: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        count: [f64; 2],
    },

    /// Copies a shape around a center.
    RepeatPolar {
        shape: usize,

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
        center: Option<[f64; 2]>,
    },

    /// Turns a shape.
    Rotate {
        shape: usize,

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
        pivot: Option<[f64; 2]>,
    },

    /// Scales a shape by a factor per axis.
    Scale {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        factor: [f64; 2],

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pivot: Option<[f64; 2]>,
    },

    /// A slice of a circle between two angles.
    Sector {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        from_degrees: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        to_degrees: f64,
    },

    /// Keeps a shape's outer wall and hollows the rest.
    Shell {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        thickness: f64,
    },

    /// An intersection with rounded corners.
    SmoothIntersect {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        radius: f64,

        shapes: Vec<usize>,
    },

    /// A subtraction with rounded corners.
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

    /// A smooth curve through points, closed into an outline or open with a
    /// width.
    Spline {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        points: Vec<[f64; 2]>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        width: Option<f64>,
    },

    /// A star.
    Star {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        center: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        points: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        outer_radius: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        inner_radius: f64,
    },

    /// A base shape with the cutters removed.
    Subtract { base: usize, cutters: Vec<usize> },

    /// A line of text in a font of [`SdfjFile::fonts`](crate::SdfjFile::fonts).
    Text {
        font: usize,

        string: String,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        size: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        align: Option<SdfjTextAlign>,
    },

    /// Moves a shape.
    Translate {
        shape: usize,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        offset: [f64; 2],
    },

    /// The area any shape covers.
    Union { shapes: Vec<usize> },

    /// A pointed lens between two points.
    Vesica {
        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        a: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        b: [f64; 2],

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        width: f64,
    },
}
