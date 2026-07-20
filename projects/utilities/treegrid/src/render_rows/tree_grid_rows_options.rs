use crate::TreeGridLabelMode;

/// Options the `rows` layout consumes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridRowsOptions {
    /// How rows spend the ancestor path.
    pub label: TreeGridLabelMode,

    /// Wrap budget in visible columns; `None` never wraps.
    pub width: Option<usize>,
}

impl TreeGridRowsOptions {
    /// Sets the label mode.
    pub fn with_label(mut self, label: TreeGridLabelMode) -> Self {
        self.label = label;
        self
    }

    /// Sets the wrap budget.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }
}
