use crate::CliValue;

/// The frame a view's or light's transform is read in, before the shape
/// checks which frames its entity takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformFrame {
    /// The document's frame.
    World,

    /// World axes centered on the subject's bounds.
    Subject,

    /// The view being rendered, so a light follows every view.
    Camera,

    /// One hierarchy node path's world transform, scale included, so a view
    /// or a light rides the node.
    Node,
}

impl CliValue for TransformFrame {
    const VARIANTS: &'static [Self] = &[
        TransformFrame::World,
        TransformFrame::Subject,
        TransformFrame::Camera,
        TransformFrame::Node,
    ];

    fn name(self) -> &'static str {
        match self {
            TransformFrame::World => "world",
            TransformFrame::Subject => "subject",
            TransformFrame::Camera => "camera",
            TransformFrame::Node => "node",
        }
    }

    fn help(self) -> &'static str {
        match self {
            TransformFrame::World => "The document's frame",
            TransformFrame::Subject => "World axes centered on the subject's bounds",
            TransformFrame::Camera => "The view being rendered, so a light follows every view",
            TransformFrame::Node => {
                "The node path a --view-node or --light-node gives, scale included"
            }
        }
    }
}
