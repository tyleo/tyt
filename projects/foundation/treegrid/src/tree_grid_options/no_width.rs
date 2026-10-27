use crate::{TreeGridError, TreeGridOptions};

impl TreeGridOptions {
    pub(crate) fn no_width(&self) -> Result<(), TreeGridError> {
        if self.width.is_some() {
            return Err(TreeGridError::WidthWithoutTextRows);
        }
        Ok(())
    }
}
