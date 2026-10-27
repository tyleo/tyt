use crate::operations::node::{NodeListLayout, NodeListViews, PatternView};

/// The options [`node_list`](crate::operations::node::node_list()) takes. The
/// default renders the whole scene as a box-glyph tree with no views.
#[derive(Clone, Debug, Default)]
pub struct NodeListOptions {
    /// When set, only matched nodes and objects and their ancestors render.
    pub pattern: Option<PatternView>,

    /// The rendering to draw the populated grid through.
    pub layout: NodeListLayout,

    /// Collapse repeat instances to a stub after the first placement.
    pub collapse_instances: bool,

    /// The per-node and per-object subtrees to append.
    pub views: NodeListViews,
}
