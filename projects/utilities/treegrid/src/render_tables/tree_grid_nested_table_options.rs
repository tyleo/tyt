use crate::TreeGridTableLabelMode;
use std::num::NonZeroU8;

/// Options nested tables consume.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeGridNestedTableOptions {
    /// How the nested headings spend the ancestor path.
    pub label: TreeGridTableLabelMode,

    /// The level of the shallowest heading, default `1`. A heading
    /// that nests past level `6` renders as a bold label.
    pub level: NonZeroU8,
}

impl TreeGridNestedTableOptions {
    /// Sets the heading label mode.
    pub fn with_label(mut self, label: TreeGridTableLabelMode) -> Self {
        self.label = label;
        self
    }

    /// Sets the shallowest heading level.
    pub fn with_level(mut self, level: NonZeroU8) -> Self {
        self.level = level;
        self
    }
}

impl Default for TreeGridNestedTableOptions {
    fn default() -> Self {
        Self {
            label: TreeGridTableLabelMode::default(),
            level: NonZeroU8::MIN,
        }
    }
}
