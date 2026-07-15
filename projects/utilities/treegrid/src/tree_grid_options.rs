use crate::{
    TreeGridColumnsOptions, TreeGridError, TreeGridHeaderOptions, TreeGridHierarchyOptions,
    TreeGridLabelKind, TreeGridLabelMode, TreeGridLayout, TreeGridLayoutKind,
    TreeGridNestedTableOptions, TreeGridRowsOptions, TreeGridTableLabelMode, TreeGridTableShape,
    TreeGridTableShapeKind,
};
use std::num::NonZeroU8;

/// Loose render options: every option as its own independent field.
///
/// [`resolve`](Self::resolve) maps them into the structural
/// [`TreeGridLayout`], rejecting any option the chosen layout does
/// not consume.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridOptions {
    /// The layout to resolve.
    pub layout: TreeGridLayoutKind,

    /// The label-mode flag, consumed by `rows`, `columns`, and
    /// `tables`; unset means `concat` there.
    pub label: Option<TreeGridLabelKind>,

    /// The wrap-budget flag, consumed by `rows`.
    pub width: Option<usize>,

    /// The shallowest-heading-level flag, consumed by
    /// heading-emitting renders; unset means `1` there.
    pub header_level: Option<NonZeroU8>,

    /// The bare-roots flag, consumed by `hierarchy`.
    pub bare_roots: bool,

    /// The value-children flag, consumed by `hierarchy`.
    pub value_children: bool,

    /// The table-shape flag, consumed by `tables`; unset means
    /// `nested` there.
    pub table_shape: Option<TreeGridTableShapeKind>,
}

impl TreeGridOptions {
    /// Sets the layout.
    pub fn with_layout(mut self, layout: TreeGridLayoutKind) -> Self {
        self.layout = layout;
        self
    }

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

    /// Maps the flags into a structural [`TreeGridLayout`], rejecting
    /// any flag the chosen layout does not consume.
    pub fn resolve(&self) -> Result<TreeGridLayout, TreeGridError> {
        match self.layout {
            TreeGridLayoutKind::Hierarchy => {
                self.no_label()?;
                self.no_width()?;
                self.no_header_level()?;
                self.no_table_shape()?;
                Ok(TreeGridLayout::Hierarchy(TreeGridHierarchyOptions {
                    bare_roots: self.bare_roots,
                    value_children: self.value_children,
                }))
            }
            TreeGridLayoutKind::Rows => {
                self.no_hierarchy_flags()?;
                self.no_table_shape()?;
                Ok(TreeGridLayout::Rows(TreeGridRowsOptions {
                    label: self.text_label()?,
                    width: self.width,
                }))
            }
            TreeGridLayoutKind::Columns => {
                self.no_hierarchy_flags()?;
                self.no_width()?;
                self.no_table_shape()?;
                Ok(TreeGridLayout::Columns(TreeGridColumnsOptions {
                    label: self.text_label()?,
                }))
            }
            TreeGridLayoutKind::Tables => {
                self.no_hierarchy_flags()?;
                self.no_width()?;
                Ok(TreeGridLayout::Tables(self.resolved_table_shape()?))
            }
            #[cfg(feature = "json")]
            TreeGridLayoutKind::JsonPretty => {
                self.no_flags()?;
                Ok(TreeGridLayout::JsonPretty)
            }
            #[cfg(feature = "json")]
            TreeGridLayoutKind::JsonCompact => {
                self.no_flags()?;
                Ok(TreeGridLayout::JsonCompact)
            }
        }
    }

    /// The `rows` / `columns` label mode, with the level flag folded
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

    /// The `tables` shape, with the label and level flags folded into
    /// the nested payload.
    fn resolved_table_shape(&self) -> Result<TreeGridTableShape, TreeGridError> {
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

    fn level(&self) -> NonZeroU8 {
        self.header_level.unwrap_or(NonZeroU8::MIN)
    }

    fn no_label(&self) -> Result<(), TreeGridError> {
        if self.label.is_some() {
            return Err(TreeGridError::LabelModeWithoutLabels);
        }
        Ok(())
    }

    fn no_width(&self) -> Result<(), TreeGridError> {
        if self.width.is_some() {
            return Err(TreeGridError::WidthWithoutRows);
        }
        Ok(())
    }

    fn no_header_level(&self) -> Result<(), TreeGridError> {
        if self.header_level.is_some() {
            return Err(TreeGridError::HeaderLevelWithoutHeaders);
        }
        Ok(())
    }

    fn no_table_shape(&self) -> Result<(), TreeGridError> {
        if self.table_shape.is_some() {
            return Err(TreeGridError::TableShapeWithoutTables);
        }
        Ok(())
    }

    fn no_hierarchy_flags(&self) -> Result<(), TreeGridError> {
        if self.bare_roots {
            return Err(TreeGridError::BareRootsWithoutHierarchy);
        }
        if self.value_children {
            return Err(TreeGridError::ValueChildrenWithoutHierarchy);
        }
        Ok(())
    }

    #[cfg(feature = "json")]
    fn no_flags(&self) -> Result<(), TreeGridError> {
        self.no_label()?;
        self.no_width()?;
        self.no_header_level()?;
        self.no_table_shape()?;
        self.no_hierarchy_flags()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridColumnsOptions, TreeGridError, TreeGridHeaderOptions, TreeGridHierarchyOptions,
        TreeGridLabelKind, TreeGridLabelMode, TreeGridLayout, TreeGridLayoutKind,
        TreeGridNestedTableOptions, TreeGridOptions, TreeGridRowsOptions, TreeGridTableLabelMode,
        TreeGridTableShape, TreeGridTableShapeKind,
    };
    use std::num::NonZeroU8;

    #[test]
    fn the_builder_sets_every_field() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Tables)
            .with_label(TreeGridLabelKind::Header)
            .with_width(80)
            .with_header_level(NonZeroU8::new(2).unwrap())
            .with_bare_roots(true)
            .with_value_children(true)
            .with_table_shape(TreeGridTableShapeKind::Flat);

