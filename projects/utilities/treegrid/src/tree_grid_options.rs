use crate::{
    TreeGridColumnsOptions, TreeGridError, TreeGridHeaderOptions, TreeGridHierarchyOptions,
    TreeGridLabelKind, TreeGridLabelMode, TreeGridNestedTableOptions, TreeGridRowsOptions,
    TreeGridTableLabelMode, TreeGridTableShape, TreeGridTableShapeKind,
};
use std::num::NonZeroU8;

/// Loose render options: every option as its own independent field.
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

    /// The hierarchy render's options, rejecting every option it does
    /// not consume.
    pub fn resolve_hierarchy(&self) -> Result<TreeGridHierarchyOptions, TreeGridError> {
        self.no_label()?;
        self.no_width()?;
        self.no_header_level()?;
        self.no_table_shape()?;
        Ok(TreeGridHierarchyOptions {
            bare_roots: self.bare_roots,
            value_children: self.value_children,
        })
    }

    /// The rows render's options, rejecting every option it does not
    /// consume.
    pub fn resolve_rows(&self) -> Result<TreeGridRowsOptions, TreeGridError> {
        self.no_hierarchy_options()?;
        self.no_table_shape()?;
        Ok(TreeGridRowsOptions {
            label: self.text_label()?,
            width: self.width,
        })
    }

    /// The columns render's options, rejecting every option it does
    /// not consume.
    pub fn resolve_columns(&self) -> Result<TreeGridColumnsOptions, TreeGridError> {
        self.no_hierarchy_options()?;
        self.no_width()?;
        self.no_table_shape()?;
        Ok(TreeGridColumnsOptions {
            label: self.text_label()?,
        })
    }

    /// The tables render's shape, rejecting every option it does not
    /// consume.
    pub fn resolve_tables(&self) -> Result<TreeGridTableShape, TreeGridError> {
        self.no_hierarchy_options()?;
        self.no_width()?;
        let label = match self.label.unwrap_or(TreeGridLabelKind::Concat) {
            TreeGridLabelKind::None => return Err(TreeGridError::LabelNoneWithTables),
            TreeGridLabelKind::Concat => TreeGridTableLabelMode::Concat,
            TreeGridLabelKind::Header => TreeGridTableLabelMode::Header,
        };
        match self.table_shape.unwrap_or(TreeGridTableShapeKind::Nested) {
            TreeGridTableShapeKind::Nested => {
                Ok(TreeGridTableShape::Nested(TreeGridNestedTableOptions {
                    label,
                    level: self.level(),
                }))
            }
            TreeGridTableShapeKind::Flat => {
                if label == TreeGridTableLabelMode::Header {
                    return Err(TreeGridError::HeaderLabelWithFlatTables);
                }
                self.no_header_level()?;
                Ok(TreeGridTableShape::Flat)
            }
        }
    }

    /// The rows / columns label mode, with the header level folded
    /// into `header` labels.
    fn text_label(&self) -> Result<TreeGridLabelMode, TreeGridError> {
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

    fn level(&self) -> NonZeroU8 {
        self.header_level.unwrap_or(NonZeroU8::MIN)
    }

    pub(crate) fn no_label(&self) -> Result<(), TreeGridError> {
        if self.label.is_some() {
            return Err(TreeGridError::LabelModeWithoutLabels);
        }
        Ok(())
    }

    pub(crate) fn no_width(&self) -> Result<(), TreeGridError> {
        if self.width.is_some() {
            return Err(TreeGridError::WidthWithoutRows);
        }
        Ok(())
    }

    pub(crate) fn no_header_level(&self) -> Result<(), TreeGridError> {
        if self.header_level.is_some() {
            return Err(TreeGridError::HeaderLevelWithoutHeaders);
        }
        Ok(())
    }

    pub(crate) fn no_table_shape(&self) -> Result<(), TreeGridError> {
        if self.table_shape.is_some() {
            return Err(TreeGridError::TableShapeWithoutTables);
        }
        Ok(())
    }

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
    use crate::{
        TreeGridColumnsOptions, TreeGridError, TreeGridHeaderOptions, TreeGridHierarchyOptions,
        TreeGridLabelKind, TreeGridLabelMode, TreeGridNestedTableOptions, TreeGridOptions,
        TreeGridRowsOptions, TreeGridTableLabelMode, TreeGridTableShape, TreeGridTableShapeKind,
    };
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

    #[test]
    fn default_options_resolve_to_concat_rows() {
        assert_eq!(
            TreeGridOptions::default().resolve_rows(),
            Ok(TreeGridRowsOptions::default())
        );
    }

    #[test]
    fn hierarchy_options_resolve_into_the_hierarchy_payload() {
        let options = TreeGridOptions::default()
            .with_bare_roots(true)
            .with_value_children(true);

        assert_eq!(
            options.resolve_hierarchy(),
            Ok(TreeGridHierarchyOptions::default()
                .with_bare_roots(true)
                .with_value_children(true))
        );
    }

    #[test]
    fn a_header_label_carries_the_level() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Header)
            .with_header_level(NonZeroU8::new(3).unwrap())
            .with_width(72);

        assert_eq!(
            options.resolve_rows(),
            Ok(TreeGridRowsOptions::default()
                .with_label(TreeGridLabelMode::Header(
                    TreeGridHeaderOptions::default().with_level(NonZeroU8::new(3).unwrap())
                ))
                .with_width(72))
        );
    }

    #[test]
    fn an_unset_level_defaults_to_one() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::Header);

        assert_eq!(
            options.resolve_columns(),
            Ok(TreeGridColumnsOptions::default()
                .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default())))
        );
    }

    #[test]
    fn tables_resolve_nested_with_header_labels_and_level() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Header)
            .with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve_tables(),
            Ok(TreeGridTableShape::Nested(
                TreeGridNestedTableOptions::default()
                    .with_label(TreeGridTableLabelMode::Header)
                    .with_level(NonZeroU8::new(2).unwrap())
            ))
        );
    }

    #[test]
    fn flat_tables_resolve_with_concat_labels() {
        let options = TreeGridOptions::default().with_table_shape(TreeGridTableShapeKind::Flat);

        assert_eq!(options.resolve_tables(), Ok(TreeGridTableShape::Flat));
    }

    #[test]
    fn a_label_off_the_text_renders_is_rejected() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::Concat);

        assert_eq!(
            options.resolve_hierarchy(),
            Err(TreeGridError::LabelModeWithoutLabels)
        );
    }

    #[test]
    fn none_labels_with_tables_are_rejected() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::None);

        assert_eq!(
            options.resolve_tables(),
            Err(TreeGridError::LabelNoneWithTables)
        );
    }

    #[test]
    fn header_labels_with_flat_tables_are_rejected() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Header)
            .with_table_shape(TreeGridTableShapeKind::Flat);

        assert_eq!(
            options.resolve_tables(),
            Err(TreeGridError::HeaderLabelWithFlatTables)
        );
    }

    #[test]
    fn a_level_without_headings_is_rejected() {
        let options = TreeGridOptions::default().with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve_rows(),
            Err(TreeGridError::HeaderLevelWithoutHeaders)
        );
    }

    #[test]
    fn a_width_off_rows_is_rejected() {
        let options = TreeGridOptions::default().with_width(80);

        assert_eq!(
            options.resolve_columns(),
            Err(TreeGridError::WidthWithoutRows)
        );
    }

    #[test]
    fn a_table_shape_off_tables_is_rejected() {
        let options = TreeGridOptions::default().with_table_shape(TreeGridTableShapeKind::Nested);

        assert_eq!(
            options.resolve_rows(),
            Err(TreeGridError::TableShapeWithoutTables)
        );
    }

    #[test]
    fn hierarchy_options_off_hierarchy_are_rejected() {
        let bare = TreeGridOptions::default().with_bare_roots(true);
        let children = TreeGridOptions::default().with_value_children(true);

        assert_eq!(
            bare.resolve_rows(),
            Err(TreeGridError::BareRootsWithoutHierarchy)
        );
        assert_eq!(
            children.resolve_columns(),
            Err(TreeGridError::ValueChildrenWithoutHierarchy)
        );
    }
}
