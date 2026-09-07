/// How an object's grid-min corner renders under
/// [`HierarchyViews`](crate::operations::hierarchy_show::HierarchyViews).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OriginView {
    /// Render in world space; local otherwise.
    pub world: bool,

    /// Decimal places for each component.
    pub precision: usize,
}
