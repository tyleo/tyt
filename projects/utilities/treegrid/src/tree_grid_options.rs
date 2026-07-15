use crate::{TreeGridLabelMode, TreeGridLayout};
use std::num::NonZeroU8;

/// Options for a render.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeGridOptions {
    /// The arrangement to render.
    pub layout: TreeGridLayout,

    /// How the `rows`, `columns`, and `tables` layouts spend the
    /// ancestor path; `None` means `Concat`. Setting a mode with the
    /// `hierarchy` or JSON layouts, which carry labels structurally,
    /// is
    /// [`LabelModeWithoutLabels`](crate::TreeGridError::LabelModeWithoutLabels).
    pub label: Option<TreeGridLabelMode>,

    /// Wrap budget in visible columns, consumed by the `rows` layout
    /// only; `None` never wraps.
    pub width: Option<usize>,

    /// The level of the shallowest heading on a heading-emitting
    /// render; `None` means `1`. A heading that nests past level `6`,
    /// markdown's deepest, renders as a bold label (`**label**`)
    /// instead of a deeper `#` run. Setting the option on a render
    /// that emits no headings is
    /// [`HeaderLevelWithoutHeaders`](crate::TreeGridError::HeaderLevelWithoutHeaders).
    pub header_level: Option<NonZeroU8>,

    /// When true, each `hierarchy`-layout root prints its label alone
    /// on an unprefixed line, its children below with connectors;
    /// when false, roots take connectors like any child.
    pub bare_roots: bool,
}

impl TreeGridOptions {
    /// Sets the layout.
    pub fn with_layout(mut self, layout: TreeGridLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Sets the label mode.
    pub fn with_label(mut self, label: TreeGridLabelMode) -> Self {
        self.label = Some(label);
        self
    }

    /// Sets the wrap budget.
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the level of the shallowest heading.
    pub fn with_header_level(mut self, header_level: NonZeroU8) -> Self {
        self.header_level = Some(header_level);
        self
    }

    /// Sets whether roots print bare.
    pub fn with_bare_roots(mut self, bare_roots: bool) -> Self {
        self.bare_roots = bare_roots;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridLabelMode, TreeGridLayout, TreeGridOptions};
    use std::num::NonZeroU8;

    #[test]
    fn the_builder_sets_every_field() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayout::Tables)
            .with_label(TreeGridLabelMode::Header)
            .with_width(80)
            .with_header_level(NonZeroU8::new(2).unwrap())
            .with_bare_roots(true);

        assert_eq!(
            options,
            TreeGridOptions {
                layout: TreeGridLayout::Tables,
                label: Some(TreeGridLabelMode::Header),
                width: Some(80),
                header_level: NonZeroU8::new(2),
                bare_roots: true,
            }
        );
    }
}
