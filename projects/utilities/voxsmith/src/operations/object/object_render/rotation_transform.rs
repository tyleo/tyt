use crate::operations::object::Rotation;

/// The transform a directional light takes: a rotation read in a frame. The
/// light sits at the frame's origin.
#[derive(Clone, Debug, PartialEq)]
pub enum RotationTransform {
    /// The document's frame.
    World {
        /// The rotation.
        rotation: Rotation,
    },

    /// The view being rendered, so the light follows every view.
    Camera {
        /// The rotation.
        rotation: Rotation,
    },

    /// One hierarchy node path's world transform, so the light rides the
    /// node.
    Node {
        /// A glob over node paths that matches exactly one.
        path: String,

        /// The rotation.
        rotation: Rotation,
    },
}
