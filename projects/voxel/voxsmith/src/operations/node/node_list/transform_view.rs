/// How each node's transform renders under
/// [`NodeListViews`](crate::operations::node::NodeListViews).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransformView {
    /// Render in world space; local otherwise.
    pub world: bool,

    /// Render rotation in degrees; radians otherwise.
    pub degrees: bool,

    /// Decimal places for each component.
    pub precision: usize,
}
