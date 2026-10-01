use crate::operations::object::Rotation;
use ty_math::TyVector3F64;

/// The transform a spot light takes: a position and a rotation read in any
/// of the five frames.
#[derive(Clone, Debug, PartialEq)]
pub enum SpotTransform {
    /// The document's frame.
    World {
        /// The position, in meters.
        position: TyVector3F64,

        /// The rotation.
        rotation: Rotation,
    },

    /// World axes centered on the subject's bounds.
    Subject {
        /// The position from the subject's center, in meters.
        position: TyVector3F64,

        /// The rotation.
        rotation: Rotation,
    },

    /// The view being rendered, so the light follows every view.
    Camera {
        /// The position from the camera, on its axes, in meters.
        position: TyVector3F64,

        /// The rotation.
        rotation: Rotation,
    },

    /// A position on a sphere about the subject's center, facing it.
    Orbit {
        /// Degrees from +Z toward +X.
        azimuth: f64,

        /// Degrees from the azimuth's direction toward +Y.
        elevation: f64,

        /// The sphere's radius, in meters.
        distance: f64,
    },

    /// One hierarchy node path's world transform, scale included, so the
    /// light rides the node.
    Node {
        /// A glob over node paths that matches exactly one.
        path: String,

        /// The position from the node's origin, on its axes, in meters.
        position: TyVector3F64,

        /// The rotation.
        rotation: Rotation,
    },
}
