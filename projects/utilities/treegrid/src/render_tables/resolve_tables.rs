use crate::{
    TreeGridError, TreeGridLabelKind, TreeGridNestedTableOptions, TreeGridOptions,
    TreeGridRecordsTableOptions, TreeGridTableLabelMode, TreeGridTableShape,
    TreeGridTableShapeKind,
};

impl TreeGridOptions {
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
            // Record sections are roots, whose concat path is their
            // leaf segment, so both heading label modes render alike.
            TreeGridTableShapeKind::Records => {
                Ok(TreeGridTableShape::Records(TreeGridRecordsTableOptions {
                    level: self.level(),
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridError, TreeGridLabelKind, TreeGridNestedTableOptions, TreeGridOptions,
        TreeGridRecordsTableOptions, TreeGridTableLabelMode, TreeGridTableShape,
        TreeGridTableShapeKind,
    };
    use std::num::NonZeroU8;

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
    fn none_labels_with_tables_are_rejected() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::None);

        assert_eq!(
            options.resolve_tables(),
            Err(TreeGridError::LabelNoneWithTables)
        );
    }

    #[test]
    fn records_resolve_with_the_header_level() {
        let options = TreeGridOptions::default()
            .with_table_shape(TreeGridTableShapeKind::Records)
            .with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve_tables(),
            Ok(TreeGridTableShape::Records(
                TreeGridRecordsTableOptions::default().with_level(NonZeroU8::new(2).unwrap())
            ))
        );
    }

    #[test]
    fn records_accept_both_heading_label_modes_alike() {
        let records = TreeGridOptions::default().with_table_shape(TreeGridTableShapeKind::Records);

        assert_eq!(
            records
                .with_label(TreeGridLabelKind::Concat)
                .resolve_tables(),
            records
                .with_label(TreeGridLabelKind::Header)
                .resolve_tables()
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
}
