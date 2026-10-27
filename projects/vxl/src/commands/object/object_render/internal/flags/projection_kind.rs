use crate::CliValue;

/// A view's projection kind, before its field of view or scale joins it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionKind {
    /// Rays fan out from the view's position.
    Perspective,

    /// Rays run parallel along the view's -Z.
    Orthographic,
}

impl CliValue for ProjectionKind {
    const VARIANTS: &'static [Self] = &[ProjectionKind::Perspective, ProjectionKind::Orthographic];

    fn name(self) -> &'static str {
        match self {
            ProjectionKind::Perspective => "perspective",
            ProjectionKind::Orthographic => "orthographic",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ProjectionKind::Perspective => {
                "Rays fan out from the view's position by its field of view"
            }

            ProjectionKind::Orthographic => "Rays run parallel across the view's scale",
        }
    }
}
