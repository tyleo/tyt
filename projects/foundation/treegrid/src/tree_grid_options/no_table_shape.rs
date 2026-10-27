use crate::{TreeGridError, TreeGridOptions};

impl TreeGridOptions {
    pub(crate) fn no_table_shape(&self) -> Result<(), TreeGridError> {
        if self.table_shape.is_some() {
            return Err(TreeGridError::TableShapeWithoutTables);
        }
        Ok(())
    }
}
