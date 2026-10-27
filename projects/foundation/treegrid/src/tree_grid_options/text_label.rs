use crate::{
    TreeGridError, TreeGridHeaderOptions, TreeGridLabelKind, TreeGridLabelMode, TreeGridOptions,
};

impl TreeGridOptions {
    /// The text rows / columns label mode, with the header level folded
    /// into `header` labels.
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
}
