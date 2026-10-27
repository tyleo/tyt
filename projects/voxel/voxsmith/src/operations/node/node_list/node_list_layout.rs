/// How [`node_list`](crate::operations::node::node_list()) renders the scene
/// graph.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NodeListLayout {
    /// The scene graph as a box-glyph tree, each entity's tag and view rows
    /// inline on its nodes.
    #[default]
    BoxHierarchy,

    /// The scene graph as single-line JSON records.
    JsonCompact,

    /// The scene graph as indented JSON records.
    JsonPretty,
}
