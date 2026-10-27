use crate::{TreeGridBoxHierarchyOptions, TreeGridError, TreeGridOptions};

impl TreeGridOptions {
    /// The box hierarchy render's options, rejecting every option it does
    /// not consume.
    pub fn resolve_box_hierarchy(&self) -> Result<TreeGridBoxHierarchyOptions, TreeGridError> {
        self.no_label()?;
        self.no_width()?;
        self.no_header_level()?;
        self.no_table_shape()?;
        Ok(TreeGridBoxHierarchyOptions {
            bare_roots: self.bare_roots,
            value_children: self.value_children,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridBoxHierarchyOptions, TreeGridError, TreeGridLabelKind, TreeGridOptions};

    #[test]
    fn hierarchy_options_resolve_into_the_hierarchy_payload() {
        let options = TreeGridOptions::default()
            .with_bare_roots(true)
            .with_value_children(true);

        assert_eq!(
            options.resolve_box_hierarchy(),
            Ok(TreeGridBoxHierarchyOptions::default()
                .with_bare_roots(true)
                .with_value_children(true))
        );
    }

    #[test]
    fn a_label_off_the_text_renders_is_rejected() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::Concat);

        assert_eq!(
            options.resolve_box_hierarchy(),
            Err(TreeGridError::LabelModeWithoutLabels)
        );
    }
}
