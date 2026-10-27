use crate::{TreeGridError, TreeGridOptions, TreeGridTextRowsOptions};

impl TreeGridOptions {
    /// The text rows render's options, rejecting every option it does not
    /// consume.
    pub fn resolve_text_rows(&self) -> Result<TreeGridTextRowsOptions, TreeGridError> {
        self.no_box_hierarchy_options()?;
        self.no_table_shape()?;
        Ok(TreeGridTextRowsOptions {
            label: self.text_label()?,
            width: self.width,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridError, TreeGridHeaderOptions, TreeGridLabelKind, TreeGridLabelMode,
        TreeGridOptions, TreeGridTableShapeKind, TreeGridTextRowsOptions,
    };
    use std::num::NonZeroU8;

    #[test]
    fn default_options_resolve_to_concat_rows() {
        assert_eq!(
            TreeGridOptions::default().resolve_text_rows(),
            Ok(TreeGridTextRowsOptions::default())
        );
    }

    #[test]
    fn a_header_label_carries_the_level() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Header)
            .with_header_level(NonZeroU8::new(3).unwrap())
            .with_width(72);

        assert_eq!(
            options.resolve_text_rows(),
            Ok(TreeGridTextRowsOptions::default()
                .with_label(TreeGridLabelMode::Header(
                    TreeGridHeaderOptions::default().with_level(NonZeroU8::new(3).unwrap())
                ))
                .with_width(72))
        );
    }

    #[test]
    fn a_level_without_headings_is_rejected() {
        let options = TreeGridOptions::default().with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve_text_rows(),
            Err(TreeGridError::HeaderLevelWithoutHeaders)
        );
    }

    #[test]
    fn a_table_shape_off_tables_is_rejected() {
        let options = TreeGridOptions::default().with_table_shape(TreeGridTableShapeKind::Nested);

        assert_eq!(
            options.resolve_text_rows(),
            Err(TreeGridError::TableShapeWithoutTables)
        );
    }

    #[test]
    fn bare_roots_off_hierarchy_are_rejected() {
        let options = TreeGridOptions::default().with_bare_roots(true);

        assert_eq!(
            options.resolve_text_rows(),
            Err(TreeGridError::BareRootsWithoutBoxHierarchy)
        );
    }
}
