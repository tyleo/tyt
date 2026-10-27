use crate::{TreeGridError, TreeGridOptions};

impl TreeGridOptions {
    pub(crate) fn no_header_level(&self) -> Result<(), TreeGridError> {
        if self.header_level.is_some() {
            return Err(TreeGridError::HeaderLevelWithoutHeaders);
        }
        Ok(())
    }
}
