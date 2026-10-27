use crate::{TreeGridError, TreeGridOptions};

impl TreeGridOptions {
    pub(crate) fn no_box_hierarchy_options(&self) -> Result<(), TreeGridError> {
        if self.bare_roots {
            return Err(TreeGridError::BareRootsWithoutBoxHierarchy);
        }
        if self.value_children {
            return Err(TreeGridError::ValueChildrenWithoutBoxHierarchy);
        }
        Ok(())
    }
}
