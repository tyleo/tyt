use crate::TreeGridLabelMode;

/// Options the `text-columns` layout consumes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridTextColumnsOptions {
    /// How columns spend the ancestor path.
    pub label: TreeGridLabelMode,
}

impl TreeGridTextColumnsOptions {
    /// Sets the label mode.
    pub fn with_label(mut self, label: TreeGridLabelMode) -> Self {
        self.label = label;
        self
    }
}