        assert_eq!(
            options,
            TreeGridOptions {
                layout: TreeGridLayoutKind::Tables,
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
            TreeGridOptions::default().resolve(),
            Ok(TreeGridLayout::Rows(TreeGridRowsOptions::default()))
        );
    }

    #[test]
    fn hierarchy_flags_resolve_into_the_hierarchy_payload() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Hierarchy)
            .with_bare_roots(true)
            .with_value_children(true);

        assert_eq!(
            options.resolve(),
            Ok(TreeGridLayout::Hierarchy(
                TreeGridHierarchyOptions::default()
                    .with_bare_roots(true)
                    .with_value_children(true)
            ))
        );
    }

    #[test]
    fn a_header_label_carries_the_level() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Header)
            .with_header_level(NonZeroU8::new(3).unwrap())
            .with_width(72);

        assert_eq!(
            options.resolve(),
            Ok(TreeGridLayout::Rows(
                TreeGridRowsOptions::default()
                    .with_label(TreeGridLabelMode::Header(
                        TreeGridHeaderOptions::default().with_level(NonZeroU8::new(3).unwrap())
                    ))
                    .with_width(72)
            ))
        );
    }

    #[test]
    fn an_unset_level_defaults_to_one() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Columns)
            .with_label(TreeGridLabelKind::Header);

        assert_eq!(
            options.resolve(),
            Ok(TreeGridLayout::Columns(
                TreeGridColumnsOptions::default()
                    .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default()))
            ))
        );
    }

    #[test]
    fn tables_resolve_nested_with_header_labels_and_level() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Tables)
            .with_label(TreeGridLabelKind::Header)
            .with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve(),
            Ok(TreeGridLayout::Tables(TreeGridTableShape::Nested(
                TreeGridNestedTableOptions::default()
                    .with_label(TreeGridTableLabelMode::Header)
                    .with_level(NonZeroU8::new(2).unwrap())
            )))
        );
    }

    #[test]
    fn flat_tables_resolve_with_concat_labels() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Tables)
            .with_table_shape(TreeGridTableShapeKind::Flat);

        assert_eq!(
            options.resolve(),
            Ok(TreeGridLayout::Tables(TreeGridTableShape::Flat))
        );
    }

    #[test]
    fn a_label_off_the_text_layouts_is_rejected() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Hierarchy)
            .with_label(TreeGridLabelKind::Concat);

        assert_eq!(
            options.resolve(),
            Err(TreeGridError::LabelModeWithoutLabels)
        );
    }

    #[test]
    fn none_labels_with_tables_are_rejected() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Tables)
            .with_label(TreeGridLabelKind::None);

        assert_eq!(options.resolve(), Err(TreeGridError::LabelNoneWithTables));
    }

    #[test]
    fn header_labels_with_flat_tables_are_rejected() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Tables)
            .with_label(TreeGridLabelKind::Header)
            .with_table_shape(TreeGridTableShapeKind::Flat);

        assert_eq!(
            options.resolve(),
            Err(TreeGridError::HeaderLabelWithFlatTables)
        );
    }

    #[test]
    fn a_level_without_headings_is_rejected() {
        let options = TreeGridOptions::default().with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve(),
            Err(TreeGridError::HeaderLevelWithoutHeaders)
        );
    }

    #[test]
    fn a_width_off_rows_is_rejected() {
        let options = TreeGridOptions::default()
            .with_layout(TreeGridLayoutKind::Columns)
            .with_width(80);

        assert_eq!(options.resolve(), Err(TreeGridError::WidthWithoutRows));
    }

    #[test]
    fn a_table_shape_off_tables_is_rejected() {
        let options = TreeGridOptions::default().with_table_shape(TreeGridTableShapeKind::Nested);

        assert_eq!(
            options.resolve(),
            Err(TreeGridError::TableShapeWithoutTables)
        );
    }

    #[test]
    fn hierarchy_flags_off_hierarchy_are_rejected() {
        let bare = TreeGridOptions::default().with_bare_roots(true);
        let children = TreeGridOptions::default().with_value_children(true);

        assert_eq!(
            bare.resolve(),
            Err(TreeGridError::BareRootsWithoutHierarchy)
        );
        assert_eq!(
            children.resolve(),
            Err(TreeGridError::ValueChildrenWithoutHierarchy)
        );
    }

    #[cfg(feature = "json")]
    #[test]
    fn json_layouts_resolve_bare_and_reject_every_flag() {
        assert_eq!(
            TreeGridOptions::default()
                .with_layout(TreeGridLayoutKind::JsonCompact)
                .resolve(),
            Ok(TreeGridLayout::JsonCompact)
        );
        assert_eq!(
            TreeGridOptions::default()
                .with_layout(TreeGridLayoutKind::JsonPretty)
                .with_width(80)
                .resolve(),
            Err(TreeGridError::WidthWithoutRows)
        );
    }
}
