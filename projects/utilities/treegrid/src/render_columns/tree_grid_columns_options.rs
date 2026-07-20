use crate::TreeGridLabelMode;

/// Options the `columns` layout consumes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridColumnsOptions {
    /// How columns spend the ancestor path.
    pub label: TreeGridLabelMode,
}

impl TreeGridColumnsOptions {
    /// Sets the label mode.
    pub fn with_label(mut self, label: TreeGridLabelMode) -> Self {
        self.label = label;
        self
    }
}
