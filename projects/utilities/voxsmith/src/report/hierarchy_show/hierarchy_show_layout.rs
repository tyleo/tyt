/// How [`render_hierarchy_show`](crate::render_hierarchy_show) renders the
/// scene graph.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HierarchyShowLayout {
    /// The scene graph as a box-glyph tree, each entity's tag and view rows
    /// inline on its nodes.
    #[default]
    Hierarchy,

    /// The scene graph as indented JSON records.
    JsonPretty,

    /// The scene graph as single-line JSON records.
    JsonCompact,
}
