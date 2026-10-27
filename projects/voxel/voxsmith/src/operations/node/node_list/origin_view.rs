/// How an object's grid-min corner renders under
/// [`NodeListViews`](crate::operations::node::NodeListViews).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OriginView {
    /// Render in world space; local otherwise.
    pub world: bool,

    /// Decimal places for each component.
    pub precision: usize,
}
