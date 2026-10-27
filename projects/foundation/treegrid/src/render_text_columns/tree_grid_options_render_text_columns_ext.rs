use crate::{TreeGridError, TreeGridOptions, TreeGridTextColumnsOptions};

impl TreeGridOptions {
    /// The text columns render's options, rejecting every option it does
    /// not consume.
    pub fn resolve_text_columns(&self) -> Result<TreeGridTextColumnsOptions, TreeGridError> {
        self.no_box_hierarchy_options()?;
        self.no_width()?;
        self.no_table_shape()?;
        Ok(TreeGridTextColumnsOptions {
            label: self.text_label()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridError, TreeGridHeaderOptions, TreeGridLabelKind, TreeGridLabelMode,
        TreeGridOptions, TreeGridTextColumnsOptions,
    };

    #[test]
    fn an_unset_level_defaults_to_one() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::Header);

        assert_eq!(
            options.resolve_text_columns(),
            Ok(TreeGridTextColumnsOptions::default()
                .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default())))
        );
    }

    #[test]
    fn a_width_off_rows_is_rejected() {
        let options = TreeGridOptions::default().with_width(80);

        assert_eq!(
            options.resolve_text_columns(),
            Err(TreeGridError::WidthWithoutTextRows)
        );
    }

    #[test]
    fn value_children_off_hierarchy_are_rejected() {
        let options = TreeGridOptions::default().with_value_children(true);

        assert_eq!(
            options.resolve_text_columns(),
            Err(TreeGridError::ValueChildrenWithoutBoxHierarchy)
        );
    }
}
