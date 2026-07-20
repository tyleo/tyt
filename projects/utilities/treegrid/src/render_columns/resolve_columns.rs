use crate::{TreeGridColumnsOptions, TreeGridError, TreeGridOptions};

impl TreeGridOptions {
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
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridColumnsOptions, TreeGridError, TreeGridHeaderOptions, TreeGridLabelKind,
        TreeGridLabelMode, TreeGridOptions,
    };

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
    fn a_width_off_rows_is_rejected() {
        let options = TreeGridOptions::default().with_width(80);

        assert_eq!(
            options.resolve_columns(),
            Err(TreeGridError::WidthWithoutRows)
        );
    }

    #[test]
    fn value_children_off_hierarchy_are_rejected() {
        let options = TreeGridOptions::default().with_value_children(true);

        assert_eq!(
            options.resolve_columns(),
            Err(TreeGridError::ValueChildrenWithoutHierarchy)
        );
    }
}
