use crate::RenderShadow;
use ty_math::{TyLinSrgbF64, TyQuaternionF64, TyVector3F64};

/// A light in world space. Each kind carries the part of a pose it reads
/// and nothing more.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderLight {
    /// A light shining down its rotation's -Z from infinitely far away.
    Directional {
        /// The rotation, a unit quaternion.
        rotation: TyQuaternionF64,

        /// The color, in linear light.
        color: TyLinSrgbF64,

        /// The strength scaling the color.
        strength: f64,

        /// The shadow granularity.
        shadow: RenderShadow,
    },

    /// A light at a position, falling off by the inverse square of the
    /// distance in meters and cut off smoothly at `range`.
    Point {
        /// The position, in meters.
        position: TyVector3F64,

        /// The color, in linear light.
        color: TyLinSrgbF64,

        /// The strength scaling the color.
        strength: f64,

        /// The distance the light reaches, in meters, or `None` for no
        /// cutoff.
        range: Option<f64>,

        /// The shadow granularity.
        shadow: RenderShadow,
    },

    /// A light at a position shining down its rotation's -Z. It falls off
    /// as a point light does, times glTF's cone falloff: full strength
    /// inside `inner_cone`, none past `outer_cone`, and a smooth ramp
    /// between.
    Spot {
        /// The position, in meters.
        position: TyVector3F64,

        /// The rotation, a unit quaternion.
        rotation: TyQuaternionF64,

        /// The color, in linear light.
        color: TyLinSrgbF64,

        /// The strength scaling the color.
        strength: f64,

        /// The distance the light reaches, in meters, or `None` for no
        /// cutoff.
        range: Option<f64>,

        /// The half-angle in radians the full strength holds within, from
        /// `0` up to but excluding `outer_cone`.
        inner_cone: f64,

        /// The half-angle in radians past which no light reaches, at most
        /// a quarter turn.
        outer_cone: f64,

        /// The shadow granularity.
        shadow: RenderShadow,
    },

    /// The ambient term: a sky color above and a ground color below, mixed
    /// by a normal's world +Y.
    Hemisphere {
        /// The color from above, in linear light.
        sky: TyLinSrgbF64,

        /// The color from below, in linear light.
        ground: TyLinSrgbF64,

        /// The strength scaling both colors.
        strength: f64,
    },
}
