use crate::{TreeGridError, TreeGridLabelKind, TreeGridOptions, TreeGridTableShapeKind};

impl TreeGridOptions {
    /// The box tables render's shape, rejecting every option it does not
    /// consume. The bare heading lines carry full paths, so only the
    /// `concat` label mode fits.
    pub fn resolve_box_tables(&self) -> Result<TreeGridTableShapeKind, TreeGridError> {
        self.no_box_hierarchy_options()?;
        self.no_width()?;
        self.no_header_level()?;
        match self.label.unwrap_or(TreeGridLabelKind::Concat) {
            TreeGridLabelKind::Concat => {}
            TreeGridLabelKind::Header => return Err(TreeGridError::HeaderLabelWithBoxTables),
            TreeGridLabelKind::None => return Err(TreeGridError::LabelNoneWithTables),
        }
        Ok(self.table_shape.unwrap_or(TreeGridTableShapeKind::Nested))
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridError, TreeGridLabelKind, TreeGridOptions, TreeGridTableShapeKind};
    use std::num::NonZeroU8;

    #[test]
    fn an_unset_shape_resolves_nested() {
        assert_eq!(
            TreeGridOptions::default().resolve_box_tables(),
            Ok(TreeGridTableShapeKind::Nested)
        );
    }

    #[test]
    fn a_set_shape_resolves_as_given_under_concat_labels() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Concat)
            .with_table_shape(TreeGridTableShapeKind::Records);

        assert_eq!(
            options.resolve_box_tables(),
            Ok(TreeGridTableShapeKind::Records)
        );
    }

    #[test]
    fn header_labels_are_rejected() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::Header);

        assert_eq!(
            options.resolve_box_tables(),
            Err(TreeGridError::HeaderLabelWithBoxTables)
        );
    }

    #[test]
    fn none_labels_are_rejected() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::None);

        assert_eq!(
            options.resolve_box_tables(),
            Err(TreeGridError::LabelNoneWithTables)
        );
    }

    #[test]
    fn a_header_level_is_rejected() {
        let options = TreeGridOptions::default().with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve_box_tables(),
            Err(TreeGridError::HeaderLevelWithoutHeaders)
        );
    }
}
