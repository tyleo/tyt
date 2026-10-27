use crate::{
    TreeGridError, TreeGridHeaderOptions, TreeGridLabelKind, TreeGridMdListsLabelMode,
    TreeGridMdListsOptions, TreeGridOptions,
};

impl TreeGridOptions {
    /// The lists render's options, rejecting every option it does not
    /// consume. An unset label means `header`.
    pub fn resolve_md_lists(&self) -> Result<TreeGridMdListsOptions, TreeGridError> {
        self.no_box_hierarchy_options()?;
        self.no_width()?;
        self.no_table_shape()?;
        let label = match self.label.unwrap_or(TreeGridLabelKind::Header) {
            TreeGridLabelKind::None => {
                self.no_header_level()?;
                TreeGridMdListsLabelMode::None
            }

            TreeGridLabelKind::Concat => return Err(TreeGridError::LabelConcatWithMdLists),

            TreeGridLabelKind::Header => TreeGridMdListsLabelMode::Header(TreeGridHeaderOptions {
                level: self.level(),
            }),
        };
        Ok(TreeGridMdListsOptions { label })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridError, TreeGridHeaderOptions, TreeGridLabelKind, TreeGridMdListsLabelMode,
        TreeGridMdListsOptions, TreeGridOptions, TreeGridTableShapeKind,
    };
    use std::num::NonZeroU8;

    #[test]
    fn default_options_resolve_to_header_lists() {
        assert_eq!(
            TreeGridOptions::default().resolve_md_lists(),
            Ok(TreeGridMdListsOptions::default())
        );
    }

    #[test]
    fn a_header_label_carries_the_level() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Header)
            .with_header_level(NonZeroU8::new(3).unwrap());

        assert_eq!(
            options.resolve_md_lists(),
            Ok(
                TreeGridMdListsOptions::default().with_label(TreeGridMdListsLabelMode::Header(
                    TreeGridHeaderOptions::default().with_level(NonZeroU8::new(3).unwrap())
                ))
            )
        );
    }

    #[test]
    fn a_none_label_drops_the_headings() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::None);

        assert_eq!(
            options.resolve_md_lists(),
            Ok(TreeGridMdListsOptions::default().with_label(TreeGridMdListsLabelMode::None))
        );
    }

    #[test]
    fn a_concat_label_is_rejected() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::Concat);

        assert_eq!(
            options.resolve_md_lists(),
            Err(TreeGridError::LabelConcatWithMdLists)
        );
    }

    #[test]
    fn a_level_without_headings_is_rejected() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::None)
            .with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve_md_lists(),
            Err(TreeGridError::HeaderLevelWithoutHeaders)
        );
    }

    #[test]
    fn options_of_other_layouts_are_rejected() {
        assert_eq!(
            TreeGridOptions::default().with_width(72).resolve_md_lists(),
            Err(TreeGridError::WidthWithoutTextRows)
        );
        assert_eq!(
            TreeGridOptions::default()
                .with_table_shape(TreeGridTableShapeKind::Nested)
                .resolve_md_lists(),
            Err(TreeGridError::TableShapeWithoutTables)
        );
        assert_eq!(
            TreeGridOptions::default()
                .with_value_children(true)
                .resolve_md_lists(),
            Err(TreeGridError::ValueChildrenWithoutBoxHierarchy)
        );
    }
}
