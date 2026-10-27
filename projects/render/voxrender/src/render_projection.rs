/// How a view maps the scene onto the image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderProjection {
    /// Rays fan out from the view's position.
    Perspective {
        /// The vertical field of view, in radians.
        fov: f64,
    },

    /// Rays run parallel along the view's -Z.
    Orthographic {
        /// The world units across the shorter image axis.
        scale: f64,
    },
}
