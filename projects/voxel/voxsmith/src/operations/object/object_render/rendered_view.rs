use voxrender::{RenderOutput, RenderView};

/// One view's output from a render run.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderedView {
    /// The view resolved to world space.
    pub view: RenderView,

    /// The image.
    pub image: RenderOutput,
}
