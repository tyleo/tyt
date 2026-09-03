/// The node-path patterns [`HierarchyShowOptions`](crate::HierarchyShowOptions)
/// filters by, bundled with the collapse flags so a collapse flag cannot be set
/// without a pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternView {
    /// The gitignore-style node-path patterns, in order. The last one to match
    /// a path decides whether it is selected.
    pub globs: Vec<String>,

    /// Hide each match's ancestor chain behind an `ancestors` marker.
    pub collapse_ancestors: bool,

    /// Hide each match's descendants behind a `descendants` marker.
    pub collapse_descendants: bool,
}
