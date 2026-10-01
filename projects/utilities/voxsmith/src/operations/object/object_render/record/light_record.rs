use crate::operations::object::{PositionTransform, RotationTransform, SpotTransform};
use ty_math::TyLinSrgbF64;
use voxrender::RenderShadow;

/// One light of a render run.
#[derive(Clone, Debug, PartialEq)]
pub enum LightRecord {
    /// A light shining down its rotation's -Z from infinitely far away.
    Directional {
        /// The rotation.
        transform: RotationTransform,

        /// The shadow granularity.
        shadow: RenderShadow,

        /// The color, in linear light.
        color: TyLinSrgbF64,

        /// The strength scaling the color.
        strength: f64,
    },

    /// A light at a position, falling off by the inverse square of the
    /// distance in meters and cut off smoothly at `range`.
    Point {
        /// The position.
        transform: PositionTransform,

        /// The shadow granularity.
        shadow: RenderShadow,

        /// The color, in linear light.
        color: TyLinSrgbF64,

        /// The strength scaling the color.
        strength: f64,

        /// The distance the light reaches, in meters, or `None` for no
        /// cutoff.
        range: Option<f64>,
    },

    /// A light at a position shining down its rotation's -Z. It falls off
    /// as a point light does, times glTF's cone falloff: full strength
    /// inside `inner_cone`, none past `outer_cone`, and a smooth ramp
    /// between.
    Spot {
        /// The pose.
        transform: SpotTransform,

        /// The shadow granularity.
        shadow: RenderShadow,

        /// The color, in linear light.
        color: TyLinSrgbF64,

        /// The strength scaling the color.
        strength: f64,

        /// The distance the light reaches, in meters, or `None` for no
        /// cutoff.
        range: Option<f64>,

        /// The half-angle in degrees the full strength holds within, from
        /// `0` up to but excluding `outer_cone`.
        inner_cone: f64,

        /// The half-angle in degrees past which no light reaches, at most
        /// `90`.
        outer_cone: f64,
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
