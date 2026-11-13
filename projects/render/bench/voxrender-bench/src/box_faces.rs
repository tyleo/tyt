/// Which of its boxes' faces a run of a [`BoxDraw`](crate::BoxDraw) draws.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoxFaces {
    /// The faces toward the view.
    Front,

    /// The faces away from the view, drawn nearer than they lie by
    /// `mask_march.wgsl`'s `NEARER`.
    NearBack,

    /// The faces away from the view at the nearest depth, where the view's
    /// eye lies within reach of the box. The module's `place_held` places
    /// them.
    Held,
}
