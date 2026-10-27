use crate::operations::object::{PoseTransform, ViewProjection};

/// One view of a render run.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewRecord {
    /// The name errors identify the view by.
    pub name: String,

    /// The pose.
    pub transform: PoseTransform,

    /// The projection.
    pub projection: ViewProjection,

    /// Hierarchy-path globs narrowing the subject to the rendered objects
    /// they match, or empty for every rendered object.
    pub select: Vec<String>,
}
