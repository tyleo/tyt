use crate::{HierarchyBounds, HierarchyTransform};

/// One object in the hierarchy payload; entries arrive in pre-order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyEntry {
    /// The object's name, the last segment of its path.
    pub name: String,

    /// The object's `/`-joined path from its scene root.
    pub path: String,

    /// The object's type, e.g. `MESH` or `EMPTY`.
    pub object_type: String,

    /// The transform payload, present when transforms were requested.
    pub transform: Option<HierarchyTransform>,

    /// The bounds payload, present when bounds were requested and the
    /// object's subtree has mesh geometry.
    pub bounds: Option<HierarchyBounds>,

    /// The extents components, present when extents were requested and
    /// the object's subtree has mesh geometry.
    pub extents: Option<[String; 3]>,
}
