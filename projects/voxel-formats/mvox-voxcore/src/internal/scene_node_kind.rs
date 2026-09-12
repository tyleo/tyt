/// The scene-graph chunk a hierarchy node writes as.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SceneNodeKind {
    /// An `nTRN` over one child node.
    Transform,

    /// An `nGRP` over child nodes.
    Group,

    /// An `nSHP` over placed objects.
    Shape,
}
