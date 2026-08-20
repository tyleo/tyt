#[cfg(any(
    feature = "render_columns",
    feature = "render_hierarchy",
    feature = "json",
    feature = "render_rows",
    feature = "render_tables"
))]
use crate::TreeGridError;
#[cfg(any(feature = "render_columns", feature = "render_rows"))]
use crate::{TreeGridHeaderOptions, TreeGridLabelMode};
use crate::{TreeGridLabelKind, TreeGridTableShapeKind};
use std::num::NonZeroU8;

/// Loose render options: every option as its own field.
///
/// The per-layout `resolve_*` methods map them into the payload the
/// matching render method consumes, rejecting any option that render
/// does not consume.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridOptions {
    /// The label mode, consumed by the rows, columns, and tables
    /// renders; unset means `concat` there.
    pub label: Option<TreeGridLabelKind>,

    /// The wrap budget, consumed by the rows render.
    pub width: Option<usize>,

    /// The level of the shallowest heading, consumed by
    /// heading-emitting renders; unset means `1` there.
    pub header_level: Option<NonZeroU8>,

    /// Whether roots print bare, consumed by the hierarchy render.
    pub bare_roots: bool,

    /// Whether values print as child lines, consumed by the hierarchy
    /// render.
    pub value_children: bool,

    /// The table shape, consumed by the tables render; unset means
    /// `nested` there.
    pub table_shape: Option<TreeGridTableShapeKind>,
}

impl TreeGridOptions {
    /// Sets the label mode.
    pub fn with_label(mut self, label: TreeGridLabelKind) -> Self {
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

    /// Sets whether values print as child lines.
    pub fn with_value_children(mut self, value_children: bool) -> Self {
        self.value_children = value_children;
        self
    }

    /// Sets the table shape.
    pub fn with_table_shape(mut self, table_shape: TreeGridTableShapeKind) -> Self {
        self.table_shape = Some(table_shape);
        self
    }

    /// The rows / columns label mode, with the header level folded
    /// into `header` labels.
    #[cfg(any(feature = "render_columns", feature = "render_rows"))]
    pub(crate) fn text_label(&self) -> Result<TreeGridLabelMode, TreeGridError> {
        match self.label.unwrap_or(TreeGridLabelKind::Concat) {
            TreeGridLabelKind::None => {
                self.no_header_level()?;
                Ok(TreeGridLabelMode::None)
            }
            TreeGridLabelKind::Concat => {
                self.no_header_level()?;
                Ok(TreeGridLabelMode::Concat)
            }
            TreeGridLabelKind::Header => Ok(TreeGridLabelMode::Header(TreeGridHeaderOptions {
                level: self.level(),
            })),
        }
    }

    #[cfg(any(
        feature = "render_columns",
        feature = "render_rows",
        feature = "render_tables"
    ))]
    pub(crate) fn level(&self) -> NonZeroU8 {
        self.header_level.unwrap_or(NonZeroU8::MIN)
    }

    #[cfg(any(feature = "render_hierarchy", feature = "json"))]
    pub(crate) fn no_label(&self) -> Result<(), TreeGridError> {
        if self.label.is_some() {
            return Err(TreeGridError::LabelModeWithoutLabels);
        }
        Ok(())
    }

    #[cfg(any(
        feature = "render_columns",
        feature = "render_hierarchy",
        feature = "json",
        feature = "render_tables"
    ))]
    pub(crate) fn no_width(&self) -> Result<(), TreeGridError> {
        if self.width.is_some() {
            return Err(TreeGridError::WidthWithoutRows);
        }
        Ok(())
    }

    #[cfg(any(
        feature = "render_columns",
        feature = "render_hierarchy",
        feature = "json",
        feature = "render_rows",
        feature = "render_tables"
    ))]
    pub(crate) fn no_header_level(&self) -> Result<(), TreeGridError> {
        if self.header_level.is_some() {
            return Err(TreeGridError::HeaderLevelWithoutHeaders);
        }
        Ok(())
    }

    #[cfg(any(
        feature = "render_columns",
        feature = "render_hierarchy",
        feature = "json",
        feature = "render_rows"
    ))]
    pub(crate) fn no_table_shape(&self) -> Result<(), TreeGridError> {
        if self.table_shape.is_some() {
            return Err(TreeGridError::TableShapeWithoutTables);
        }
        Ok(())
    }

    #[cfg(any(
        feature = "render_columns",
        feature = "json",
        feature = "render_rows",
        feature = "render_tables"
    ))]
    pub(crate) fn no_hierarchy_options(&self) -> Result<(), TreeGridError> {
        if self.bare_roots {
            return Err(TreeGridError::BareRootsWithoutHierarchy);
        }
        if self.value_children {
            return Err(TreeGridError::ValueChildrenWithoutHierarchy);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridLabelKind, TreeGridOptions, TreeGridTableShapeKind};
    use std::num::NonZeroU8;

    #[test]
    fn the_builder_sets_every_field() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Header)
            .with_width(80)
            .with_header_level(NonZeroU8::new(2).unwrap())
            .with_bare_roots(true)
            .with_value_children(true)
            .with_table_shape(TreeGridTableShapeKind::Flat);

        assert_eq!(
            options,
            TreeGridOptions {
                label: Some(TreeGridLabelKind::Header),
                width: Some(80),
                header_level: NonZeroU8::new(2),
                bare_roots: true,
                value_children: true,
                table_shape: Some(TreeGridTableShapeKind::Flat),
            }
        );
    }
}
