use ty_math::{TyAngleUnit, TyQuaternionF64, TyVector3F64};

/// A rotation read in a frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rotation {
    /// A unit quaternion, the form a node stores.
    Quaternion {
        /// The quaternion.
        value: TyQuaternionF64,
    },

    /// Angles about the fixed x, y, then z axes.
    Euler {
        /// The angles.
        value: TyVector3F64,

        /// The unit of the angles.
        unit: TyAngleUnit,
    },

    /// Aims -Z from the entity's position at a point in the frame.
    LookAt {
        /// The point, or the frame's origin when `None`.
        target: Option<TyVector3F64>,
    },

    /// The rotation that faces the frame's origin from the direction of
    /// `azimuth` and `elevation`. A light shines from that direction.
    Angles {
        /// Degrees from +Z toward +X.
        azimuth: f64,

        /// Degrees from the azimuth's direction toward +Y.
        elevation: f64,
    },
}
