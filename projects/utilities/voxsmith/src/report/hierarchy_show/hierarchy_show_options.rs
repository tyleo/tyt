use crate::{HierarchyShowLayout, HierarchyViews, PatternView};

/// The options [`render_hierarchy_show`](crate::render_hierarchy_show) takes.
/// The default renders the whole scene as a box-glyph tree with no views.
#[derive(Clone, Debug, Default)]
pub struct HierarchyShowOptions {
    /// When set, only matched nodes and objects and their ancestors render.
    pub pattern: Option<PatternView>,

    /// The rendering to draw the populated grid through.
    pub layout: HierarchyShowLayout,

    /// Collapse repeat instances to a stub after the first placement.
    pub collapse_instances: bool,

    /// The per-node and per-object subtrees to append.
    pub views: HierarchyViews,
}
