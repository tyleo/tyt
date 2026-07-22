use clap::ValueEnum;

/// How `hierarchy show` renders the scene graph.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum HierarchyShowLayout {
    /// The scene graph as a box-glyph tree, each entity's tag and view rows
    /// inline on its nodes.
    #[value(name = "hierarchy")]
    Hierarchy,

    /// The scene graph as indented JSON records.
    #[value(name = "json-pretty")]
    JsonPretty,

    /// The scene graph as single-line JSON records.
    #[value(name = "json-compact")]
    JsonCompact,
}
