/// Which of its boxes' faces a run of a [`BoxDraw`](crate::BoxDraw) draws.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoxFaces {
    /// The faces toward the view.
    Front,

    /// The faces away from the view.
    Back,

    /// The faces away from the view, drawn nearer than they lie by
    /// `mask_march.wgsl`'s `NEARER`.
    NearBack,
}
