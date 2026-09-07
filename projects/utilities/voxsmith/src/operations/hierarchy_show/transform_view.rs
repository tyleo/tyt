/// How each node's transform renders under
/// [`HierarchyViews`](crate::operations::hierarchy_show::HierarchyViews).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransformView {
    /// Render in world space; local otherwise.
    pub world: bool,

    /// Render rotation in degrees; radians otherwise.
    pub degrees: bool,

    /// Decimal places for each component.
    pub precision: usize,
}
