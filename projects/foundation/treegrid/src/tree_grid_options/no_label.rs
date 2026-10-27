use crate::{TreeGridError, TreeGridOptions};

impl TreeGridOptions {
    pub(crate) fn no_label(&self) -> Result<(), TreeGridError> {
        if self.label.is_some() {
            return Err(TreeGridError::LabelModeWithoutLabels);
        }
        Ok(())
    }
}
