/// Resolved `pattern` arguments: the node-path glob and the collapse flags that
/// act only alongside it, bundled so a collapse flag cannot be set without a
/// pattern.
#[derive(Clone, Debug)]
pub struct PatternView {
    /// The node-path glob.
    pub glob: String,

    /// Hide each match's ancestor chain behind an `ancestors` marker.
    pub collapse_ancestors: bool,

    /// Hide each match's descendants behind a `descendants` marker.
    pub collapse_descendants: bool,
}
