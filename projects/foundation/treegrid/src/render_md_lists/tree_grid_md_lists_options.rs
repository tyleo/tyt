use crate::TreeGridMdListsLabelMode;

/// Options the `md-lists` layout consumes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridMdListsOptions {
    /// How lists spend the ancestor path.
    pub label: TreeGridMdListsLabelMode,
}

impl TreeGridMdListsOptions {
    /// Sets the label mode.
    pub fn with_label(mut self, label: TreeGridMdListsLabelMode) -> Self {
        self.label = label;
        self
    }
}
