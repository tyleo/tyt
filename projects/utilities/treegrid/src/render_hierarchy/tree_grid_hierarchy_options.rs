/// Options the `hierarchy` layout consumes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridHierarchyOptions {
    /// When true, each root prints its label alone on an unprefixed
    /// line, its children below with connectors; when false, roots
    /// take connectors like any child.
    pub bare_roots: bool,

    /// When true, each data node prints its label alone and its
    /// values print as child lines beneath, one cell per line before
    /// the node's children; when false, values print inline after the
    /// label.
    pub value_children: bool,
}

impl TreeGridHierarchyOptions {
    /// Sets whether roots print bare.
    pub fn with_bare_roots(mut self, bare_roots: bool) -> Self {
        self.bare_roots = bare_roots;
        self
    }

    /// Sets whether values print as child lines.
    pub fn with_value_children(mut self, value_children: bool) -> Self {
        self.value_children = value_children;
        self
    }
}
